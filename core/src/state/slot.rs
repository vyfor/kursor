use std::{
    cell::UnsafeCell, marker::PhantomData, mem::MaybeUninit, ops::Deref, ptr, sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering},
};

use crate::state::{deps, id::AtomId, queue::dirty_queue, scope};

pub static CURRENT_FRAME_EPOCH: AtomicU32 = AtomicU32::new(1);

pub trait State: Clone + PartialEq + Send + Sync + 'static {}
impl<T: Clone + PartialEq + Send + Sync + 'static> State for T {}

const WRITING: u32 = 1 << 31;
const READERS: u32 = !WRITING;
const DEAD: u32 = u32::MAX;

struct Node<T> {
    value: UnsafeCell<MaybeUninit<T>>,
    flags: AtomicU32,
}

struct Retired<T> {
    node: *mut Node<T>,
    epoch: u64,
}

#[repr(align(64))]
struct Head<T> {
    current: AtomicPtr<Node<T>>,
    writer: AtomicBool,
    _pad: [u8; 55],
}

#[repr(align(64))]
pub struct Slot<T: State> {
    head: Head<T>,
    id: AtomId,
    frame: AtomicU32,
    nodes: UnsafeCell<Vec<Box<Node<T>>>>,
    free: UnsafeCell<Vec<*mut Node<T>>>,
    retired: UnsafeCell<Vec<Retired<T>>>,
}

unsafe impl<T: State> Sync for Slot<T> {}
unsafe impl<T: State> Send for Slot<T> {}

impl<T: State> Slot<T> {
    pub fn new(id: AtomId, initial: T) -> Self {
        let mut node = Box::new(Node {
            value: UnsafeCell::new(MaybeUninit::new(initial)),
            flags: AtomicU32::new(0),
        });
        let current = &mut *node as *mut Node<T>;
        Self {
            head: Head {
                current: AtomicPtr::new(current),
                writer: AtomicBool::new(false),
                _pad: [0; 55],
            },
            id,
            frame: AtomicU32::new(0),
            nodes: UnsafeCell::new(vec![node]),
            free: UnsafeCell::new(Vec::new()),
            retired: UnsafeCell::new(Vec::new()),
        }
    }

    pub fn id(&self) -> AtomId {
        self.id
    }

