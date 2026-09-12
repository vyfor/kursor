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
    state::{Transition, Value},
};

/// box-drawing border characters.
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

    pub const HEAVY: Self = Self {
        horizontal: '━',
        vertical: '┃',
        top_left: '┏',
        top_right: '┓',
        bottom_left: '┗',
        bottom_right: '┛',
    };

    pub const ASCII: Self = Self {
        horizontal: '-',
        vertical: '|',
        top_left: '+',
        top_right: '+',
        bottom_left: '+',
        bottom_right: '+',
    };
}

/// border kind for a `Block`.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Border {
    None,
    /// ```text
    /// ┌──┐
    /// │  │
    /// └──┘
    /// ```
    #[default]
    Plain,

    /// ```text
    /// ╭──╮
    /// │  │
    /// ╰──╯
    /// ```
    Rounded,
    /// ```text
    /// ╔══╗
    /// ║  ║
    /// ╚══╝
    /// ```
    Double,
    /// ```text
    /// ┏━━┓
    /// ┃  ┃
    /// ┗━━┛
    /// ```
    Heavy,
    Custom(BorderChars),
}

impl Border {
    pub const fn chars(&self) -> BorderChars {
        match self {
            Self::Plain => BorderChars::PLAIN,
            Self::Rounded => BorderChars::ROUNDED,
            Self::Double => BorderChars::DOUBLE,
            Self::Heavy => BorderChars::HEAVY,
            Self::Custom(chars) => *chars,
            Self::None => BorderChars {
                horizontal: ' ',
                vertical: ' ',
                top_left: ' ',
                top_right: ' ',
                bottom_left: ' ',
                bottom_right: ' ',
            },
        }
    }
}

crate::core::into_value!(Border);

#[derive(Clone, PartialEq)]
pub struct BlockProps {
    pub border: Value<Border>,
    pub style: Value<Option<Style>>,
    pub transition: Option<Transition>,
}

impl Default for BlockProps {
    fn default() -> Self {
        Self {
            border: Value::plain(Border::Plain),
            style: Value::plain(None),
            transition: None,
        }
    }
}

/// draws a [`Border`] around a child component. the child is inset by 1 cell on
/// each side so that its content doesn't collide with the border.
///
/// border merging / collapsing is handled by the [`Canvas`]
pub struct Block {
    border: Border,
    style: Option<Style>,
    transition: Option<Transition>,
}

impl Block {
    pub fn new(child: impl IntoBlueprint) -> BlockBuilder {
        BlockBuilder::new().children(child)
    }

    pub fn plain(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BlockProps {
                border: Value::plain(Border::Plain),
                style: Value::plain(None),
                transition: None,
            },
            child,
        )
    }

    pub fn rounded(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BlockProps {
                border: Value::plain(Border::Rounded),
                style: Value::plain(None),
                transition: None,
            },
            child,
        )
    }

    pub fn double(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BlockProps {
                border: Value::plain(Border::Double),
                style: Value::plain(None),
                transition: None,
            },
            child,
        )
    }

    pub fn heavy(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BlockProps {
                border: Value::plain(Border::Heavy),
                style: Value::plain(None),
                transition: None,
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
            transition: None,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let border = props.border.get();
        let style = props.style.get();
        let transition = props.transition.clone();
        let border_changed = self.border != border;
        let style_changed =
            self.style != style || self.transition != transition;
        self.border = border;
        self.style = style;
        self.transition = transition;
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

    fn layout(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        area: Rect,
        children: &mut LayoutCx,
    ) {
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

    fn paint(&self, cx: &mut Cx, props: &Self::Props, canvas: &mut Canvas) {
        let rect = cx.rect;
        if rect.width == 0 || rect.height == 0 {
            return;
        }

        let fallback = cx.theme().surface;
        let style = cx.resolve_or(
            "style",
            &props.style,
            fallback,
            self.transition.clone(),
        );

        let has_border = !matches!(self.border, Border::None);
        let fill_rect = if has_border {
            Rect::new(
                rect.x.saturating_add(1),
                rect.y.saturating_add(1),
                rect.width.saturating_sub(2),
                rect.height.saturating_sub(2),
            )
        } else {
            rect
        };
        canvas.fill(fill_rect, ' ', style);

        let chars = match self.border {
            Border::None => return,
            Border::Plain => BorderChars::PLAIN,
            Border::Rounded => BorderChars::ROUNDED,
            Border::Double => BorderChars::DOUBLE,
            Border::Heavy => BorderChars::HEAVY,
            Border::Custom(chars) => chars,
        };

        let right = rect.right().saturating_sub(1);
        let bottom = rect.bottom().saturating_sub(1);

        if right > rect.x {
            for x in (rect.x + 1)..right {
                canvas.set(x, rect.y, chars.horizontal, style);
                canvas.set(x, bottom, chars.horizontal, style);
            }
        }
        if bottom > rect.y {
            for y in (rect.y + 1)..bottom {
                canvas.set(rect.x, y, chars.vertical, style);
                canvas.set(right, y, chars.vertical, style);
            }
        }

        canvas.set(rect.x, rect.y, chars.top_left, style);
        canvas.set(right, rect.y, chars.top_right, style);
        canvas.set(rect.x, bottom, chars.bottom_left, style);
        canvas.set(right, bottom, chars.bottom_right, style);
    }
}
