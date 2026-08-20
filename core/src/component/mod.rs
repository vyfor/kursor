use std::any::Any;

use crate::{
    component::{blueprint::Blueprint, context::Cx},
    event::{Event, EventResult, Phase},
    layout::{
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    render::canvas::Canvas,
};
pub mod action;
pub mod blueprint;
pub mod context;
pub mod environment;
pub mod behavior;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Focus {
    pub focusable: bool,
    pub trap: bool,
}

pub trait Component: 'static {
    type Props: Clone;

    fn create(cx: &mut Cx, props: &Self::Props) -> Self;

    fn build(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        children: Vec<Blueprint>,
    ) -> Vec<Blueprint> {
        children
    }

    fn focus(&self, _props: &Self::Props) -> Focus {
        Focus {
            focusable: false,
            trap: false,
        }
    }

    fn event(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        _event: &Event,
        _phase: Phase,
    ) -> EventResult {
        EventResult::Ignored
    }

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

    fn layout(&mut self, _cx: &mut Cx, _props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        for index in 0..children.len() {
            children.set(index, area);
        }
    }
    fn paint(&self, _cx: &mut Cx, _props: &Self::Props, _canvas: &mut Canvas) {}

    fn changed(&self, _old: &Self::Props, _new: &Self::Props) -> bool {
        true
    }

    fn drop(&mut self, _cx: &mut Cx) {}
}

pub trait AnyComponent {
    fn build_any(
        &mut self,
        cx: &mut Cx,
        props: &dyn Any,
        children: Vec<Blueprint>,
    ) -> Vec<Blueprint>;
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
    fn layout_any(&mut self, cx: &mut Cx, props: &dyn Any, area: Rect, children: &mut LayoutCx);
    fn paint_any(&self, cx: &mut Cx, props: &dyn Any, canvas: &mut Canvas);
    fn changed_any(&self, old: &dyn Any, new: &dyn Any) -> bool;
    fn drop_any(&mut self, cx: &mut Cx);
}

impl<C: Component> AnyComponent for C {
    fn build_any(
        &mut self,
        cx: &mut Cx,
        props: &dyn Any,
        children: Vec<Blueprint>,
    ) -> Vec<Blueprint> {
        let props = props.downcast_ref::<C::Props>().unwrap();
        self.build(cx, props, children)
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

    fn layout_any(&mut self, cx: &mut Cx, props: &dyn Any, area: Rect, children: &mut LayoutCx) {
        let props = props.downcast_ref::<C::Props>().unwrap();
        self.layout(cx, props, area, children)
    }

    fn paint_any(&self, cx: &mut Cx, props: &dyn Any, canvas: &mut Canvas) {
        let props = props.downcast_ref::<C::Props>().unwrap();
        self.paint(cx, props, canvas)
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
