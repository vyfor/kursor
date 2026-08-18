use std::{
    cell::UnsafeCell,
    ops::Deref,
    sync::atomic::{AtomicU8, AtomicU32, Ordering},
};

use crate::state::{deps, id::AtomId, queue::dirty_queue, spin::SpinLock};

pub static CURRENT_FRAME_EPOCH: AtomicU32 = AtomicU32::new(1);

pub trait State: Clone + Send + Sync + 'static {}
impl<T: Clone + Send + Sync + 'static> State for T {}

#[repr(align(64))]
pub struct Slot<T: State> {
    buffers: [UnsafeCell<T>; 3],
    read_idx: AtomicU8,
    lock: SpinLock,
    epoch: AtomicU32,
    id: AtomId,
}

unsafe impl<T: State> Sync for Slot<T> {}
unsafe impl<T: State> Send for Slot<T> {}

impl<T: State> Slot<T> {
    pub fn new(id: AtomId, initial: T) -> Self {
        Self {
            buffers: [
                UnsafeCell::new(initial.clone()),
                UnsafeCell::new(initial.clone()),
                UnsafeCell::new(initial),
            ],
            read_idx: AtomicU8::new(0),
            lock: SpinLock::new(),
            epoch: AtomicU32::new(0),
            id,
        }
    }

    pub fn id(&self) -> AtomId {
        self.id
    }

    #[inline(always)]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        deps::record(self.id);
        let idx = self.read_idx.load(Ordering::Acquire) as usize;
        let val = unsafe { &*self.buffers[idx].get() };
        f(val)
    }

    #[inline(always)]
    pub fn borrow(&self) -> Guard<'_, T> {
        deps::record(self.id);
        let idx = self.read_idx.load(Ordering::Acquire) as usize;
        Guard { slot: self, idx }
    }

    #[inline(always)]
    pub fn read(&self) -> T {
        self.with(|v| v.clone())
    }

    #[inline(always)]
    pub fn peek(&self) -> T {
        let idx = self.read_idx.load(Ordering::Acquire) as usize;
        let val = unsafe { &*self.buffers[idx].get() };
        val.clone()
    }

    #[inline(always)]
    pub fn set(&self, val: T) {
        self.lock.lock();
        let read_idx = self.read_idx.load(Ordering::Relaxed);
        let target_idx = ((read_idx + 1) % 3) as usize;

        unsafe {
            *self.buffers[target_idx].get() = val;
        }

        self.read_idx.store(target_idx as u8, Ordering::Release);
        self.lock.unlock();
        self.mark_dirty();
    }

    #[inline(always)]
    pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.lock.lock();
        let read_idx = self.read_idx.load(Ordering::Relaxed);
        let target_idx = ((read_idx + 1) % 3) as usize;

        let val = unsafe { &*self.buffers[read_idx as usize].get() };
        let target_buf = unsafe { &mut *self.buffers[target_idx].get() };
        target_buf.clone_from(val);

        let res = f(target_buf);

        self.read_idx.store(target_idx as u8, Ordering::Release);
        self.lock.unlock();
        self.mark_dirty();
        res
    }

    #[inline(always)]
    fn mark_dirty(&self) {
        let current = CURRENT_FRAME_EPOCH.load(Ordering::Relaxed);
        if self.epoch.swap(current, Ordering::Relaxed) != current {
            dirty_queue().mark(self.id);
        }
    }
}

pub struct Guard<'a, T: State> {
    slot: &'a Slot<T>,
    idx: usize,
}

impl<T: State> Deref for Guard<'_, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.slot.buffers[self.idx].get() }
    }
}
