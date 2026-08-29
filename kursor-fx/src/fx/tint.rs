use std::time::Duration;

use animate::{Activity, Time};
use kursor_core::render::{color::Color, style::Style};

use super::color::mix;
use crate::{EffectCx, Fx, Mask, Spread};

#[derive(Clone)]
pub struct Tint {
    fg: Option<Color>,
    bg: Option<Color>,
    duration: Duration,
    start: Option<Time>,
    mask: Mask,
    spread: Spread,
}

pub fn tint_fg(color: Color, duration: Duration) -> Tint {
    Tint {
        fg: Some(color),
        bg: None,
        duration,
        start: None,
        mask: Mask::all(),
        spread: Spread::uniform(),
    }
}

pub fn tint_bg(color: Color, duration: Duration) -> Tint {
    Tint {
        fg: None,
        bg: Some(color),
        duration,
        start: None,
        mask: Mask::all(),
        spread: Spread::uniform(),
    }
}

pub fn tint_style(style: Style, duration: Duration) -> Tint {
    Tint {
        fg: Some(style.fg),
        bg: Some(style.bg),
        duration,
        start: None,
        mask: Mask::all(),
        spread: Spread::uniform(),
    }
}

impl Tint {
    pub fn mask(mut self, mask: Mask) -> Self {
        self.mask = mask;
        self
    }

    pub fn spread(mut self, spread: Spread) -> Self {
        self.spread = spread;
        self
    }
}

impl Fx for Tint {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity {
        let start = *self.start.get_or_insert(cx.time);
        let progress = cx.progress(start, self.duration);

        for y in 0..cx.layer.height() {
            for x in 0..cx.layer.width() {
                if !cx.includes(&self.mask, x, y) {
                    continue;
                }

                let p = cx.spread(self.spread, progress, x, y);
                let mut cell = cx.source(x, y);
                if let Some(fg) = self.fg {
                    cell.style.fg = mix(cell.style.fg, fg, p, Color::White);
                }
                if let Some(bg) = self.bg {
                    cell.style.bg = mix(cell.style.bg, bg, p, Color::Black);
                }
                cx.set(x, y, cell);
            }
        }

        if progress >= 1.0 {
            Activity::FINISHED
        } else {
            Activity::RUNNING
        }
    }

    fn reset(&mut self) {
        self.start = None;
    }
}
