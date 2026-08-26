use std::{
    cell::Cell,
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
};

const IDLE: u64 = u64::MAX;
pub(crate) const MAX_PINS: usize = 8;

struct Domain {
    epoch: AtomicU64,
    active: AtomicU32,
    pinned: [AtomicU64; MAX_PINS],
}

static DOMAIN: Domain = Domain::new();

impl Domain {
    const fn new() -> Self {
        Self {
            epoch: AtomicU64::new(0),
            active: AtomicU32::new(0),
            pinned: [const { AtomicU64::new(IDLE) }; MAX_PINS],
        }
    }
}

thread_local! {
    static PINNED: Cell<usize> = const { Cell::new(usize::MAX) };
    static NESTING: Cell<u32> = const { Cell::new(0) };
}

#[inline(always)]
pub(crate) fn is_active() -> bool {
    PINNED.with(|p| p.get() != usize::MAX)
}

#[inline(always)]
pub fn with<R>(f: impl FnOnce() -> R) -> R {
    let _scope = enter();
    f()
}

pub(crate) fn enter() -> Scope {
    let nesting = NESTING.with(|n| {
        let v = n.get();
        n.set(v + 1);
        v
    });

    if nesting == 0 {
        let participant = PINNED.with(|p| {
            let idx = p.get();
            if idx != usize::MAX {
                return Some(idx);
            }
            let epoch = DOMAIN.epoch.load(Ordering::Relaxed);
            for i in 0..MAX_PINS {
                if DOMAIN.pinned[i]
                    .compare_exchange(IDLE, epoch, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
                {
                    p.set(i);
                    return Some(i);
                }
            }
            None
        });

        if let Some(p) = participant {
            let epoch = DOMAIN.epoch.load(Ordering::Relaxed);
            DOMAIN.pinned[p].store(epoch, Ordering::Release);
            DOMAIN.active.fetch_add(1, Ordering::SeqCst);

            Scope {
                participant: Some(p),
                first: true,
            }
        } else {
            Scope {
                participant: None,
                first: true,
            }
        }
    } else {
        let participant = PINNED.with(|p| {
            let idx = p.get();
            if idx != usize::MAX { Some(idx) } else { None }
        });
        Scope {
            participant,
            first: false,
        }
    }
}

// todo: possibly rename deps::Scope to avoid confusion
pub struct Scope {
    participant: Option<usize>,
    first: bool,
}

impl Drop for Scope {
    fn drop(&mut self) {
        let nesting = NESTING.with(|n| {
            let v = n.get() - 1;
            n.set(v);
            v
        });
        if nesting == 0
            && self.first
            && let Some(participant) = self.participant
        {
            DOMAIN.pinned[participant].store(IDLE, Ordering::Release);
            DOMAIN.active.fetch_sub(1, Ordering::SeqCst);
            PINNED.with(|p| p.set(usize::MAX));
        }
    }
}

#[inline(always)]
pub(crate) fn bump_epoch() -> u64 {
    if DOMAIN.active.load(Ordering::SeqCst) != 0 {
        DOMAIN.epoch.fetch_add(1, Ordering::SeqCst)
    } else {
        DOMAIN.epoch.load(Ordering::Relaxed)
    }
}

pub(crate) fn reclaimable(retired: u64) -> bool {
    DOMAIN.pinned.iter().all(|p| {
        let active = p.load(Ordering::SeqCst);
        active == IDLE || active > retired
    })
}
