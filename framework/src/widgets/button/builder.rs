use std::sync::Arc;

use kursor_core::component::{
    behavior::Behavior,
    blueprint::{Blueprint, IntoBlueprint},
    context::Cx,
};

use super::{Border, Button, ButtonIntent, ButtonProps, ButtonState, ButtonStyles};

pub struct ButtonBuilder {
    props: ButtonProps,
}

impl ButtonBuilder {
    pub(crate) fn new(label: impl Into<String>) -> Self {
        Self {
            props: ButtonProps::new(label, |_| {}),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.props.label = label.into();
        self
    }

    pub fn border(mut self, border: Border) -> Self {
        self.props.border = border;
        self
    }

    pub fn styles(mut self, styles: ButtonStyles) -> Self {
        self.props.styles = styles;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.props.disabled = disabled;
        self
    }

    pub fn behavior<B>(mut self, behavior: B) -> Self
    where
        B: Behavior<State = ButtonState, Intent = ButtonIntent>,
    {
        self.props.behavior = Arc::new(behavior);
        self
    }

    pub fn on_press(mut self, on_press: impl Fn(&mut Cx) + 'static) -> Self {
        self.props.on_press = Arc::new(on_press);
        self
    }
}

impl IntoBlueprint for ButtonBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Button>(self.props)]
    }
}
