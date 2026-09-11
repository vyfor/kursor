pub mod builder;
pub use builder::SpacerBuilder;

use kursor_core::{
    component::{Component, Update, blueprint::Blueprint, context::Cx},
    layout::{context::MeasureCx, size::Size},
    state::{IntoValue, Value},
};

#[derive(Clone, PartialEq, Eq)]
pub struct SpacerProps {
    pub size: Value<u16>,
}

/// reserves a fixed number of cells.
pub struct Spacer {
    size: u16,
}

impl Spacer {
    pub fn builder() -> SpacerBuilder {
        SpacerBuilder::new()
    }

    pub fn new(size: impl IntoValue<u16>) -> Blueprint {
        Blueprint::new::<Self>(SpacerProps {
            size: size.into_value(),
        })
    }
}

impl Component for Spacer {
    type Props = SpacerProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self { size: 0 }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let size = props.size.get();
        if self.size == size {
            Update::NONE
        } else {
            self.size = size;
            Update::MEASURE
        }
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        available: Size,
        _children: &mut MeasureCx,
    ) -> Size {
        let size = self.size.min(available.width).min(available.height);
        Size::new(size, size)
    }
}
