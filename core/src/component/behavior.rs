use crate::{
    event::{Event, Phase},
    layout::rect::Rect,
};

pub struct BehaviorCx {
    pub phase: Phase,
    pub rect: Rect,
}

pub trait Behavior: Send + Sync + 'static {
    type State;
    type Intent;

    fn event(&self, cx: &BehaviorCx, event: &Event, state: &Self::State) -> Option<Self::Intent>;
}

pub trait BehaviorBuilder: Sized {
    type State;
    type Intent;

    fn behavior<B>(self, behavior: B) -> Self
    where
        B: Behavior<State = Self::State, Intent = Self::Intent>;
}
