pub mod builder;
pub use builder::ModalBuilder;

use std::rc::Rc;
use std::sync::Arc;

use kursor_core::{
    component::{
        Component, Focus, MountChildren, Update,
        behavior::{Behavior, BehaviorCx},
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    event::{
        Event, EventResult, Phase,
        key::KeyCode,
        mouse::{MouseButton, MouseKind},
    },
    layout::{
        Alignment, HAlign, VAlign,
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    render::{canvas::Canvas, color::Color, style::Style},
    state::Value,
};

use crate::widgets::CxOverlayExt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModalState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalIntent {
    Dismiss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModalBehavior {
    close_on_esc: bool,
    close_on_backdrop: bool,
}

impl ModalBehavior {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn esc(mut self, close: bool) -> Self {
        self.close_on_esc = close;
        self
    }

    pub fn backdrop(mut self, close: bool) -> Self {
        self.close_on_backdrop = close;
        self
    }
}

impl Default for ModalBehavior {
    fn default() -> Self {
        Self {
            close_on_esc: true,
            close_on_backdrop: true,
        }
    }
}

impl Behavior for ModalBehavior {
    type State = ModalState;
    type Intent = ModalIntent;

    fn event(
        &self,
        cx: &BehaviorCx,
        event: &Event,
        _state: &ModalState,
    ) -> Option<ModalIntent> {
        match event {
            Event::Key(key)
                if key.code == KeyCode::Esc && self.close_on_esc =>
            {
                Some(ModalIntent::Dismiss)
            }
            Event::Mouse(mouse)
                if self.close_on_backdrop
                    && matches!(
                        mouse.kind,
                        MouseKind::Down(MouseButton::Left)
                    )
                    && !cx.rect.contains(mouse.column, mouse.row) =>
            {
                Some(ModalIntent::Dismiss)
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct ModalProps {
    pub(crate) content: Rc<Blueprint>,
    pub(crate) backdrop: Option<Value<Color>>,
    pub(crate) alignment: Value<Alignment>,
    pub(crate) behavior:
        Arc<dyn Behavior<State = ModalState, Intent = ModalIntent>>,
    pub(crate) on_dismiss: Option<Rc<dyn Fn(&mut Cx)>>,
    pub(crate) trap_focus: bool,
}

impl PartialEq for ModalProps {
    fn eq(&self, other: &Self) -> bool {
        self.backdrop == other.backdrop
            && self.alignment == other.alignment
            && self.trap_focus == other.trap_focus
            && Rc::ptr_eq(&self.content, &other.content)
            && Arc::ptr_eq(&self.behavior, &other.behavior)
            && match (&self.on_dismiss, &other.on_dismiss) {
                (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Modal {
    size: Size,
    rect: Rect,
}

impl Modal {
    pub fn new(content: impl IntoBlueprint) -> ModalBuilder {
        ModalBuilder::new(content)
    }
}

impl Component for Modal {
    type Props = ModalProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self::default()
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn mount(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        children: &mut MountChildren,
    ) {
        cx.focus(true);

        children.replace(vec![(*props.content).clone()]);
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        Update::children(vec![(*props.content).clone()])
    }

    fn focus(&self, props: &Self::Props) -> Focus {
        Focus {
            focusable: true,
            trap: props.trap_focus,
        }
    }

    fn event(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        event: &Event,
        phase: Phase,
    ) -> EventResult {
        if phase == Phase::Capture {
            return EventResult::Ignored;
        }

        let bcx = BehaviorCx {
            phase,
            rect: self.rect,
        };

        if let Some(ModalIntent::Dismiss) =
            props.behavior.event(&bcx, event, &ModalState)
        {
            if let Some(on_dismiss) = &props.on_dismiss {
                on_dismiss(cx);
            }
            cx.close_layer(());
            return EventResult::Consumed;
        }

        match event {
            Event::Mouse(mouse)
                if !self.rect.contains(mouse.column, mouse.row) =>
            {
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        available: Size,
        children: &mut MeasureCx,
    ) -> Size {
        if !children.is_empty() {
            self.size = children.measure(0, available);
        }
        available
    }

    fn layout(
        &mut self,
        _cx: &mut Cx,
        props: &Self::Props,
        area: Rect,
        children: &mut LayoutCx,
    ) {
        if children.is_empty() {
            self.rect = Rect::default();
            return;
        }

        let alignment = props.alignment.get();
        let width = self.size.width.min(area.width);
        let height = self.size.height.min(area.height);
        let max_x = area.right().saturating_sub(width);
        let max_y = area.bottom().saturating_sub(height);

        let x = match alignment.horizontal {
            HAlign::Left => area.x,
            HAlign::Center => {
                area.x.saturating_add(area.width.saturating_sub(width) / 2)
            }
            HAlign::Right => max_x,
        };

        let y = match alignment.vertical {
            VAlign::Top => area.y,
            VAlign::Center => area
                .y
                .saturating_add(area.height.saturating_sub(height) / 2),
            VAlign::Bottom => max_y,
        };

        self.rect = Rect::new(x.min(max_x), y.min(max_y), width, height);
        children.set(0, self.rect);
    }

    fn pre_paint(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        canvas: &mut Canvas,
    ) {
        if let Some(backdrop) = &props.backdrop {
            let style = Style::new().bg(backdrop.get());
            canvas.fill(
                Rect::new(0, 0, cx.rect.width, cx.rect.height),
                ' ',
                style,
            );
        }
    }
}
