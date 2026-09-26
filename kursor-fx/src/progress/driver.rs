use std::{cell::Cell, rc::Rc, time::Duration};

use animate::{Driver, Time, Transition};

use super::Advance;

/// state-driven progress.
#[derive(Clone)]
pub struct DriverState {
    pub(crate) driver: Driver<f32>,
    pub(crate) target: Rc<Cell<f32>>,
    pub(crate) current: Rc<Cell<f32>>,
}

impl DriverState {
    pub fn advance(&mut self, time: Time) -> Advance {
        let target = self.target.get();
        if target != *self.driver.target() {
            self.driver.to(target);
        }

        let activity = self.driver.advance(time);
        let value = *self.driver.value();
        self.current.set(value);

        Advance {
            value,
            settled: !activity.running,
        }
    }
}

/// state-driven progress.
#[derive(Clone)]
pub struct Driven {
    pub(crate) transition: Transition,
    pub(crate) initial: f32,
    pub(crate) target: Rc<Cell<f32>>,
    pub(crate) current: Rc<Cell<f32>>,
}

impl Driven {
    pub fn new(transition: Transition, initial: f32) -> Self {
        Self {
            transition,
            initial,
            target: Rc::new(Cell::new(initial)),
            current: Rc::new(Cell::new(initial)),
        }
    }

    pub fn tween(initial: f32, duration: Duration) -> Self {
        Self::new(Transition::Tween(Transition::tween(duration)), initial)
    }

    pub fn spring(initial: f32) -> Self {
        Self::new(Transition::Spring(Transition::spring()), initial)
    }

    pub fn target(&self) -> f32 {
        self.target.get()
    }

    pub fn set_target(&self, target: f32) {
        self.target.set(target);
    }

    pub fn value(&self) -> f32 {
        self.current.get()
    }
}
