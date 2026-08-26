use std::{
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

    pub fn drain_into(&self, output: &mut Vec<AtomId>) {
        let mut queue = self
            .pending
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        output.append(&mut queue);
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
