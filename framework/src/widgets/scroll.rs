use kursor_core::{
    component::{
        Component,
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

#[derive(Clone, Copy, Debug, Default)]
pub struct ScrollProps {
    pub direction: ScrollDirection,
}

pub struct Scroll {
    offset_x: u16,
    offset_y: u16,
    content_width: u16,
    content_height: u16,
}

impl Scroll {
    pub fn new(child: impl IntoBlueprint) -> Blueprint {
        Self::vertical(child)
    }

    pub fn vertical(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ScrollProps {
                direction: ScrollDirection::Vertical,
            },
            child,
        )
    }

    pub fn horizontal(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ScrollProps {
                direction: ScrollDirection::Horizontal,
            },
            child,
        )
    }

    pub fn both(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ScrollProps {
                direction: ScrollDirection::Both,
            },
            child,
        )
    }

    pub fn with(props: ScrollProps, child: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(props).child(child)
    }
}

impl Component for Scroll {
    type Props = ScrollProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            content_width: 0,
            content_height: 0,
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

        let child_available = match _props.direction {
            ScrollDirection::Vertical => Size::new(available.width, u16::MAX),
            ScrollDirection::Horizontal => Size::new(u16::MAX, available.height),
            ScrollDirection::Both => Size::new(u16::MAX, u16::MAX),
        };
        let child_size = children.measure(0, child_available);
        self.content_width = child_size.width;
        self.content_height = child_size.height;

        available
    }

    fn layout(&mut self, _cx: &mut Cx, props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        if children.is_empty() {
            return;
        }

        let max_offset_x = self.content_width.saturating_sub(area.width);
        let max_offset_y = self.content_height.saturating_sub(area.height);

        match props.direction {
            ScrollDirection::Vertical => {
                self.offset_y = self.offset_y.min(max_offset_y);
                self.offset_x = 0;
            }
            ScrollDirection::Horizontal => {
                self.offset_x = self.offset_x.min(max_offset_x);
                self.offset_y = 0;
            }
            ScrollDirection::Both => {
                self.offset_x = self.offset_x.min(max_offset_x);
                self.offset_y = self.offset_y.min(max_offset_y);
            }
        }

        children.set(0, Rect::new(0, 0, self.content_width, self.content_height));
        children.translate(
            0,
            Offset::new(-i32::from(self.offset_x), -i32::from(self.offset_y)),
        );
    }

    fn event(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        event: &Event,
        phase: Phase,
    ) -> EventResult {
        if phase != Phase::Descending {
            return EventResult::Ignored;
        }

        let Event::Mouse(mouse) = event else {
            return EventResult::Ignored;
        };

        let viewport = cx.rect;
        let horizontal = matches!(
            props.direction,
            ScrollDirection::Horizontal | ScrollDirection::Both
        );
        let vertical = matches!(
            props.direction,
            ScrollDirection::Vertical | ScrollDirection::Both
        );
        let max_x = self.content_width.saturating_sub(viewport.width);
        let max_y = self.content_height.saturating_sub(viewport.height);
        let previous = (self.offset_x, self.offset_y);

        match mouse.kind {
            MouseKind::ScrollUp if vertical && !mouse.modifiers.shift => {
                self.offset_y = self.offset_y.saturating_sub(3);
            }
            MouseKind::ScrollDown if vertical && !mouse.modifiers.shift => {
                self.offset_y = self.offset_y.saturating_add(3).min(max_y);
            }
            MouseKind::ScrollUp if horizontal && mouse.modifiers.shift => {
                self.offset_x = self.offset_x.saturating_sub(3);
            }
            MouseKind::ScrollDown if horizontal && mouse.modifiers.shift => {
                self.offset_x = self.offset_x.saturating_add(3).min(max_x);
            }
            MouseKind::ScrollLeft if horizontal => {
                self.offset_x = self.offset_x.saturating_sub(3);
            }
            MouseKind::ScrollRight if horizontal => {
                self.offset_x = self.offset_x.saturating_add(3).min(max_x);
            }
            _ => {}
        }

        ((self.offset_x, self.offset_y) != previous)
            .then_some(EventResult::Consumed)
            .unwrap_or(EventResult::Ignored)
    }
}
