use std::{
    cell::{Cell, UnsafeCell},
    mem,
    mem::ManuallyDrop,
    rc::Rc,
};

use super::{LocalState, deps, id::AtomId};

pub(crate) struct LocalQueue {
    epoch: Cell<u64>,
    pending: UnsafeCell<Vec<AtomId>>,
}

impl LocalQueue {
    pub(crate) fn new() -> Self {
        Self {
            epoch: Cell::new(1),
            pending: UnsafeCell::new(Vec::new()),
        }
    }

    #[inline(always)]
    fn mark(&self, id: AtomId, queued_epoch: &Cell<u64>) {
        let epoch = self.epoch.get();
        if queued_epoch.get() == epoch {
            return;
        }
        queued_epoch.set(epoch);
        unsafe { (&mut *self.pending.get()).push(id) };
    }

    pub(crate) fn drain_into(&self, output: &mut Vec<AtomId>) {
        output.append(unsafe { &mut *self.pending.get() });
        self.epoch.set(self.epoch.get().wrapping_add(1));
    }
}

thread_local! {
    static ACTIVE_QUEUE: UnsafeCell<Option<Rc<LocalQueue>>> = const { UnsafeCell::new(None) };
}

pub(crate) struct QueueScope {
    previous: Option<Rc<LocalQueue>>,
}

pub(crate) fn enter(queue: &Rc<LocalQueue>) -> QueueScope {
    let previous = ACTIVE_QUEUE
        .with(|active| unsafe { mem::replace(&mut *active.get(), Some(queue.clone())) });
    QueueScope { previous }
}

impl Drop for QueueScope {
    fn drop(&mut self) {
        ACTIVE_QUEUE.with(|active| unsafe {
            *active.get() = self.previous.take();
        });
    }
}

fn active_queue() -> Rc<LocalQueue> {
    ACTIVE_QUEUE.with(|active| unsafe {
        (*active.get())
            .as_ref()
            .cloned()
            .expect("signals must be created inside a runtime")
    })
}

struct SignalInner<T: LocalState> {
    id: AtomId,
    value: UnsafeCell<T>,
    readers: Cell<u32>,
    writing: Cell<bool>,
    queued_epoch: Cell<u64>,
    queue: Rc<LocalQueue>,
}

struct ReadGuard<'a> {
    readers: &'a Cell<u32>,
}

impl Drop for ReadGuard<'_> {
    fn drop(&mut self) {
        self.readers.set(self.readers.get() - 1);
    }
}

struct WriteGuard<'a> {
    writing: &'a Cell<bool>,
}

struct UpdateGuard<'a, T> {
    write: Option<WriteGuard<'a>>,
    before: ManuallyDrop<T>,
}

impl<T> Drop for UpdateGuard<'_, T> {
    fn drop(&mut self) {
        drop(self.write.take());
        unsafe { ManuallyDrop::drop(&mut self.before) };
    }
}

impl Drop for WriteGuard<'_> {
    fn drop(&mut self) {
        self.writing.set(false);
    }
}

#[derive(Clone)]
pub struct Signal<T: LocalState> {
    inner: Rc<SignalInner<T>>,
}

impl<T: LocalState> Signal<T> {
    pub(crate) fn new(value: T) -> Self {
        Self::new_in(active_queue(), value)
    }

    pub(crate) fn new_in(queue: Rc<LocalQueue>, value: T) -> Self {
        Self {
            inner: Rc::new(SignalInner {
                id: AtomId::next(),
                value: UnsafeCell::new(value),
                readers: Cell::new(0),
                writing: Cell::new(false),
                queued_epoch: Cell::new(0),
                queue,
            }),
        }
    }

    pub fn id(&self) -> AtomId {
        self.inner.id
    }

    #[inline(always)]
    fn read_guard(&self) -> ReadGuard<'_> {
        if self.inner.writing.get() {
            panic!("signal is being written");
        }
        let readers = self.inner.readers.get();
        let Some(next) = readers.checked_add(1) else {
            panic!("signal has too many readers");
        };
        self.inner.readers.set(next);
        ReadGuard {
            readers: &self.inner.readers,
        }
    }

    #[inline(always)]
    fn write_guard(&self) -> WriteGuard<'_> {
        if self.inner.readers.get() != 0 || self.inner.writing.replace(true) {
            panic!("signal is borrowed");
        }
        WriteGuard {
            writing: &self.inner.writing,
        }
    }

    #[inline(always)]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        deps::record(self.id());
        let _guard = self.read_guard();
        f(unsafe { &*self.inner.value.get() })
    }

    #[inline(always)]
    pub fn read(&self) -> T {
        self.with(Clone::clone)
    }

    #[inline(always)]
    pub fn peek(&self) -> T {
        let _guard = self.read_guard();
        unsafe { (&*self.inner.value.get()).clone() }
    }

    #[inline(always)]
    pub fn set(&self, value: T) {
        let guard = self.write_guard();
        let current = unsafe { &mut *self.inner.value.get() };
        if *current == value {
            return;
        }
        let previous = mem::replace(current, value);
        self.inner.queue.mark(self.id(), &self.inner.queued_epoch);
        drop(guard);
        drop(previous);
    }

    #[inline(always)]
    pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let guard = self.write_guard();
        let current = unsafe { &mut *self.inner.value.get() };
        let guard = UpdateGuard {
            write: Some(guard),
            before: ManuallyDrop::new(current.clone()),
        };
        let result = f(current);
        if *current != *guard.before {
            self.inner.queue.mark(self.id(), &self.inner.queued_epoch);
        }
        drop(guard);
        result
    }
}

impl<T: LocalState> PartialEq for Signal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T: LocalState> Eq for Signal<T> {}
