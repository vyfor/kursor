use std::time::Duration;

use animate::Activity;
use kursor_core::{layout::size::Size, render::color::Color};

use super::color::mix;
use crate::{Advance, Driven, EffectCx, Fx, Ink, Mask, Progress, Spread};

pub fn colorize(ink: impl Ink + Clone) -> Colorize {
    Colorize {
        ink: Box::new(ink),
        target: Target::Both,
        mode: Mode::Replace,
        mask: Mask::all(),
        spread: Spread::uniform(),
    }
}

/// recolors cells.
#[derive(Clone)]
pub struct Colorize {
    ink: Box<dyn Ink>,
    target: Target,
    mode: Mode,
    mask: Mask,
    spread: Spread,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Target {
    Fg,
    Bg,
    Both,
}

#[derive(Clone)]
enum Mode {
    Replace,
    Animate,
    Blend { progress: Progress },
}

impl Colorize {
    pub fn fg(mut self) -> Self {
        self.target = Target::Fg;
        self
    }

    pub fn bg(mut self) -> Self {
        self.target = Target::Bg;
        self
    }

    pub fn both(mut self) -> Self {
        self.target = Target::Both;
        self
    }

    pub fn animate(mut self) -> Self {
        self.mode = Mode::Animate;
        self
    }

    pub fn blend(mut self, duration: Duration) -> Self {
        self.mode = Mode::Blend { progress: Progress::clock(duration) };
        self
    }

    pub fn driven(mut self, driven: &Driven) -> Self {
        self.mode = Mode::Blend { progress: Progress::driven(driven) };
        self
    }

    pub fn mask(mut self, mask: Mask) -> Self {
        self.mask = mask;
        self
    }

    pub fn spread(mut self, spread: Spread) -> Self {
        self.spread = spread;
        self
    }
}

impl Fx for Colorize {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity {
        let adv = match &mut self.mode {
            Mode::Blend { progress } => progress.advance(cx.time),
            _ => Advance { value: 1.0, settled: true },
        };
        let size = Size::new(cx.layer.width(), cx.layer.height());
        let time = cx.time;

        for y in 0..cx.layer.height() {
            for x in 0..cx.layer.width() {
                let cell = cx.source(x, y);
                if !cx.includes(&self.mask, x, y) {
                    continue;
                }

                let inked = self.ink.color(x, y, cell, size, time);
                let amount = match &self.mode {
                    Mode::Replace | Mode::Animate => 1.0,
                    Mode::Blend { .. } => {
                        cx.spread(self.spread, adv.value, x, y)
                    }
                };

                let mut cell = cell;
                if amount > 0.0 {
                    match self.target {
                        Target::Fg => {
                            cell.style.fg =
                                mix(cell.style.fg, inked, amount, Color::White);
                        }
                        Target::Bg => {
                            cell.style.bg =
                                mix(cell.style.bg, inked, amount, Color::Black);
                        }
                        Target::Both => {
                            cell.style.fg =
                                mix(cell.style.fg, inked, amount, Color::White);
                            cell.style.bg =
                                mix(cell.style.bg, inked, amount, Color::Black);
                        }
                    }
                }
                cx.set(x, y, cell);
            }
        }

        match &self.mode {
            Mode::Replace => Activity::FINISHED,
            Mode::Animate => Activity::RUNNING,
            Mode::Blend { .. } => {
                if adv.settled {
                    Activity::FINISHED
                } else {
                    Activity::RUNNING
                }
            }
        }
    }

    fn reset(&mut self) {
        if let Mode::Blend { progress } = &mut self.mode {
            progress.reset();
        }
    }
}
