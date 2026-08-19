use kursor_core::{
    component::{Component, blueprint::Blueprint, context::Cx},
    layout::{Orientation, context::MeasureCx, size::Size},
    render::{canvas::Canvas, style::Style},
};

#[derive(Clone)]
pub struct DividerProps {
    pub orientation: Orientation,
    pub style: Option<Style>,
    pub glyph: char,
}

impl Default for DividerProps {
    fn default() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            style: None,
            glyph: '─',
        }
    }
}

pub struct Divider;

impl Divider {
    pub fn new() -> Blueprint {
        Self::horizontal()
    }

    pub fn horizontal() -> Blueprint {
        Blueprint::new::<Self>(DividerProps::default())
    }

    pub fn vertical() -> Blueprint {
        Blueprint::new::<Self>(DividerProps {
            orientation: Orientation::Vertical,
            glyph: '│',
            ..DividerProps::default()
        })
    }
}

impl Component for Divider {
    type Props = DividerProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self
    }

    fn build(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        _children: Vec<Blueprint>,
    ) -> Vec<Blueprint> {
        Vec::new()
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        props: &Self::Props,
        available: Size,
        _children: &mut MeasureCx,
    ) -> Size {
        match props.orientation {
            Orientation::Horizontal => Size::new(available.width, available.height.min(1)),
            Orientation::Vertical => Size::new(available.width.min(1), available.height),
        }
    }

    fn paint(&self, cx: &mut Cx, props: &Self::Props, canvas: &mut Canvas) {
        let rect = cx.rect;
        if rect.width == 0 || rect.height == 0 {
            return;
        }

        let style = props.style.unwrap_or_default();
        match props.orientation {
            Orientation::Horizontal => {
                for x in rect.left()..rect.right() {
                    canvas.set(x, rect.top(), props.glyph, style);
                }
            }
            Orientation::Vertical => {
                for y in rect.top()..rect.bottom() {
                    canvas.set(rect.left(), y, props.glyph, style);
                }
            }
        }
    }
}
