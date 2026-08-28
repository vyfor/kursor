use std::time::Duration;

use animate::{Activity, Time};
use kursor_core::layout::Offset;

use crate::{EffectCx, Fx};

#[derive(Clone)]
pub struct Translate {
    offset: Offset,
    duration: Duration,
    start: Option<Time>,
}

pub fn translate(offset: Offset, duration: Duration) -> Translate {
    Translate {
        offset,
        duration,
        start: None,
    }
}

impl Fx for Translate {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity {
        let start = *self.start.get_or_insert(cx.time);
        let progress = cx.progress(start, self.duration);
        let dx = (self.offset.x as f32 * progress).round() as i32;
        let dy = (self.offset.y as f32 * progress).round() as i32;

        cx.layer.clear();
        for y in 0..cx.layer.height() {
            for x in 0..cx.layer.width() {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0
                    && ny >= 0
                    && nx < cx.layer.width() as i32
                    && ny < cx.layer.height() as i32
                {
                    cx.set(nx as u16, ny as u16, cx.source(x, y));
                }
            }
        }

        if progress >= 1.0 { Activity::FINISHED } else { Activity::RUNNING }
    }

    fn reset(&mut self) {
        self.start = None;
    }
}
