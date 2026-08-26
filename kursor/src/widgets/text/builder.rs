use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    layout::WrapMode,
    render::style::Style,
    state::value::{IntoValue, Value},
};

use super::{Text, TextProps};

pub struct TextBuilder {
    props: TextProps,
}

impl TextBuilder {
    pub(crate) fn new(text: impl IntoValue<String>) -> Self {
        Self {
            props: TextProps {
                text: text.into_value(),
                style: Value::plain(None),
                wrap: Value::plain(WrapMode::None),
            },
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.props.style = Value::plain(Some(style));
        self
    }

    pub fn wrap(mut self, mode: impl IntoValue<WrapMode>) -> Self {
        self.props.wrap = mode.into_value();
        self
    }
}

impl IntoBlueprint for TextBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Text>(self.props)]
    }
}
