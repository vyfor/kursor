use crate::{
    layout::{Alignment, Insets, Margin, Orientation, ScrollDirection, WrapMode},
    render::{color::Color, style::Style},
    theme::Theme,
};

use super::Transition;
use super::{LocalState, atom::Atom, memo::Memo, signal::Signal};

#[derive(Clone)]
pub struct Value<T: LocalState> {
    pub(crate) source: ValueSource<T>,
    pub(crate) transition: Option<Transition>,
}

#[derive(Clone)]
pub enum ValueSource<T: LocalState> {
    Plain(T),
    Signal(Signal<T>),
    Atom(Atom<T>),
    Memo(Memo<T>),
}

impl<T: LocalState> Value<T> {
    pub fn plain(value: T) -> Self {
        Self {
            source: ValueSource::Plain(value),
            transition: None,
        }
    }

    pub fn atom(atom: Atom<T>) -> Self {
        Self {
            source: ValueSource::Atom(atom),
            transition: None,
        }
    }

    pub fn signal(signal: Signal<T>) -> Self {
        Self {
            source: ValueSource::Signal(signal),
            transition: None,
        }
    }

    pub fn memo(memo: Memo<T>) -> Self {
        Self {
            source: ValueSource::Memo(memo),
            transition: None,
        }
    }

    pub fn source(&self) -> &ValueSource<T> {
        &self.source
    }

    pub fn as_plain(&self) -> Option<&T> {
        match &self.source {
            ValueSource::Plain(value) => Some(value),
            _ => None,
        }
    }

    pub fn get(&self) -> T {
        match &self.source {
            ValueSource::Plain(value) => value.clone(),
            ValueSource::Signal(signal) => signal.read(),
            ValueSource::Atom(atom) => atom.read(),
            ValueSource::Memo(memo) => memo.get(),
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        match &self.source {
            ValueSource::Plain(value) => f(value),
            ValueSource::Signal(signal) => signal.with(f),
            ValueSource::Atom(atom) => atom.with(f),
            ValueSource::Memo(memo) => memo.with(f),
        }
    }

    pub fn transition(mut self, transition: impl Into<Transition>) -> Self {
        self.transition = Some(transition.into());
        self
    }

    pub fn get_transition(&self) -> Option<Transition> {
        self.transition
    }
}

impl<T: LocalState + Default> Default for Value<T> {
    fn default() -> Self {
        Self::plain(T::default())
    }
}

impl<T: LocalState> PartialEq for Value<T> {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.transition == other.transition
    }
}

impl<T: LocalState> Eq for Value<T> {}

impl<T: LocalState> PartialEq for ValueSource<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Plain(a), Self::Plain(b)) => a == b,
            (Self::Signal(a), Self::Signal(b)) => a == b,
            (Self::Atom(a), Self::Atom(b)) => a == b,
            (Self::Memo(a), Self::Memo(b)) => a == b,
            _ => false,
        }
    }
}

impl<T: LocalState> Eq for ValueSource<T> {}

impl<T: LocalState> From<T> for Value<T> {
    fn from(value: T) -> Self {
        Value::plain(value)
    }
}

impl<T: LocalState> From<Signal<T>> for Value<T> {
    fn from(signal: Signal<T>) -> Self {
        Value::signal(signal)
    }
}

impl<T: LocalState> From<Memo<T>> for Value<T> {
    fn from(memo: Memo<T>) -> Self {
        Value::memo(memo)
    }
}

impl<T: LocalState + Send + Sync> From<Atom<T>> for Value<T> {
    fn from(atom: Atom<T>) -> Self {
        Value::atom(atom)
    }
}

pub trait IntoValue<T: LocalState> {
    fn into_value(self) -> Value<T>;

    fn transition(self, transition: impl Into<Transition>) -> Value<T>
    where
        Self: Sized,
    {
        self.into_value().transition(transition)
    }
}

#[macro_export]
macro_rules! into_value {
    ($($type:ty),+ $(,)?) => {
        $(
            impl $crate::state::value::IntoValue<$type> for $type {
                fn into_value(self) -> $crate::state::value::Value<$type> {
                    $crate::state::value::Value::plain(self)
                }
            }
        )+
    };
}

impl<T: LocalState> IntoValue<T> for Value<T> {
    fn into_value(self) -> Value<T> {
        self
    }
}

impl<T: LocalState + Send + Sync> IntoValue<T> for Atom<T> {
    fn into_value(self) -> Value<T> {
        Value::atom(self)
    }
}

impl<T: LocalState> IntoValue<T> for Signal<T> {
    fn into_value(self) -> Value<T> {
        Value::signal(self)
    }
}

impl<T: LocalState> IntoValue<T> for Memo<T> {
    fn into_value(self) -> Value<T> {
        Value::memo(self)
    }
}

pub struct Plain<T: LocalState>(pub T);

impl<T: LocalState> Plain<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T: LocalState> IntoValue<T> for Plain<T> {
    fn into_value(self) -> Value<T> {
        Value::plain(self.0)
    }
}

into_value!(
    bool,
    char,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    String,
    Alignment,
    Insets,
    Margin,
    Orientation,
    ScrollDirection,
    WrapMode,
    Color,
    Style,
    Theme,
);

impl<T: LocalState> IntoValue<Option<T>> for Option<T> {
    fn into_value(self) -> Value<Option<T>> {
        Value::plain(self)
    }
}

impl IntoValue<Option<Color>> for Color {
    fn into_value(self) -> Value<Option<Color>> {
        Value::plain(Some(self))
    }
}

impl IntoValue<Option<Style>> for Style {
    fn into_value(self) -> Value<Option<Style>> {
        Value::plain(Some(self))
    }
}

impl IntoValue<String> for &str {
    fn into_value(self) -> Value<String> {
        Value::plain(self.to_owned())
    }
}
