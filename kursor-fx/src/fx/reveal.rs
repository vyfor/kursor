use std::time::Duration;

use animate::{Activity, Time};
use kursor_core::{
    layout::Direction,
    render::{cell::Cell, color::Color, style::Style, subcell::Subcell},
};

use super::color::mix;
use crate::{EffectCx, Feather, Fx, Mask, Spread};

#[derive(Clone)]
pub struct Reveal {
    duration: Duration,
    inward: bool,
    start: Option<Time>,
    mask: Mask,
    subcell: Subcell,
    feather: Feather,
    spread: Spread,
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
    }
}

impl Reveal {
    pub fn out(mut self) -> Self {
        self.inward = false;
        self
    }

    pub fn feather(mut self, feather: Feather) -> Self {
        self.feather = feather;
        self
    }

    pub fn subcell(mut self, mode: Subcell) -> Self {
        self.subcell = mode;
        self
    }

    pub fn spread(mut self, spread: Spread) -> Self {
        self.spread = spread;
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

        if self.feather.is_full() {
            self.apply_blend(cx, progress);
        } else {
            match self.spread {
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
                    if self.feather.is_hard() {
                        self.apply_hard_wipe(cx, amount, dir);
                    } else {
                        self.apply_soft_wipe(cx, amount, dir);
                    }
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
                Spread::Uniform => {
                    self.apply_blend(cx, progress);
                }
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
                    } else if let Some(cell) =
                        self.subcell_blend(cx, x, y, progress, source, underlay, true)
                    {
                        cell
                    } else {
                        self.blend_cell(source, underlay, amount, true)
                    }
                } else {
                    if amount <= 0.0 {
                        source
                    } else if amount >= 1.0 {
                        underlay
                    } else if let Some(cell) =
                        self.subcell_blend(cx, x, y, progress, source, underlay, false)
                    {
                        cell
                    } else {
                        self.blend_cell(source, underlay, amount, false)
                    }
                };

                cx.set(x, y, cell);
            }
        }
    }

    fn blend_cell(&self, source: Cell, underlay: Cell, amount: f32, entering: bool) -> Cell {
        let (fg_from, fg_to, bg_from, bg_to) = if entering {
            let fg_from = match underlay.style.bg {
                Color::Reset => Color::Black,
                other => other,
            };
            (fg_from, source.style.fg, underlay.style.bg, source.style.bg)
        } else {
            let fg_to = match underlay.style.bg {
                Color::Reset => Color::Black,
                other => other,
            };
            (source.style.fg, fg_to, source.style.bg, underlay.style.bg)
        };

        let mut cell = source;
        cell.style.fg = mix(fg_from, fg_to, amount, Color::White);
        if bg_from != bg_to {
            cell.style.bg = mix(bg_from, bg_to, amount, Color::Black);
        }
        cell
    }

    fn subcell_blend(
        &self,
        cx: &EffectCx<'_>,
        x: u16,
        y: u16,
        progress: f32,
        source: Cell,
        underlay: Cell,
        entering: bool,
    ) -> Option<Cell> {
        if self.subcell == Subcell::None || source.ch != ' ' || source.style.bg == Color::Reset {
            return None;
        }

        let direction = match self.spread {
            Spread::Towards(direction) => direction,
            _ => return None,
        };

        let a_here = cx.spread(self.spread, progress, x, y);
        let spread = |x: u16, y: u16| cx.spread(self.spread, progress, x, y);
        let a_next = match direction {
            Direction::Right => {
                if x + 1 < cx.layer.width() {
                    spread(x + 1, y)
                } else {
                    let step = spread(x.saturating_sub(1), y) - a_here;
                    (a_here - step).max(0.0)
                }
            }
            Direction::Left => {
                if x > 0 {
                    spread(x - 1, y)
                } else {
                    let step = spread((x + 1).min(cx.layer.width() - 1), y) - a_here;
                    (a_here - step).max(0.0)
                }
            }
            Direction::Down => {
                if y + 1 < cx.layer.height() {
                    spread(x, y + 1)
                } else {
                    let step = spread(x, y.saturating_sub(1)) - a_here;
                    (a_here - step).max(0.0)
                }
            }
            Direction::Up => {
                if y > 0 {
                    spread(x, y - 1)
                } else {
                    let step = spread(x, (y + 1).min(cx.layer.height() - 1)) - a_here;
                    (a_here - step).max(0.0)
                }
            }
        };

        let gamma = 0.6;
        let ag_here = a_here.powf(gamma);
        let ag_next = a_next.powf(gamma);

        let (from, to) = if entering {
            (underlay.style.bg, source.style.bg)
        } else {
            (source.style.bg, underlay.style.bg)
        };
        let c_here = mix(from, to, ag_here, Color::Black);
        let c_next = mix(from, to, ag_next, Color::Black);

        let ch = match direction {
            Direction::Right => self.subcell.fill_left(ag_here.clamp(0.0, 1.0)),
            Direction::Left => self.subcell.fill_right(ag_here.clamp(0.0, 1.0)),
            Direction::Down => self.subcell.fill_top(ag_here.clamp(0.0, 1.0)),
            Direction::Up => self.subcell.fill_bottom(ag_here.clamp(0.0, 1.0)),
        };

        let bright_on_fill = true;
        let (fg, bg) = if bright_on_fill {
            (c_here, c_next)
        } else {
            (c_next, c_here)
        };

        Some(Cell::new(ch, Style::new().fg(fg).bg(bg)))
    }

    fn apply_hard_wipe(&self, cx: &mut EffectCx<'_>, amount: f32, direction: Direction) {
        let total = match direction {
            Direction::Left | Direction::Right => cx.layer.width() as f32,
            Direction::Up | Direction::Down => cx.layer.height() as f32,
        };

        let pos = total * amount;

        let (whole, frac) = if self.subcell == Subcell::None {
            (pos.round() as usize, 0.0)
        } else {
            (pos.floor() as usize, pos.fract())
        };

        cx.layer.clear();

        match direction {
            Direction::Down => self.wipe_down(cx, whole, frac),
            Direction::Up => self.wipe_up(cx, whole, frac),
            Direction::Right => self.wipe_right(cx, whole, frac),
            Direction::Left => self.wipe_left(cx, whole, frac),
        }
    }

    fn apply_soft_wipe(&self, cx: &mut EffectCx<'_>, amount: f32, direction: Direction) {
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
                    Direction::Left => pos - x as f32,
                    Direction::Right => pos - (length - 1.0 - x as f32),
                    Direction::Up => pos - (length - 1.0 - y as f32),
                    Direction::Down => pos - y as f32,
                };
                let source = cx.source(x, y);
                let underlay = cx.underlay(x, y);
                let boundary = distance > 0.0 && distance < 1.0;
                let cell = if self.subcell != Subcell::None && boundary {
                    let edge = self
                        .subcell
                        .edge_cell(source, underlay, direction, distance);
                    self.feather.soften(edge, underlay, distance)
                } else {
                    self.feather.soften(source, underlay, distance)
                };

                cx.set(x, y, cell);
            }
        }
    }

    fn apply_radial_edge(&self, cx: &mut EffectCx<'_>, amount: f32) {
        let w = cx.layer.width() as f32;
        let h = cx.layer.height() as f32;
        let max_dist = (0.25f32 + 0.25).sqrt();
        let pos = max_dist * amount;

        for y in 0..cx.layer.height() {
            for x in 0..cx.layer.width() {
                if !cx.includes(&self.mask, x, y) {
                    continue;
                }

                let nx = (x as f32 + 0.5) / w.max(1.0) - 0.5;
                let ny = (y as f32 + 0.5) / h.max(1.0) - 0.5;
                let cell_dist = (nx * nx + ny * ny).sqrt();
                let distance = pos - cell_dist;
                let source = cx.source(x, y);
                let underlay = cx.underlay(x, y);
                let boundary = distance > 0.0 && distance < 1.0;
                let cell = if self.subcell != Subcell::None && boundary {
                    let dir = if nx.abs() > ny.abs() {
                        if nx > 0.0 {
                            Direction::Right
                        } else {
                            Direction::Left
                        }
                    } else if ny > 0.0 {
                        Direction::Down
                    } else {
                        Direction::Up
                    };
                    let edge = self.subcell.edge_cell(source, underlay, dir, distance);
                    self.feather.soften(edge, underlay, distance)
                } else {
                    self.feather.soften(source, underlay, distance)
                };

                cx.set(x, y, cell);
            }
        }
    }

    fn edge(&self, src: Cell, underlay: Cell, direction: Direction, frac: f32) -> Cell {
        let cell = self.subcell.edge_cell(src, underlay, direction, frac);
        self.feather.soften(cell, underlay, frac)
    }

    fn wipe_down(&self, cx: &mut EffectCx, whole: usize, frac: f32) {
        let w = cx.layer.width();
        let h = cx.layer.height() as usize;

        if frac == 0.0 {
            for y in 0..whole.min(h) {
                for x in 0..w {
                    cx.set(x, y as u16, cx.source(x, y as u16));
                }
            }
            return;
        }

        for y in 0..whole.min(h) {
            for x in 0..w {
                cx.set(x, y as u16, cx.source(x, y as u16));
            }
        }

        if whole < h {
            let edge_y = whole as u16;
            for x in 0..w {
                if cx.includes(&self.mask, x, edge_y) {
                    let src = cx.source(x, edge_y);
                    let underlay = cx.underlay(x, edge_y);
                    let cell = self.edge(src, underlay, Direction::Down, frac);
                    cx.set(x, edge_y, cell);
                }
            }
        }
    }

    fn wipe_up(&self, cx: &mut EffectCx, whole: usize, frac: f32) {
        let w = cx.layer.width();
        let h = cx.layer.height() as usize;

        if frac == 0.0 {
            let start = h.saturating_sub(whole);
            for y in start..h {
                for x in 0..w {
                    cx.set(x, y as u16, cx.source(x, y as u16));
                }
            }
            return;
        }

        let start = h.saturating_sub(whole);
        for y in start..h {
            for x in 0..w {
                cx.set(x, y as u16, cx.source(x, y as u16));
            }
        }

        if start > 0 {
            let edge_y = (start - 1) as u16;
            for x in 0..w {
                if cx.includes(&self.mask, x, edge_y) {
                    let src = cx.source(x, edge_y);
                    let underlay = cx.underlay(x, edge_y);
                    let cell = self.edge(src, underlay, Direction::Up, frac);
                    cx.set(x, edge_y, cell);
                }
            }
        }
    }

    fn wipe_right(&self, cx: &mut EffectCx, whole: usize, frac: f32) {
        let w = cx.layer.width() as usize;
        let h = cx.layer.height();

        if frac == 0.0 {
            for x in 0..whole.min(w) {
                for y in 0..h {
                    cx.set(x as u16, y, cx.source(x as u16, y));
                }
            }
            return;
        }

        for x in 0..whole.min(w) {
            for y in 0..h {
                cx.set(x as u16, y, cx.source(x as u16, y));
            }
        }

        if whole < w {
            let edge_x = whole as u16;
            for y in 0..h {
                if cx.includes(&self.mask, edge_x, y) {
                    let src = cx.source(edge_x, y);
                    let underlay = cx.underlay(edge_x, y);
                    let cell = self.edge(src, underlay, Direction::Right, frac);
                    cx.set(edge_x, y, cell);
                }
            }
        }
    }

    fn wipe_left(&self, cx: &mut EffectCx, whole: usize, frac: f32) {
        let w = cx.layer.width() as usize;
        let h = cx.layer.height();

        if frac == 0.0 {
            let start = w.saturating_sub(whole);
            for x in start..w {
                for y in 0..h {
                    cx.set(x as u16, y, cx.source(x as u16, y));
                }
            }
            return;
        }

        let start = w.saturating_sub(whole);
        for x in start..w {
            for y in 0..h {
                cx.set(x as u16, y, cx.source(x as u16, y));
            }
        }

        if start > 0 {
            let edge_x = (start - 1) as u16;
            for y in 0..h {
                if cx.includes(&self.mask, edge_x, y) {
                    let src = cx.source(edge_x, y);
                    let underlay = cx.underlay(edge_x, y);
                    let cell = self.edge(src, underlay, Direction::Left, frac);
                    cx.set(edge_x, y, cell);
                }
            }
        }
    }
}
