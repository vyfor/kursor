use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    state::{IntoValue, Value},
};

use super::{Spacer, SpacerProps};

pub struct SpacerBuilder {
    size: Value<u16>,
}

impl SpacerBuilder {
    pub(crate) fn new() -> Self {
        Self {
            size: Value::plain(0),
        }
    }

    pub fn size(mut self, size: impl IntoValue<u16>) -> Self {
        self.size = size.into_value();
        self
    }

    pub fn build(self) -> Blueprint {
        Blueprint::new::<Spacer>(SpacerProps { size: self.size })
    }
}

impl IntoBlueprint for SpacerBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}
