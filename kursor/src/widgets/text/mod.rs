pub mod builder;
pub mod line;
pub mod span;
pub use builder::TextBuilder;
pub use line::Line;
pub use span::Span;

use kursor_core::{
    component::{Component, Update, blueprint::Blueprint, context::Cx},
    layout::{WrapMode, context::MeasureCx, size::Size},
    render::{canvas::Canvas, style::Style},
    state::{IntoValue, Signal, Value, atom::Atom, memo::Memo},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Clone, PartialEq)]
#[doc(hidden)]
pub enum TextContent {
    Plain(Value<String>),
    Lines(Value<Vec<Line>>),
}

pub trait IntoText {
    fn into_text(self) -> TextContent;
}

impl IntoText for String {
    fn into_text(self) -> TextContent {
        TextContent::Plain(Value::plain(self))
    }
}

impl IntoText for &str {
    fn into_text(self) -> TextContent {
        TextContent::Plain(Value::plain(self.to_owned()))
    }
}

impl IntoText for Span {
    fn into_text(self) -> TextContent {
        TextContent::Lines(Value::plain(vec![Line::from(self)]))
    }
}

impl IntoText for Line {
    fn into_text(self) -> TextContent {
        TextContent::Lines(Value::plain(vec![self]))
    }
}

impl IntoText for Vec<Line> {
    fn into_text(self) -> TextContent {
        TextContent::Lines(Value::plain(self))
    }
}

impl<const N: usize> IntoText for [Line; N] {
    fn into_text(self) -> TextContent {
        TextContent::Lines(Value::plain(self.into()))
    }
}

impl IntoText for Value<String> {
    fn into_text(self) -> TextContent {
        TextContent::Plain(self)
    }
}

impl IntoText for Signal<String> {
    fn into_text(self) -> TextContent {
        TextContent::Plain(Value::signal(self))
    }
}

impl IntoText for Atom<String> {
    fn into_text(self) -> TextContent {
        TextContent::Plain(Value::atom(self))
    }
}

impl IntoText for Memo<String> {
    fn into_text(self) -> TextContent {
        TextContent::Plain(Value::memo(self))
    }
}

impl IntoText for Value<Vec<Line>> {
    fn into_text(self) -> TextContent {
        TextContent::Lines(self)
    }
}

impl IntoText for Signal<Vec<Line>> {
    fn into_text(self) -> TextContent {
        TextContent::Lines(Value::signal(self))
    }
}

impl IntoText for Atom<Vec<Line>> {
    fn into_text(self) -> TextContent {
        TextContent::Lines(Value::atom(self))
    }
}

impl IntoText for Memo<Vec<Line>> {
    fn into_text(self) -> TextContent {
        TextContent::Lines(Value::memo(self))
    }
}

impl IntoText for TextContent {
    fn into_text(self) -> TextContent {
        self
    }
}

#[derive(Clone, PartialEq)]
pub struct TextProps {
    pub text: TextContent,
    pub style: Value<Option<Style>>,
    pub wrap: Value<WrapMode>,
}

pub struct Text {
    lines: Vec<Line>,
    style: Option<Style>,
    wrap: WrapMode,
}

impl Text {
    pub fn builder(text: impl IntoText) -> TextBuilder {
        TextBuilder::new(text)
    }

    pub fn new(text: impl IntoText) -> Blueprint {
        Self::with(TextProps {
            text: text.into_text(),
            style: Value::plain(None),
            wrap: Value::plain(WrapMode::None),
        })
    }

    pub fn styled(text: impl IntoText, style: impl IntoValue<Option<Style>>) -> Blueprint {
        Self::with(TextProps {
            text: text.into_text(),
            style: style.into_value(),
            wrap: Value::plain(WrapMode::None),
        })
    }

    pub fn line(line: impl Into<Line>) -> Blueprint {
        Self::new(line.into())
    }

    pub fn lines(lines: impl IntoIterator<Item = Line>) -> Blueprint {
        Self::new(lines.into_iter().collect::<Vec<_>>())
    }

    pub fn with(props: TextProps) -> Blueprint {
        Blueprint::new::<Self>(props)
    }
}

fn wrap_lines(lines: &[Line], wrap: WrapMode, width: u16) -> Vec<Line> {
    if width == 0 {
        return Vec::new();
    }
    if wrap == WrapMode::None {
        return lines.to_vec();
    }

    let mut res = Vec::new();
    for line in lines {
        let mut current = Line::default();
        let mut current_width = 0usize;
        for span in &line.spans {
            let mut start = 0;
            for (index, ch) in span.text.char_indices() {
                let end = index + ch.len_utf8();
                let char_width = ch.width().unwrap_or(0);
                if current_width > 0 && current_width + char_width > width as usize {
                    res.push(current);
                    current = Line::default();
                    current_width = 0;
                    start = index;
                }
                current.spans.push(Span {
                    text: span.text[start..end].to_owned(),
                    style: span.style,
                });
                current_width += char_width;
                start = end;
            }
        }
        res.push(current);
    }
    res
}

impl Component for Text {
    type Props = TextProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            lines: Vec::new(),
            style: None,
            wrap: WrapMode::None,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let lines = match &props.text {
            TextContent::Plain(text) => text.get().split('\n').map(Line::from).collect(),
            TextContent::Lines(lines) => lines.get(),
        };
        let style = props.style.get();
        let wrap = props.wrap.get();
        let text_changed = self.lines != lines;
        let style_changed = self.style != style;
        let wrap_changed = self.wrap != wrap;
        self.lines = lines;
        self.style = style;
        self.wrap = wrap;
        if text_changed || wrap_changed {
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
        _children: &mut MeasureCx,
    ) -> Size {
        let lines = wrap_lines(&self.lines, self.wrap, available.width);
        let width = lines.iter().map(Line::width).max().unwrap_or(0);
        Size::new(
            (width as u16).min(available.width),
            (lines.len() as u16).min(available.height),
        )
    }

    fn paint(&self, cx: &mut Cx, _props: &Self::Props, canvas: &mut Canvas) {
        let default_style = self.style.unwrap_or(cx.theme().text);
        let lines = wrap_lines(&self.lines, self.wrap, cx.rect.width);
        for (row, line) in lines.iter().enumerate() {
            let y = cx.rect.y.saturating_add(row as u16);
            if y >= cx.rect.bottom() {
                break;
            }
            let mut x = cx.rect.x;
            for span in &line.spans {
                let style = span.style.unwrap_or(default_style);
                canvas.set_str(x, y, &span.text, style);
                x = x.saturating_add(UnicodeWidthStr::width(span.text.as_str()) as u16);
            }
        }
    }
}
