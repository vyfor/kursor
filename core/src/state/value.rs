use super::{atom::Atom, memo::Memo, slot::State};

pub enum Value<T: State> {
    Static(T),
    Atom(Atom<T>),
    Memo(Memo<T>),
}

impl<T: State> Value<T> {
    pub fn static_value(value: T) -> Self {
        Self::Static(value)
    }

    pub fn atom(atom: Atom<T>) -> Self {
        Self::Atom(atom)
    }

    pub fn memo(memo: Memo<T>) -> Self {
        Self::Memo(memo)
    }

    pub fn get(&self) -> T {
        match self {
            Self::Static(value) => value.clone(),
            Self::Atom(atom) => atom.read(),
            Self::Memo(memo) => memo.get(),
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        match self {
            Self::Static(value) => f(value),
            Self::Atom(atom) => atom.with(f),
            Self::Memo(memo) => memo.with(f),
        }
    }
}

impl<T: State> Clone for Value<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Static(value) => Self::Static(value.clone()),
            Self::Atom(atom) => Self::Atom(*atom),
            Self::Memo(memo) => Self::Memo(memo.clone()),
        }
    }
}

impl<T: State> PartialEq for Value<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Static(a), Self::Static(b)) => a == b,
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
        Value::Static(self.0)
    }
}

impl IntoValue<String> for String {
    fn into_value(self) -> Value<String> {
        Value::Static(self)
    }
}

impl IntoValue<String> for &str {
    fn into_value(self) -> Value<String> {
        Value::Static(self.to_owned())
    }
}
