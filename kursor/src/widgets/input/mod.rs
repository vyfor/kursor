pub mod builder;
pub use builder::InputBuilder;

use std::sync::Arc;

use kursor_core::{
    component::{
        Component, Focus, Update,
        behavior::{Behavior, BehaviorCx},
        blueprint::Blueprint,
        context::Cx,
    },
    event::{Event, EventResult, Phase, key::KeyCode, mouse::MouseButton, mouse::MouseKind},
    layout::{context::MeasureCx, size::Size},
    render::{canvas::Canvas, cell::Cell, style::Style},
    state::{IntoValue, Transition, Value},
    theme::Theme,
};
use unicode_width::UnicodeWidthChar;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct InputState {
    pub focused: bool,
    pub text: String,
    pub cursor: usize,
    pub selection: Option<(usize, usize)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputIntent {
    Insert(char),
    Paste(String),
    Replace(String),
    DeleteBack,
    DeleteForward,
    DeleteWordBack,
    DeleteWordForward,
    CursorLeft,
    CursorRight,
    WordLeft,
    WordRight,
    Home,
    End,
    MoveTo(usize),
    SelectLeft,
    SelectRight,
    SelectAll,
    SelectToStart,
    SelectToEnd,
    Select(usize, usize),
    Copy,
    Cut,
    Submit,
    Cancel,
}

#[derive(Clone, Default, PartialEq)]
pub struct InputStyles {
    pub normal: Option<Value<Style>>,
    pub focused: Option<Value<Style>>,
    pub disabled: Option<Value<Style>>,
    pub placeholder: Option<Value<Style>>,
    pub selection: Option<Value<Style>>,
}

kursor_core::into_value!(InputStyles);

impl Eq for InputStyles {}

impl InputStyles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn normal(mut self, style: impl IntoValue<Style>) -> Self {
        self.normal = Some(style.into_value());
        self
    }

    pub fn focused(mut self, style: impl IntoValue<Style>) -> Self {
        self.focused = Some(style.into_value());
        self
    }

    pub fn disabled(mut self, style: impl IntoValue<Style>) -> Self {
        self.disabled = Some(style.into_value());
        self
    }

    pub fn placeholder(mut self, style: impl IntoValue<Style>) -> Self {
        self.placeholder = Some(style.into_value());
        self
    }

    pub fn selection(mut self, style: impl IntoValue<Style>) -> Self {
        self.selection = Some(style.into_value());
        self
    }
}

pub type InputDisplay = Arc<dyn Fn(char, usize, usize) -> char>;

#[derive(Default)]
pub struct InputBehavior;

impl Behavior for InputBehavior {
    type State = InputState;
    type Intent = InputIntent;

