use std::{
    future::Future,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use kursor_core::{
    component::context::Cx,
    state::{AtomId, SharedState, deps, dirty_queue},
};

use crate::executor::TaskExecutor;

/// handle to an asynchronous task.
#[derive(Clone)]
pub struct Task<T: SharedState> {
    id: AtomId,
    value: Arc<OnceLock<T>>,
    cancelled: Arc<AtomicBool>,
    sentinel: Arc<()>,
}

impl<T: SharedState> Task<T> {
    /// reads the current value, reactively subscribing the calling component.
    ///
    /// returns `None` while the task is running, or `Some(value)`
    /// once the task has completed.
    pub fn read(&self) -> Option<T> {
        deps::record(self.id);
        self.value.get().cloned()
    }

    /// peeks at the current value without subscribing.
    pub fn peek(&self) -> Option<T> {
        self.value.get().cloned()
    }

    pub fn id(&self) -> AtomId {
        self.id
    }

    /// cancels the task.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// whether the task has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    /// whether the task has been completed and has a value available.
    pub fn is_ready(&self) -> bool {
        self.value.get().is_some()
    }
}

impl<T: SharedState> Drop for Task<T> {
    fn drop(&mut self) {
        if Arc::strong_count(&self.sentinel) == 1 {
            self.cancelled.store(true, Ordering::Release);
        }
    }
}

pub trait CxAsyncExt {
    /// spawns a future returning a reactive [`Task`] that wakes and updates the
    /// component when complete.
    fn task<T, F>(&mut self, future: F) -> Task<T>
    where
        T: SharedState,
        F: Future<Output = T> + Send + 'static;

    /// spawns a fire and forget future.
    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static;
}

impl CxAsyncExt for Cx<'_> {
    fn task<T, F>(&mut self, future: F) -> Task<T>
    where
        T: SharedState,
        F: Future<Output = T> + Send + 'static,
    {
        let executor = self
            .get::<TaskExecutor>()
            .expect("no executor was configured")
            .clone();

        let id = AtomId::next();
        let value = Arc::new(OnceLock::new());
        let cancelled = Arc::new(AtomicBool::new(false));

        let task = Task {
            id,
            value: value.clone(),
            cancelled: cancelled.clone(),
            sentinel: Arc::new(()),
        };

        let w_val = value;
        let w_cancelled = cancelled;

        executor.spawn(async move {
            let result = future.await;
            if !w_cancelled.load(Ordering::Acquire) {
                let _ = w_val.set(result);
                dirty_queue().mark(id);
            }
        });

        task
    }

    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let executor = self
            .get::<TaskExecutor>()
            .expect("no executor was configured")
            .clone();

        executor.spawn(future);
    }
}
