pub mod builder;
pub use builder::AlignBuilder;

use kursor_core::{
    component::{
        Component, Update,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::{
        Alignment, HAlign, VAlign,
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    state::{IntoValue, Value},
};

#[derive(Clone, PartialEq, Eq)]
pub struct AlignProps {
    pub alignment: Value<Alignment>,
}

pub struct Align {
    alignment: Alignment,
}

impl Align {
    pub fn builder(child: impl IntoBlueprint) -> AlignBuilder {
        AlignBuilder::new(child)
    }

    pub fn new(alignment: impl IntoValue<Alignment>, child: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(AlignProps {
            alignment: alignment.into_value(),
        })
        .child(child)
    }

    pub fn top_left(child: impl IntoBlueprint) -> Blueprint {
        Self::new(Alignment::TOP_LEFT, child)
    }

    pub fn top_center(child: impl IntoBlueprint) -> Blueprint {
        Self::new(Alignment::TOP_CENTER, child)
    }

    pub fn top_right(child: impl IntoBlueprint) -> Blueprint {
        Self::new(Alignment::TOP_RIGHT, child)
    }

    pub fn center_left(child: impl IntoBlueprint) -> Blueprint {
        Self::new(Alignment::CENTER_LEFT, child)
    }

    pub fn center(child: impl IntoBlueprint) -> Blueprint {
        Self::new(Alignment::CENTER, child)
    }

    pub fn center_right(child: impl IntoBlueprint) -> Blueprint {
        Self::new(Alignment::CENTER_RIGHT, child)
    }

    pub fn bottom_left(child: impl IntoBlueprint) -> Blueprint {
        Self::new(Alignment::BOTTOM_LEFT, child)
    }

    pub fn bottom_center(child: impl IntoBlueprint) -> Blueprint {
        Self::new(Alignment::BOTTOM_CENTER, child)
    }

    pub fn bottom_right(child: impl IntoBlueprint) -> Blueprint {
        Self::new(Alignment::BOTTOM_RIGHT, child)
    }
}

impl Component for Align {
    type Props = AlignProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            alignment: Alignment::TOP_LEFT,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let alignment = props.alignment.get();
        if self.alignment == alignment {
            Update::NONE
        } else {
            self.alignment = alignment;
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
        let Some(size) = (!children.is_empty()).then(|| children.size(0)) else {
            return Size::default();
        };
        Size::new(
            size.width.min(available.width),
            size.height.min(available.height),
        )
    }

    fn layout(&mut self, _cx: &mut Cx, _props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        for index in 0..children.len() {
            let size = children.size(index);
            let width = size.width.min(area.width);
            let height = size.height.min(area.height);
            let x = match self.alignment.horizontal {
                HAlign::Left => area.x,
                HAlign::Center => area.x.saturating_add(area.width.saturating_sub(width) / 2),
                HAlign::Right => area.x.saturating_add(area.width.saturating_sub(width)),
            };
            let y = match self.alignment.vertical {
                VAlign::Top => area.y,
                VAlign::Center => area
                    .y
                    .saturating_add(area.height.saturating_sub(height) / 2),
                VAlign::Bottom => area.y.saturating_add(area.height.saturating_sub(height)),
            };
            children.set(index, Rect::new(x, y, width, height));
        }
    }
}
