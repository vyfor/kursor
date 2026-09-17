use kursor_core::{
    render::{color::Color, style::Style},
    state::{Atom, Memo, Signal, Value},
};

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
        Span::from_static(self.to_owned(), None)
    }
}

impl IntoSpan for String {
    fn into_span(self) -> Span {
        Span::from_static(self, None)
    }
}

impl IntoSpan for Span {
    fn into_span(self) -> Span {
        self
    }
}

impl IntoSpan for Value<String> {
    fn into_span(self) -> Span {
        Span::from_reactive(self)
    }
}

impl IntoSpan for Signal<String> {
    fn into_span(self) -> Span {
        Span::from_reactive(Value::signal(self))
    }
}

impl IntoSpan for Atom<String> {
    fn into_span(self) -> Span {
        Span::from_reactive(Value::atom(self))
    }
}

impl IntoSpan for Memo<String> {
    fn into_span(self) -> Span {
        Span::from_reactive(Value::memo(self))
    }
}

impl<T> IntoSpanExt for T where T: IntoSpan {}
