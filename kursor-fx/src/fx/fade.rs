use std::time::Duration;

use animate::{Activity, Time};
use kursor_core::render::color::Color;

use super::color::mix;
use crate::{EffectCx, Fx, Mask, Spread};

#[derive(Clone)]
pub struct Fade {
    duration: Duration,
    mode: FadeMode,
    start: Option<Time>,
    mask: Mask,
    spread: Spread,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FadeMode {
    In,
    Out,
    From(Color),
    To(Color),
}

pub fn fade_in(duration: Duration) -> Fade {
    Fade::new(duration, FadeMode::In)
}

pub fn fade_out(duration: Duration) -> Fade {
    Fade::new(duration, FadeMode::Out)
}

pub fn fade_from(color: Color, duration: Duration) -> Fade {
    Fade::new(duration, FadeMode::From(color))
}

pub fn fade_to(color: Color, duration: Duration) -> Fade {
    Fade::new(duration, FadeMode::To(color))
}

impl Fade {
    fn new(duration: Duration, mode: FadeMode) -> Self {
        Self {
            duration,
            mode,
            start: None,
            mask: Mask::all(),
            spread: Spread::uniform(),
        }
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

impl Fx for Fade {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity {
        let start = *self.start.get_or_insert(cx.time);
        let progress = cx.progress(start, self.duration);

        for y in 0..cx.layer.height() {
            for x in 0..cx.layer.width() {
                let source = cx.source(x, y);
                let underlay = cx.underlay(x, y);
                let amount = cx.spread(self.spread, progress, x, y);

                if !cx.includes(&self.mask, x, y) {
                    continue;
                }

                let cell = match self.mode {
                    FadeMode::In => {
                        if amount <= 0.0 {
                            underlay
                        } else if amount >= 1.0 {
                            source
                        } else {
                            let fg_from = match underlay.style.bg {
                                Color::Reset => Color::Black,
                                other => other,
                            };
                            let fg_to = source.style.fg;
                            let bg_from = underlay.style.bg;
                            let bg_to = source.style.bg;

                            let mut cell = source;
                            cell.style.fg = mix(fg_from, fg_to, amount, Color::White);
                            if bg_from != bg_to {
                                cell.style.bg =
                                    mix(bg_from, bg_to, amount, Color::Black);
                            }
                            cell
                        }
                    }
                    FadeMode::Out => {
                        if amount <= 0.0 {
                            source
                        } else if amount >= 1.0 {
                            underlay
                        } else {
                            let fg_from = source.style.fg;
                            let fg_to = match underlay.style.bg {
                                Color::Reset => Color::Black,
                                other => other,
                            };
                            let bg_from = source.style.bg;
                            let bg_to = underlay.style.bg;

                            let mut cell = source;
                            cell.style.fg = mix(fg_from, fg_to, amount, Color::Black);
                            if bg_from != bg_to {
                                cell.style.bg =
                                    mix(bg_from, bg_to, amount, Color::Black);
                            }
                            cell
                        }
                    }
                    FadeMode::From(color) => {
                        if amount >= 1.0 {
                            source
                        } else {
                            let mut cell = source;
                            cell.style.fg =
                                mix(color, source.style.fg, amount, Color::White);
                            cell.style.bg =
                                mix(color, source.style.bg, amount, Color::Black);
                            cell
                        }
                    }
                    FadeMode::To(color) => {
                        if amount >= 1.0 {
                            let mut cell = source;
                            cell.style.fg = color;
                            cell.style.bg = color;
                            cell
                        } else {
                            let mut cell = source;
                            cell.style.fg =
                                mix(source.style.fg, color, amount, Color::Black);
                            cell.style.bg =
                                mix(source.style.bg, color, amount, Color::Black);
                            cell
                        }
                    }
                };

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
