use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    layout::Alignment,
};

use super::{Align, AlignProps};

pub struct AlignBuilder {
    alignment: Alignment,
    children: Vec<Blueprint>,
}

impl AlignBuilder {
    pub(crate) fn new(child: impl IntoBlueprint) -> Self {
        Self {
            alignment: Alignment::TOP_LEFT,
            children: child.into_blueprint(),
        }
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl IntoBlueprint for AlignBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![
            Blueprint::new::<Align>(AlignProps {
                alignment: self.alignment,
            })
            .children(self.children),
        ]
    }
}
