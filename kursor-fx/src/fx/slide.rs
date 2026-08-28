use std::time::Duration;

use animate::{Activity, Time};

use crate::{Direction, EffectCx, Fx};

#[derive(Clone)]
pub struct Slide {
    direction: Direction,
    duration: Duration,
    inward: bool,
    start: Option<Time>,
}

pub fn slide_in(direction: Direction, duration: Duration) -> Slide {
    Slide {
        direction,
        duration,
        inward: true,
        start: None,
    }
}

pub fn slide_out(direction: Direction, duration: Duration) -> Slide {
    Slide {
        direction,
        duration,
        inward: false,
        start: None,
    }
}

impl Fx for Slide {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity {
        let start = *self.start.get_or_insert(cx.time);
        let progress = cx.progress(start, self.duration);
        let amount = if self.inward { 1.0 - progress } else { progress };
        let distance = match self.direction {
            Direction::Left | Direction::Right => cx.layer.width(),
            Direction::Up | Direction::Down => cx.layer.height(),
        } as f32;
        let distance = (distance * amount).round() as i32;
        let (dx, dy) = match self.direction {
            Direction::Left => (-distance, 0),
            Direction::Right => (distance, 0),
            Direction::Up => (0, -distance),
            Direction::Down => (0, distance),
        };

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
