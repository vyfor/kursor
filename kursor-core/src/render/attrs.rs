#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Underline {
    #[default]
    None,
    Single,
    Double,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Blink {
    #[default]
    None,
    Slow,
    Rapid,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Attrs {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: Underline,
    pub blink: Blink,
    pub reverse: bool,
    pub hidden: bool,
    pub strikethrough: bool,
    pub overline: bool,
}

impl Attrs {
    pub const DEFAULT: Self = Self {
        bold: false,
        dim: false,
        italic: false,
        underline: Underline::None,
        blink: Blink::None,
        reverse: false,
        hidden: false,
        strikethrough: false,
        overline: false,
    };
}
