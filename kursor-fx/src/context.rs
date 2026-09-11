use std::time::Duration;

use animate::{Activity, Time};
use kursor_core::{layout::rect::Rect, render::cell::Cell};

use crate::{EffectLayer, Mask, Spread};

pub struct EffectCx<'a> {
    pub time: Time,
    pub area: Rect,
    pub layer: &'a mut EffectLayer,
}

impl EffectCx<'_> {
    pub fn progress(&self, start: Time, duration: Duration) -> f32 {
        let elapsed = self.time.elapsed.saturating_sub(start.elapsed);
        let duration = duration.max(Duration::from_millis(1));

        (elapsed.as_secs_f32() / duration.as_secs_f32()).min(1.0)
    }

    pub fn source(&self, x: u16, y: u16) -> Cell {
        self.layer.source(x, y)
    }

    pub fn underlay(&self, x: u16, y: u16) -> Cell {
        self.layer.underlay(x, y)
    }

    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        self.layer.set(x, y, cell);
    }

    pub fn includes(&self, mask: &Mask, x: u16, y: u16) -> bool {
        mask.includes(self.source(x, y), x, y, self.area)
    }

    pub fn spread(&self, spread: Spread, progress: f32, x: u16, y: u16) -> f32 {
        spread.progress(progress, x, y, self.area)
    }
}

/// visual post-processing "shader" for terminal cells.
pub trait Fx: FxClone + 'static {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity;

    fn reset(&mut self) {}
}

pub trait FxClone {
    fn clone_box(&self) -> Box<dyn Fx>;
}

impl<T> FxClone for T
where
    T: Fx + Clone,
{
    fn clone_box(&self) -> Box<dyn Fx> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Fx> {
    fn clone(&self) -> Self {
        self.as_ref().clone_box()
    }
}

impl Fx for Box<dyn Fx> {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity {
        (**self).apply(cx)
    }

    fn reset(&mut self) {
        (**self).reset();
    }
}
