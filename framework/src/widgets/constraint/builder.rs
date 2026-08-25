use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    state::IntoValue,
};

use super::{Constraint, ConstraintProps};

pub struct ConstraintBuilder {
    props: ConstraintProps,
    children: Vec<Blueprint>,
}

impl ConstraintBuilder {
    pub(crate) fn new(child: impl IntoBlueprint) -> Self {
        Self {
            props: ConstraintProps::default(),
            children: child.into_blueprint(),
        }
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
}

impl IntoBlueprint for ConstraintBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Constraint>(self.props).children(self.children)]
    }
}
