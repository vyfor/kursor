pub mod builder;
pub use builder::ColumnBuilder;

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
pub struct ColumnProps {
    pub gap: u16,
}

pub struct Column;

impl Column {
    pub fn builder(children: impl IntoBlueprint) -> ColumnBuilder {
        ColumnBuilder::new(children)
    }

    pub fn new(children: impl IntoBlueprint) -> Blueprint {
        Self::spaced(0, children)
    }

    pub fn spaced(gap: u16, children: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(ColumnProps { gap }).children(children)
    }
}

impl Component for Column {
    type Props = ColumnProps;

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
        children: &mut MeasureCx,
    ) -> Size {
        let count = children.len();
        if count == 0 {
            return Size::default();
        }

        let gaps = usize::from(props.gap).saturating_mul(count.saturating_sub(1));
        let sizes: Vec<Size> = (0..count).map(|index| children.size(index)).collect();
        let height = sizes
            .iter()
            .map(|size| usize::from(size.height))
            .sum::<usize>()
            .saturating_add(gaps)
            .min(usize::from(available.height)) as u16;
        let width = sizes
            .iter()
            .map(|size| size.width)
            .max()
            .unwrap_or(0)
            .min(available.width);

        Size::new(width, height)
    }

    fn layout(&mut self, _cx: &mut Cx, props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        let mut y = area.y;
        for index in 0..children.len() {
            let size = children.size(index);
            let height = size.height.min(area.bottom().saturating_sub(y));
            children.set(index, Rect::new(area.x, y, area.width, height));
            y = y.saturating_add(height).saturating_add(
                props
                    .gap
                    .min(area.bottom().saturating_sub(y.saturating_add(height))),
            );
        }
    }
}
