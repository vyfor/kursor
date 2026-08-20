use unicode_width::UnicodeWidthChar;

use crate::{
    layout::{offset::Offset, rect::Rect},
    render::{buffer::Buffer, cell::Cell, style::Style},
};

pub struct Canvas<'a> {
    buffer: &'a mut Buffer,
    clip: Rect,
    origin: Offset,
}

impl<'a> Canvas<'a> {
    pub fn new(buffer: &'a mut Buffer, clip: Rect, origin: Offset) -> Self {
        Self {
            buffer,
            clip,
            origin,
        }
    }

    pub fn set(&mut self, x: u16, y: u16, ch: char, style: Style) {
        self.set_cell(x, y, Cell::new(ch, style));
    }

    pub fn set_style(&mut self, x: u16, y: u16, ch: char, style: Style) {
        self.set(x, y, ch, style);
    }

    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) {
        let x = self.origin.x.saturating_add(i32::from(x));
        let y = self.origin.y.saturating_add(i32::from(y));
        if x >= 0 && y >= 0 && self.clip.contains(x as u16, y as u16) {
            self.buffer.set(x as u16, y as u16, cell);
        }
    }

    pub fn set_str(&mut self, mut x: u16, y: u16, text: &str, style: Style) {
        for ch in text.chars() {
            let width = ch.width().unwrap_or(0) as u16;
            if width > 0 {
                self.set(x, y, ch, style);
                if width == 2 {
                    self.set(x + 1, y, ' ', style);
                }
                x = x.saturating_add(width);
            }
        }
    }

    pub fn fill(&mut self, rect: Rect, ch: char, style: Style) {
        for y in rect.top()..rect.bottom() {
            for x in rect.left()..rect.right() {
                self.set_cell(x, y, Cell::new(ch, style));
            }
        }
    }

    pub fn clear(&mut self) {
        for y in self.clip.top()..self.clip.bottom() {
            for x in self.clip.left()..self.clip.right() {
                self.buffer.set(x, y, Cell::default());
            }
        }
    }
}
