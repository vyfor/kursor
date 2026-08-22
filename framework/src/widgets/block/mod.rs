pub mod builder;
pub use builder::BlockBuilder;

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
    render::{canvas::Canvas, style::Style},
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

#[derive(Clone, Default, PartialEq, Eq)]
pub struct BlockProps {
    pub border: Border,
    pub style: Option<Style>,
}

pub struct Block;

impl Block {
    pub fn builder(child: impl IntoBlueprint) -> BlockBuilder {
        BlockBuilder::new(child)
    }

    pub fn new(child: impl IntoBlueprint) -> Blueprint {
        Self::with(BlockProps::default(), child)
    }

    pub fn plain(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BlockProps {
                border: Border::Plain,
                style: None,
            },
            child,
        )
    }

    pub fn rounded(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BlockProps {
                border: Border::Rounded,
                style: None,
            },
            child,
        )
    }

    pub fn double(child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BlockProps {
                border: Border::Double,
                style: None,
            },
            child,
        )
    }

    pub fn styled(style: Style, child: impl IntoBlueprint) -> Blueprint {
        Self::with(
            BlockProps {
                style: Some(style),
                ..BlockProps::default()
            },
            child,
        )
    }

    pub fn with(props: BlockProps, child: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(props).child(child)
    }

    fn inset(props: &BlockProps) -> u16 {
        u16::from(!matches!(props.border, Border::None))
    }
}

impl Component for Block {
    type Props = BlockProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        props: &Self::Props,
        available: Size,
        children: &mut MeasureCx,
    ) -> Size {
        let inset = Self::inset(props).saturating_mul(2);
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

    fn layout(&mut self, _cx: &mut Cx, props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        if children.is_empty() {
            return;
        }
        let inset = Self::inset(props);
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

        let style = props.style.unwrap_or(cx.theme().surface);
        canvas.fill(rect, ' ', style);

        let chars = match props.border {
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
