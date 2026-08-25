pub mod builder;
pub use builder::DividerBuilder;

use kursor_core::{
    component::{Component, Update, blueprint::Blueprint, context::Cx},
    layout::{Orientation, context::MeasureCx, size::Size},
    render::{canvas::Canvas, style::Style},
    state::Value,
};

#[derive(Clone, PartialEq, Eq)]
pub struct DividerProps {
    pub orientation: Value<Orientation>,
    pub style: Value<Option<Style>>,
    pub glyph: Value<char>,
}

impl Default for DividerProps {
    fn default() -> Self {
        Self {
            orientation: Value::plain(Orientation::Horizontal),
            style: Value::plain(None),
            glyph: Value::plain('─'),
        }
    }
}

pub struct Divider {
    orientation: Orientation,
    style: Option<Style>,
    glyph: char,
}

impl Divider {
    pub fn builder() -> DividerBuilder {
        DividerBuilder::new()
    }

    pub fn new() -> Blueprint {
        Self::horizontal()
    }

    pub fn horizontal() -> Blueprint {
        Self::with(DividerProps::default())
    }

    pub fn vertical() -> Blueprint {
        Self::with(DividerProps {
            orientation: Value::plain(Orientation::Vertical),
            glyph: Value::plain('│'),
            ..DividerProps::default()
        })
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
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let orientation = props.orientation.get();
        let style = props.style.get();
        let glyph = props.glyph.get();
        let orientation_changed = self.orientation != orientation;
        let style_changed = self.style != style;
        let glyph_changed = self.glyph != glyph;
        self.orientation = orientation;
        self.style = style;
        self.glyph = glyph;
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
            Orientation::Horizontal => Size::new(available.width, available.height.min(1)),
            Orientation::Vertical => Size::new(available.width.min(1), available.height),
        }
    }

    fn paint(&self, cx: &mut Cx, _props: &Self::Props, canvas: &mut Canvas) {
        let rect = cx.rect;
        if rect.width == 0 || rect.height == 0 {
            return;
        }

        let style = self.style.unwrap_or(cx.theme().surface);
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
