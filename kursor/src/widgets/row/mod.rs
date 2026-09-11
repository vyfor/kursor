pub mod builder;
pub use builder::RowBuilder;

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
pub struct RowProps {
    pub gap: Value<u16>,
}

impl Default for RowProps {
    fn default() -> Self {
        Self {
            gap: Value::plain(0),
        }
    }
}

/// places children in a horizontal row.
pub struct Row {
    gap: u16,
}

impl Row {
    pub fn builder(children: impl IntoBlueprint) -> RowBuilder {
        RowBuilder::new(children)
    }

    pub fn new(children: impl IntoBlueprint) -> Blueprint {
        Self::spaced(0, children)
    }

    pub fn spaced(
        gap: impl IntoValue<u16>,
        children: impl IntoBlueprint,
    ) -> Blueprint {
        Blueprint::new::<Self>(RowProps {
            gap: gap.into_value(),
        })
        .children(children)
    }
}

impl Component for Row {
    type Props = RowProps;

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
        let width = sizes
            .iter()
            .map(|size| usize::from(size.width))
            .sum::<usize>()
            .saturating_add(gaps)
            .min(usize::from(available.width)) as u16;
        let height = sizes
            .iter()
            .map(|size| size.height)
            .max()
            .unwrap_or(0)
            .min(available.height);

        Size::new(width, height)
    }

    fn layout(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        area: Rect,
        children: &mut LayoutCx,
    ) {
        let mut x = area.x;
        for index in 0..children.len() {
            let size = children.size(index);
            let width = size.width.min(area.right().saturating_sub(x));
            children.set(index, Rect::new(x, area.y, width, area.height));
            x = x.saturating_add(width).saturating_add(
                self.gap
                    .min(area.right().saturating_sub(x.saturating_add(width))),
            );
        }
    }
}
