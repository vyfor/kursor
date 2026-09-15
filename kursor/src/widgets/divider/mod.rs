pub mod builder;
pub use builder::DividerBuilder;

use kursor_core::{
    component::{Component, Update, blueprint::Blueprint, context::Cx},
    layout::{Orientation, context::MeasureCx, size::Size},
    render::{canvas::Canvas, style::Style},
    state::{Transition, Value},
};

#[derive(Clone, PartialEq)]
pub struct DividerProps {
    pub orientation: Value<Orientation>,
    pub style: Value<Option<Style>>,
    pub glyph: Value<char>,
    pub transition: Option<Transition>,
}

impl Default for DividerProps {
    fn default() -> Self {
        Self {
            orientation: Value::plain(Orientation::Horizontal),
            style: Value::plain(None),
            glyph: Value::plain('─'),
            transition: None,
        }
    }
}

/// draws a single character repeated across the full width or height of its
/// given rect.
pub struct Divider {
    orientation: Orientation,
    style: Option<Style>,
    glyph: char,
    transition: Option<Transition>,
}

impl Divider {
    pub fn new() -> DividerBuilder {
        DividerBuilder::new()
    }

    pub fn horizontal() -> Blueprint {
        Self::new().build()
    }

    pub fn vertical() -> Blueprint {
        Self::new()
            .orientation(Orientation::Vertical)
            .glyph('│')
            .build()
    }

    pub fn with(props: DividerProps) -> Blueprint {
        Blueprint::new::<Self>(props)
    }
}

impl Component for Divider {
    type Props = DividerProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            orientation: Orientation::Horizontal,
            style: None,
            glyph: '─',
            transition: None,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn update(&mut self, cx: &mut Cx, props: &Self::Props) -> Update {
        let _ = cx.inherited_style();
        let orientation = props.orientation.get();
        let style = props.style.get();
        let glyph = props.glyph.get();
        let transition = props.transition;
        let orientation_changed = self.orientation != orientation;
        let style_changed =
            self.style != style || self.transition != transition;
        let glyph_changed = self.glyph != glyph;
        self.orientation = orientation;
        self.style = style;
        self.glyph = glyph;
        self.transition = transition;
        if orientation_changed {
            Update::MEASURE
        } else if style_changed || glyph_changed {
            Update::PAINT
        } else {
            Update::NONE
        }
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        available: Size,
        _children: &mut MeasureCx,
    ) -> Size {
        match self.orientation {
            Orientation::Horizontal => {
                Size::new(available.width, available.height.min(1))
            }
            Orientation::Vertical => {
                Size::new(available.width.min(1), available.height)
            }
        }
    }

    fn paint(&self, cx: &mut Cx, props: &Self::Props, canvas: &mut Canvas) {
        let rect = cx.rect;
        if rect.width == 0 || rect.height == 0 {
            return;
        }

        let fallback = cx.theme().surface.patch(cx.inherited_style());
        let style =
            cx.resolve_or("style", &props.style, fallback, self.transition);
        match self.orientation {
            Orientation::Horizontal => {
                for x in rect.left()..rect.right() {
                    canvas.set(x, rect.top(), self.glyph, style);
                }
            }
            Orientation::Vertical => {
                for y in rect.top()..rect.bottom() {
                    canvas.set(rect.left(), y, self.glyph, style);
                }
            }
        }
    }
}
