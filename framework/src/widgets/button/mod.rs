pub mod builder;
pub use builder::ButtonBuilder;

use std::sync::Arc;

use kursor_core::{
    component::{
        Children, Component, Focus,
        behavior::{Behavior, BehaviorCx},
        blueprint::Blueprint,
        context::Cx,
    },
    event::{
        Event, EventResult, Phase,
        key::KeyCode,
        mouse::{MouseButton, MouseKind},
    },
    layout::{context::MeasureCx, size::Size},
    render::style::Style,
    theme::Theme,
};

use super::{Block, BlockProps, Border, Text};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonState {
    pub hovered: bool,
    pub focused: bool,
    pub pressed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonIntent {
    Activate,
}

#[derive(Clone, Copy, Default)]
pub struct ButtonStyles {
    pub normal: Option<Style>,
    pub hovered: Option<Style>,
    pub focused: Option<Style>,
    pub pressed: Option<Style>,
    pub disabled: Option<Style>,
}

impl PartialEq for ButtonStyles {
    fn eq(&self, other: &Self) -> bool {
        self.normal == other.normal
            && self.hovered == other.hovered
            && self.focused == other.focused
            && self.pressed == other.pressed
            && self.disabled == other.disabled
    }
}

impl Eq for ButtonStyles {}

#[derive(Default)]
pub struct ButtonBehavior;

impl Behavior for ButtonBehavior {
    type State = ButtonState;
    type Intent = ButtonIntent;

    fn event(&self, cx: &BehaviorCx, event: &Event, _state: &ButtonState) -> Option<ButtonIntent> {
        if cx.phase != Phase::Descending {
            return None;
        }

        match event {
            Event::Mouse(mouse) => match mouse.kind {
                MouseKind::Click(MouseButton::Left) => Some(ButtonIntent::Activate),
                _ => None,
            },
            Event::Key(key) if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) => {
                Some(ButtonIntent::Activate)
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct ButtonProps {
    pub label: String,
    pub border: Border,
    pub styles: ButtonStyles,
    pub disabled: bool,
    pub behavior: Arc<dyn Behavior<State = ButtonState, Intent = ButtonIntent>>,
    pub on_press: Arc<dyn Fn(&mut Cx)>,
}

impl ButtonProps {
    pub fn new(label: impl Into<String>, on_press: impl Fn(&mut Cx) + 'static) -> Self {
        Self {
            label: label.into(),
            border: Border::Rounded,
            styles: ButtonStyles::default(),
            disabled: false,
            behavior: Arc::new(ButtonBehavior),
            on_press: Arc::new(on_press),
        }
    }
}

pub struct Button {
    state: ButtonState,
}

impl Button {
    pub fn builder(label: impl Into<String>) -> ButtonBuilder {
        ButtonBuilder::new(label)
    }

    pub fn new(label: impl Into<String>, on_press: impl Fn(&mut Cx) + 'static) -> Blueprint {
        Self::with(ButtonProps::new(label, on_press))
    }

    pub fn with(props: ButtonProps) -> Blueprint {
        Blueprint::new::<Self>(props)
    }

    fn style(&self, props: &ButtonProps, theme: Theme) -> Style {
        if props.disabled {
            return props.styles.disabled.unwrap_or(theme.disabled);
        }
        if self.state.pressed {
            return props.styles.pressed.unwrap_or(theme.focus);
        }
        if self.state.focused {
            return props.styles.focused.unwrap_or(theme.focus);
        }
        if self.state.hovered {
            return props.styles.hovered.unwrap_or(theme.primary);
        }
        props.styles.normal.unwrap_or(theme.surface)
    }
}

impl Component for Button {
    type Props = ButtonProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            state: ButtonState::default(),
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old.label != new.label
            || old.border != new.border
            || old.styles != new.styles
            || old.disabled != new.disabled
            || !Arc::ptr_eq(&old.behavior, &new.behavior)
            || !Arc::ptr_eq(&old.on_press, &new.on_press)
    }

    fn focus(&self, props: &Self::Props) -> Focus {
        Focus {
            focusable: !props.disabled,
            trap: false,
        }
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        available: Size,
        children: &mut MeasureCx,
    ) -> Size {
        if children.is_empty() {
            return Size::default();
        }
        let size = children.size(0);
        Size::new(
            size.width.min(available.width),
            size.height.min(available.height),
        )
    }

    fn build(&mut self, cx: &mut Cx, props: &Self::Props, children: &mut Children) {
        let theme = *cx.theme();
        let style = self.style(props, theme);
        children.replace(Block::with(
            BlockProps {
                border: props.border,
                style: Some(style),
            },
            Text::styled(props.label.clone(), style),
        ));
    }

    fn event(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        event: &Event,
        phase: Phase,
    ) -> EventResult {
        if props.disabled {
            return EventResult::Ignored;
        }

        if phase != Phase::Descending {
            return EventResult::Ignored;
        }

        let mut state_changed = false;
        match event {
            Event::Mouse(mouse) => match mouse.kind {
                MouseKind::Enter if !self.state.hovered => {
                    self.state.hovered = true;
                    state_changed = true;
                }
                MouseKind::Leave if self.state.hovered => {
                    self.state.hovered = false;
                    self.state.pressed = false;
                    state_changed = true;
                }
                MouseKind::Down(MouseButton::Left) => {
                    self.state.pressed = true;
                    cx.focus();
                    cx.capture();
                    state_changed = true;
                }
                MouseKind::Up(MouseButton::Left) if self.state.pressed => {
                    self.state.pressed = false;
                    cx.release_capture();
                    state_changed = true;
                }
                _ => {}
            },
            Event::FocusIn if !self.state.focused => {
                self.state.focused = true;
                state_changed = true;
            }
            Event::FocusOut if self.state.focused || self.state.pressed => {
                self.state.focused = false;
                self.state.pressed = false;
                state_changed = true;
            }
            _ => {}
        }

        let bcx = BehaviorCx {
            phase,
            rect: cx.rect,
        };
        if let Some(ButtonIntent::Activate) = props.behavior.event(&bcx, event, &self.state) {
            (props.on_press)(cx);
            return EventResult::Consumed;
        }

        if state_changed {
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }
}
