use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    state::IntoValue,
};

use super::{Column, ColumnProps};

pub struct ColumnBuilder {
    props: ColumnProps,
    children: Vec<Blueprint>,
}

impl ColumnBuilder {
    pub(crate) fn new(children: impl IntoBlueprint) -> Self {
        Self {
            props: ColumnProps::default(),
            children: children.into_blueprint(),
        }
    }

    pub fn gap(mut self, gap: impl IntoValue<u16>) -> Self {
        self.props.gap = gap.into_value();
        self
    }
}

impl IntoBlueprint for ColumnBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Column>(self.props).children(self.children)]
    }
}
