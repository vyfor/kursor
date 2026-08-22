pub mod builder;
pub use builder::WrapBuilder;

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
pub struct WrapProps {
    pub gap: u16,
    pub line_gap: u16,
}

pub struct Wrap;

impl Wrap {
    pub fn builder(children: impl IntoBlueprint) -> WrapBuilder {
        WrapBuilder::new(children)
    }

    pub fn new(children: impl IntoBlueprint) -> Blueprint {
        Self::uniform(0, children)
    }

    pub fn uniform(gap: u16, children: impl IntoBlueprint) -> Blueprint {
        Self::spaced(gap, gap, children)
    }

    pub fn spaced(gap: u16, line_gap: u16, children: impl IntoBlueprint) -> Blueprint {
        Self::with(WrapProps { gap, line_gap }, children)
    }

    pub fn with(props: WrapProps, children: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(props).children(children)
    }
}

impl Component for Wrap {
    type Props = WrapProps;

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
        let mut lw = 0u16;
        let mut lh = 0u16;
        let mut w = 0u16;
        let mut h = 0u16;
        let mut empty = true;

        for index in 0..children.len() {
            let size = children.measure(index, available);
            let child_w = size.width.min(available.width);
            let gap = if empty { 0 } else { props.gap };
            let next_w = u32::from(lw) + u32::from(gap) + u32::from(child_w);

            if !empty && next_w > u32::from(available.width) {
                w = w.max(lw);
                h = h.saturating_add(lh).saturating_add(props.line_gap);
                lw = child_w;
                lh = size.height;
            } else {
                lw = lw.saturating_add(gap).saturating_add(child_w);
                lh = lh.max(size.height);
            }
            empty = false;
        }

        if !empty {
            w = w.max(lw);
            h = h.saturating_add(lh);
        }

        Size::new(w.min(available.width), h.min(available.height))
    }

    fn layout(&mut self, _cx: &mut Cx, props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        let mut lw = 0u16;
        let mut lh = 0u16;
        let mut y = area.y;
        let mut empty = true;

        for index in 0..children.len() {
            let size = children.size(index);
            let child_w = size.width.min(area.width);
            let gap = if empty { 0 } else { props.gap };
            let next_w = u32::from(lw) + u32::from(gap) + u32::from(child_w);

            if !empty && next_w > u32::from(area.width) {
                y = y.saturating_add(lh).saturating_add(props.line_gap);
                lw = 0;
                lh = 0;
                empty = true;
            }

            let gap = if empty { 0 } else { props.gap };
            let x = area.x.saturating_add(lw).saturating_add(gap);
            children.set(index, Rect::new(x, y, child_w, size.height));
            lw = lw.saturating_add(gap).saturating_add(child_w);
            lh = lh.max(size.height);
            empty = false;
        }
    }
}
