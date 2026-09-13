use std::any::Any;
use std::ops::{BitOr, BitOrAssign};

use crate::{
    component::blueprint::{Blueprint, IntoBlueprint},
    component::context::Cx,
    event::{Event, EventResult, Phase},
    layout::{
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    render::canvas::Canvas,
};
pub use context::InheritedStyle;
pub mod action;
pub mod behavior;
pub mod blueprint;
pub mod children;
pub use children::Children;
pub type MountChildren = Children;
pub mod context;
pub mod environment;
pub mod key;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Invalidation(u8);

impl Invalidation {
    pub const NONE: Self = Self(0);
    pub const MEASURE: Self = Self(1 << 0);
    pub const LAYOUT: Self = Self(1 << 1);
    pub const PAINT: Self = Self(1 << 2);
    pub const ALL: Self =
        Self(Self::MEASURE.0 | Self::LAYOUT.0 | Self::PAINT.0);
    pub const LAYOUT_AND_PAINT: Self = Self(Self::LAYOUT.0 | Self::PAINT.0);

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for Invalidation {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Invalidation {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// describes what work is required after a component updates.
///
/// an update can:
/// 1. indicate that nothing changed, skipping the rest of the frame.
/// 2. invalidate painting, layout, or measurement, causing only the relevant
///    stages to be re-evaluated.
/// 3. or provide an entirely new set of children.
pub struct Update {
    pub(crate) invalidation: Invalidation,
    pub(crate) children: Option<Vec<Blueprint>>,
}

impl Update {
    pub const NONE: Self = Self {
        invalidation: Invalidation::NONE,
        children: None,
    };

    pub const PAINT: Self = Self {
        invalidation: Invalidation::PAINT,
        children: None,
    };

    pub const LAYOUT: Self = Self {
        invalidation: Invalidation::LAYOUT_AND_PAINT,
        children: None,
    };

    pub const MEASURE: Self = Self {
        invalidation: Invalidation::ALL,
        children: None,
    };

    pub fn children(children: impl IntoBlueprint) -> Self {
        Self {
            invalidation: Invalidation::ALL,
            children: Some(children.into_blueprint()),
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Focus {
    pub focusable: bool,
    pub trap: bool,
}

/// a retained, stateful widget.
pub trait Component: 'static {
    type Props: Clone;

    /// creates a dummy instance of the struct so that the runtime has something
    /// to attach to the tree.
    fn create(cx: &mut Cx, props: &Self::Props) -> Self;

    /// describes the initial structure of the component (i.e. the child tree).
    fn mount(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        _children: &mut MountChildren,
    ) {
    }

    /// runs when:
    /// 1. the component's props change.
    /// 2. reactive state that the component has subscribed to observes a
    ///    change.
    /// 3. the component handles an event.
    ///
    /// returns an [`Update`].
    fn update(&mut self, _cx: &mut Cx, _props: &Self::Props) -> Update {
        Update::NONE
    }

    /// declares whether this component is focusable (i.e. wants to receive
    /// keyboard events).
    ///
    /// alternatively, to opt into global input handling see
    /// [`Cx::global_input`].
    fn focus(&self, _props: &Self::Props) -> Focus {
        Focus {
            focusable: false,
            trap: false,
        }
    }

    /// receives [`Event`]s aimed at this component.
    ///
    /// events arrive in multiple [`Phase`]s, each representing a different
    /// stage of event propagation. typically:
    /// - `Capture` will propagate down from the root to this component.
    /// - `Bubble` will propagate up from this component to the root.
    /// - `Global` will only arrive if this component has opted into global
    ///   input.
    ///
    /// the return value [`EventResult`] tells the runtime whether the event was
    /// handled or not.
    ///
    /// *notes:*
    /// - *key events only arrive if this component (or a descendant) is
    ///   currently focused, OR has opted into global input.*
    fn event(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        _event: &Event,
        _phase: Phase,
    ) -> EventResult {
        EventResult::Ignored
    }

    /// reports how much space the component needs given what's available.
    fn measure(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        available: Size,
        children: &mut MeasureCx,
    ) -> Size {
        for index in 0..children.len() {
            children.size(index);
        }
        available
    }

    /// positions component's children inside the allocated rect.
    fn layout(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        area: Rect,
        children: &mut LayoutCx,
    ) {
        for index in 0..children.len() {
            children.set(index, area);
        }
    }

    /// draws the component to the canvas.
    fn paint(&self, _cx: &mut Cx, _props: &Self::Props, _canvas: &mut Canvas) {}

    /// runs before children paint.
    fn pre_paint(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        _canvas: &mut Canvas,
    ) {
    }

    /// runs after children and this component itself are done painting.
    fn post_paint(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        _canvas: &mut Canvas,
    ) -> bool {
        false
    }

    /// decides whether [`Self::update`] should re-run.
    fn changed(&self, _old: &Self::Props, _new: &Self::Props) -> bool {
        true
    }

    /// called before the component is unmounted.
    fn drop(&mut self, _cx: &mut Cx) {}
}

pub trait AnyComponent {
    fn mount_any(
        &mut self,
        cx: &mut Cx,
        props: &dyn Any,
        children: &mut MountChildren,
    );
    fn update_any(&mut self, cx: &mut Cx, props: &dyn Any) -> Update;
    fn focus_any(&self, props: &dyn Any) -> Focus;
    fn event_any(
        &mut self,
        cx: &mut Cx,
        props: &dyn Any,
        event: &Event,
        phase: Phase,
    ) -> EventResult;
    fn measure_any(
        &mut self,
        cx: &mut Cx,
        props: &dyn Any,
        available: Size,
        children: &mut MeasureCx,
    ) -> Size;
    fn layout_any(
        &mut self,
        cx: &mut Cx,
        props: &dyn Any,
        area: Rect,
        children: &mut LayoutCx,
    );
    fn paint_any(&self, cx: &mut Cx, props: &dyn Any, canvas: &mut Canvas);
    fn pre_paint_any(
        &mut self,
        cx: &mut Cx,
        props: &dyn Any,
        canvas: &mut Canvas,
    );
    fn post_paint_any(
        &mut self,
        cx: &mut Cx,
        props: &dyn Any,
        canvas: &mut Canvas,
    ) -> bool;
    fn changed_any(&self, old: &dyn Any, new: &dyn Any) -> bool;
    fn drop_any(&mut self, cx: &mut Cx);
}

impl<C: Component> AnyComponent for C {
    fn mount_any(
        &mut self,
        cx: &mut Cx,
        props: &dyn Any,
        children: &mut MountChildren,
    ) {
        let props = props.downcast_ref::<C::Props>().unwrap();
        self.mount(cx, props, children)
    }

    fn update_any(&mut self, cx: &mut Cx, props: &dyn Any) -> Update {
        let props = props.downcast_ref::<C::Props>().unwrap();
        self.update(cx, props)
    }

    fn focus_any(&self, props: &dyn Any) -> Focus {
        let props = props.downcast_ref::<C::Props>().unwrap();
        self.focus(props)
    }

    fn event_any(
        &mut self,
        cx: &mut Cx,
        props: &dyn Any,
        event: &Event,
        phase: Phase,
    ) -> EventResult {
        let props = props.downcast_ref::<C::Props>().unwrap();
        self.event(cx, props, event, phase)
    }

    fn measure_any(
        &mut self,
        cx: &mut Cx,
        props: &dyn Any,
        available: Size,
        children: &mut MeasureCx,
    ) -> Size {
        let props = props.downcast_ref::<C::Props>().unwrap();
        self.measure(cx, props, available, children)
    }

    fn layout_any(
        &mut self,
        cx: &mut Cx,
        props: &dyn Any,
        area: Rect,
        children: &mut LayoutCx,
    ) {
        let props = props.downcast_ref::<C::Props>().unwrap();
        self.layout(cx, props, area, children)
    }

    fn paint_any(&self, cx: &mut Cx, props: &dyn Any, canvas: &mut Canvas) {
        let props = props.downcast_ref::<C::Props>().unwrap();
        self.paint(cx, props, canvas)
    }

    fn pre_paint_any(
        &mut self,
        cx: &mut Cx,
        props: &dyn Any,
        canvas: &mut Canvas,
    ) {
        let props = props.downcast_ref::<C::Props>().unwrap();
        self.pre_paint(cx, props, canvas)
    }

    fn post_paint_any(
        &mut self,
        cx: &mut Cx,
        props: &dyn Any,
        canvas: &mut Canvas,
    ) -> bool {
        let props = props.downcast_ref::<C::Props>().unwrap();
        self.post_paint(cx, props, canvas)
    }

    fn changed_any(&self, old: &dyn Any, new: &dyn Any) -> bool {
        let old = old.downcast_ref::<C::Props>().unwrap();
        let new = new.downcast_ref::<C::Props>().unwrap();
        self.changed(old, new)
    }

    fn drop_any(&mut self, cx: &mut Cx) {
        self.drop(cx)
    }
}
