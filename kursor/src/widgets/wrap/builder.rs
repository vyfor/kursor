use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    state::IntoValue,
};

use super::{Wrap, WrapProps};

pub struct WrapBuilder {
    props: WrapProps,
    children: Vec<Blueprint>,
}

impl WrapBuilder {
    pub(crate) fn new(children: impl IntoBlueprint) -> Self {
        Self {
            props: WrapProps::default(),
            children: children.into_blueprint(),
        }
    }

    pub fn gap(mut self, gap: impl IntoValue<u16>) -> Self {
        self.props.gap = gap.into_value();
        self
    }

    pub fn line_gap(mut self, gap: impl IntoValue<u16>) -> Self {
        self.props.line_gap = gap.into_value();
        self
    }

    pub fn build(self) -> Blueprint {
        Blueprint::new::<Wrap>(self.props).children(self.children)
    }
}

impl IntoBlueprint for WrapBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}