    fn event(&self, _cx: &BehaviorCx, event: &Event, _state: &Self::State) -> Option<Self::Intent> {
        match event {
            Event::Key(key) => {
                let ctrl = key.modifiers.ctrl;
                let shift = key.modifiers.shift;
                match (key.code, ctrl, shift) {
                    (KeyCode::Char(c), false, _) if !c.is_control() => Some(InputIntent::Insert(c)),
                    (KeyCode::Backspace, false, _) => Some(InputIntent::DeleteBack),
                    (KeyCode::Delete, false, _) => Some(InputIntent::DeleteForward),
                    (KeyCode::Left, false, false) => Some(InputIntent::CursorLeft),
                    (KeyCode::Right, false, false) => Some(InputIntent::CursorRight),
                    (KeyCode::Left, false, true) => Some(InputIntent::SelectLeft),
                    (KeyCode::Right, false, true) => Some(InputIntent::SelectRight),
                    (KeyCode::Home, false, false) => Some(InputIntent::Home),
                    (KeyCode::End, false, false) => Some(InputIntent::End),
                    (KeyCode::Home, false, true) => Some(InputIntent::SelectToStart),
                    (KeyCode::End, false, true) => Some(InputIntent::SelectToEnd),
                    (KeyCode::Left, true, _) => Some(InputIntent::WordLeft),
                    (KeyCode::Right, true, _) => Some(InputIntent::WordRight),
                    (KeyCode::Backspace, true, _) => Some(InputIntent::DeleteWordBack),
                    (KeyCode::Delete, true, _) => Some(InputIntent::DeleteWordForward),
                    (KeyCode::Char('a'), true, _) => Some(InputIntent::SelectAll),
                    (KeyCode::Char('c'), true, _) => Some(InputIntent::Copy),
                    (KeyCode::Char('x'), true, _) => Some(InputIntent::Cut),
                    (KeyCode::Enter, _, _) => Some(InputIntent::Submit),
                    (KeyCode::Esc, _, _) => Some(InputIntent::Cancel),
                    _ => None,
                }
            }
            Event::Paste(text) => {
                let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
                Some(InputIntent::Paste(filtered))
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct InputProps {
    pub value: Value<String>,
    pub placeholder: Value<String>,
    pub styles: Value<InputStyles>,
    pub disabled: Value<bool>,
    pub behavior: Arc<dyn Behavior<State = InputState, Intent = InputIntent>>,
    pub display: InputDisplay,
    pub transition: Option<Transition>,
    pub on_change: Arc<dyn Fn(&mut Cx, &str)>,
    pub on_submit: Arc<dyn Fn(&mut Cx)>,
}

impl InputProps {
    pub fn new(value: impl IntoValue<String>, on_change: impl Fn(&mut Cx, &str) + 'static) -> Self {
        Self {
            value: value.into_value(),
            placeholder: Value::plain(String::new()),
            styles: Value::plain(InputStyles::default()),
            disabled: Value::plain(false),
            behavior: Arc::new(InputBehavior),
            display: Arc::new(|ch, _, _| ch),
            transition: None,
            on_change: Arc::new(on_change),
            on_submit: Arc::new(|_| {}),
        }
    }
}

pub struct Input {
    state: InputState,
    styles: InputStyles,
    placeholder: String,
    disabled: bool,
    scroll: usize,
    display: InputDisplay,
    transition: Option<Transition>,
}

impl Input {
    pub fn builder(value: impl IntoValue<String>) -> InputBuilder {
        InputBuilder::new(value)
    }

    pub fn new(
        value: impl IntoValue<String>,
        on_change: impl Fn(&mut Cx, &str) + 'static,
    ) -> Blueprint {
        Self::with(InputProps::new(value, on_change))
    }

    pub fn with(props: InputProps) -> Blueprint {
        Blueprint::new::<Self>(props)
    }

    fn style(&self, cx: &mut Cx, theme: Theme) -> Style {
        let (val, fallback) = if self.disabled {
            (&self.styles.disabled, theme.disabled)
        } else if self.state.focused {
            (&self.styles.focused, theme.text)
        } else {
            (&self.styles.normal, theme.text)
        };

        cx.resolve_or("style", val, fallback, self.transition.clone())
    }

    fn char_count(&self) -> usize {
        self.state.text.chars().count()
    }

    fn char_to_byte(&self, idx: usize) -> usize {
        self.state
            .text
            .char_indices()
            .nth(idx)
            .map(|(i, _)| i)
            .unwrap_or_else(|| self.state.text.len())
    }

    fn display_char(&self, ch: char, idx: usize, total: usize) -> char {
        (self.display)(ch, idx, total)
    }

    fn display_width(&self, from: usize, to: usize) -> usize {
        let total = self.char_count();
        self.state
            .text
            .chars()
            .skip(from)
            .take(to.saturating_sub(from))
            .enumerate()
            .map(|(offset, ch)| {
                let idx = from + offset;
                let c = self.display_char(ch, idx, total);
                UnicodeWidthChar::width(c).unwrap_or(0)
            })
            .sum()
    }

    fn clamp_cursor(&mut self) {
        let max = self.char_count();
        if self.state.cursor > max {
            self.state.cursor = max;
        }
    }

    fn delete_selection(&mut self) {
        if let Some((s, e)) = self.state.selection {
            let byte_s = self.char_to_byte(s.min(e));
            let byte_e = self.char_to_byte(s.max(e));
            self.state.text.replace_range(byte_s..byte_e, "");
            self.state.cursor = s.min(e);
            self.state.selection = None;
        }
    }

    fn insert_char(&mut self, c: char) {
        self.delete_selection();
        let byte = self.char_to_byte(self.state.cursor);
        self.state.text.insert(byte, c);
        self.state.cursor += 1;
    }

    fn paste(&mut self, text: &str) {
        for c in text.chars() {
            self.insert_char(c);
        }
    }

    fn replace_text(&mut self, text: String) {
        self.state.text = text;
        self.clamp_cursor();
        self.state.selection = None;
    }

    fn delete_back(&mut self) {
        if self.state.selection.is_some() {
            self.delete_selection();
            return;
        }
        if self.state.cursor > 0 {
            let prev = self.char_to_byte(self.state.cursor - 1);
            let curr = self.char_to_byte(self.state.cursor);
            self.state.text.replace_range(prev..curr, "");
            self.state.cursor -= 1;
        }
    }

    fn delete_forward(&mut self) {
        if self.state.selection.is_some() {
            self.delete_selection();
            return;
        }
        let total = self.char_count();
        if self.state.cursor < total {
            let curr = self.char_to_byte(self.state.cursor);
            let next = self.char_to_byte(self.state.cursor + 1);
            self.state.text.replace_range(curr..next, "");
        }
    }

    fn delete_word_back(&mut self) {
        if self.state.cursor == 0 {
            self.delete_selection();
            return;
        }
        let start = self.state.cursor;
        let chars: Vec<char> = self.state.text.chars().collect();
        let mut end = start;
        while end > 0 && !is_valid(chars[end - 1]) {
            end -= 1;
        }
        while end > 0 && is_valid(chars[end - 1]) {
            end -= 1;
        }
        let byte_s = self.char_to_byte(end);
        let byte_e = self.char_to_byte(start);
        self.state.text.replace_range(byte_s..byte_e, "");
        self.state.cursor = end;
        self.state.selection = None;
    }

    fn delete_word_forward(&mut self) {
        let total = self.char_count();
        if self.state.cursor >= total {
            self.delete_selection();
            return;
        }
        let start = self.state.cursor;
        let chars: Vec<char> = self.state.text.chars().collect();
        let mut end = start;
        while end < total && is_valid(chars[end]) {
            end += 1;
        }
        while end < total && !is_valid(chars[end]) {
            end += 1;
        }
        let byte_s = self.char_to_byte(start);
        let byte_e = self.char_to_byte(end);
        self.state.text.replace_range(byte_s..byte_e, "");
        self.state.selection = None;
    }

    fn cursor_left(&mut self) {
        self.state.cursor = self.state.cursor.saturating_sub(1);
        self.state.selection = None;
    }

    fn cursor_right(&mut self) {
        if self.state.cursor < self.char_count() {
            self.state.cursor += 1;
        }
        self.state.selection = None;
    }

    fn word_left(&mut self) {
        if self.state.cursor == 0 {
            return;
        }
        let chars: Vec<char> = self.state.text.chars().collect();
        let mut pos = self.state.cursor;
        while pos > 0 && !is_valid(chars[pos - 1]) {
            pos -= 1;
        }
        while pos > 0 && is_valid(chars[pos - 1]) {
            pos -= 1;
        }
        self.state.cursor = pos;
        self.state.selection = None;
    }

    fn word_right(&mut self) {
        let total = self.char_count();
        if self.state.cursor >= total {
            return;
        }
        let chars: Vec<char> = self.state.text.chars().collect();
        let mut pos = self.state.cursor;
        while pos < total && is_valid(chars[pos]) {
            pos += 1;
        }
        while pos < total && !is_valid(chars[pos]) {
            pos += 1;
        }
        self.state.cursor = pos;
        self.state.selection = None;
    }

    fn goto_start(&mut self) {
        self.state.cursor = 0;
        self.state.selection = None;
    }

    fn goto_end(&mut self) {
        self.state.cursor = self.char_count();
        self.state.selection = None;
    }

    fn goto(&mut self, pos: usize) {
        self.state.cursor = pos;
        self.clamp_cursor();
        self.state.selection = None;
    }

    fn select_left(&mut self) {
        let anchor = self.selection_anchor();
        self.state.cursor = self.state.cursor.saturating_sub(1);
        self.update_selection(anchor);
    }

    fn select_right(&mut self) {
        let anchor = self.selection_anchor();
        if self.state.cursor < self.char_count() {
            self.state.cursor += 1;
        }
        self.update_selection(anchor);
    }

    fn select_all(&mut self) {
        let total = self.char_count();
        if total > 0 {
            self.state.selection = Some((0, total));
            self.state.cursor = total;
        }
    }

    fn select_to_start(&mut self) {
        let anchor = self.selection_anchor();
        self.state.cursor = 0;
        self.update_selection(anchor);
    }

    fn select_to_end(&mut self) {
        let anchor = self.selection_anchor();
        self.state.cursor = self.char_count();
        self.update_selection(anchor);
    }

    fn select(&mut self, anchor: usize, cursor: usize) {
        self.state.cursor = cursor;
        self.clamp_cursor();
        let mut anchor = anchor;
        if anchor > self.char_count() {
            anchor = self.char_count();
        }
        self.update_selection(anchor);
    }

    fn selection_anchor(&self) -> usize {
        match self.state.selection {
            Some((s, e)) if s == self.state.cursor => e,
            Some((s, _)) => s,
            None => self.state.cursor,
        }
    }

    fn update_selection(&mut self, anchor: usize) {
        if anchor != self.state.cursor {
            self.state.selection = Some((anchor, self.state.cursor));
        } else {
            self.state.selection = None;
        }
    }

    fn in_selection(&self, char_idx: usize) -> bool {
        match self.state.selection {
            Some((s, e)) => char_idx >= s.min(e) && char_idx < s.max(e),
            None => false,
        }
    }
    
    fn cut_sel(&mut self) -> bool {
        let had = self.state.selection.is_some();

        // todo: copy into clipboard

        if had {
            self.delete_selection();
        }
        had
    }

    fn click_to(&self, click_x: u16, rect_x: u16) -> usize {
        let rel = click_x.saturating_sub(rect_x) as usize + self.scroll;
        let total = self.char_count();
        let mut pos = 0usize;
        for (idx, ch) in self.state.text.chars().enumerate() {
            let c = self.display_char(ch, idx, total);
            let w = UnicodeWidthChar::width(c).unwrap_or(0);
            if pos + w / 2 >= rel {
                return idx;
            }
            pos += w;
        }
        total
    }

    fn update_scroll(&mut self, width: usize) {
        if width == 0 {
            self.scroll = 0;
            return;
        }
        let cursor_pos = self.display_width(0, self.state.cursor);
        if cursor_pos < self.scroll {
            self.scroll = cursor_pos;
        } else if cursor_pos >= self.scroll + width {
            self.scroll = cursor_pos + 1 - width;
        }
        let max_scroll = self
            .display_width(0, self.char_count())
            .saturating_sub(width);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }
    }

    fn execute(&mut self, intent: InputIntent) -> (bool, bool) {
        match intent {
            InputIntent::Insert(c) => {
                self.insert_char(c);
                (true, false)
            }
            InputIntent::Paste(s) => {
                self.paste(&s);
                (true, false)
            }
            InputIntent::Replace(s) => {
                self.replace_text(s);
                (true, false)
            }
            InputIntent::DeleteBack => {
                self.delete_back();
                (true, false)
            }
            InputIntent::DeleteForward => {
                self.delete_forward();
                (true, false)
            }
            InputIntent::DeleteWordBack => {
                self.delete_word_back();
                (true, false)
            }
            InputIntent::DeleteWordForward => {
                self.delete_word_forward();
                (true, false)
            }
            InputIntent::CursorLeft => {
                self.cursor_left();
                (false, false)
            }
            InputIntent::CursorRight => {
                self.cursor_right();
                (false, false)
            }
            InputIntent::WordLeft => {
                self.word_left();
                (false, false)
            }
            InputIntent::WordRight => {
                self.word_right();
                (false, false)
            }
            InputIntent::Home => {
                self.goto_start();
                (false, false)
            }
            InputIntent::End => {
                self.goto_end();
                (false, false)
            }
            InputIntent::MoveTo(pos) => {
                self.goto(pos);
                (false, false)
            }
            InputIntent::SelectLeft => {
                self.select_left();
                (false, false)
            }
            InputIntent::SelectRight => {
                self.select_right();
                (false, false)
            }
            InputIntent::SelectAll => {
                self.select_all();
                (false, false)
            }
            InputIntent::SelectToStart => {
                self.select_to_start();
                (false, false)
            }
            InputIntent::SelectToEnd => {
                self.select_to_end();
                (false, false)
            }
            InputIntent::Select(anchor, cursor) => {
                self.select(anchor, cursor);
                (false, false)
            }
            InputIntent::Copy => {
                // todo

                (false, false)
            }
            InputIntent::Cut => {
                let changed = self.cut_sel();
                (changed, false)
            }
            InputIntent::Submit => (false, true),
            InputIntent::Cancel => {
                self.state.selection = None;
                (false, false)
            }
        }
    }
}

fn is_valid(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl Component for Input {
    type Props = InputProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            state: InputState::default(),
            styles: InputStyles::default(),
            placeholder: String::new(),
            disabled: false,
            scroll: 0,
            display: Arc::new(|ch, _, _| ch),
            transition: None,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old.value != new.value
            || old.placeholder != new.placeholder
            || old.styles != new.styles
            || old.disabled != new.disabled
            || old.transition != new.transition
            || !Arc::ptr_eq(&old.behavior, &new.behavior)
            || !Arc::ptr_eq(&old.display, &new.display)
            || !Arc::ptr_eq(&old.on_change, &new.on_change)
            || !Arc::ptr_eq(&old.on_submit, &new.on_submit)
    }

    fn focus(&self, _props: &Self::Props) -> Focus {
        Focus {
            focusable: !self.disabled,
            trap: false,
        }
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        available: Size,
        _children: &mut MeasureCx,
    ) -> Size {
        Size::new(available.width, 1)
    }

    fn update(&mut self, cx: &mut Cx, props: &Self::Props) -> Update {
        let value = props.value.get();
        let placeholder = props.placeholder.get();
        let styles = props.styles.get();
        let disabled = props.disabled.get();

        let text_changed = value != self.state.text;
        if text_changed {
            self.state.text = value;
            self.clamp_cursor();
            self.state.selection = None;
        }
        self.placeholder = placeholder;
        self.styles = styles;
        self.disabled = disabled;
        self.display = props.display.clone();
        self.transition = props.transition.clone();
        self.update_scroll(cx.rect.width as usize);

        if text_changed {
            Update::MEASURE
        } else {
            Update::PAINT
        }
    }

    fn event(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        event: &Event,
        phase: Phase,
    ) -> EventResult {
        if phase != Phase::Bubble {
            return EventResult::Ignored;
        }

        let mut state_changed = false;

        match event {
            Event::Mouse(mouse) if !self.disabled => match mouse.kind {
                MouseKind::Down(MouseButton::Left) => {
                    cx.focus(true);
                    if !self.state.focused {
                        self.state.focused = true;
                        state_changed = true;
                    }
                    let target = self.click_to(mouse.column, cx.rect.x);
                    if target != self.state.cursor {
                        self.state.cursor = target;
                        state_changed = true;
                    }
                    if self.state.selection.is_some() {
                        self.state.selection = None;
                        state_changed = true;
                    }
                    if state_changed {
                        cx.capture(true);
                    }
                }
                MouseKind::Drag(MouseButton::Left) => {
                    let target = self.click_to(mouse.column, cx.rect.x);
                    if target != self.state.cursor {
                        let anchor = self.selection_anchor();
                        self.state.cursor = target;
                        self.update_selection(anchor);
                        state_changed = true;
                    }
                }
                _ => {}
            },
            Event::FocusIn if !self.state.focused => {
                self.state.focused = true;
                state_changed = true;
            }
            Event::FocusOut if self.state.focused => {
                self.state.focused = false;
                self.state.selection = None;
                state_changed = true;
            }
            _ => {}
        }

        if self.disabled || !self.state.focused {
            if state_changed {
                self.update_scroll(cx.rect.width as usize);
                cx.repaint_self();
                return EventResult::Consumed;
            }
            return EventResult::Ignored;
        }

        let bcx = BehaviorCx {
            phase,
            rect: cx.rect,
        };
        if let Some(intent) = props.behavior.event(&bcx, event, &self.state) {
            let (text_changed, is_submit) = self.execute(intent);
            if text_changed {
                (props.on_change)(cx, &self.state.text);
            }
            if is_submit {
                (props.on_submit)(cx);
            }
            self.update_scroll(cx.rect.width as usize);
            cx.repaint_self();
            return EventResult::Consumed;
        }

        if state_changed {
            self.update_scroll(cx.rect.width as usize);
            cx.repaint_self();
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }

    fn paint(&self, cx: &mut Cx, _props: &Self::Props, canvas: &mut Canvas) {
        let theme = *cx.theme();
        let style = self.style(cx, theme);
        let width = cx.rect.width as usize;
        let total = self.char_count();
        let chars: Vec<char> = self.state.text.chars().collect();

        let mut display_pos = 0usize;
        let mut idx = 0usize;
        let x = cx.rect.x;
        let y = cx.rect.y;

        for x in 0..cx.rect.width {
            let style = canvas
                .cell(x, 0)
                .map(|cell| cell.style.patch(style))
                .unwrap_or(style);
            canvas.set_cell(x, 0, Cell::new(' ', style));
        }

        while idx < total && display_pos < self.scroll + width {
            let raw_ch = chars[idx];
            let ch = self.display_char(raw_ch, idx, total);
            let w = UnicodeWidthChar::width(ch).unwrap_or(0);
            if display_pos + w > self.scroll {
                let render_x = x + (display_pos.saturating_sub(self.scroll)) as u16;
                let cell_style = if self.in_selection(idx) {
                    self.styles
                        .selection
                        .as_ref()
                        .map(Value::get)
                        .unwrap_or(theme.focus)
                } else {
                    style
                };
                canvas.set_cell(render_x, y, Cell::new(ch, cell_style));
            }
            display_pos += w;
            idx += 1;
        }

        if self.state.text.is_empty() && !self.placeholder.is_empty() {
            let placeholder = self.styles.placeholder.as_ref().map(Value::get);
            let mut ph_style_inner = placeholder.unwrap_or_default();
            if placeholder.is_none() {
                ph_style_inner.attrs.dim = true;
            }
            let ph_style = style.patch(ph_style_inner);
            let budget = width;
            let mut rendered = 0usize;
            let mut ph = String::new();
            for c in self.placeholder.chars() {
                let cw = UnicodeWidthChar::width(c).unwrap_or(0);
                if rendered + cw > budget {
                    break;
                }
                ph.push(c);
                rendered += cw;
            }
            canvas.set_str(x, y, &ph, ph_style);
        }

        if self.state.focused && !self.disabled {
            let cursor_pos = self.display_width(0, self.state.cursor);
            let cursor_x = x + (cursor_pos.saturating_sub(self.scroll)) as u16;
            if cursor_x < x + width as u16 {
                cx.cursor(Some((cursor_x, y)));
            }
        }
    }
}
