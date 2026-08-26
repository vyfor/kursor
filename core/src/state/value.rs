#[cfg(feature = "animate")]
use super::Animated;
use super::{LocalState, atom::Atom, memo::Memo, signal::Signal};
#[cfg(feature = "animate")]
use std::rc::Rc;

pub enum Value<T: LocalState> {
    Plain(T),
    Signal(Signal<T>),
    Atom(Atom<T>),
    Memo(Memo<T>),
    #[cfg(feature = "animate")]
    Animated(Animated<T>),
}

impl<T: LocalState> Value<T> {
    pub fn plain(value: T) -> Self {
        Self::Plain(value)
    }

    pub fn atom(atom: Atom<T>) -> Self {
        Self::Atom(atom)
    }

    pub fn signal(signal: Signal<T>) -> Self {
        Self::Signal(signal)
    }

    pub fn memo(memo: Memo<T>) -> Self {
        Self::Memo(memo)
    }

    pub fn get(&self) -> T {
        match self {
            Self::Plain(value) => value.clone(),
            Self::Signal(signal) => signal.read(),
            Self::Atom(atom) => atom.read(),
            Self::Memo(memo) => memo.get(),
            #[cfg(feature = "animate")]
            Self::Animated(animated) => animated.get(),
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        match self {
            Self::Plain(value) => f(value),
            Self::Signal(signal) => signal.with(f),
            Self::Atom(atom) => atom.with(f),
            Self::Memo(memo) => memo.with(f),
            #[cfg(feature = "animate")]
            Self::Animated(animated) => f(&animated.get()),
        }
    }
}

impl<T: LocalState> Clone for Value<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Plain(value) => Self::Plain(value.clone()),
            Self::Signal(signal) => Self::Signal(signal.clone()),
            Self::Atom(atom) => Self::Atom(*atom),
            Self::Memo(memo) => Self::Memo(memo.clone()),
            #[cfg(feature = "animate")]
            Self::Animated(animated) => Self::Animated(animated.clone()),
        }
    }
}

impl<T: LocalState> PartialEq for Value<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Plain(a), Self::Plain(b)) => a == b,
            (Self::Signal(a), Self::Signal(b)) => a == b,
            (Self::Atom(a), Self::Atom(b)) => a == b,
            (Self::Memo(a), Self::Memo(b)) => a == b,
            #[cfg(feature = "animate")]
            (Self::Animated(a), Self::Animated(b)) => Rc::ptr_eq(&a.inner, &b.inner),
            _ => false,
        }
    }
}

impl<T: LocalState> Eq for Value<T> {}

#[cfg(feature = "animate")]
impl<T: LocalState> Value<T> {
    pub fn animate<A>(self, animation: A) -> Self
    where
        A: animate::Animation<Value = T> + 'static,
    {
        Self::Animated(Animated::with(self, animation))
    }
}

pub trait IntoValue<T: LocalState> {
    fn into_value(self) -> Value<T>;

    #[cfg(feature = "animate")]
    fn animate<A>(self, animation: A) -> Value<T>
    where
        A: animate::Animation<Value = T> + 'static,
        Self: Sized,
    {
        self.into_value().animate(animation)
    }
}

#[macro_export]
macro_rules! into_value {
    ($($type:ty),+ $(,)?) => {
        $(
            impl $crate::state::value::IntoValue<$type> for $type {
                fn into_value(self) -> $crate::state::value::Value<$type> {
                    $crate::state::value::Value::Plain(self)
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
        Value::Atom(self)
    }
}

impl<T: LocalState> IntoValue<T> for Signal<T> {
    fn into_value(self) -> Value<T> {
        Value::Signal(self)
    }
}

impl<T: LocalState> IntoValue<T> for Memo<T> {
    fn into_value(self) -> Value<T> {
        Value::Memo(self)
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
        Value::Plain(self.0)
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
    crate::layout::Alignment,
    crate::layout::Insets,
    crate::layout::Orientation,
    crate::layout::ScrollDirection,
    crate::layout::WrapMode,
    crate::render::style::Style,
    crate::theme::Theme,
);

impl<T: LocalState> IntoValue<Option<T>> for Option<T> {
    fn into_value(self) -> Value<Option<T>> {
        Value::Plain(self)
    }
}

impl IntoValue<String> for &str {
    fn into_value(self) -> Value<String> {
        Value::Plain(self.to_owned())
    }
}
