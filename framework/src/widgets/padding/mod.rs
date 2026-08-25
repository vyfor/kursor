pub mod builder;
pub use builder::PaddingBuilder;

use kursor_core::{
    component::{
        Component,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::{
        Insets,
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    state::{IntoValue, Value},
};

#[derive(Clone, PartialEq, Eq)]
pub struct PaddingProps {
    pub insets: Value<Insets>,
}

pub struct Padding {
    insets: Insets,
}

impl Padding {
    pub fn builder(child: impl IntoBlueprint) -> PaddingBuilder {
        PaddingBuilder::new(child)
    }

    pub fn all(value: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(Insets::all(value), child)
    }

    pub fn symmetric(horizontal: u16, vertical: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(Insets::symmetric(horizontal, vertical), child)
    }

    pub fn horizontal(value: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(Insets::horizontal(value), child)
    }

    pub fn vertical(value: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(Insets::vertical(value), child)
    }

    pub fn top(value: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(Insets::top(value), child)
    }

    pub fn bottom(value: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(Insets::bottom(value), child)
    }

    pub fn left(value: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(Insets::left(value), child)
    }

    pub fn right(value: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(Insets::right(value), child)
    }

    pub fn new(insets: impl IntoValue<Insets>, child: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(PaddingProps {
            insets: insets.into_value(),
        })
        .child(child)
    }
}

impl Component for Padding {
    type Props = PaddingProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            insets: Insets::default(),
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn build(&mut self, _cx: &mut Cx, props: &Self::Props, _children: &mut kursor_core::component::Children) {
        self.insets = props.insets.get();
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

        let horizontal = self.insets.width();
        let vertical = self.insets.height();

        let child_available = Size::new(
            available.width.saturating_sub(horizontal),
            available.height.saturating_sub(vertical),
        );
        let child_size = children.measure(0, child_available);

        Size::new(
            child_size
                .width
                .saturating_add(horizontal)
                .min(available.width),
            child_size
                .height
                .saturating_add(vertical)
                .min(available.height),
        )
    }

    fn layout(&mut self, _cx: &mut Cx, _props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        if children.is_empty() {
            return;
        }

        let insets = self.insets;
        let horizontal = insets.width();
        let vertical = insets.height();

        children.set(
            0,
            Rect::new(
                area.x.saturating_add(insets.left),
                area.y.saturating_add(insets.top),
                area.width.saturating_sub(horizontal),
                area.height.saturating_sub(vertical),
            ),
        );
    }
}
