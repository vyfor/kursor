pub mod key;
pub mod mouse;

use crate::event::{key::KeyEvent, mouse::MouseEvent};

pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    FocusIn,
    FocusOut,
    WindowFocus(bool),
    Paste(String),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EventResult {
    #[default]
    Ignored,
    Consumed,
    Stop,
}

impl EventResult {
    pub fn is_handled(self) -> bool {
        !matches!(self, Self::Ignored)
    }

    pub fn should_stop(self) -> bool {
        matches!(self, Self::Stop)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub system: bool,
    pub meta: bool,
    pub hyper: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Capture,
    Bubble,
    Global,
}
