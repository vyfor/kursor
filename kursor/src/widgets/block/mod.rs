pub mod builder;
pub use builder::BlockBuilder;

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
    render::{canvas::Canvas, style::Style},
    state::Value,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BorderChars {
    pub horizontal: char,
    pub vertical: char,
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
}

impl BorderChars {
    pub const PLAIN: Self = Self {
        horizontal: '─',
        vertical: '│',
        top_left: '┌',
        top_right: '┐',
        bottom_left: '└',
        bottom_right: '┘',
    };

    pub const ROUNDED: Self = Self {
        horizontal: '─',
        vertical: '│',
        top_left: '╭',
        top_right: '╮',
        bottom_left: '╰',
        bottom_right: '╯',
    };

    pub const DOUBLE: Self = Self {
        horizontal: '═',
        vertical: '║',
        top_left: '╔',
        top_right: '╗',
        bottom_left: '╚',
        bottom_right: '╝',
    };
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Border {
    #[default]
    Plain,
    None,
    Rounded,
    Double,
    Custom(BorderChars),
}

crate::core::into_value!(Border);

#[derive(Clone, PartialEq, Eq)]
pub struct BlockProps {
    pub border: Value<Border>,
    pub style: Value<Option<Style>>,
}

impl Default for BlockProps {
    fn default() -> Self {
        Self {
            border: Value::plain(Border::Plain),
            style: Value::plain(None),
        }
    }
}

pub struct Block {
    border: Border,
    style: Option<Style>,
}

impl Block {
    pub fn builder() -> BlockBuilder {
        BlockBuilder::new()
    }

    pub fn new(child: impl IntoBlueprint) -> Blueprint {
        Self::with(BlockProps::default(), child)
    }

    pub fn plain(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BlockProps {
                border: Value::plain(Border::Plain),
                style: Value::plain(None),
            },
            child,
        )
    }

    pub fn rounded(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BlockProps {
                border: Value::plain(Border::Rounded),
                style: Value::plain(None),
            },
            child,
        )
    }

    pub fn double(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BlockProps {
                border: Value::plain(Border::Double),
                style: Value::plain(None),
            },
            child,
        )
    }

    pub fn styled(style: Style, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BlockProps {
                style: Value::plain(Some(style)),
                ..BlockProps::default()
            },
            child,
        )
    }

    pub fn with(props: BlockProps, child: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(props).children(child)
    }

    fn inset(border: Border) -> u16 {
        u16::from(!matches!(border, Border::None))
    }
}

impl Component for Block {
    type Props = BlockProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            border: Border::Plain,
            style: None,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let border = props.border.get();
        let style = props.style.get();
        let border_changed = self.border != border;
        let style_changed = self.style != style;
        self.border = border;
        self.style = style;
        if border_changed {
            Update::MEASURE
        } else if style_changed {
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
        children: &mut MeasureCx,
    ) -> Size {
        let inset = Self::inset(self.border).saturating_mul(2);
        let child = if children.is_empty() {
            Size::default()
        } else {
            children.size(0)
        };
        Size::new(
            child.width.saturating_add(inset).min(available.width),
            child.height.saturating_add(inset).min(available.height),
        )
    }

    fn layout(&mut self, _cx: &mut Cx, _props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        if children.is_empty() {
            return;
        }
        let inset = Self::inset(self.border);
        let double = inset.saturating_mul(2);
        children.set(
            0,
            Rect::new(
                area.x.saturating_add(inset),
                area.y.saturating_add(inset),
                area.width.saturating_sub(double),
                area.height.saturating_sub(double),
            ),
        );
    }

    fn paint(&self, cx: &mut Cx, _props: &Self::Props, canvas: &mut Canvas) {
        let rect = cx.rect;
        if rect.width == 0 || rect.height == 0 {
            return;
        }

        let style = self.style.unwrap_or(cx.theme().surface);
        canvas.fill(rect, ' ', style);

        let chars = match self.border {
            Border::None => return,
            Border::Plain => BorderChars::PLAIN,
            Border::Rounded => BorderChars::ROUNDED,
            Border::Double => BorderChars::DOUBLE,
            Border::Custom(chars) => chars,
        };

        let right = rect.right().saturating_sub(1);
        let bottom = rect.bottom().saturating_sub(1);

        for x in rect.x..=right {
            canvas.set(x, rect.y, chars.horizontal, style);
            canvas.set(x, bottom, chars.horizontal, style);
        }
        for y in rect.y..=bottom {
            canvas.set(rect.x, y, chars.vertical, style);
            canvas.set(right, y, chars.vertical, style);
        }

        canvas.set(rect.x, rect.y, chars.top_left, style);
        canvas.set(right, rect.y, chars.top_right, style);
        canvas.set(rect.x, bottom, chars.bottom_left, style);
        canvas.set(right, bottom, chars.bottom_right, style);
    }
}
