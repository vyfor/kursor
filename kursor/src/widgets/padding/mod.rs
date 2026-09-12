pub mod builder;
pub use builder::PaddingBuilder;

use kursor_core::{
    component::{
        Component, Update,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::{
        Insets,
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    state::Value,
};

#[derive(Clone, PartialEq, Eq)]
pub struct PaddingProps {
    pub insets: Value<Insets>,
}

/// shrinks the child's rect by given `insets` on each side.
pub struct Padding {
    insets: Insets,
}

impl Padding {
    pub fn new(child: impl IntoBlueprint) -> PaddingBuilder {
        PaddingBuilder::new().children(child)
    }

    pub fn all(value: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).all(value).build()
    }

    pub fn symmetric(
        horizontal: u16,
        vertical: u16,
        child: impl IntoBlueprint,
    ) -> Blueprint {
        Self::new(child)
            .insets(Insets::symmetric(horizontal, vertical))
            .build()
    }

    pub fn horizontal(value: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).horizontal(value).build()
    }

    pub fn vertical(value: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).vertical(value).build()
    }

    pub fn top(value: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).insets(Insets::top(value)).build()
    }

    pub fn bottom(value: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).insets(Insets::bottom(value)).build()
    }

    pub fn left(value: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).insets(Insets::left(value)).build()
    }

    pub fn right(value: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).insets(Insets::right(value)).build()
    }

    pub fn with(props: PaddingProps, child: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(props).children(child)
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

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let insets = props.insets.get();
        if self.insets == insets {
            Update::NONE
        } else {
            self.insets = insets;
            Update::MEASURE
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

    fn layout(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        area: Rect,
        children: &mut LayoutCx,
    ) {
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
