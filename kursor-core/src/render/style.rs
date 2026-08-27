use super::{attrs::{Attrs, Underline}, color::Color};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub attrs: Attrs,
}

impl Style {
    pub const DEFAULT: Self = Self {
        fg: Color::Reset,
        bg: Color::Reset,
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
        if other.fg != Color::Reset {
            self.fg = other.fg;
        }
        if other.bg != Color::Reset {
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
