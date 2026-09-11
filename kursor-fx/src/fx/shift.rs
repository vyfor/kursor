use std::time::Duration;

use animate::{Activity, Time};
use kursor_core::{
    layout::{Direction, Offset},
    render::{cell::Cell, color::Color, style::Style, subcell::Subcell},
};

use crate::{EffectCx, Feather, Fx, Mask, Spread};

/// moves its content in or out of the area.
#[derive(Clone)]
pub struct Shift {
    duration: Duration,
    inward: bool,
    start: Option<Time>,
    mask: Mask,
    subcell: Subcell,
    feather: Feather,
    spread: Spread,
    target: ShiftTarget,
}

#[derive(Clone)]
pub enum ShiftTarget {
    Direction(Direction),
    Offset(Offset),
}

impl From<Direction> for ShiftTarget {
    fn from(dir: Direction) -> Self {
        Self::Direction(dir)
    }
}

impl From<Offset> for ShiftTarget {
    fn from(offset: Offset) -> Self {
        Self::Offset(offset)
    }
}

pub fn shift(duration: Duration, target: impl Into<ShiftTarget>) -> Shift {
    Shift {
        duration,
        inward: true,
        start: None,
        mask: Mask::all(),
        subcell: Subcell::None,
        feather: Feather::hard(),
        spread: Spread::uniform(),
        target: target.into(),
    }
}

impl Shift {
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

impl Fx for Shift {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity {
        let start = *self.start.get_or_insert(cx.time);
        let progress = cx.progress(start, self.duration);

