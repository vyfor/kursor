use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    render::style::Style,
    state::{IntoValue, Transition},
};

use super::{Block, BlockProps, Border};

#[derive(Default)]
pub struct BlockBuilder {
    props: BlockProps,
    children: Vec<Blueprint>,
}

impl BlockBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn border(mut self, border: impl IntoValue<Border>) -> Self {
        self.props.border = border.into_value();
        self
    }

    pub fn style(mut self, style: impl IntoValue<Option<Style>>) -> Self {
        self.props.style = style.into_value();
        self
    }

    pub fn transition(mut self, transition: impl Into<Transition>) -> Self {
        self.props.transition = Some(transition.into());
        self
    }

    pub fn children(mut self, children: impl IntoBlueprint) -> Self {
        self.children.extend(children.into_blueprint());
        self
    }
}

impl IntoBlueprint for BlockBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}

impl From<BlockBuilder> for Blueprint {
    fn from(builder: BlockBuilder) -> Self {
        builder.build()
    }
}

impl BlockBuilder {
    pub fn build(self) -> Blueprint {
        Blueprint::new::<Block>(self.props).children(self.children)
    }
}
