use crate::render::{attrs::Attrs, color::Color, style::Style};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub fg: Color,
    pub bg: Color,
    pub primary: Color,
    pub accent: Color,
    pub muted: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Palette {
    pub const DEFAULT: Self = Self {
        fg: Color::White,
        bg: Color::Reset,
        primary: Color::LightBlue,
        accent: Color::LightCyan,
        muted: Color::Gray,
        success: Color::LightGreen,
        warning: Color::LightYellow,
        error: Color::LightRed,
    };
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub palette: Palette,
    pub text: Style,
    pub surface: Style,
    pub primary: Style,
    pub accent: Style,
    pub focus: Style,
    pub active: Style,
    pub disabled: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Theme {
    pub const DEFAULT: Self = Self {
        palette: Palette::DEFAULT,
        text: Style {
            fg: Color::White,
            bg: Color::Unset,
            attrs: Attrs::DEFAULT,
        },
        surface: Style {
            fg: Color::White,
            bg: Color::Unset,
            attrs: Attrs::DEFAULT,
        },
        primary: Style {
            fg: Color::LightBlue,
            bg: Color::Unset,
            attrs: Attrs::DEFAULT,
        },
        accent: Style {
            fg: Color::LightCyan,
            bg: Color::Unset,
            attrs: Attrs::DEFAULT,
        },
        focus: Style {
            fg: Color::LightBlue,
            bg: Color::Unset,
            attrs: Attrs::DEFAULT,
        },
        active: Style {
            fg: Color::Black,
            bg: Color::LightBlue,
            attrs: Attrs::DEFAULT,
        },
        disabled: Style {
            fg: Color::Gray,
            bg: Color::Unset,
            attrs: Attrs::DEFAULT,
        },
    };

    pub fn default_ref() -> &'static Self {
        &Self::DEFAULT
    }

    pub fn from_palette(palette: Palette) -> Self {
        Self {
            palette,
            text: Style {
                fg: palette.fg,
                bg: Color::Unset,
                attrs: Default::default(),
            },
            surface: Style {
                fg: palette.fg,
                bg: palette.bg,
                attrs: Default::default(),
            },
            primary: Style {
                fg: palette.primary,
                bg: Color::Unset,
                attrs: Default::default(),
            },
            accent: Style {
                fg: palette.accent,
                bg: Color::Unset,
                attrs: Default::default(),
            },
            focus: Style {
                fg: palette.primary,
                bg: Color::Unset,
                attrs: Default::default(),
            },
            active: Style {
                fg: palette.bg,
                bg: palette.primary,
                attrs: Default::default(),
            },
            disabled: Style {
                fg: palette.muted,
                bg: Color::Unset,
                attrs: Default::default(),
            },
        }
    }
}
