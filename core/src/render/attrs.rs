#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Attrs {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    // todo: add rest
}

impl Attrs {
    pub const DEFAULT: Self = Self {
        bold: false,
        dim: false,
        italic: false,
        underline: false,
    };
}
