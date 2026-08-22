use kursor_core::component::blueprint::{Blueprint, IntoBlueprint};

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

    pub fn gap(mut self, gap: u16) -> Self {
        self.props.gap = gap;
        self
    }

    pub fn line_gap(mut self, gap: u16) -> Self {
        self.props.line_gap = gap;
        self
    }
}

impl IntoBlueprint for WrapBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Wrap>(self.props).children(self.children)]
    }
}
