use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    layout::Alignment,
    state::{IntoValue, Value},
};

use super::{Align, AlignProps};

pub struct AlignBuilder {
    alignment: Value<Alignment>,
    children: Vec<Blueprint>,
}

impl AlignBuilder {
    pub(crate) fn new(child: impl IntoBlueprint) -> Self {
        Self {
            alignment: Value::plain(Alignment::TOP_LEFT),
            children: child.into_blueprint(),
        }
    }

    pub fn alignment(mut self, alignment: impl IntoValue<Alignment>) -> Self {
        self.alignment = alignment.into_value();
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
