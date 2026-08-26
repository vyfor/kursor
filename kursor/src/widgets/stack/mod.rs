use kursor_core::{
    component::{
        Component,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::{
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct StackProps;

pub struct Stack;

impl Stack {
    pub fn new(children: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(StackProps).children(children)
    }
}

impl Component for Stack {
    type Props = StackProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self
    }

    fn changed(&self, _old: &Self::Props, _new: &Self::Props) -> bool {
        false
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        available: Size,
        children: &mut MeasureCx,
    ) -> Size {
        let mut size = Size::default();
        for index in 0..children.len() {
            let child = children.size(index);
            size.width = size.width.max(child.width);
            size.height = size.height.max(child.height);
        }
        Size::new(
            size.width.min(available.width),
            size.height.min(available.height),
        )
    }

    fn layout(&mut self, _cx: &mut Cx, _props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        for index in 0..children.len() {
            children.set(index, area);
        }
    }
}
