use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    layout::Insets,
};

use super::{Padding, PaddingProps};

pub struct PaddingBuilder {
    insets: Insets,
    children: Vec<Blueprint>,
}

impl PaddingBuilder {
    pub(crate) fn new(child: impl IntoBlueprint) -> Self {
        Self {
            insets: Insets::default(),
            children: child.into_blueprint(),
        }
    }

    pub fn insets(mut self, insets: Insets) -> Self {
        self.insets = insets;
        self
    }

    pub fn all(mut self, value: u16) -> Self {
        self.insets = Insets::all(value);
        self
    }

    pub fn horizontal(mut self, value: u16) -> Self {
        self.insets.left = value;
        self.insets.right = value;
        self
    }

    pub fn vertical(mut self, value: u16) -> Self {
        self.insets.top = value;
        self.insets.bottom = value;
        self
    }
}

impl IntoBlueprint for PaddingBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![
            Blueprint::new::<Padding>(PaddingProps {
                insets: self.insets,
            })
            .children(self.children),
        ]
    }
}
