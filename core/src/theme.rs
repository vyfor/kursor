use crate::render::{color::Color, style::Style};

#[derive(Clone, Copy)]
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
        Self {
            fg: Color::White,
            bg: Color::Reset,
            primary: Color::Cyan,
            accent: Color::Magenta,
            muted: Color::LightGray,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub palette: Palette,
    pub text: Style,
    pub surface: Style,
    pub primary: Style,
    pub focus: Style,
    pub disabled: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_palette(Palette::default())
    }
}

impl Theme {
    pub fn from_palette(palette: Palette) -> Self {
        Self {
            palette,
            text: Style {
                fg: palette.fg,
                bg: palette.bg,
                attrs: Default::default(),
            },
            surface: Style {
                fg: palette.fg,
                bg: palette.bg,
                attrs: Default::default(),
            },
            primary: Style {
                fg: palette.primary,
                bg: palette.bg,
                attrs: Default::default(),
            },
            focus: Style {
                fg: palette.bg,
                bg: palette.primary,
                attrs: Default::default(),
            },
            disabled: Style {
                fg: palette.muted,
                bg: palette.bg,
                attrs: Default::default(),
            },
        }
    }
}
