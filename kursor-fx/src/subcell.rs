use kursor_core::render::{cell::Cell, color::Color, style::Style};

use crate::fx::Direction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Subcell {
    None,
    Half,
    Eighth,
    // eighth + symbols for legacy computing
    EighthExt,
    Braille,
}

impl Default for Subcell {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Vertical,
    Horizontal,
}

impl Subcell {
    pub fn levels(self, axis: Axis) -> u8 {
        match (self, axis) {
            (Self::None, _) => 1,
            (Self::Half, _) => 2,
            (Self::Eighth, Axis::Vertical) => 8,
            (Self::Eighth, Axis::Horizontal) => 2,
            (Self::EighthExt, _) => 8,
            (Self::Braille, Axis::Vertical) => 4,
            (Self::Braille, Axis::Horizontal) => 2,
        }
    }

    pub fn fill_bottom(self, amount: f32) -> char {
        let chars: &[char] = match self {
            Self::None => &[' '],
            Self::Half => &[' ', '\u{2584}', '\u{2588}'],
            Self::Eighth | Self::EighthExt => {
                &[' ', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}',
                  '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}']
            }
            Self::Braille => &['\u{2800}', '\u{28C0}', '\u{28E4}', '\u{28F6}', '\u{28FF}'],
        };
        pick_char(chars, amount)
    }

    pub fn fill_top(self, amount: f32) -> char {
        let chars: &[char] = match self {
            Self::None => &[' '],
            Self::Half => &[' ', '\u{2580}', '\u{2588}'],
            Self::Eighth => &[' ', '\u{2580}', '\u{2588}'],
            Self::EighthExt => {
                &[' ', '\u{2594}', '\u{1FB82}', '\u{1FB83}',
                  '\u{2580}', '\u{1FB84}', '\u{1FB85}', '\u{1FB86}', '\u{2588}']
            }
            Self::Braille => &['\u{2800}', '\u{2809}', '\u{281B}', '\u{283F}', '\u{28FF}'],
        };
        pick_char(chars, amount)
    }

    pub fn fill_left(self, amount: f32) -> char {
        let chars: &[char] = match self {
            Self::None => &[' '],
            Self::Half => &[' ', '\u{258C}', '\u{2588}'],
            Self::Eighth | Self::EighthExt => {
                &[' ', '\u{258F}', '\u{258E}', '\u{258D}', '\u{258C}',
                  '\u{258B}', '\u{258A}', '\u{2589}', '\u{2588}']
            }
            Self::Braille => &['\u{2800}', '\u{2847}', '\u{28FF}'],
        };
        pick_char(chars, amount)
    }

    pub fn fill_right(self, amount: f32) -> char {
        let chars: &[char] = match self {
            Self::None => &[' '],
            Self::Half => &[' ', '\u{2590}', '\u{2588}'],
            Self::Eighth => &[' ', '\u{2590}', '\u{2588}'],
            Self::EighthExt => {
                &[' ', '\u{2595}', '\u{1FB87}', '\u{1FB88}',
                  '\u{2590}', '\u{1FB89}', '\u{1FB8A}', '\u{1FB8B}', '\u{2588}']
            }
            Self::Braille => &['\u{2800}', '\u{28B8}', '\u{28FF}'],
        };
        pick_char(chars, amount)
    }

    pub fn render_entering(self, source: Cell, amount: f32, direction: Direction) -> Cell {
        let ch = match direction {
            Direction::Up => self.fill_bottom(amount),
            Direction::Down => self.fill_top(amount),
            Direction::Left => self.fill_right(amount),
            Direction::Right => self.fill_left(amount),
        };
        Cell::new(ch, source.style)
    }

    pub fn render_leaving(self, source: Cell, amount: f32, direction: Direction) -> Cell {
        let ch = match direction {
            Direction::Up => self.fill_top(amount),
            Direction::Down => self.fill_bottom(amount),
            Direction::Left => self.fill_left(amount),
            Direction::Right => self.fill_right(amount),
        };
        Cell::new(ch, source.style)
    }

    pub fn edge_cell(
        self,
        src: Cell,
        underlay: Cell,
        direction: Direction,
        amount: f32,
    ) -> Cell {
        let ch = match direction {
            Direction::Left => self.fill_left(amount),
            Direction::Right => self.fill_right(amount),
            Direction::Up => self.fill_top(amount),
            Direction::Down => self.fill_bottom(amount),
        };
        let fg = match src.style.bg {
            Color::Reset => src.style.fg,
            bg => bg,
        };
        Cell::new(ch, Style::new().fg(fg).bg(underlay.style.bg))
    }
}

fn pick_char(chars: &[char], amount: f32) -> char {
    let idx = ((amount.clamp(0.0, 1.0) * (chars.len() - 1) as f32).round() as usize)
        .min(chars.len() - 1);
    chars[idx]
}
