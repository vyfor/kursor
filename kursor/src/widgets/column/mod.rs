pub mod builder;
pub use builder::ColumnBuilder;

use kursor_core::{
    component::{
        Component, Update,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::{
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    state::{IntoValue, Value},
};

#[derive(Clone, PartialEq, Eq)]
pub struct ColumnProps {
    pub gap: Value<u16>,
}

impl Default for ColumnProps {
    fn default() -> Self {
        Self {
            gap: Value::plain(0),
        }
    }
}

/// places children in a vertical column.
pub struct Column {
    gap: u16,
}

impl Column {
    pub fn new(children: impl IntoBlueprint) -> ColumnBuilder {
        ColumnBuilder::new(children)
    }

    pub fn spaced(
        gap: impl IntoValue<u16>,
        children: impl IntoBlueprint,
    ) -> Blueprint {
        Self::new(children).gap(gap).build()
    }
}

impl Component for Column {
    type Props = ColumnProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self { gap: 0 }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let gap = props.gap.get();
        if self.gap == gap {
            Update::NONE
        } else {
            self.gap = gap;
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
        let count = children.len();
        if count == 0 {
            return Size::default();
        }

        let gaps =
            usize::from(self.gap).saturating_mul(count.saturating_sub(1));
        let sizes: Vec<Size> =
            (0..count).map(|index| children.size(index)).collect();
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

    fn layout(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        area: Rect,
        children: &mut LayoutCx,
    ) {
        let mut y = area.y;
        for index in 0..children.len() {
            let size = children.size(index);
            let height = size.height.min(area.bottom().saturating_sub(y));
            children.set(index, Rect::new(area.x, y, area.width, height));
            y =
                y.saturating_add(height).saturating_add(self.gap.min(
                    area.bottom().saturating_sub(y.saturating_add(height)),
                ));
        }
    }
}
