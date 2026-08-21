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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StackProps {
    pub gap: u16,
}

pub struct Row;

impl Row {
    pub fn new(children: impl IntoBlueprint) -> Blueprint {
        Self::spaced(0, children)
    }

    pub fn spaced(gap: u16, children: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(StackProps { gap }).children(children)
    }
}

impl Component for Row {
    type Props = StackProps;

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
        let sizes: Vec<Size> = (0..count).map(|i| children.size(i)).collect();
        let width = sizes
            .iter()
            .map(|s| usize::from(s.width))
            .sum::<usize>()
            .saturating_add(gaps)
            .min(usize::from(available.width)) as u16;
        let height = sizes
            .iter()
            .map(|s| s.height)
            .max()
            .unwrap_or(0)
            .min(available.height);

        Size::new(width, height)
    }

    fn layout(&mut self, _cx: &mut Cx, props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        let mut x = area.x;
        for index in 0..children.len() {
            let size = children.size(index);
            let width = size.width.min(area.right().saturating_sub(x));
            children.set(index, Rect::new(x, area.y, width, area.height));
            x = x.saturating_add(width).saturating_add(
                props
                    .gap
                    .min(area.right().saturating_sub(x.saturating_add(width))),
            );
        }
    }
}

pub struct Column;

impl Column {
    pub fn new(children: impl IntoBlueprint) -> Blueprint {
        Self::spaced(0, children)
    }

    pub fn spaced(gap: u16, children: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(StackProps { gap }).children(children)
    }
}

impl Component for Column {
    type Props = StackProps;

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
        let sizes: Vec<Size> = (0..count).map(|i| children.size(i)).collect();
        let height = sizes
            .iter()
            .map(|s| usize::from(s.height))
            .sum::<usize>()
            .saturating_add(gaps)
            .min(usize::from(available.height)) as u16;
        let width = sizes
            .iter()
            .map(|s| s.width)
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
