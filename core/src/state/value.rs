use super::{atom::Atom, memo::Memo, slot::State};

pub enum Value<T: State> {
    Plain(T),
    Atom(Atom<T>),
    Memo(Memo<T>),
}

impl<T: State> Value<T> {
    pub fn plain(value: T) -> Self {
        Self::Plain(value)
    }

    pub fn atom(atom: Atom<T>) -> Self {
        Self::Atom(atom)
    }

    pub fn memo(memo: Memo<T>) -> Self {
        Self::Memo(memo)
    }

    pub fn get(&self) -> T {
        match self {
            Self::Plain(value) => value.clone(),
            Self::Atom(atom) => atom.read(),
            Self::Memo(memo) => memo.get(),
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        match self {
            Self::Plain(value) => f(value),
            Self::Atom(atom) => atom.with(f),
            Self::Memo(memo) => memo.with(f),
        }
    }
}

impl<T: State> Clone for Value<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Plain(value) => Self::Plain(value.clone()),
            Self::Atom(atom) => Self::Atom(*atom),
            Self::Memo(memo) => Self::Memo(memo.clone()),
        }
    }
}

impl<T: State> PartialEq for Value<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Plain(a), Self::Plain(b)) => a == b,
            (Self::Atom(a), Self::Atom(b)) => a == b,
            (Self::Memo(a), Self::Memo(b)) => a == b,
            _ => false,
        }
    }
}

impl<T: State> Eq for Value<T> {}

pub trait IntoValue<T: State> {
    fn into_value(self) -> Value<T>;
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

impl<T: State> IntoValue<T> for Value<T> {
    fn into_value(self) -> Value<T> {
        self
    }
}

impl<T: State> IntoValue<T> for Atom<T> {
    fn into_value(self) -> Value<T> {
        Value::Atom(self)
    }
}

impl<T: State> IntoValue<T> for Memo<T> {
    fn into_value(self) -> Value<T> {
        Value::Memo(self)
    }
}

pub struct Static<T: State>(pub T);

impl<T: State> Static<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T: State> IntoValue<T> for Static<T> {
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

impl<T: State> IntoValue<Option<T>> for Option<T> {
    fn into_value(self) -> Value<Option<T>> {
        Value::Plain(self)
    }
}

impl IntoValue<String> for &str {
    fn into_value(self) -> Value<String> {
        Value::Plain(self.to_owned())
    }
}
