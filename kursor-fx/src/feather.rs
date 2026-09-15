use kursor_core::render::{cell::Cell, color::Color};

use crate::fx::color::mix;

/// controls how soft is the boundary between the effect and the untouched
/// cells.
#[derive(Clone, Copy, Debug)]
pub struct Feather {
    width: f32,
    curve: Curve,
}

#[derive(Clone, Copy, Debug)]
enum Curve {
    Linear,
    Soft,
    Custom(fn(f32) -> f32),
}

impl Default for Feather {
    fn default() -> Self {
        Self::full()
    }
}

impl Feather {
    pub fn full() -> Self {
        Self {
            width: -1.0,
            curve: Curve::Linear,
        }
    }

    pub fn hard() -> Self {
        Self {
            width: 0.0,
            curve: Curve::Linear,
        }
    }

    pub fn linear(width: f32) -> Self {
        Self {
            width: width.max(0.0),
            curve: Curve::Linear,
        }
    }

    pub fn soft(width: f32) -> Self {
        Self {
            width: width.max(0.0),
            curve: Curve::Soft,
        }
    }

    pub fn custom(width: f32, curve: fn(f32) -> f32) -> Self {
        Self {
            width: width.max(0.0),
            curve: Curve::Custom(curve),
        }
    }

    pub fn is_full(&self) -> bool {
        self.width < 0.0
    }

    pub fn is_hard(&self) -> bool {
        self.width == 0.0
    }

    pub fn width_val(&self) -> f32 {
        self.width.max(0.0)
    }

    fn factor(&self, distance: f32) -> f32 {
        if distance <= 0.0 {
            return 0.0;
        }
        if self.is_full() {
            return distance.clamp(0.0, 1.0);
        }
        if self.is_hard() {
            return 1.0;
        }

        let t = (distance / self.width).clamp(0.0, 1.0);
        match self.curve {
            Curve::Linear => t,
            Curve::Soft => t * t * (3.0 - 2.0 * t),
            Curve::Custom(curve) => curve(t).clamp(0.0, 1.0),
        }
    }

    pub fn soften(&self, cell: Cell, underlay: Cell, distance: f32) -> Cell {
        let factor = self.factor(distance);
        if factor >= 1.0 {
            return cell;
        }
        if factor <= 0.0 {
            return underlay;
        }

        let base_bg = match underlay.style.bg {
            Color::Reset | Color::Unset => Color::Black,
            bg => bg,
        };
        let target_fg = match cell.style.fg {
            Color::Reset | Color::Unset => Color::White,
            fg => fg,
        };

        let mut cell = cell;
        cell.style.fg = mix(base_bg, target_fg, factor, base_bg);
        if cell.style.bg != underlay.style.bg
            && cell.style.bg != Color::Unset
            && underlay.style.bg != Color::Unset
        {
            cell.style.bg =
                mix(underlay.style.bg, cell.style.bg, factor, base_bg);
        }
        cell
    }
}
