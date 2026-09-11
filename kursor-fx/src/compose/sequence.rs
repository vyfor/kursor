use animate::Activity;

use crate::{EffectCx, Fx};

/// runs effects one after another.
pub fn seq() -> Sequence {
    Sequence {
        effects: Vec::new(),
        index: 0,
    }
}

/// runs effects one after another.
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
        let len = self.effects.len();
        for (index, effect) in self.effects.iter_mut().enumerate() {
            let activity = effect.apply(cx);
            if !activity.finished {
                self.index = index;
                return activity;
            }

            if index + 1 < len {
                cx.layer.advance();
            }
        }

        self.index = len;
        Activity::FINISHED
    }

    fn reset(&mut self) {
        self.index = 0;
        for effect in &mut self.effects {
            effect.reset();
        }
    }
}
