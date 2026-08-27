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

impl Default for AlignBuilder {
    fn default() -> Self {
        Self {
            alignment: Value::plain(Alignment::TOP_LEFT),
            children: Vec::new(),
        }
    }
}

impl AlignBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alignment(mut self, alignment: impl IntoValue<Alignment>) -> Self {
        self.alignment = alignment.into_value();
        self
    }

    pub fn top_left(mut self) -> Self {
        self.alignment = Value::plain(Alignment::TOP_LEFT);
        self
    }

    pub fn top_center(mut self) -> Self {
        self.alignment = Value::plain(Alignment::TOP_CENTER);
        self
    }

    pub fn top_right(mut self) -> Self {
        self.alignment = Value::plain(Alignment::TOP_RIGHT);
        self
    }

    pub fn center_left(mut self) -> Self {
        self.alignment = Value::plain(Alignment::CENTER_LEFT);
        self
    }

    pub fn center(mut self) -> Self {
        self.alignment = Value::plain(Alignment::CENTER);
        self
    }

    pub fn center_right(mut self) -> Self {
        self.alignment = Value::plain(Alignment::CENTER_RIGHT);
        self
    }

    pub fn bottom_left(mut self) -> Self {
        self.alignment = Value::plain(Alignment::BOTTOM_LEFT);
        self
    }

    pub fn bottom_center(mut self) -> Self {
        self.alignment = Value::plain(Alignment::BOTTOM_CENTER);
        self
    }

    pub fn bottom_right(mut self) -> Self {
        self.alignment = Value::plain(Alignment::BOTTOM_RIGHT);
        self
    }

    pub fn children(mut self, children: impl IntoBlueprint) -> Self {
        self.children.extend(children.into_blueprint());
        self
    }

    pub fn build(self) -> Blueprint {
        Blueprint::new::<Align>(AlignProps {
            alignment: self.alignment,
        })
        .children(self.children)
    }
}

impl IntoBlueprint for AlignBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}

impl From<AlignBuilder> for Blueprint {
    fn from(builder: AlignBuilder) -> Self {
        builder.build()
    }
}
