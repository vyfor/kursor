mod clock;
mod driver;

pub use clock::ClockState;
pub use driver::{Driven, DriverState};

use std::time::Duration;

use animate::Time;

#[derive(Clone, Copy)]
pub struct Advance {
    pub value: f32,
    pub settled: bool,
}

#[derive(Clone)]
pub enum Progress {
    Clock(ClockState),
    Driven(DriverState),
}

impl Progress {
    pub fn clock(duration: Duration) -> Self {
        Self::Clock(ClockState {
            start: None,
            duration,
        })
    }

    pub fn driven(source: &Driven) -> Self {
        Self::Driven(DriverState {
            driver: source.transition.build(source.initial),
            target: source.target.clone(),
            current: source.current.clone(),
        })
    }

    pub fn advance(&mut self, time: Time) -> Advance {
        match self {
            Self::Clock(state) => state.advance(time),
            Self::Driven(state) => state.advance(time),
        }
    }

    pub fn reset(&mut self) {
        if let Self::Clock(state) = self {
            state.reset();
        }
    }
}
