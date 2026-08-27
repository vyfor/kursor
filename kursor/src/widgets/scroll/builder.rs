use std::sync::Arc;

use kursor_core::{
    component::{
        behavior::{Behavior, BehaviorBuilder},
        blueprint::{Blueprint, IntoBlueprint},
    },
    layout::ScrollDirection,
    state::IntoValue,
};

use super::{Scroll, ScrollIntent, ScrollProps, ScrollState, WheelScroll};

pub struct ScrollBuilder {
    props: ScrollProps,
    children: Vec<Blueprint>,
}

impl Default for ScrollBuilder {
    fn default() -> Self {
        Self {
            props: ScrollProps {
                direction: ScrollDirection::Vertical.into_value(),
                behavior: Arc::new(WheelScroll::default()),
            },
            children: Vec::new(),
        }
    }
}

impl ScrollBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn direction(mut self, direction: impl IntoValue<ScrollDirection>) -> Self {
        self.props.direction = direction.into_value();
        self
    }

    pub fn vertical(mut self) -> Self {
        self.props.direction = ScrollDirection::Vertical.into_value();
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.props.direction = ScrollDirection::Horizontal.into_value();
        self
    }

    pub fn both(mut self) -> Self {
        self.props.direction = ScrollDirection::Both.into_value();
        self
    }

    pub fn children(mut self, children: impl IntoBlueprint) -> Self {
        self.children.extend(children.into_blueprint());
        self
    }

    pub fn behavior<B>(mut self, behavior: B) -> Self
    where
        B: Behavior<State = ScrollState, Intent = ScrollIntent>,
    {
        self.props.behavior = Arc::new(behavior);
        self
    }

    pub fn build(self) -> Blueprint {
        Blueprint::new::<Scroll>(self.props).children(self.children)
    }
}

impl BehaviorBuilder for ScrollBuilder {
    type State = ScrollState;
    type Intent = ScrollIntent;

    fn behavior<B>(self, behavior: B) -> Self
    where
        B: Behavior<State = Self::State, Intent = Self::Intent>,
    {
        self.behavior(behavior)
    }
}

impl IntoBlueprint for ScrollBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}

impl From<ScrollBuilder> for Blueprint {
    fn from(builder: ScrollBuilder) -> Self {
        builder.build()
    }
}
