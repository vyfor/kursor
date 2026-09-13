use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    render::style::Style,
    state::{IntoValue, Value},
};

use super::Styled;

pub struct StyledBuilder {
    style: Value<Style>,
    children: Vec<Blueprint>,
}

impl StyledBuilder {
    pub(crate) fn new(child: impl IntoBlueprint) -> Self {
        Self {
            style: Value::plain(Style::default()),
            children: child.into_blueprint(),
        }
    }

    pub fn style(mut self, style: impl IntoValue<Style>) -> Self {
        self.style = style.into_value();
        self
    }

    pub fn build(self) -> Blueprint {
        Blueprint::new::<Styled>(self.style).children(self.children)
    }
}

impl IntoBlueprint for StyledBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}

impl From<StyledBuilder> for Blueprint {
    fn from(builder: StyledBuilder) -> Self {
        builder.build()
    }
}
