use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, Mutex, OnceLock},
};

use crate::state::{id::AtomId, slot::CURRENT_FRAME_EPOCH};

pub type WakerFn = Arc<dyn Fn() + Send + Sync + 'static>;

static DIRTY_QUEUE: OnceLock<DirtyQueue> = OnceLock::new();

pub struct DirtyQueue {
    pending: Mutex<Vec<AtomId>>,
    waker: Mutex<Option<WakerFn>>,
    wake_pending: AtomicBool,
}

impl Default for DirtyQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl DirtyQueue {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(Vec::new()),
            waker: Mutex::new(None),
            wake_pending: AtomicBool::new(false),
        }
    }

    pub fn set_waker(&self, waker: Option<WakerFn>) {
        *self.waker.lock().unwrap_or_else(|error| error.into_inner()) = waker;
    }

    pub fn wake(&self) {
        let waker = self
            .waker
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();

        if let Some(waker) = waker {
            waker();
        }
    }

    #[inline(always)]
    pub fn mark(&self, id: AtomId) {
        self.pending
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(id);

        if !self.wake_pending.swap(true, Ordering::AcqRel) {
            self.wake();
        }
    }

    pub fn drain_into(&self, output: &mut Vec<AtomId>) {
        let mut queue = self
            .pending
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        output.append(&mut queue);
        self.wake_pending.store(false, Ordering::Release);
        CURRENT_FRAME_EPOCH.fetch_add(1, Ordering::Relaxed);
    }

    pub fn len(&self) -> usize {
        self.pending
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub fn dirty_queue() -> &'static DirtyQueue {
    DIRTY_QUEUE.get_or_init(DirtyQueue::new)
}
