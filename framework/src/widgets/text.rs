use kursor_core::{
    component::{Component, blueprint::Blueprint, context::Cx},
    layout::{context::MeasureCx, size::Size},
    render::{canvas::Canvas, style::Style},
};
use unicode_width::UnicodeWidthStr;

#[derive(Clone)]
pub struct TextProps {
    pub text: String,
    pub style: Option<Style>,
}

pub struct Text;

impl Text {
    pub fn new(text: impl Into<String>) -> Blueprint {
        Blueprint::new::<Self>(TextProps {
            text: text.into(),
            style: None,
        })
    }

    pub fn styled(text: impl Into<String>, style: Style) -> Blueprint {
        Blueprint::new::<Self>(TextProps {
            text: text.into(),
            style: Some(style),
        })
    }
}

impl Component for Text {
    type Props = TextProps;

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
        let width = props
            .text
            .lines()
            .map(UnicodeWidthStr::width)
            .max()
            .unwrap_or(0);
        let height = props.text.lines().count();

        Size::new(
            (width as u16).min(available.width),
            (height as u16).min(available.height),
        )
    }

    fn paint(&self, cx: &mut Cx, props: &Self::Props, canvas: &mut Canvas) {
        let style = props.style.unwrap_or_default();
        for (row, line) in props.text.lines().enumerate() {
            let y = cx.rect.y.saturating_add(row as u16);
            if y >= cx.rect.bottom() {
                break;
            }
            canvas.set_str(cx.rect.x, y, line, style);
        }
    }
}
