use std::{
    mem,
    sync::atomic::Ordering,
    sync::{Mutex, OnceLock},
};

use crate::state::{id::AtomId, slot::CURRENT_FRAME_EPOCH};

static DIRTY_QUEUE: OnceLock<DirtyQueue> = OnceLock::new();

pub struct DirtyQueue {
    pending: Mutex<Vec<AtomId>>,
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
        }
    }

    #[inline(always)]
    pub fn mark(&self, id: AtomId) {
        self.pending
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(id);
    }

    pub fn drain(&self) -> Vec<AtomId> {
        let mut queue = self
            .pending
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let drained = mem::take(&mut *queue);
        CURRENT_FRAME_EPOCH.fetch_add(1, Ordering::Relaxed);
        drained
    }

    pub fn len(&self) -> usize {
        self.pending
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .is_empty()
    }
}

pub fn dirty_queue() -> &'static DirtyQueue {
    DIRTY_QUEUE.get_or_init(DirtyQueue::new)
}
