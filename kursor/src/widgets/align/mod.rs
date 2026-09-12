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
    state::Value,
};

#[derive(Clone, PartialEq, Eq)]
pub struct AlignProps {
    pub alignment: Value<Alignment>,
}

/// positions a child at a specific `alignment` within the allocated rect.
pub struct Align {
    alignment: Alignment,
}

impl Align {
    pub fn new(child: impl IntoBlueprint) -> AlignBuilder {
        AlignBuilder::new().children(child)
    }

    pub fn left(child: impl IntoBlueprint) -> Blueprint {
        Self::center_left(child)
    }

    pub fn right(child: impl IntoBlueprint) -> Blueprint {
        Self::center_right(child)
    }

    pub fn top(child: impl IntoBlueprint) -> Blueprint {
        Self::top_center(child)
    }

    pub fn bottom(child: impl IntoBlueprint) -> Blueprint {
        Self::bottom_center(child)
    }

    pub fn top_left(child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).top_left().build()
    }

    pub fn top_center(child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).top_center().build()
    }

    pub fn top_right(child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).top_right().build()
    }

    pub fn center_left(child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).center_left().build()
    }

    pub fn center(child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).center().build()
    }

    pub fn center_right(child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).center_right().build()
    }

    pub fn bottom_left(child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).bottom_left().build()
    }

    pub fn bottom_center(child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).bottom_center().build()
    }

    pub fn bottom_right(child: impl IntoBlueprint) -> Blueprint {
        Self::new(child).bottom_right().build()
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
        let Some(size) = (!children.is_empty()).then(|| children.size(0))
        else {
            return Size::default();
        };
        Size::new(
            size.width.min(available.width),
            size.height.min(available.height),
        )
    }

    fn layout(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        area: Rect,
        children: &mut LayoutCx,
    ) {
        for index in 0..children.len() {
            let size = children.size(index);
            let width = size.width.min(area.width);
            let height = size.height.min(area.height);
            let x = match self.alignment.horizontal {
                HAlign::Left => area.x,
                HAlign::Center => {
                    area.x.saturating_add(area.width.saturating_sub(width) / 2)
                }
                HAlign::Right => {
                    area.x.saturating_add(area.width.saturating_sub(width))
                }
            };
            let y = match self.alignment.vertical {
                VAlign::Top => area.y,
                VAlign::Center => area
                    .y
                    .saturating_add(area.height.saturating_sub(height) / 2),
                VAlign::Bottom => {
                    area.y.saturating_add(area.height.saturating_sub(height))
                }
            };
            children.set(index, Rect::new(x, y, width, height));
        }
    }
}
