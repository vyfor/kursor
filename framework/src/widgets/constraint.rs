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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConstraintProps {
    pub min_width: Option<u16>,
    pub max_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_height: Option<u16>,
}

pub struct Constraint;

impl Constraint {
    pub fn exact(width: u16, height: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ConstraintProps {
                min_width: Some(width),
                max_width: Some(width),
                min_height: Some(height),
                max_height: Some(height),
            },
            child,
        )
    }

    pub fn width(width: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ConstraintProps {
                min_width: Some(width),
                max_width: Some(width),
                ..Default::default()
            },
            child,
        )
    }

    pub fn height(height: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ConstraintProps {
                min_height: Some(height),
                max_height: Some(height),
                ..Default::default()
            },
            child,
        )
    }

    pub fn square(size: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::exact(size, size, child)
    }

    pub fn min(width: u16, height: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ConstraintProps {
                min_width: Some(width),
                min_height: Some(height),
                ..Default::default()
            },
            child,
        )
    }

    pub fn max(width: u16, height: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ConstraintProps {
                max_width: Some(width),
                max_height: Some(height),
                ..Default::default()
            },
            child,
        )
    }

    pub fn min_width(width: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ConstraintProps {
                min_width: Some(width),
                ..Default::default()
            },
            child,
        )
    }

    pub fn max_width(width: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ConstraintProps {
                max_width: Some(width),
                ..Default::default()
            },
            child,
        )
    }

    pub fn min_height(height: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ConstraintProps {
                min_height: Some(height),
                ..Default::default()
            },
            child,
        )
    }

    pub fn max_height(height: u16, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ConstraintProps {
                max_height: Some(height),
                ..Default::default()
            },
            child,
        )
    }

    pub fn with(props: ConstraintProps, child: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(props).child(child)
    }
}

impl Component for Constraint {
    type Props = ConstraintProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        props: &Self::Props,
        available: Size,
        children: &mut MeasureCx,
    ) -> Size {
        let child_avail_width = props
            .max_width
            .map_or(available.width, |m| m.min(available.width));
        let child_avail_height = props
            .max_height
            .map_or(available.height, |m| m.min(available.height));

        let child_size = if children.is_empty() {
            Size::default()
        } else {
            children.measure(0, Size::new(child_avail_width, child_avail_height))
        };

        let mut width = child_size.width;
        if let Some(min) = props.min_width {
            width = width.max(min);
        }
        if let Some(max) = props.max_width {
            width = width.min(max);
        }

        let mut height = child_size.height;
        if let Some(min) = props.min_height {
            height = height.max(min);
        }
        if let Some(max) = props.max_height {
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
