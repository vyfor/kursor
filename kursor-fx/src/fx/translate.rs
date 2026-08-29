use std::time::Duration;

use animate::{Activity, Time};
use kursor_core::{
    layout::Offset,
    render::{cell::Cell, color::Color, style::Style},
};

use crate::{EffectCx, Fx, Mask, Subcell};

#[derive(Clone)]
pub struct Translate {
    offset: Offset,
    duration: Duration,
    start: Option<Time>,
    subcell: Subcell,
    mask: Mask,
}

pub fn translate(offset: Offset, duration: Duration) -> Translate {
    Translate {
        offset,
        duration,
        start: None,
        subcell: Subcell::None,
        mask: Mask::all(),
    }
}

impl Translate {
    pub fn subcell(mut self, mode: Subcell) -> Self {
        self.subcell = mode;
        self
    }

    pub fn mask(mut self, mask: Mask) -> Self {
        self.mask = mask;
        self
    }
}

struct Shift {
    whole: i32,
    frac: f32,
    positive: bool,
}

impl Shift {
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

impl Fx for Translate {
    fn apply(&mut self, cx: &mut EffectCx<'_>) -> Activity {
        let start = *self.start.get_or_insert(cx.time);
        let progress = cx.progress(start, self.duration);
        let finished = progress >= 1.0;

        let (px, py) = if finished {
            ((self.offset.x as f32).abs(), (self.offset.y as f32).abs())
        } else {
            (
                self.offset.x as f32 * progress,
                self.offset.y as f32 * progress,
            )
        };

        let (px, py) = if self.subcell == Subcell::None {
            (px.round(), py.round())
        } else {
            (px, py)
        };

        let x = Shift::new(px.abs(), self.offset.x >= 0);
        let y = Shift::new(py.abs(), self.offset.y >= 0);

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

        if finished {
            Activity::FINISHED
        } else {
            Activity::RUNNING
        }
    }

    fn reset(&mut self) {
        self.start = None;
    }
}

impl Translate {
    fn strip_x(&self, cx: &mut EffectCx, x: &Shift, y: &Shift, src_x: i32) {
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

    fn strip_y(&self, cx: &mut EffectCx, y: &Shift, x: &Shift, src_y: i32) {
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
