use std::time::Duration;

use animate::{Activity, Time};
use kursor_core::{
    layout::Direction,
    render::{cell::Cell, color::Color, subcell::Subcell},
};

use super::color::mix;
use crate::{EffectCx, Feather, Fx, Mask, Spread};

/// reveals or hides its content across the area.
#[derive(Clone)]
pub struct Reveal {
    duration: Duration,
    inward: bool,
    start: Option<Time>,
    mask: Mask,
    subcell: Subcell,
    feather: Feather,
    spread: Spread,
    apply_feather: bool,
}

pub fn reveal(duration: Duration) -> Reveal {
    Reveal {
        duration,
        inward: true,
        start: None,
        mask: Mask::all(),
        subcell: Subcell::None,
        feather: Feather::full(),
        spread: Spread::uniform(),
        apply_feather: false,
    }
}

impl Reveal {
    pub fn out(mut self) -> Self {
        self.inward = false;
        self
    }

    pub fn feather(mut self, feather: Feather) -> Self {
        self.feather = feather;
        self.apply_feather = true;
        self
    }

    pub fn subcell(mut self, mode: Subcell) -> Self {
        self.subcell = mode;
        self
    }

    pub fn spread(mut self, spread: Spread) -> Self {
        self.spread = spread;
        if !self.apply_feather && !matches!(spread, Spread::Uniform) {
            self.feather = Feather::hard();
        }
        self
    }

    pub fn mask(mut self, mask: Mask) -> Self {
        self.mask = mask;
        self
    }
}

impl Fx for Reveal {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity {
        let start = *self.start.get_or_insert(cx.time);
        let progress = cx.progress(start, self.duration);

        match self.spread {
            Spread::Uniform => {
                self.apply_blend(cx, progress);
            }
            Spread::Towards(dir) => {
                if progress >= 1.0 {
                    self.finished(cx);
                    return Activity::FINISHED;
                }
                let amount = if self.inward {
                    progress
                } else {
                    1.0 - progress
                };
                let wipe_dir = if self.inward {
                    dir
                } else {
                    match dir {
                        Direction::Right => Direction::Left,
                        Direction::Left => Direction::Right,
                        Direction::Up => Direction::Down,
                        Direction::Down => Direction::Up,
                    }
                };
                self.apply_soft_wipe(cx, amount, wipe_dir);
            }
            Spread::Radial => {
                if progress >= 1.0 {
                    self.finished(cx);
                    return Activity::FINISHED;
                }
                let amount = if self.inward {
                    progress
                } else {
                    1.0 - progress
                };
                self.apply_radial_edge(cx, amount);
            }
        }

        if progress >= 1.0 {
            self.finished(cx);
            Activity::FINISHED
        } else {
            Activity::RUNNING
        }
    }

    fn reset(&mut self) {
        self.start = None;
    }
}

impl Reveal {
    fn finished(&self, cx: &mut EffectCx<'_>) {
        if self.inward {
            for y in 0..cx.layer.height() {
                for x in 0..cx.layer.width() {
                    cx.set(x, y, cx.source(x, y));
                }
            }
        } else {
            cx.layer.clear();
        }
    }

    fn apply_blend(&self, cx: &mut EffectCx<'_>, progress: f32) {
        for y in 0..cx.layer.height() {
            for x in 0..cx.layer.width() {
                let source = cx.source(x, y);
                let underlay = cx.underlay(x, y);
                let amount = cx.spread(self.spread, progress, x, y);

                if !cx.includes(&self.mask, x, y) {
                    continue;
                }

                let cell = if self.inward {
                    if amount <= 0.0 {
                        underlay
                    } else if amount >= 1.0 {
                        source
                    } else {
                        self.blend_cell(source, underlay, amount, true)
                    }
                } else {
                    if amount <= 0.0 {
                        source
                    } else if amount >= 1.0 {
                        underlay
                    } else {
                        self.blend_cell(source, underlay, amount, false)
                    }
                };

                cx.set(x, y, cell);
            }
        }
    }

