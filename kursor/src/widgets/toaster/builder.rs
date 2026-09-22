use std::{rc::Rc, time::Duration};

#[cfg(feature = "fx")]
use kursor_core::layout::Direction;
use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    layout::Alignment,
};

#[cfg(feature = "fx")]
use crate::fx::{Feather, Fx, Spread, reveal};

use super::{Toaster, ToasterProps};

pub struct ToasterBuilder {
    props: ToasterProps,
}

impl ToasterBuilder {
    pub fn new(child: impl IntoBlueprint) -> Self {
        let mut blueprints = child.into_blueprint();
        let content = blueprints.pop().expect("no child");

        Self {
            props: ToasterProps {
                gap: 1,
                soft_limit: None,
                hard_limit: None,
                pace: Duration::from_millis(60),
                delay: Duration::from_millis(40),
                anchor: Alignment::TOP_RIGHT,
                #[cfg(feature = "fx")]
                enter: Box::new(
                    reveal(Duration::from_millis(300))
                        .spread(Spread::towards(Direction::Left))
                        .feather(Feather::soft(2.0)),
                ),
                #[cfg(feature = "fx")]
                exit: Box::new(
                    reveal(Duration::from_millis(300))
                        .spread(Spread::towards(Direction::Right))
                        .feather(Feather::soft(2.0))
                        .out(),
                ),
                content: Rc::new(content),
            },
        }
    }

    #[cfg(feature = "fx")]
    pub fn enter(mut self, fx: impl Fx) -> Self {
        self.props.enter = Box::new(fx);
        self
    }

    #[cfg(feature = "fx")]
    pub fn exit(mut self, fx: impl Fx) -> Self {
        self.props.exit = Box::new(fx);
        self
    }

    pub fn gap(mut self, gap: u16) -> Self {
        self.props.gap = gap;
        self
    }

    pub fn soft_limit(mut self, limit: usize) -> Self {
        self.props.soft_limit = Some(limit);
        self
    }

    pub fn hard_limit(mut self, limit: usize) -> Self {
        self.props.hard_limit = Some(limit);
        self
    }

    pub fn pace(mut self, pace: Duration) -> Self {
        self.props.pace = pace;
        self
    }

    pub fn delay(mut self, delay: Duration) -> Self {
        self.props.delay = delay;
        self
    }

    pub fn anchor(mut self, anchor: Alignment) -> Self {
        self.props.anchor = anchor;
        self
    }

    pub fn build(self) -> Blueprint {
        Toaster::with(self.props)
    }
}

impl IntoBlueprint for ToasterBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}
