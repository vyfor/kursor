use animate::Activity;

use crate::{EffectCx, Fx};

/// runs multiple effects concurrently.
pub fn par() -> Parallel {
    Parallel {
        effects: Vec::new(),
    }
}

/// runs multiple effects concurrently.
pub struct Parallel {
    effects: Vec<Box<dyn Fx>>,
}

impl Parallel {
    pub fn with(mut self, effect: impl Fx) -> Self {
        self.effects.push(Box::new(effect));
        self
    }
}

impl Clone for Parallel {
    fn clone(&self) -> Self {
        Self {
            effects: self.effects.clone(),
        }
    }
}

impl Fx for Parallel {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity {
        let mut running = false;
        let mut changed = false;

        for effect in &mut self.effects {
            let activity = effect.apply(cx);
            running |= activity.running;
            changed |= activity.changed || activity.finished;
        }

        if running {
            Activity::RUNNING
        } else if changed {
            Activity::FINISHED
        } else {
            Activity::NONE
        }
    }

    fn reset(&mut self) {
        for effect in &mut self.effects {
            effect.reset();
        }
    }
}
