#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Integer(u64),
    String(String),
}

impl From<u64> for Key {
    fn from(value: u64) -> Self {
        Self::Integer(value)
    }
}

impl From<u32> for Key {
    fn from(value: u32) -> Self {
        Self::Integer(u64::from(value))
    }
}

impl From<usize> for Key {
    fn from(value: usize) -> Self {
        Self::Integer(value as u64)
    }
}

impl From<String> for Key {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Key {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}
