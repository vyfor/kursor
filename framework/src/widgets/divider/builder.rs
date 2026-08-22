use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    layout::Orientation,
    render::style::Style,
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

    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.props.orientation = orientation;
        self
    }

    pub fn glyph(mut self, glyph: char) -> Self {
        self.props.glyph = glyph;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.props.style = Some(style);
        self
    }
}

impl IntoBlueprint for DividerBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Divider>(self.props)]
    }
}
