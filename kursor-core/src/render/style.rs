use super::{attrs::Attrs, color::Color};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub attrs: Attrs,
}
