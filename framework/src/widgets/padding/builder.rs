use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    layout::Insets,
    state::{IntoValue, Value},
};

use super::{Padding, PaddingProps};

pub struct PaddingBuilder {
    insets: Value<Insets>,
    children: Vec<Blueprint>,
}

impl PaddingBuilder {
    pub(crate) fn new(child: impl IntoBlueprint) -> Self {
        Self {
            insets: Value::plain(Insets::default()),
            children: child.into_blueprint(),
        }
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
        let mut insets = match self.insets {
            Value::Plain(insets) => insets,
            _ => Insets::default(),
        };
        insets.left = value;
        insets.right = value;
        self.insets = Value::plain(insets);
        self
    }

    pub fn vertical(mut self, value: u16) -> Self {
        let mut insets = match self.insets {
            Value::Plain(insets) => insets,
            _ => Insets::default(),
        };
        insets.top = value;
        insets.bottom = value;
        self.insets = Value::plain(insets);
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
