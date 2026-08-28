use animate::Activity;

use crate::{EffectCx, Fx};

pub fn seq() -> Sequence {
    Sequence {
        effects: Vec::new(),
        index: 0,
    }
}

pub struct Sequence {
    effects: Vec<Box<dyn Fx>>,
    index: usize,
}

impl Sequence {
    pub fn with(mut self, effect: impl Fx) -> Self {
        self.effects.push(Box::new(effect));
        self
    }

    pub fn then(self, effect: impl Fx) -> Self {
        self.with(effect)
    }
}

impl Clone for Sequence {
    fn clone(&self) -> Self {
        Self {
            effects: self.effects.clone(),
            index: self.index,
        }
    }
}

impl Fx for Sequence {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity {
        while self.index < self.effects.len() {
            let activity = self.effects[self.index].apply(cx);
            if activity.finished {
                self.index += 1;
                continue;
            }
            return activity;
        }

        Activity::FINISHED
    }

    fn reset(&mut self) {
        self.index = 0;
        for effect in &mut self.effects {
            effect.reset();
        }
    }
}
