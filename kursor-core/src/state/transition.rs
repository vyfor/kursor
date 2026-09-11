use std::any::TypeId;

/// identifier for an animation/transition on a widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Channel {
    Name(&'static str),
    Index(u64),
    Type(TypeId),
}

pub trait IntoChannel {
    fn into_channel(self) -> Channel;
}

impl IntoChannel for &'static str {
    fn into_channel(self) -> Channel {
        Channel::Name(self)
    }
}

impl IntoChannel for usize {
    fn into_channel(self) -> Channel {
        Channel::Index(self as u64)
    }
}

impl IntoChannel for u64 {
    fn into_channel(self) -> Channel {
        Channel::Index(self)
    }
}

impl IntoChannel for u32 {
    fn into_channel(self) -> Channel {
        Channel::Index(self as u64)
    }
}

impl IntoChannel for i32 {
    fn into_channel(self) -> Channel {
        Channel::Index(self as u64)
    }
}

impl IntoChannel for Channel {
    fn into_channel(self) -> Channel {
        self
    }
}

impl IntoChannel for () {
    fn into_channel(self) -> Channel {
        Channel::Index(0)
    }
}

#[cfg(feature = "animate")]
pub use animate::{SpringSpec, Transition, TweenSpec};

#[cfg(not(feature = "animate"))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition;
