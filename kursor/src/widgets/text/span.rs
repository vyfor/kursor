use std::borrow::Cow;

use super::span_ext::IntoSpan;
use kursor_core::{
    render::style::Style,
    state::{Value, ValueSource},
};

/// text content of a [`Span`], either static or reactive.
#[derive(Clone, PartialEq, Eq)]
#[doc(hidden)]
pub enum SpanContent {
    Static(String),
    Reactive(Value<String>),
}

/// a piece of optionally styled text.
///
/// the hierarchy is: [`Span`] -> [`Line`](super::Line) -> [`Text`](super::Text)
#[derive(Clone, PartialEq, Eq)]
pub struct Span {
    content: SpanContent,
    pub style: Option<Style>,
}

impl Span {
    pub fn new(text: impl IntoSpan) -> Self {
        text.into_span()
    }

    pub fn styled(text: impl IntoSpan, style: Style) -> Self {
        Self::new(text).patch_style(style)
    }

    /// returns the current text content without subscribing to reactive values.
    pub fn text(&self) -> Cow<'_, str> {
        match &self.content {
            SpanContent::Static(text) => Cow::Borrowed(text),
            SpanContent::Reactive(value) => match value.source() {
                ValueSource::Plain(text) => Cow::Borrowed(text),
                _ => Cow::Owned(value.peek()),
            },
        }
    }

    pub(crate) fn content(&self) -> &SpanContent {
        &self.content
    }

    pub(crate) fn from_reactive(value: Value<String>) -> Self {
        Self {
            content: SpanContent::Reactive(value),
            style: None,
        }
    }

    pub(crate) fn from_static(text: String, style: Option<Style>) -> Self {
        Self {
            content: SpanContent::Static(text),
            style,
        }
    }

    pub(crate) fn patch_style(mut self, style: Style) -> Self {
        self.style = Some(self.style.unwrap_or_default().patch(style));
        self
    }
}

impl From<&str> for Span {
    fn from(text: &str) -> Self {
        Span::from_static(text.to_owned(), None)
    }
}

impl From<String> for Span {
    fn from(text: String) -> Self {
        Span::from_static(text, None)
    }
}
