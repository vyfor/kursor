use std::{
    hint::spin_loop,
    sync::atomic::{AtomicBool, Ordering},
    thread::yield_now,
};

pub struct SpinLock(AtomicBool);

impl Default for SpinLock {
    fn default() -> Self {
        Self::new()
    }
}

impl SpinLock {
    pub const fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    #[inline(always)]
    pub fn lock(&self) {
        let mut spins = 0u32;
        loop {
            if self
                .0
                .compare_exchange_weak(
                    false,
                    true,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                return;
            }
            while self.0.load(Ordering::Relaxed) {
                if spins < 50 {
                    spins += 1;
                    spin_loop();
                } else {
                    spins = 0;
                    yield_now();
                }
            }
        }
    }

    #[inline(always)]
    pub fn unlock(&self) {
        self.0.store(false, Ordering::Release);
    }
}
