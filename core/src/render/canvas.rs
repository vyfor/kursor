use unicode_width::UnicodeWidthChar;

use crate::{
    layout::rect::Rect,
    render::{buffer::Buffer, cell::Cell, style::Style},
};

pub struct Canvas<'a> {
    buffer: &'a mut Buffer,
    rect: Rect,
}

impl<'a> Canvas<'a> {
    pub fn new(buffer: &'a mut Buffer, rect: Rect) -> Self {
        Self { buffer, rect }
    }

    pub fn set(&mut self, x: u16, y: u16, ch: char, style: Style) {
        self.set_cell(x, y, Cell::new(ch, style));
    }

    pub fn set_style(&mut self, x: u16, y: u16, ch: char, style: Style) {
        self.set(x, y, ch, style);
    }

    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) {
        if self.rect.contains(x, y) {
            self.buffer.set(x, y, cell);
        }
    }

    pub fn set_str(&mut self, mut x: u16, y: u16, text: &str, style: Style) {
        for ch in text.chars() {
            if x >= self.rect.right() {
                break;
            }
            let width = ch.width().unwrap_or(0) as u16;
            if width > 0 {
                self.set(x, y, ch, style);
                if width == 2 && x + 1 < self.rect.right() {
                    self.set(x + 1, y, ' ', style);
                }
                x = x.saturating_add(width);
            }
        }
    }

    pub fn fill(&mut self, rect: Rect, ch: char, style: Style) {
        let Some(rect) = rect.intersection(&self.rect) else {
            return;
        };
        for y in rect.top()..rect.bottom() {
            for x in rect.left()..rect.right() {
                self.set_cell(x, y, Cell::new(ch, style));
            }
        }
    }

    pub fn clear(&mut self) {
        self.fill(self.rect, ' ', Style::default());
    }
}
