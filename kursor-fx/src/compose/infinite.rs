use std::time::Duration;

use animate::{Activity, Time};

use crate::{EffectCx, Fx};

/// repeats an effect indefinitely.
///
/// to infinity and beyond!
pub fn infinite(effect: impl Fx) -> Infinite {
    Infinite {
        effect: Box::new(effect),
        delay: Duration::ZERO,
        waiting_since: None,
    }
}

/// repeats an effect indefinitely.
///
/// to infinity and beyond!
pub struct Infinite {
    effect: Box<dyn Fx>,
    delay: Duration,
    waiting_since: Option<Time>,
}

impl Infinite {
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }
}

impl Clone for Infinite {
    fn clone(&self) -> Self {
        Self {
            effect: self.effect.clone(),
            delay: self.delay,
            waiting_since: self.waiting_since,
        }
    }
}

impl Fx for Infinite {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity {
        if let Some(waiting_since) = self.waiting_since {
            if cx.time.elapsed.saturating_sub(waiting_since.elapsed)
                < self.delay
            {
                self.effect.apply(cx);
                return Activity::RUNNING;
            }
            self.waiting_since = None;
            self.effect.reset();
        }

        if self.effect.apply(cx).finished {
            self.waiting_since = Some(cx.time);
        }

        Activity::RUNNING
    }

    fn reset(&mut self) {
        self.waiting_since = None;
        self.effect.reset();
    }
}
