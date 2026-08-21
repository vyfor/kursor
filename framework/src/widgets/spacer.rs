use kursor_core::{
    component::{Component, blueprint::Blueprint, context::Cx},
    layout::{context::MeasureCx, size::Size},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SpacerProps {
    pub size: u16,
}

pub struct Spacer;

impl Spacer {
    pub fn new(size: u16) -> Blueprint {
        Blueprint::new::<Self>(SpacerProps { size })
    }
}

impl Component for Spacer {
    type Props = SpacerProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        props: &Self::Props,
        available: Size,
        _children: &mut MeasureCx,
    ) -> Size {
        let size = props.size.min(available.width).min(available.height);
        Size::new(size, size)
    }
}
