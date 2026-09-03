use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    layout::Insets,
    state::{IntoValue, Value},
};

use super::{Padding, PaddingProps};

#[derive(Default)]
pub struct PaddingBuilder {
    insets: Value<Insets>,
    children: Vec<Blueprint>,
}

impl PaddingBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insets(mut self, insets: impl IntoValue<Insets>) -> Self {
        self.insets = insets.into_value();
        self
    }

    pub fn all(mut self, value: u16) -> Self {
        self.insets = Value::plain(Insets::all(value));
        self
    }

    pub fn horizontal(mut self, value: u16) -> Self {
        let mut insets = self.insets.as_plain().copied().unwrap_or_default();
        insets.left = value;
        insets.right = value;
        self.insets = Value::plain(insets);
        self
    }

    pub fn vertical(mut self, value: u16) -> Self {
        let mut insets = self.insets.as_plain().copied().unwrap_or_default();
        insets.top = value;
        insets.bottom = value;
        self.insets = Value::plain(insets);
        self
    }

    pub fn children(mut self, children: impl IntoBlueprint) -> Self {
        self.children.extend(children.into_blueprint());
        self
    }

    pub fn build(self) -> Blueprint {
        Blueprint::new::<Padding>(PaddingProps {
            insets: self.insets,
        })
        .children(self.children)
    }
}

impl IntoBlueprint for PaddingBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}

impl From<PaddingBuilder> for Blueprint {
    fn from(builder: PaddingBuilder) -> Self {
        builder.build()
    }
}
