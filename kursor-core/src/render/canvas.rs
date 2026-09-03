use unicode_width::UnicodeWidthChar;

use crate::{
    layout::{offset::Offset, rect::Rect},
    render::{
        buffer::{Buffer, GraphicsOp},
        cell::Cell,
        color::Color,
        style::Style,
    },
    tree::id::NodeId,
};

pub struct Canvas<'a> {
    buffer: &'a mut Buffer,
    clip: Rect,
    origin: Offset,
    node: NodeId,
}

impl<'a> Canvas<'a> {
    pub fn origin(&self) -> Offset {
        self.origin
    }

    pub fn new(buffer: &'a mut Buffer, clip: Rect, origin: Offset, node: NodeId) -> Self {
        Self {
            buffer,
            clip,
            origin,
            node,
        }
    }

    pub fn set(&mut self, x: u16, y: u16, ch: char, style: Style) {
        self.set_cell(x, y, Cell::new(ch, style));
    }

    pub fn set_style(&mut self, x: u16, y: u16, ch: char, style: Style) {
        self.set(x, y, ch, style);
    }

    pub fn set_cell(&mut self, x: u16, y: u16, mut cell: Cell) {
        let x = self.origin.x.saturating_add(i32::from(x));
        let y = self.origin.y.saturating_add(i32::from(y));
        if x >= 0 && y >= 0 && self.clip.contains(x as u16, y as u16) {
            let x = x as u16;
            let y = y as u16;
            if cell.style.bg == Color::Unset || cell.style.fg == Color::Unset {
                if let Some(existing) = self.buffer.cell(x, y) {
                    if cell.style.bg == Color::Unset {
                        cell.style.bg = existing.style.bg;
                    }
                    if cell.style.fg == Color::Unset {
                        cell.style.fg = existing.style.fg;
                    }
                }
            }
            self.buffer.set(x, y, cell);
        }
    }

    pub fn cell(&self, x: u16, y: u16) -> Option<Cell> {
        let x = self.origin.x.saturating_add(i32::from(x));
        let y = self.origin.y.saturating_add(i32::from(y));
        if x < 0 || y < 0 || !self.clip.contains(x as u16, y as u16) {
            return None;
        }
        self.buffer.cell(x as u16, y as u16).copied()
    }

    pub fn set_str(&mut self, mut x: u16, y: u16, text: &str, style: Style) {
        for ch in text.chars() {
            let width = if ch.is_ascii() && ch >= ' ' {
                1
            } else {
                ch.width().unwrap_or(0) as u16
            };
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

    pub fn push_graphics(&mut self, op: GraphicsOp) {
        let x = self.origin.x.max(0) as u16;
        let y = self.origin.y.max(0) as u16;
        let w = self.clip.width;
        let h = self.clip.height;
        self.buffer.push_graphics(self.node, x, y, w, h, op);
    }

    pub fn clear(&mut self) {
        for y in self.clip.top()..self.clip.bottom() {
            for x in self.clip.left()..self.clip.right() {
                self.buffer.set(x, y, Cell::default());
            }
        }
    }
}
