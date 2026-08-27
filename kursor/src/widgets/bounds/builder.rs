use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    state::IntoValue,
};

use super::{Bounds, BoundsProps};

#[derive(Default)]
pub struct BoundsBuilder {
    props: BoundsProps,
    children: Vec<Blueprint>,
}

impl BoundsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn exact(mut self, width: impl IntoValue<u16>, height: impl IntoValue<u16>) -> Self {
        let width = width.into_value();
        let height = height.into_value();
        self.props.min_width = Some(width.clone());
        self.props.max_width = Some(width);
        self.props.min_height = Some(height.clone());
        self.props.max_height = Some(height);
        self
    }

    pub fn min_width(mut self, width: impl IntoValue<u16>) -> Self {
        self.props.min_width = Some(width.into_value());
        self
    }

    pub fn max_width(mut self, width: impl IntoValue<u16>) -> Self {
        self.props.max_width = Some(width.into_value());
        self
    }

    pub fn min_height(mut self, height: impl IntoValue<u16>) -> Self {
        self.props.min_height = Some(height.into_value());
        self
    }

    pub fn max_height(mut self, height: impl IntoValue<u16>) -> Self {
        self.props.max_height = Some(height.into_value());
        self
    }

    pub fn children(mut self, children: impl IntoBlueprint) -> Self {
        self.children.extend(children.into_blueprint());
        self
    }

    pub fn build(self) -> Blueprint {
        Blueprint::new::<Bounds>(self.props).children(self.children)
    }
}

impl IntoBlueprint for BoundsBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}

impl From<BoundsBuilder> for Blueprint {
    fn from(builder: BoundsBuilder) -> Self {
        builder.build()
    }
}
