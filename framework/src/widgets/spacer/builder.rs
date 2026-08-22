use kursor_core::component::blueprint::{Blueprint, IntoBlueprint};

use super::{Spacer, SpacerProps};

pub struct SpacerBuilder {
    size: u16,
}

impl SpacerBuilder {
    pub(crate) fn new() -> Self {
        Self { size: 0 }
    }

    pub fn size(mut self, size: u16) -> Self {
        self.size = size;
        self
    }
}

impl IntoBlueprint for SpacerBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Spacer>(SpacerProps { size: self.size })]
    }
}
