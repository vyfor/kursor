use kursor_core::component::blueprint::{Blueprint, IntoBlueprint};

use super::{Row, RowProps};

pub struct RowBuilder {
    props: RowProps,
    children: Vec<Blueprint>,
}

impl RowBuilder {
    pub(crate) fn new(children: impl IntoBlueprint) -> Self {
        Self {
            props: RowProps::default(),
            children: children.into_blueprint(),
        }
    }

    pub fn gap(mut self, gap: u16) -> Self {
        self.props.gap = gap;
        self
    }
}

impl IntoBlueprint for RowBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Row>(self.props).children(self.children)]
    }
}
