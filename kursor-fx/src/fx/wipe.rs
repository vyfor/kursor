use std::time::Duration;

use animate::{Activity, Time};

use crate::{Direction, EffectCx, Fx, Mask, Subcell};

#[derive(Clone)]
pub struct Wipe {
    direction: Direction,
    duration: Duration,
    inward: bool,
    start: Option<Time>,
    subcell: Subcell,
    mask: Mask,
}

pub fn wipe_in(direction: Direction, duration: Duration) -> Wipe {
    Wipe {
        direction,
        duration,
        inward: true,
        start: None,
        subcell: Subcell::None,
        mask: Mask::all(),
    }
}

pub fn wipe_out(direction: Direction, duration: Duration) -> Wipe {
    Wipe {
        direction,
        duration,
        inward: false,
        start: None,
        subcell: Subcell::None,
        mask: Mask::all(),
    }
}

impl Wipe {
    pub fn subcell(mut self, mode: Subcell) -> Self {
        self.subcell = mode;
        self
    }

    pub fn mask(mut self, mask: Mask) -> Self {
        self.mask = mask;
        self
    }
}

impl Fx for Wipe {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity {
        let start = *self.start.get_or_insert(cx.time);
        let progress = cx.progress(start, self.duration);

        if progress >= 1.0 {
            if self.inward {
                for y in 0..cx.layer.height() {
                    for x in 0..cx.layer.width() {
                        cx.set(x, y, cx.source(x, y));
                    }
                }
            } else {
                cx.layer.clear();
            }
            return Activity::FINISHED;
        }

        let amount = if self.inward { progress } else { 1.0 - progress };
        let total = match self.direction {
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

        match self.direction {
            Direction::Down => self.wipe_down(cx, whole, frac),
            Direction::Up => self.wipe_up(cx, whole, frac),
            Direction::Right => self.wipe_right(cx, whole, frac),
            Direction::Left => self.wipe_left(cx, whole, frac),
        }

        Activity::RUNNING
    }

    fn reset(&mut self) {
        self.start = None;
    }
}

impl Wipe {
    fn wipe_left(&self, cx: &mut EffectCx, whole: usize, frac: f32) {
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
                    let cell = self.subcell.edge_cell(src, underlay, Direction::Left, frac);
                    cx.set(edge_x, y, cell);
                }
            }
        }
    }

    fn wipe_right(&self, cx: &mut EffectCx, whole: usize, frac: f32) {
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
                    let cell = self.subcell.edge_cell(src, underlay, Direction::Right, frac);
                    cx.set(edge_x, y, cell);
                }
            }
        }
    }

    fn wipe_up(&self, cx: &mut EffectCx, whole: usize, frac: f32) {
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
                    let cell = self.subcell.edge_cell(src, underlay, Direction::Up, frac);
                    cx.set(x, edge_y, cell);
                }
            }
        }
    }

    fn wipe_down(&self, cx: &mut EffectCx, whole: usize, frac: f32) {
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
                    let cell = self.subcell.edge_cell(src, underlay, Direction::Down, frac);
                    cx.set(x, edge_y, cell);
                }
            }
        }
    }
}
