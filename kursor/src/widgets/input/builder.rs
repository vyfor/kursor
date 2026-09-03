use std::sync::Arc;

use kursor_core::{
    component::{
        behavior::{Behavior, BehaviorBuilder},
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    state::{IntoValue, Transition},
};

use super::{Input, InputDisplay, InputIntent, InputProps, InputState, InputStyles};

pub struct InputBuilder {
    props: InputProps,
}

impl InputBuilder {
    pub(crate) fn new(value: impl IntoValue<String>) -> Self {
        Self {
            props: InputProps::new(value, |_, _| {}),
        }
    }

    pub fn placeholder(mut self, placeholder: impl IntoValue<String>) -> Self {
        self.props.placeholder = placeholder.into_value();
        self
    }

    pub fn styles(mut self, styles: impl IntoValue<InputStyles>) -> Self {
        self.props.styles = styles.into_value();
        self
    }

    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.props.disabled = disabled.into_value();
        self
    }

    pub fn behavior<B>(mut self, behavior: B) -> Self
    where
        B: Behavior<State = InputState, Intent = InputIntent>,
    {
        self.props.behavior = Arc::new(behavior);
        self
    }

    pub fn display(mut self, display: InputDisplay) -> Self {
        self.props.display = display;
        self
    }

    pub fn transition(mut self, transition: impl Into<Transition>) -> Self {
        self.props.transition = Some(transition.into());
        self
    }

    pub fn masked(self) -> Self {
        self.display(Arc::new(|_, _, _| '*'))
    }

    pub fn on_change(mut self, on_change: impl Fn(&mut Cx, &str) + 'static) -> Self {
        self.props.on_change = Arc::new(on_change);
        self
    }

    pub fn on_submit(mut self, on_submit: impl Fn(&mut Cx) + 'static) -> Self {
        self.props.on_submit = Arc::new(on_submit);
        self
    }
}

impl BehaviorBuilder for InputBuilder {
    type State = InputState;
    type Intent = InputIntent;

    fn behavior<B>(self, behavior: B) -> Self
    where
        B: Behavior<State = Self::State, Intent = Self::Intent>,
    {
        self.behavior(behavior)
    }
}

impl IntoBlueprint for InputBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Input>(self.props)]
    }
}
