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
    pub fn then(mut self, effect: impl Fx) -> Self {
        self.effects.push(Box::new(effect));
        self
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
        while self.index < len {
            let activity = self.effects[self.index].apply(cx);
            if !activity.finished {
                return activity;
            }

            self.index += 1;
            if self.index < len {
                cx.layer.advance();
            }
        }

        if len > 0 {
            self.effects[len - 1].apply(cx);
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
