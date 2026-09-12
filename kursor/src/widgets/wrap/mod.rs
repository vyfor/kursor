pub mod builder;
pub use builder::WrapBuilder;

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
pub struct WrapProps {
    pub gap: Value<u16>,
    pub line_gap: Value<u16>,
}

impl Default for WrapProps {
    fn default() -> Self {
        Self {
            gap: Value::plain(0),
            line_gap: Value::plain(0),
        }
    }
}

/// wraps children across lines.
///
/// if adding the next child would exceed the available width, it starts on a
/// new line.
pub struct Wrap {
    gap: u16,
    line_gap: u16,
}

impl Wrap {
    pub fn new(children: impl IntoBlueprint) -> WrapBuilder {
        WrapBuilder::new(children)
    }

    pub fn uniform(
        gap: impl IntoValue<u16>,
        children: impl IntoBlueprint,
    ) -> Blueprint {
        let gap = gap.into_value();
        Self::new(children).gap(gap.clone()).line_gap(gap).build()
    }

    pub fn spaced(
        gap: impl IntoValue<u16>,
        line_gap: impl IntoValue<u16>,
        children: impl IntoBlueprint,
    ) -> Blueprint {
        Self::new(children).gap(gap).line_gap(line_gap).build()
    }

    pub fn with(props: WrapProps, children: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(props).children(children)
    }
}

impl Component for Wrap {
    type Props = WrapProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            gap: 0,
            line_gap: 0,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let gap = props.gap.get();
        let line_gap = props.line_gap.get();
        if self.gap == gap && self.line_gap == line_gap {
            Update::NONE
        } else {
            self.gap = gap;
            self.line_gap = line_gap;
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
        let mut lw = 0u16;
        let mut lh = 0u16;
        let mut w = 0u16;
        let mut h = 0u16;
        let mut empty = true;

        for index in 0..children.len() {
            let size = children.measure(index, available);
            let child_w = size.width.min(available.width);
            let gap = if empty { 0 } else { self.gap };
            let next_w = u32::from(lw) + u32::from(gap) + u32::from(child_w);

            if !empty && next_w > u32::from(available.width) {
                w = w.max(lw);
                h = h.saturating_add(lh).saturating_add(self.line_gap);
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

    fn layout(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        area: Rect,
        children: &mut LayoutCx,
    ) {
        let mut lw = 0u16;
        let mut lh = 0u16;
        let mut y = area.y;
        let mut empty = true;

        for index in 0..children.len() {
            let size = children.size(index);
            let child_w = size.width.min(area.width);
            let gap = if empty { 0 } else { self.gap };
            let next_w = u32::from(lw) + u32::from(gap) + u32::from(child_w);

            if !empty && next_w > u32::from(area.width) {
                y = y.saturating_add(lh).saturating_add(self.line_gap);
                lw = 0;
                lh = 0;
                empty = true;
            }

            let gap = if empty { 0 } else { self.gap };
            let x = area.x.saturating_add(lw).saturating_add(gap);
            children.set(index, Rect::new(x, y, child_w, size.height));
            lw = lw.saturating_add(gap).saturating_add(child_w);
            lh = lh.max(size.height);
            empty = false;
        }
    }
}
