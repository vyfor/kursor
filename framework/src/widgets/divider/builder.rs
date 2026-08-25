use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    layout::Orientation,
    render::style::Style,
    state::IntoValue,
};

use super::{Divider, DividerProps};

pub struct DividerBuilder {
    props: DividerProps,
}

impl DividerBuilder {
    pub(crate) fn new() -> Self {
        Self {
            props: DividerProps::default(),
        }
    }

    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        self.props.orientation = orientation.into_value();
        self
    }

    pub fn glyph(mut self, glyph: impl IntoValue<char>) -> Self {
        self.props.glyph = glyph.into_value();
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.props.style = Some(style).into_value();
        self
    }
}

impl IntoBlueprint for DividerBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Divider>(self.props)]
    }
}
