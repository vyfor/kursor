use std::{cell::UnsafeCell, rc::Rc};

use animate::Activity;

use crate::{EffectCx, Fx};

#[derive(Clone, Default)]
pub struct FxCell {
    inner: Rc<UnsafeCell<Inner>>,
}

#[derive(Default)]
struct Inner {
    fx: Option<Box<dyn Fx>>,
    finished: bool,
}

impl FxCell {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_of(fx: impl Fx) -> Self {
        let cell = Self::default();
        cell.set(fx);
        cell
    }

    pub fn set(&self, fx: impl Fx) {
        let inner = unsafe { &mut *self.inner.get() };
        inner.fx = Some(Box::new(fx));
        inner.finished = false;
    }

    pub fn finished(&self) -> bool {
        unsafe { (*self.inner.get()).finished }
    }
}

impl PartialEq for FxCell {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Fx for FxCell {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity {
        let inner = unsafe { &mut *self.inner.get() };
        let Some(mut fx) = inner.fx.take() else {
            return Activity::NONE;
        };

        let activity = fx.apply(cx);
        match activity {
            Activity::FINISHED => {
                inner.fx = None;
                inner.finished = true;
                Activity::FINISHED
            }
            _ => {
                inner.fx = Some(fx);
                activity
            }
        }
    }

    fn reset(&mut self) {
        let inner = unsafe { &mut *self.inner.get() };
        if let Some(fx) = &mut inner.fx {
            fx.reset();
        }
        inner.finished = false;
    }

    fn is_finished(&self) -> bool {
        self.finished()
    }
}
