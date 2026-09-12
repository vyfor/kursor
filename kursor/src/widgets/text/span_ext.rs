use kursor_core::render::{color::Color, style::Style};

use super::Span;

pub trait IntoSpan {
    fn into_span(self) -> Span;
}

pub trait IntoSpanExt: IntoSpan + Sized {
    fn styled(self, style: Style) -> Span {
        self.into_span().patch_style(style)
    }

    fn fg(self, color: Color) -> Span {
        self.styled(Style::new().fg(color))
    }

    fn bg(self, color: Color) -> Span {
        self.styled(Style::new().bg(color))
    }

    fn bold(self) -> Span {
        self.styled(Style::new().bold())
    }

    fn dim(self) -> Span {
        self.styled(Style::new().dim())
    }

    fn italic(self) -> Span {
        self.styled(Style::new().italic())
    }

    fn underlined(self) -> Span {
        self.styled(Style::new().underline())
    }

    fn reversed(self) -> Span {
        self.styled(Style::new().reverse())
    }
}

impl IntoSpan for &str {
    fn into_span(self) -> Span {
        Span::new(self)
    }
}

impl IntoSpan for String {
    fn into_span(self) -> Span {
        Span::new(self)
    }
}

impl IntoSpan for Span {
    fn into_span(self) -> Span {
        self
    }
}

impl<T> IntoSpanExt for T where T: IntoSpan {}

impl Span {
    fn patch_style(mut self, style: Style) -> Self {
        self.style = Some(self.style.unwrap_or_default().patch(style));
        self
    }
}
