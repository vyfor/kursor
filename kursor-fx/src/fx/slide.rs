use std::time::Duration;

use animate::{Activity, Time};

use crate::{Direction, EffectCx, Fx, Mask, Subcell};

#[derive(Clone)]
pub struct Slide {
    direction: Direction,
    duration: Duration,
    inward: bool,
    start: Option<Time>,
    subcell: Subcell,
    mask: Mask,
}

pub fn slide_in(direction: Direction, duration: Duration) -> Slide {
    Slide {
        direction,
        duration,
        inward: true,
        start: None,
        subcell: Subcell::None,
        mask: Mask::all(),
    }
}

pub fn slide_out(direction: Direction, duration: Duration) -> Slide {
    Slide {
        direction,
        duration,
        inward: false,
        start: None,
        subcell: Subcell::None,
        mask: Mask::all(),
    }
}

impl Slide {
    pub fn subcell(mut self, mode: Subcell) -> Self {
        self.subcell = mode;
        self
    }

    pub fn mask(mut self, mask: Mask) -> Self {
        self.mask = mask;
        self
    }
}

impl Fx for Slide {
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

        let amount = if self.inward { 1.0 - progress } else { progress };
        let total = match self.direction {
            Direction::Left | Direction::Right => cx.layer.width() as f32,
            Direction::Up | Direction::Down => cx.layer.height() as f32,
        };

        if self.subcell == Subcell::None {
            self.apply_whole_cell(cx, amount, total);
        } else {
            self.apply_subcell(cx, amount, total);
        }

        Activity::RUNNING
    }

    fn reset(&mut self) {
        self.start = None;
    }
}

impl Slide {
    fn apply_whole_cell(&self, cx: &mut EffectCx, amount: f32, total: f32) {
        let distance = (total * amount).round() as i32;
        let (dx, dy) = self.offset(distance);

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
    }

    fn apply_subcell(&self, cx: &mut EffectCx, amount: f32, total: f32) {
        let pos = total * amount;
        let whole = pos.floor() as usize;
        let frac = pos.fract();

        cx.layer.clear();

        match self.direction {
            Direction::Down => self.slide_down(cx, whole, frac),
            Direction::Up => self.slide_up(cx, whole, frac),
            Direction::Right => self.slide_right(cx, whole, frac),
            Direction::Left => self.slide_left(cx, whole, frac),
        }
    }

    fn slide_down(&self, cx: &mut EffectCx, whole: usize, frac: f32) {
        let w = cx.layer.width();
        let h = cx.layer.height() as usize;

        if frac == 0.0 {
            for dst_y in whole..h {
                let src_y = (dst_y - whole) as u16;
                for x in 0..w {
                    cx.set(x, dst_y as u16, cx.source(x, src_y));
                }
            }
            return;
        }

        for dst_y in (whole + 1)..h {
            let src_y = (dst_y - whole) as u16;
            for x in 0..w {
                cx.set(x, dst_y as u16, cx.source(x, src_y));
            }
        }

        if whole < h {
            let edge_y = whole as u16;
            for x in 0..w {
                if cx.includes(&self.mask, x, 0) {
                    let src = cx.source(x, 0);
                    let underlay = cx.underlay(x, edge_y);
                    let cell =
                        self.subcell.edge_cell(src, underlay, Direction::Down, 1.0 - frac);
                    cx.set(x, edge_y, cell);
                }
            }
        }
    }

    fn slide_up(&self, cx: &mut EffectCx, whole: usize, frac: f32) {
        let w = cx.layer.width();
        let h = cx.layer.height() as usize;

        if frac == 0.0 {
            let visible_count = h.saturating_sub(whole);
            for dst_y in 0..visible_count {
                let src_y = (dst_y + whole) as u16;
                for x in 0..w {
                    cx.set(x, dst_y as u16, cx.source(x, src_y));
                }
            }
            return;
        }

        let edge_row = h.saturating_sub(1 + whole);

        for dst_y in 0..edge_row {
            let src_y = (dst_y + whole) as u16;
            for x in 0..w {
                cx.set(x, dst_y as u16, cx.source(x, src_y));
            }
        }

        if edge_row < h {
            let edge_y = edge_row as u16;
            let src_row = (edge_row + whole).min(h.saturating_sub(1)) as u16;
            for x in 0..w {
                if cx.includes(&self.mask, x, src_row) {
                    let src = cx.source(x, src_row);
                    let underlay = cx.underlay(x, edge_y);
                    let cell =
                        self.subcell.edge_cell(src, underlay, Direction::Up, 1.0 - frac);
                    cx.set(x, edge_y, cell);
                }
            }
        }
    }

    fn slide_right(&self, cx: &mut EffectCx, whole: usize, frac: f32) {
        let w = cx.layer.width() as usize;
        let h = cx.layer.height();

        if frac == 0.0 {
            for dst_x in whole..w {
                let src_x = (dst_x - whole) as u16;
                for y in 0..h {
                    cx.set(dst_x as u16, y, cx.source(src_x, y));
                }
            }
            return;
        }

        for dst_x in (whole + 1)..w {
            let src_x = (dst_x - whole) as u16;
            for y in 0..h {
                cx.set(dst_x as u16, y, cx.source(src_x, y));
            }
        }

        if whole < w {
            let edge_x = whole as u16;
            for y in 0..h {
                if cx.includes(&self.mask, 0, y) {
                    let src = cx.source(0, y);
                    let underlay = cx.underlay(edge_x, y);
                    let cell =
                        self.subcell.edge_cell(src, underlay, Direction::Right, 1.0 - frac);
                    cx.set(edge_x, y, cell);
                }
            }
        }
    }

    fn slide_left(&self, cx: &mut EffectCx, whole: usize, frac: f32) {
        let w = cx.layer.width() as usize;
        let h = cx.layer.height();

        if frac == 0.0 {
            let visible_count = w.saturating_sub(whole);
            for dst_x in 0..visible_count {
                let src_x = (dst_x + whole) as u16;
                for y in 0..h {
                    cx.set(dst_x as u16, y, cx.source(src_x, y));
                }
            }
            return;
        }

        let edge_col = w.saturating_sub(1 + whole);

        for dst_x in 0..edge_col {
            let src_x = (dst_x + whole) as u16;
            for y in 0..h {
                cx.set(dst_x as u16, y, cx.source(src_x, y));
            }
        }

        if edge_col < w {
            let edge_x = edge_col as u16;
            let src_col = (edge_col + whole).min(w.saturating_sub(1)) as u16;
            for y in 0..h {
                if cx.includes(&self.mask, src_col, y) {
                    let src = cx.source(src_col, y);
                    let underlay = cx.underlay(edge_x, y);
                    let cell =
                        self.subcell.edge_cell(src, underlay, Direction::Left, 1.0 - frac);
                    cx.set(edge_x, y, cell);
                }
            }
        }
    }

    fn offset(&self, distance: i32) -> (i32, i32) {
        match self.direction {
            Direction::Left => (-distance, 0),
            Direction::Right => (distance, 0),
            Direction::Up => (0, -distance),
            Direction::Down => (0, distance),
        }
    }
}
