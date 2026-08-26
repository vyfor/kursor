pub mod builder;
pub use builder::BoundsBuilder;

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

#[derive(Clone, Default, PartialEq, Eq)]
pub struct BoundsProps {
    pub min_width: Option<Value<u16>>,
    pub max_width: Option<Value<u16>>,
    pub min_height: Option<Value<u16>>,
    pub max_height: Option<Value<u16>>,
}

pub struct Bounds {
    min_width: Option<u16>,
    max_width: Option<u16>,
    min_height: Option<u16>,
    max_height: Option<u16>,
}

impl Bounds {
    pub fn builder(child: impl IntoBlueprint) -> BoundsBuilder {
        BoundsBuilder::new(child)
    }

    pub fn exact(
        width: impl IntoValue<u16>,
        height: impl IntoValue<u16>,
        child: impl IntoBlueprint,
    ) -> Blueprint {
        let width = width.into_value();
        let height = height.into_value();
        Self::with(
            BoundsProps {
                min_width: Some(width.clone()),
                max_width: Some(width),
                min_height: Some(height.clone()),
                max_height: Some(height),
            },
            child,
        )
    }

    pub fn width(width: impl IntoValue<u16>, child: impl IntoBlueprint) -> Blueprint {
        let width = width.into_value();
        Self::with(
            BoundsProps {
                min_width: Some(width.clone()),
                max_width: Some(width),
                ..Default::default()
            },
            child,
        )
    }

    pub fn height(height: impl IntoValue<u16>, child: impl IntoBlueprint) -> Blueprint {
        let height = height.into_value();
        Self::with(
            BoundsProps {
                min_height: Some(height.clone()),
                max_height: Some(height),
                ..Default::default()
            },
            child,
        )
    }

    pub fn square(size: impl IntoValue<u16>, child: impl IntoBlueprint) -> Blueprint {
        let size = size.into_value();
        Self::exact(size.clone(), size, child)
    }

    pub fn min(
        width: impl IntoValue<u16>,
        height: impl IntoValue<u16>,
        child: impl IntoBlueprint,
    ) -> Blueprint {
        Self::with(
            BoundsProps {
                min_width: Some(width.into_value()),
                min_height: Some(height.into_value()),
                ..Default::default()
            },
            child,
        )
    }

    pub fn max(
        width: impl IntoValue<u16>,
        height: impl IntoValue<u16>,
        child: impl IntoBlueprint,
    ) -> Blueprint {
        Self::with(
            BoundsProps {
                max_width: Some(width.into_value()),
                max_height: Some(height.into_value()),
                ..Default::default()
            },
            child,
        )
    }

    pub fn min_width(width: impl IntoValue<u16>, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BoundsProps {
                min_width: Some(width.into_value()),
                ..Default::default()
            },
            child,
        )
    }

    pub fn max_width(width: impl IntoValue<u16>, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BoundsProps {
                max_width: Some(width.into_value()),
                ..Default::default()
            },
            child,
        )
    }

    pub fn min_height(height: impl IntoValue<u16>, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BoundsProps {
                min_height: Some(height.into_value()),
                ..Default::default()
            },
            child,
        )
    }

    pub fn max_height(height: impl IntoValue<u16>, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BoundsProps {
                max_height: Some(height.into_value()),
                ..Default::default()
            },
            child,
        )
    }

    pub fn with(props: BoundsProps, child: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(props).child(child)
    }
}

impl Component for Bounds {
    type Props = BoundsProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let min_width = props.min_width.as_ref().map(Value::get);
        let max_width = props.max_width.as_ref().map(Value::get);
        let min_height = props.min_height.as_ref().map(Value::get);
        let max_height = props.max_height.as_ref().map(Value::get);
        if self.min_width == min_width
            && self.max_width == max_width
            && self.min_height == min_height
            && self.max_height == max_height
        {
            Update::NONE
        } else {
            self.min_width = min_width;
            self.max_width = max_width;
            self.min_height = min_height;
            self.max_height = max_height;
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
        let child_avail_width = self
            .max_width
            .map_or(available.width, |m| m.min(available.width));
        let child_avail_height = self
            .max_height
            .map_or(available.height, |m| m.min(available.height));

        let child_size = if children.is_empty() {
            Size::default()
        } else {
            children.measure(0, Size::new(child_avail_width, child_avail_height))
        };

        let mut width = child_size.width;
        if let Some(min) = self.min_width {
            width = width.max(min);
        }
        if let Some(max) = self.max_width {
            width = width.min(max);
        }

        let mut height = child_size.height;
        if let Some(min) = self.min_height {
            height = height.max(min);
        }
        if let Some(max) = self.max_height {
            height = height.min(max);
        }

        Size::new(width.min(available.width), height.min(available.height))
    }

    fn layout(&mut self, _cx: &mut Cx, _props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        if children.is_empty() {
            return;
        }
        children.set(0, area);
    }
}
