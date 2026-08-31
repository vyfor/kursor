use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    state::Value,
};
use kursor_image::{GraphicsProtocol, ImageFit, ImageSource, Transmission};

use super::{Image, ImageProps};

pub struct ImageBuilder {
    props: ImageProps,
}

impl ImageBuilder {
    pub fn new(source: impl Into<ImageSource>) -> Self {
        Self {
            props: ImageProps {
                source: Value::plain(source.into()),
                fit: Value::plain(ImageFit::Contain),
                prefer: Value::plain(None),
                transmission: Value::plain(Transmission::Direct),
            },
        }
    }

    pub fn fit(mut self, fit: ImageFit) -> Self {
        self.props.fit = Value::plain(fit);
        self
    }

    pub fn prefer(mut self, protocol: Option<GraphicsProtocol>) -> Self {
        self.props.prefer = Value::plain(protocol);
        self
    }

    pub fn transmission(mut self, transmission: Transmission) -> Self {
        self.props.transmission = Value::plain(transmission);
        self
    }

    pub fn build(self) -> Blueprint {
        Image::with(self.props)
    }
}

impl IntoBlueprint for ImageBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}

impl From<ImageBuilder> for Blueprint {
    fn from(builder: ImageBuilder) -> Self {
        builder.build()
    }
}
