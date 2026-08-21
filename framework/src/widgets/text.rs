use kursor_core::{
    component::{Component, blueprint::Blueprint, context::Cx},
    layout::{WrapMode, context::MeasureCx, size::Size},
    render::{canvas::Canvas, style::Style},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Clone)]
pub struct TextProps {
    pub text: String,
    pub style: Option<Style>,
    pub wrap: WrapMode,
}

pub struct Text;

impl Text {
    pub fn new(text: impl Into<String>) -> Blueprint {
        Self::with(TextProps {
            text: text.into(),
            style: None,
            wrap: WrapMode::None,
        })
    }

    pub fn styled(text: impl Into<String>, style: Style) -> Blueprint {
        Self::with(TextProps {
            text: text.into(),
            style: Some(style),
            wrap: WrapMode::None,
        })
    }

    pub fn with(props: TextProps) -> Blueprint {
        Blueprint::new::<Self>(props)
    }
}

fn wrapped_lines(text: &str, wrap: WrapMode, width: u16) -> Vec<&str> {
    if width == 0 {
        return Vec::new();
    }

    match wrap {
        WrapMode::None => text.lines().collect(),
        WrapMode::Character => char_wrap(text, width),
        WrapMode::Word => word_wrap(text, width),
    }
}

fn char_wrap(text: &str, width: u16) -> Vec<&str> {
    let mut lines = Vec::new();
    for line in text.lines() {
        if UnicodeWidthStr::width(line) <= width as usize {
            lines.push(line);
            continue;
        }
        let mut start = 0;
        let mut cell_width = 0;
        for (idx, ch) in line.char_indices() {
            let w = ch.width().unwrap_or(0);
            if cell_width + w > width as usize {
                lines.push(&line[start..idx]);
                start = idx;
                cell_width = 0;
            }
            cell_width += w;
        }
        lines.push(&line[start..]);
    }
    lines
}

fn word_wrap(text: &str, width: u16) -> Vec<&str> {
    let mut lines = Vec::new();
    for line in text.lines() {
        if UnicodeWidthStr::width(line) <= width as usize {
            lines.push(line);
            continue;
        }

        let mut start = 0;
        let mut line_width = 0;
        let mut last_break = None;

        for (idx, ch) in line.char_indices() {
            if ch == ' ' && line_width > 0 {
                last_break = Some((idx, line_width));
            }
            let w = ch.width().unwrap_or(0);
            if line_width + w > width as usize {
                match last_break {
                    Some((break_idx, _)) => {
                        lines.push(&line[start..break_idx]);
                        start = break_idx + 1;
                        line_width = UnicodeWidthStr::width(&line[start..=idx]);
                        last_break = None;
                    }
                    None => {
                        lines.push(&line[start..idx]);
                        start = idx;
                        line_width = w;
                    }
                }
            } else {
                line_width += w;
            }
        }
        lines.push(&line[start..]);
    }
    lines
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
        let lines = wrapped_lines(&props.text, props.wrap, available.width);
        let width = lines
            .iter()
            .map(|line| UnicodeWidthStr::width(*line))
            .max()
            .unwrap_or(0);
        let height = lines.len();

        Size::new(
            (width as u16).min(available.width),
            (height as u16).min(available.height),
        )
    }

    fn paint(&self, cx: &mut Cx, props: &Self::Props, canvas: &mut Canvas) {
        let style = props.style.unwrap_or_default();
        let lines = wrapped_lines(&props.text, props.wrap, cx.rect.width);
        for (row, line) in lines.iter().enumerate() {
            let y = cx.rect.y.saturating_add(row as u16);
            if y >= cx.rect.bottom() {
                break;
            }
            canvas.set_str(cx.rect.x, y, line, style);
        }
    }
}
