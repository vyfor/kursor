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
    pub(crate) fn new(text: impl super::IntoText) -> Self {
        Self {
            props: TextProps {
                text: text.into_text(),
                style: Value::plain(None),
                wrap: Value::plain(WrapMode::None),
            },
        }
    }

    pub fn style(mut self, style: impl IntoValue<Option<Style>>) -> Self {
        self.props.style = style.into_value();
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
