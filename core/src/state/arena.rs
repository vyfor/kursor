use std::{
    any::Any,
    sync::atomic::{AtomicUsize, Ordering},
    sync::{OnceLock, RwLock},
};

use crate::state::{SharedState, id::AtomId, slot::Slot};

type Erased = Box<dyn Any + Send + Sync>;

struct Chunk {
    slots: Vec<Option<Erased>>,
}

pub struct AtomArena {
    #[allow(clippy::vec_box)]
    chunks: RwLock<Vec<Box<Chunk>>>,
    len: AtomicUsize,
}

static ARENA: OnceLock<AtomArena> = OnceLock::new();

pub fn arena() -> &'static AtomArena {
    ARENA.get_or_init(|| AtomArena {
        chunks: RwLock::new(Vec::new()),
        len: AtomicUsize::new(0),
    })
}

const CHUNK_SIZE: usize = 64;

impl AtomArena {
    pub fn insert<T: SharedState>(&self, id: AtomId, initial: T) -> u32 {
        let slot: Box<Slot<T>> = Box::new(Slot::new(id, initial));
        let raw = Box::into_raw(slot);

        let mut chunks = self.chunks.write().unwrap_or_else(|e| e.into_inner());

        let idx = self.len.fetch_add(1, Ordering::Relaxed);
        let chunk_idx = idx / CHUNK_SIZE;
        let slot_idx = idx % CHUNK_SIZE;

        if chunk_idx >= chunks.len() {
            chunks.push(Box::new(Chunk {
                slots: (0..CHUNK_SIZE).map(|_| None).collect(),
            }));
        }

        let erased: Erased = unsafe { Box::from_raw(raw) };
        chunks[chunk_idx].slots[slot_idx] = Some(erased);

        idx as u32
    }

    #[inline(always)]
    pub fn get<T: SharedState>(&self, index: u32) -> &Slot<T> {
        let chunks = self.chunks.read().unwrap_or_else(|e| e.into_inner());
        let chunk_idx = (index as usize) / CHUNK_SIZE;
        let slot_idx = (index as usize) % CHUNK_SIZE;

        let chunk = &chunks[chunk_idx];
        let erased = chunk.slots[slot_idx].as_ref().unwrap();
        let ptr = erased.as_ref() as *const dyn Any;
        let typed = ptr as *const Slot<T>;
        unsafe { &*typed }
    }

    #[inline(always)]
    pub fn get_ptr<T: SharedState>(&self, index: u32) -> *const Slot<T> {
        let chunks = self.chunks.read().unwrap_or_else(|e| e.into_inner());
        let chunk_idx = (index as usize) / CHUNK_SIZE;
        let slot_idx = (index as usize) % CHUNK_SIZE;

        let chunk = &chunks[chunk_idx];
        let erased = chunk.slots[slot_idx].as_ref().unwrap();
        let ptr = erased.as_ref() as *const dyn Any;
        ptr as *const Slot<T>
    }
}
