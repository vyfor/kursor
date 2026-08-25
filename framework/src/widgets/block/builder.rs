use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    render::style::Style,
    state::IntoValue,
};

use super::{Block, BlockProps, Border};

pub struct BlockBuilder {
    props: BlockProps,
    children: Vec<Blueprint>,
}

impl BlockBuilder {
    pub(crate) fn new(child: impl IntoBlueprint) -> Self {
        Self {
            props: BlockProps::default(),
            children: child.into_blueprint(),
        }
    }

    pub fn border(mut self, border: impl IntoValue<Border>) -> Self {
        self.props.border = border.into_value();
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.props.style = Some(style).into_value();
        self
    }
}

impl IntoBlueprint for BlockBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Block>(self.props).children(self.children)]
    }
}
