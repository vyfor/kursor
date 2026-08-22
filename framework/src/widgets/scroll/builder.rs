use std::sync::Arc;

use kursor_core::{
    component::{
        behavior::Behavior,
        blueprint::{Blueprint, IntoBlueprint},
    },
    layout::ScrollDirection,
};

use super::{Scroll, ScrollIntent, ScrollProps, ScrollState, WheelScroll};

pub struct ScrollBuilder {
    props: ScrollProps,
    children: Vec<Blueprint>,
}

impl ScrollBuilder {
    pub(crate) fn new(child: impl IntoBlueprint) -> Self {
        Self {
            props: ScrollProps {
                direction: ScrollDirection::Vertical,
                behavior: Arc::new(WheelScroll::default()),
            },
            children: child.into_blueprint(),
        }
    }

    pub fn direction(mut self, direction: ScrollDirection) -> Self {
        self.props.direction = direction;
        self
    }

    pub fn behavior<B>(mut self, behavior: B) -> Self
    where
        B: Behavior<State = ScrollState, Intent = ScrollIntent>,
    {
        self.props.behavior = Arc::new(behavior);
        self
    }
}

impl IntoBlueprint for ScrollBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Scroll>(self.props).children(self.children)]
    }
}