    fn blend_cell(
        &self,
        source: Cell,
        underlay: Cell,
        amount: f32,
        entering: bool,
    ) -> Cell {
        let base_bg = match underlay.style.bg {
            Color::Reset | Color::Unset => Color::Black,
            bg => bg,
        };

        let (fg_from, fg_to, bg_from, bg_to) = if entering {
            let fg_target = match source.style.fg {
                Color::Reset | Color::Unset => Color::White,
                fg => fg,
            };
            (base_bg, fg_target, base_bg, source.style.bg)
        } else {
            let fg_current = match source.style.fg {
                Color::Reset | Color::Unset => Color::White,
                fg => fg,
            };
            (fg_current, base_bg, source.style.bg, base_bg)
        };

        let mut cell = source;
        cell.style.fg = mix(fg_from, fg_to, amount, base_bg);
        if bg_from != bg_to && bg_from != Color::Unset && bg_to != Color::Unset
        {
            cell.style.bg = mix(bg_from, bg_to, amount, base_bg);
        }
        cell
    }

    fn apply_soft_wipe(
        &self,
        cx: &mut EffectCx<'_>,
        amount: f32,
        direction: Direction,
    ) {
        let length = match direction {
            Direction::Left | Direction::Right => cx.layer.width() as f32,
            Direction::Up | Direction::Down => cx.layer.height() as f32,
        };
        let feather_width = self.feather.width_val();
        let pos = (length + feather_width) * amount;

        for y in 0..cx.layer.height() {
            for x in 0..cx.layer.width() {
                if !cx.includes(&self.mask, x, y) {
                    continue;
                }

                let distance = match direction {
                    Direction::Left => pos - (length - 1.0 - x as f32),
                    Direction::Right => pos - x as f32,
                    Direction::Up => pos - (length - 1.0 - y as f32),
                    Direction::Down => pos - y as f32,
                };
                let source = cx.source(x, y);
                let underlay = cx.underlay(x, y);

                let cell = if feather_width > 0.0 {
                    self.feather.soften(source, underlay, distance)
                } else if self.subcell != Subcell::None
                    && distance > 0.0
                    && distance < 1.0
                {
                    self.subcell
                        .edge_cell(source, underlay, direction, distance)
                } else if distance >= 1.0 {
                    source
                } else {
                    underlay
                };

                cx.set(x, y, cell);
            }
        }
    }

    fn apply_radial_edge(&self, cx: &mut EffectCx<'_>, amount: f32) {
        let w = cx.layer.width() as f32;
        let h = cx.layer.height() as f32;
        let cx_mid = (w - 1.0).max(0.0) / 2.0;
        let cy_mid = (h - 1.0).max(0.0) / 2.0;
        let max_dist =
            (cx_mid * cx_mid + (cy_mid * 2.0) * (cy_mid * 2.0)).sqrt();
        let feather_width = self.feather.width_val();
        let pos = (max_dist + feather_width) * amount;

        for y in 0..cx.layer.height() {
            for x in 0..cx.layer.width() {
                if !cx.includes(&self.mask, x, y) {
                    continue;
                }

                let dx = x as f32 - cx_mid;
                let dy = (y as f32 - cy_mid) * 2.0;
                let cell_dist = (dx * dx + dy * dy).sqrt();
                let distance = pos - cell_dist;
                let source = cx.source(x, y);
                let underlay = cx.underlay(x, y);

                let cell = if feather_width > 0.0 {
                    self.feather.soften(source, underlay, distance)
                } else if self.subcell != Subcell::None
                    && distance > 0.0
                    && distance < 1.0
                {
                    let dir = if dx.abs() > dy.abs() {
                        if dx > 0.0 {
                            Direction::Right
                        } else {
                            Direction::Left
                        }
                    } else if dy > 0.0 {
                        Direction::Down
                    } else {
                        Direction::Up
                    };
                    self.subcell.edge_cell(source, underlay, dir, distance)
                } else if distance >= 1.0 {
                    source
                } else {
                    underlay
                };

                cx.set(x, y, cell);
            }
        }
    }
}
