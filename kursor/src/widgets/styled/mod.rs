pub mod builder;
pub use builder::StyledBuilder;

use kursor_core::{
    component::{
        Component, InheritedStyle, MountChildren, Update,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::{
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    render::style::Style,
    state::Value,
};

/// a plain convenience wrapper that
/// [`provides`](crate::core::component::context::Cx::provide) the given
/// [`InheritedStyle`] to all descendant components via the
/// [`Environment`](crate::core::component::environment::Environment).
pub struct Styled;

impl Styled {
    pub fn new(child: impl IntoBlueprint) -> StyledBuilder {
        StyledBuilder::new(child)
    }

    pub fn with(
        style: impl Into<Value<Style>>,
        child: impl IntoBlueprint,
    ) -> Blueprint {
        Blueprint::new::<Self>(style.into()).children(child)
    }
}

impl Component for Styled {
    type Props = Value<Style>;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn mount(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        _children: &mut MountChildren,
    ) {
        cx.provide(InheritedStyle(Value::plain(Some(props.get()))));
    }

    fn update(&mut self, cx: &mut Cx, props: &Self::Props) -> Update {
        cx.provide(InheritedStyle(Value::plain(Some(props.get()))));
        Update::NONE
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        _available: Size,
        children: &mut MeasureCx,
    ) -> Size {
        if children.is_empty() {
            Size::default()
        } else {
            children.size(0)
        }
    }

    fn layout(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        area: Rect,
        children: &mut LayoutCx,
    ) {
        if !children.is_empty() {
            children.set(0, area);
        }
    }
}