    #[inline(always)]
    fn lock_writer(&self) {
        while self
            .head
            .writer
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            std::hint::spin_loop();
        }
    }

    #[inline(always)]
    fn unlock_writer(&self) {
        self.head.writer.store(false, Ordering::Release);
    }

    #[inline(always)]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        deps::record(self.id);
        if scope::is_active() {
            let node = self.head.current.load(Ordering::Acquire);
            let val = unsafe { (*node).value.get().cast::<T>().as_ref().unwrap() };
            f(val)
        } else {
            let guard = self.acquire();
            f(&*guard)
        }
    }

    #[inline(always)]
    pub fn borrow(&self) -> Guard<'_, T> {
        deps::record(self.id);
        self.acquire()
    }

    #[inline(always)]
    fn acquire(&self) -> Guard<'_, T> {
        'retry: loop {
            let node = self.head.current.load(Ordering::Acquire);
            let readers = unsafe { &(*node).flags };
            let mut state = readers.load(Ordering::Acquire);
            loop {
                if state & WRITING != 0 || state & READERS == READERS {
                    continue 'retry;
                }
                match readers.compare_exchange_weak(
                    state,
                    state + 1,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => break,
                    Err(next) => state = next,
                }
            }

            if self.head.current.load(Ordering::Acquire) == node {
                return Guard {
                    _marker: std::marker::PhantomData,
                    node,
                };
            }
            readers.fetch_sub(1, Ordering::Release);
        }
    }

    #[inline(always)]
    pub fn read(&self) -> T {
        self.with(|v| v.clone())
    }

    #[inline(always)]
    pub fn peek(&self) -> T {
        if scope::is_active() {
            let node = self.head.current.load(Ordering::Acquire);
            unsafe { (*(*node).value.get().cast::<T>()).clone() }
        } else {
            let guard = self.acquire();
            (*guard).clone()
        }
    }

    #[inline(always)]
    pub fn set(&self, val: T) {
        self.lock_writer();
        let current = self.head.current.load(Ordering::Acquire);
        let readers = unsafe { &(*current).flags };

        let current_val = unsafe { (*current).value.get().cast::<T>().as_ref().unwrap() };
        if *current_val == val {
            self.unlock_writer();
            return;
        }

        if readers.load(Ordering::Acquire) == 0
            && !scope::is_active()
            && readers
                .compare_exchange(0, WRITING, Ordering::SeqCst, Ordering::Acquire)
                .is_ok()
        {
            if !scope::is_active() {
                unsafe {
                    let target = (*current).value.get().cast::<T>().as_mut().unwrap();
                    *target = val;
                }
                readers.store(0, Ordering::Release);
                self.unlock_writer();
                self.mark_dirty();
                return;
            }
            readers.store(0, Ordering::Release);
        }

        let next = self.prepare_with(|| val);
        self.publish(next);
        self.mark_dirty();
        self.unlock_writer();
    }

    #[inline(always)]
    pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.lock_writer();
        let current = self.head.current.load(Ordering::Acquire);
        let readers = unsafe { &(*current).flags };

        let claimed = readers.load(Ordering::Acquire) == 0
            && !scope::is_active()
            && readers
                .compare_exchange(0, WRITING, Ordering::SeqCst, Ordering::Acquire)
                .is_ok();
        if claimed && !scope::is_active() {
            let before = unsafe { (*current).value.get().cast::<T>().as_ref().unwrap().clone() };
            let result = unsafe { f((*current).value.get().cast::<T>().as_mut().unwrap()) };
            let after = unsafe { (*current).value.get().cast::<T>().as_ref().unwrap() };
            let changed = *after != before;
            readers.store(0, Ordering::Release);
            self.unlock_writer();
            if changed {
                self.mark_dirty();
            }
            return result;
        }
        if claimed {
            readers.store(0, Ordering::Release);
        }

        let old_ref = unsafe { (*current).value.get().cast::<T>().as_ref().unwrap() };
        let before = old_ref.clone();
        let next = self.prepare_with(|| old_ref.clone());
        let res = unsafe { f((*next).value.get().cast::<T>().as_mut().unwrap()) };
        let after = unsafe { (*next).value.get().cast::<T>().as_ref().unwrap() };
        let changed = *after != before;
        self.publish(next);
        self.unlock_writer();
        if changed {
            self.mark_dirty();
        }

        res
    }

    #[inline(always)]
    fn mark_dirty(&self) {
        let current = CURRENT_FRAME_EPOCH.load(Ordering::Relaxed);
        if self.frame.swap(current, Ordering::Relaxed) != current {
            dirty_queue().mark(self.id);
        }
    }

    fn publish(&self, next: *mut Node<T>) {
        let old = self.head.current.swap(next, Ordering::AcqRel);
        let ret_epoch = scope::bump_epoch();
        unsafe {
            (&mut *self.retired.get()).push(Retired {
                node: old,
                epoch: ret_epoch,
            });
        }
    }

    fn prepare_with(&self, make: impl FnOnce() -> T) -> *mut Node<T> {
        if unsafe { (*self.free.get()).is_empty() } {
            self.try_reclaim();
        }
        let free = unsafe { &mut *self.free.get() };
        if let Some(node) = free.pop() {
            unsafe {
                (*(*node).value.get()).write(make());
                (*node).flags.store(0, Ordering::Release);
            }
            node
        } else {
            let mut node = Box::new(Node {
                value: UnsafeCell::new(MaybeUninit::new(make())),
                flags: AtomicU32::new(0),
            });
            let pointer = &mut *node as *mut Node<T>;
            unsafe { (&mut *self.nodes.get()).push(node) };
            pointer
        }
    }

    fn try_reclaim(&self) {
        let retired = unsafe { &mut *self.retired.get() };
        if retired.is_empty() {
            return;
        }
        let free = unsafe { &mut *self.free.get() };
        let mut index = 0;
        while index < retired.len() {
            let item = &retired[index];
            let readers = unsafe { (*item.node).flags.load(Ordering::Acquire) & READERS };
            if readers == 0 && scope::reclaimable(item.epoch) {
                let item = retired.swap_remove(index);
                unsafe {
                    ptr::drop_in_place((*item.node).value.get().cast::<T>());
                    (*item.node).flags.store(DEAD, Ordering::Relaxed)
                };
                free.push(item.node);
            } else {
                index += 1;
            }
        }
    }
}

pub struct Guard<'a, T: State> {
    _marker: PhantomData<&'a Slot<T>>,
    node: *mut Node<T>,
}

impl<T: State> Deref for Guard<'_, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { (*self.node).value.get().cast::<T>().as_ref().unwrap() }
    }
}

impl<T: State> Drop for Guard<'_, T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { (*self.node).flags.fetch_sub(1, Ordering::Release) };
    }
}

impl<T: State> Drop for Slot<T> {
    fn drop(&mut self) {
        for node in unsafe { &mut *self.nodes.get() } {
            if node.flags.load(Ordering::Relaxed) != DEAD {
                unsafe { ptr::drop_in_place(node.value.get().cast::<T>()) };
            }
        }
    }
}
