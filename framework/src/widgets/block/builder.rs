use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    render::style::Style,
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

    pub fn border(mut self, border: Border) -> Self {
        self.props.border = border;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.props.style = Some(style);
        self
    }
}

impl IntoBlueprint for BlockBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Block>(self.props).children(self.children)]
    }
}
