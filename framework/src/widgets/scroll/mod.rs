pub mod builder;
pub use builder::ScrollBuilder;

use std::sync::Arc;

use kursor_core::{
    component::{
        Component, Focus,
        behavior::{Behavior, BehaviorCx},
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    event::{Event, EventResult, Phase, mouse::MouseKind},
    layout::{
        Offset, ScrollDirection,
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
};

pub struct ScrollState {
    pub offset_x: u16,
    pub offset_y: u16,
    pub content_width: u16,
    pub content_height: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScrollIntent {
    By { x: i32, y: i32 },
}

pub struct WheelScroll {
    pub step: u16,
    pub shift_horizontal: bool,
}

impl Default for WheelScroll {
    fn default() -> Self {
        Self {
            step: 3,
            shift_horizontal: true,
        }
    }
}

impl Behavior for WheelScroll {
    type State = ScrollState;
    type Intent = ScrollIntent;

    fn event(&self, cx: &BehaviorCx, event: &Event, _state: &ScrollState) -> Option<ScrollIntent> {
        if cx.phase != Phase::Bubble {
            return None;
        }
        let Event::Mouse(mouse) = event else {
            return None;
        };
        let step = i32::from(self.step);
        match mouse.kind {
            MouseKind::ScrollUp => {
                if self.shift_horizontal && mouse.modifiers.shift {
                    Some(ScrollIntent::By { x: -step, y: 0 })
                } else {
                    Some(ScrollIntent::By { x: 0, y: -step })
                }
            }
            MouseKind::ScrollDown => {
                if self.shift_horizontal && mouse.modifiers.shift {
                    Some(ScrollIntent::By { x: step, y: 0 })
                } else {
                    Some(ScrollIntent::By { x: 0, y: step })
                }
            }
            MouseKind::ScrollLeft => Some(ScrollIntent::By { x: -step, y: 0 }),
            MouseKind::ScrollRight => Some(ScrollIntent::By { x: step, y: 0 }),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct ScrollProps {
    pub direction: ScrollDirection,
    pub behavior: Arc<dyn Behavior<State = ScrollState, Intent = ScrollIntent>>,
}

pub struct Scroll {
    state: ScrollState,
}

impl Scroll {
    pub fn builder(child: impl IntoBlueprint) -> ScrollBuilder {
        ScrollBuilder::new(child)
    }

    pub fn new(child: impl IntoBlueprint) -> Blueprint {
        Self::vertical(child)
    }

    pub fn vertical(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ScrollProps {
                direction: ScrollDirection::Vertical,
                behavior: Arc::new(WheelScroll::default()),
            },
            child,
        )
    }

    pub fn horizontal(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ScrollProps {
                direction: ScrollDirection::Horizontal,
                behavior: Arc::new(WheelScroll::default()),
            },
            child,
        )
    }

    pub fn both(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ScrollProps {
                direction: ScrollDirection::Both,
                behavior: Arc::new(WheelScroll::default()),
            },
            child,
        )
    }

    pub fn with(props: ScrollProps, child: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(props).child(child)
    }

    fn apply(&mut self, intent: ScrollIntent, direction: ScrollDirection, viewport: Size) -> bool {
        let ScrollIntent::By { x, y } = intent;
        let (dx, dy) = match direction {
            ScrollDirection::Vertical => (0, y),
            ScrollDirection::Horizontal => (x, 0),
            ScrollDirection::Both => (x, y),
        };
        let prev = (self.state.offset_x, self.state.offset_y);
        if dx != 0 {
            let max = self.state.content_width.saturating_sub(viewport.width);
            self.state.offset_x = if dx > 0 {
                self.state.offset_x.saturating_add(dx as u16).min(max)
            } else {
                self.state.offset_x.saturating_sub((-dx) as u16)
            };
        }
        if dy != 0 {
            let max = self.state.content_height.saturating_sub(viewport.height);
            self.state.offset_y = if dy > 0 {
                self.state.offset_y.saturating_add(dy as u16).min(max)
            } else {
                self.state.offset_y.saturating_sub((-dy) as u16)
            };
        }
        (self.state.offset_x, self.state.offset_y) != prev
    }
}

impl Component for Scroll {
    type Props = ScrollProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            state: ScrollState {
                offset_x: 0,
                offset_y: 0,
                content_width: 0,
                content_height: 0,
            },
        }
    }

    fn focus(&self, _props: &Self::Props) -> Focus {
        Focus {
            focusable: true,
            trap: false,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old.direction != new.direction || !Arc::ptr_eq(&old.behavior, &new.behavior)
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        props: &Self::Props,
        available: Size,
        children: &mut MeasureCx,
    ) -> Size {
        if children.is_empty() {
            return Size::default();
        }
        let child_available = match props.direction {
            ScrollDirection::Vertical => Size::new(available.width, u16::MAX),
            ScrollDirection::Horizontal => Size::new(u16::MAX, available.height),
            ScrollDirection::Both => Size::new(u16::MAX, u16::MAX),
        };
        let child_size = children.measure(0, child_available);
        self.state.content_width = child_size.width;
        self.state.content_height = child_size.height;
        available
    }

    fn layout(&mut self, _cx: &mut Cx, props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        if children.is_empty() {
            return;
        }
        let max_x = self.state.content_width.saturating_sub(area.width);
        let max_y = self.state.content_height.saturating_sub(area.height);
        match props.direction {
            ScrollDirection::Vertical => {
                self.state.offset_y = self.state.offset_y.min(max_y);
                self.state.offset_x = 0;
            }
            ScrollDirection::Horizontal => {
                self.state.offset_x = self.state.offset_x.min(max_x);
                self.state.offset_y = 0;
            }
            ScrollDirection::Both => {
                self.state.offset_x = self.state.offset_x.min(max_x);
                self.state.offset_y = self.state.offset_y.min(max_y);
            }
        }
        children.set(
            0,
            Rect::new(0, 0, self.state.content_width, self.state.content_height),
        );
        children.translate(
            0,
            Offset::new(
                -i32::from(self.state.offset_x),
                -i32::from(self.state.offset_y),
            ),
        );
    }

    fn event(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        event: &Event,
        phase: Phase,
    ) -> EventResult {
        let bcx = BehaviorCx {
            phase,
            rect: cx.rect,
        };
        let Some(intent) = props.behavior.event(&bcx, event, &self.state) else {
            return EventResult::Ignored;
        };
        let viewport = Size::new(cx.rect.width, cx.rect.height);
        if self.apply(intent, props.direction, viewport) {
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }
}
