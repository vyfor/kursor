use unicode_width::UnicodeWidthStr;

use super::Span;

#[derive(Clone, PartialEq, Eq, Default)]
pub struct Line {
    pub spans: Vec<Span>,
}

impl Line {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_spans(spans: impl IntoIterator<Item = Span>) -> Self {
        Self {
            spans: spans.into_iter().collect(),
        }
    }

    pub fn span(mut self, span: impl Into<Span>) -> Self {
        self.spans.push(span.into());
        self
    }

    pub fn width(&self) -> usize {
        self.spans
            .iter()
            .map(|span| UnicodeWidthStr::width(span.text.as_str()))
            .sum()
    }
}

impl From<&str> for Line {
    fn from(text: &str) -> Self {
        Self::from_spans([Span::from(text)])
    }
}

impl From<String> for Line {
    fn from(text: String) -> Self {
        Self::from_spans([Span::from(text)])
    }
}

impl From<Span> for Line {
    fn from(span: Span) -> Self {
        Self::from_spans([span])
    }
}

impl From<Vec<Span>> for Line {
    fn from(spans: Vec<Span>) -> Self {
        Self::from_spans(spans)
    }
}

impl<const N: usize> From<[Span; N]> for Line {
    fn from(spans: [Span; N]) -> Self {
        Self::from_spans(spans)
    }
}

impl std::iter::FromIterator<Span> for Line {
    fn from_iter<T: IntoIterator<Item = Span>>(iter: T) -> Self {
        Self::from_spans(iter)
    }
}
