use super::{
    attrs::{Attrs, Underline},
    color::Color,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub attrs: Attrs,
}

impl Style {
    pub const DEFAULT: Self = Self {
        fg: Color::Unset,
        bg: Color::Unset,
        attrs: Attrs::DEFAULT,
    };

    pub const fn new() -> Self {
        Self::DEFAULT
    }

    pub const fn fg(mut self, color: Color) -> Self {
        self.fg = color;
        self
    }

    pub const fn bg(mut self, color: Color) -> Self {
        self.bg = color;
        self
    }

    pub const fn bold(mut self) -> Self {
        self.attrs.bold = true;
        self
    }

    pub const fn dim(mut self) -> Self {
        self.attrs.dim = true;
        self
    }

    pub const fn italic(mut self) -> Self {
        self.attrs.italic = true;
        self
    }

    pub const fn underline(mut self) -> Self {
        self.attrs.underline = Underline::Single;
        self
    }

    pub const fn reverse(mut self) -> Self {
        self.attrs.reverse = true;
        self
    }

    pub fn patch(mut self, other: Style) -> Self {
        if other.fg != Color::Unset {
            self.fg = other.fg;
        }
        if other.bg != Color::Unset {
            self.bg = other.bg;
        }
        if other.attrs.bold {
            self.attrs.bold = true;
        }
        if other.attrs.dim {
            self.attrs.dim = true;
        }
        if other.attrs.italic {
            self.attrs.italic = true;
        }
        if other.attrs.underline != Underline::None {
            self.attrs.underline = other.attrs.underline;
        }
        if other.attrs.reverse {
            self.attrs.reverse = true;
        }
        if other.attrs.strikethrough {
            self.attrs.strikethrough = true;
        }
        self
    }
}

#[cfg(feature = "animate")]
impl animate::Interpolate for Style {
    fn lerp(start: &Self, end: &Self, t: f32) -> Self {
        Self {
            fg: animate::Interpolate::lerp(&start.fg, &end.fg, t),
            bg: animate::Interpolate::lerp(&start.bg, &end.bg, t),
            attrs: if t < 1.0 { start.attrs } else { end.attrs },
        }
    }
}

#[cfg(feature = "animate")]
impl animate::Distance for Style {
    fn distance(&self, other: &Self) -> f32 {
        let d_fg = animate::Distance::distance(&self.fg, &other.fg);
        let d_bg = animate::Distance::distance(&self.bg, &other.bg);
        (d_fg * d_fg + d_bg * d_bg).sqrt()
    }
}

#[cfg(feature = "animate")]
impl animate::Integrate for Style {
    type Velocity = [f32; 6];

    fn integrate(
        &self,
        target: &Self,
        velocity: &Self::Velocity,
        params: animate::SpringSpec,
        dt: f32,
    ) -> (Self, Self::Velocity) {
        let fg_vel = [velocity[0], velocity[1], velocity[2]];
        let bg_vel = [velocity[3], velocity[4], velocity[5]];

        let (fg, fg_nvel) = self.fg.integrate(&target.fg, &fg_vel, params, dt);
        let (bg, bg_nvel) = self.bg.integrate(&target.bg, &bg_vel, params, dt);

        (
            Style {
                fg,
                bg,
                attrs: if fg == target.fg && bg == target.bg {
                    target.attrs
                } else {
                    self.attrs
                },
            },
            [
                fg_nvel[0], fg_nvel[1], fg_nvel[2], bg_nvel[0], bg_nvel[1],
                bg_nvel[2],
            ],
        )
    }
}
