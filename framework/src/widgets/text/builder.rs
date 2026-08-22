use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    layout::WrapMode,
    render::style::Style,
};

use super::{Text, TextProps};

pub struct TextBuilder {
    props: TextProps,
}

impl TextBuilder {
    pub(crate) fn new(text: impl Into<String>) -> Self {
        Self {
            props: TextProps {
                text: text.into(),
                style: None,
                wrap: WrapMode::None,
            },
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.props.style = Some(style);
        self
    }

    pub fn wrap(mut self, mode: WrapMode) -> Self {
        self.props.wrap = mode;
        self
    }
}

impl IntoBlueprint for TextBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Text>(self.props)]
    }
}
