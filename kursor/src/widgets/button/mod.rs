pub mod builder;
pub use builder::ButtonBuilder;

use std::rc::Rc;
use std::sync::Arc;

use kursor_core::{
    component::{
        Component, Focus, Update,
        behavior::{Behavior, BehaviorCx},
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    event::{
        Event, EventResult, Phase,
        key::KeyCode,
        mouse::{MouseButton, MouseKind},
    },
    layout::{context::MeasureCx, size::Size},
    render::style::Style,
    state::{IntoValue, Signal, Transition, Value},
    theme::Theme,
};

use super::text::IntoText;
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

#[derive(Clone, Default, PartialEq)]
pub struct ButtonStyles {
    pub normal: Option<Value<Style>>,
    pub hovered: Option<Value<Style>>,
    pub focused: Option<Value<Style>>,
    pub pressed: Option<Value<Style>>,
    pub disabled: Option<Value<Style>>,
}

kursor_core::into_value!(ButtonStyles);

impl Eq for ButtonStyles {}

impl ButtonStyles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn normal(mut self, style: impl IntoValue<Style>) -> Self {
        self.normal = Some(style.into_value());
        self
    }

    pub fn hovered(mut self, style: impl IntoValue<Style>) -> Self {
        self.hovered = Some(style.into_value());
        self
    }

    pub fn focused(mut self, style: impl IntoValue<Style>) -> Self {
        self.focused = Some(style.into_value());
        self
    }

    pub fn pressed(mut self, style: impl IntoValue<Style>) -> Self {
        self.pressed = Some(style.into_value());
        self
    }

    pub fn disabled(mut self, style: impl IntoValue<Style>) -> Self {
        self.disabled = Some(style.into_value());
        self
    }
}

#[derive(Default)]
pub struct ButtonBehavior;

impl Behavior for ButtonBehavior {
    type State = ButtonState;
    type Intent = ButtonIntent;

    fn event(
        &self,
        cx: &BehaviorCx,
        event: &Event,
        _state: &ButtonState,
    ) -> Option<ButtonIntent> {
        if cx.phase != Phase::Bubble {
            return None;
        }

        match event {
            Event::Mouse(mouse) => match mouse.kind {
                MouseKind::Click(MouseButton::Left) => {
                    Some(ButtonIntent::Activate)
                }
                _ => None,
            },
            Event::Key(key)
                if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) =>
            {
                Some(ButtonIntent::Activate)
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct ButtonProps {
    pub children: Rc<[Blueprint]>,
    pub border: Value<Border>,
    pub styles: Value<ButtonStyles>,
    pub disabled: Value<bool>,
    pub behavior: Arc<dyn Behavior<State = ButtonState, Intent = ButtonIntent>>,
    pub on_press: Arc<dyn Fn(&mut Cx)>,
    pub transition: Option<Transition>,
}

impl ButtonProps {
    pub fn new(
        child: impl IntoBlueprint,
        on_press: impl Fn(&mut Cx) + 'static,
    ) -> Self {
        Self {
            children: child.into_blueprint().into(),
            border: Value::plain(Border::Rounded),
            styles: Value::plain(ButtonStyles::default()),
            disabled: Value::plain(false),
            behavior: Arc::new(ButtonBehavior),
            on_press: Arc::new(on_press),
            transition: None,
        }
    }
}

/// a clickable button.
///
/// under the hood, it renders a `Block` with a `Text` label inside.
pub struct Button {
    state: ButtonState,
    styles: ButtonStyles,
    disabled: bool,

    border: Signal<Border>,
    style: Signal<Option<Style>>,
    transition: Option<Transition>,
}

impl Button {
    pub fn builder(label: impl IntoText) -> ButtonBuilder {
        ButtonBuilder::new(label)
    }

    pub fn new(
        child: impl IntoBlueprint,
        on_press: impl Fn(&mut Cx) + 'static,
    ) -> Blueprint {
        Self::with(ButtonProps::new(child, on_press))
    }

    pub fn label(
        label: impl IntoText,
        on_press: impl Fn(&mut Cx) + 'static,
    ) -> Blueprint {
        Self::new(Text::new(label), on_press)
    }

    pub fn with(props: ButtonProps) -> Blueprint {
        Blueprint::new::<Self>(props)
    }

    fn style(&self, cx: &mut Cx, theme: Theme) -> Style {
        let (val, fallback) = if self.disabled {
            (&self.styles.disabled, theme.disabled)
        } else if self.state.pressed {
            (&self.styles.pressed, theme.focus)
        } else if self.state.focused {
            (&self.styles.focused, theme.focus)
        } else if self.state.hovered {
            (&self.styles.hovered, theme.primary)
        } else {
            (&self.styles.normal, theme.surface)
        };

        cx.resolve_or("style", val, fallback, self.transition.clone())
    }
}

impl Component for Button {
    type Props = ButtonProps;

    fn create(cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            state: ButtonState::default(),
            styles: ButtonStyles::default(),
            disabled: false,

            border: cx.signal(Border::Rounded),
            style: cx.signal(None),
            transition: None,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        !Rc::ptr_eq(&old.children, &new.children)
            || old.border != new.border
            || old.styles != new.styles
            || old.disabled != new.disabled
            || old.transition != new.transition
            || !Arc::ptr_eq(&old.behavior, &new.behavior)
            || !Arc::ptr_eq(&old.on_press, &new.on_press)
    }

    fn focus(&self, _props: &Self::Props) -> Focus {
        Focus {
            focusable: !self.disabled,
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

    fn mount(
        &mut self,
        _cx: &mut Cx,
        props: &Self::Props,
        children: &mut kursor_core::component::MountChildren,
    ) {
        children.replace(Block::with(
            BlockProps {
                border: self.border.clone().into_value(),
                style: self.style.clone().into_value(),
                transition: None,
            },
            props.children.to_vec(),
        ));
    }

    fn update(&mut self, cx: &mut Cx, props: &Self::Props) -> Update {
        let border = props.border.get();
        let styles = props.styles.get();
        let disabled = props.disabled.get();
        let old_style = self.style.peek();
        self.styles = styles;
        self.disabled = disabled;
        self.transition = props.transition.clone();

        self.border.set(border);
        let theme = *cx.theme();
        let style = self.style(cx, theme);
        if old_style != Some(style) {
            self.style.set(Some(style));
        }
        Update::NONE
    }

    fn event(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        event: &Event,
        phase: Phase,
    ) -> EventResult {
        if self.disabled {
            return EventResult::Ignored;
        }

        if phase != Phase::Bubble {
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
                    cx.focus(true);
                    cx.capture(true);
                    state_changed = true;
                }
                MouseKind::Up(MouseButton::Left) if self.state.pressed => {
                    self.state.pressed = false;
                    cx.capture(false);
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
        if let Some(ButtonIntent::Activate) =
            props.behavior.event(&bcx, event, &self.state)
        {
            (props.on_press)(cx);
            return EventResult::Consumed;
        }

        if state_changed {
            cx.relayout_self();
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }
}