        match &self.target {
            ShiftTarget::Direction(dir) => {
                if progress >= 1.0 {
                    self.finished(cx);
                    return Activity::FINISHED;
                }
                let amount = if self.inward {
                    1.0 - progress
                } else {
                    progress
                };
                self.apply_slide(cx, amount, *dir);
            }
            ShiftTarget::Offset(offset) => {
                let t = if self.inward {
                    progress
                } else {
                    1.0 - progress
                };
                self.apply_move(cx, t, progress, *offset);
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

impl Shift {
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

    fn apply_slide(
        &self,
        cx: &mut EffectCx<'_>,
        amount: f32,
        direction: Direction,
    ) {
        let total = match direction {
            Direction::Left | Direction::Right => cx.layer.width() as f32,
            Direction::Up | Direction::Down => cx.layer.height() as f32,
        };

        if self.subcell == Subcell::None {
            let distance = (total * amount).round() as i32;
            let (dx, dy) = match direction {
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
            return;
        }

        let pos = total * amount;
        let whole = pos.floor() as usize;
        let frac = pos.fract();

        cx.layer.clear();

        match direction {
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
                    let cell = self.subcell.edge_cell(
                        src,
                        underlay,
                        Direction::Down,
                        1.0 - frac,
                    );
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
                    let cell = self.subcell.edge_cell(
                        src,
                        underlay,
                        Direction::Up,
                        1.0 - frac,
                    );
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
                    let cell = self.subcell.edge_cell(
                        src,
                        underlay,
                        Direction::Right,
                        1.0 - frac,
                    );
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
                    let cell = self.subcell.edge_cell(
                        src,
                        underlay,
                        Direction::Left,
                        1.0 - frac,
                    );
                    cx.set(edge_x, y, cell);
                }
            }
        }
    }

    fn apply_move(
        &self,
        cx: &mut EffectCx<'_>,
        t: f32,
        progress: f32,
        offset: Offset,
    ) {
        let finished = progress >= 1.0;
        let (px, py) = if finished {
            if self.inward {
                ((offset.x as f32).abs(), (offset.y as f32).abs())
            } else {
                (0.0, 0.0)
            }
        } else {
            (offset.x as f32 * t, offset.y as f32 * t)
        };

        let (px, py) = if self.subcell == Subcell::None {
            (px.round(), py.round())
        } else {
            (px, py)
        };

        let x = ShiftData::new(px.abs(), offset.x >= 0);
        let y = ShiftData::new(py.abs(), offset.y >= 0);

        cx.layer.clear();

        let w = cx.layer.width() as i32;
        let h = cx.layer.height() as i32;
        let x_edge = x.edge_src(w);
        let y_edge = y.edge_src(h);

        for src_y in 0..h {
            if Some(src_y) == y_edge {
                continue;
            }
            let dst_y = src_y + y.body_offset();
            if dst_y < 0 || dst_y >= h {
                continue;
            }
            for src_x in 0..w {
                if Some(src_x) == x_edge {
                    continue;
                }
                let dst_x = src_x + x.body_offset();
                if dst_x < 0 || dst_x >= w {
                    continue;
                }
                cx.set(
                    dst_x as u16,
                    dst_y as u16,
                    cx.source(src_x as u16, src_y as u16),
                );
            }
        }

        if self.subcell != Subcell::None {
            if let Some(src_x) = x_edge {
                self.strip_x(cx, &x, &y, src_x);
            }
            if let Some(src_y) = y_edge {
                self.strip_y(cx, &y, &x, src_y);
            }
        }
    }

    fn strip_x(
        &self,
        cx: &mut EffectCx,
        x: &ShiftData,
        y: &ShiftData,
        src_x: i32,
    ) {
        let w = cx.layer.width() as i32;
        let h = cx.layer.height() as i32;
        let edge_x = src_x + x.body_offset();
        if edge_x < 0 || edge_x >= w {
            return;
        }

        let ch = if x.positive {
            self.subcell.fill_right(1.0 - x.frac)
        } else {
            self.subcell.fill_left(1.0 - x.frac)
        };

        for src_y in 0..h {
            let dst_y = src_y + y.body_offset();
            if dst_y < 0 || dst_y >= h {
                continue;
            }
            if !cx.includes(&self.mask, src_x as u16, src_y as u16) {
                continue;
            }
            let src = cx.source(src_x as u16, src_y as u16);
            let underlay = cx.underlay(edge_x as u16, dst_y as u16);
            let fg = if src.style.bg != Color::Reset {
                src.style.bg
            } else {
                src.style.fg
            };
            let style = Style::new().fg(fg).bg(underlay.style.bg);
            cx.set(edge_x as u16, dst_y as u16, Cell::new(ch, style));
        }
    }

    fn strip_y(
        &self,
        cx: &mut EffectCx,
        y: &ShiftData,
        x: &ShiftData,
        src_y: i32,
    ) {
        let w = cx.layer.width() as i32;
        let h = cx.layer.height() as i32;
        let edge_y = src_y + y.body_offset();
        if edge_y < 0 || edge_y >= h {
            return;
        }

        let ch = if y.positive {
            self.subcell.fill_bottom(1.0 - y.frac)
        } else {
            self.subcell.fill_top(1.0 - y.frac)
        };

        for src_x in 0..w {
            let dst_x = src_x + x.body_offset();
            if dst_x < 0 || dst_x >= w {
                continue;
            }
            if !cx.includes(&self.mask, src_x as u16, src_y as u16) {
                continue;
            }
            let src = cx.source(src_x as u16, src_y as u16);
            let underlay = cx.underlay(dst_x as u16, edge_y as u16);
            let fg = if src.style.bg != Color::Reset {
                src.style.bg
            } else {
                src.style.fg
            };
            let style = Style::new().fg(fg).bg(underlay.style.bg);
            cx.set(dst_x as u16, edge_y as u16, Cell::new(ch, style));
        }
    }
}

struct ShiftData {
    whole: i32,
    frac: f32,
    positive: bool,
}

impl ShiftData {
    fn new(position: f32, positive: bool) -> Self {
        Self {
            whole: position.floor() as i32,
            frac: position.fract(),
            positive,
        }
    }

    fn body_offset(&self) -> i32 {
        if self.positive {
            self.whole
        } else {
            -self.whole
        }
    }

    fn edge_src(&self, len: i32) -> Option<i32> {
        if self.frac > 0.0 {
            Some(if self.positive { 0 } else { len - 1 })
        } else {
            None
        }
    }
}
