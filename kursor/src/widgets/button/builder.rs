use std::sync::Arc;

use kursor_core::{
    component::{
        behavior::{Behavior, BehaviorBuilder},
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::WrapMode,
    state::{IntoValue, Transition, Value},
};

use super::{
    Border, Button, ButtonIntent, ButtonProps, ButtonState, ButtonStyles, Text,
};
use crate::widgets::{TextProps, text::IntoText};

pub struct ButtonBuilder {
    props: ButtonProps,
}

impl ButtonBuilder {
    pub(crate) fn new(label: impl IntoText) -> Self {
        Self {
            props: ButtonProps::new(
                Text::with(TextProps {
                    text: label.into_text(),
                    style: Value::plain(None),
                    wrap: Value::plain(WrapMode::None),
                    transition: None,
                }),
                |_| {},
            ),
        }
    }

    pub fn label(mut self, label: impl IntoText) -> Self {
        self.props.children = vec![Text::with(TextProps {
            text: label.into_text(),
            style: Value::plain(None),
            wrap: Value::plain(WrapMode::None),
            transition: None,
        })]
        .into();
        self
    }

    pub fn border(mut self, border: impl IntoValue<Border>) -> Self {
        self.props.border = border.into_value();
        self
    }

    pub fn styles(mut self, styles: impl IntoValue<ButtonStyles>) -> Self {
        self.props.styles = styles.into_value();
        self
    }

    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.props.disabled = disabled.into_value();
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

    pub fn transition(mut self, transition: impl Into<Transition>) -> Self {
        self.props.transition = Some(transition.into());
        self
    }

    pub fn build(self) -> Blueprint {
        Blueprint::new::<Button>(self.props)
    }
}

impl BehaviorBuilder for ButtonBuilder {
    type State = ButtonState;
    type Intent = ButtonIntent;

    fn behavior<B>(self, behavior: B) -> Self
    where
        B: Behavior<State = Self::State, Intent = Self::Intent>,
    {
        self.behavior(behavior)
    }
}

impl IntoBlueprint for ButtonBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Button>(self.props)]
    }
}
