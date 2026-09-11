use kursor_core::render::style::Style;

/// a piece of optionally styled text.
///
/// the hierarchy is: [`Span`] -> [`Line`](super::Line) -> [`Text`](super::Text)
#[derive(Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub style: Option<Style>,
}

impl Span {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: None,
        }
    }

    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style: Some(style),
        }
    }
}

impl From<&str> for Span {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}

impl From<String> for Span {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}
