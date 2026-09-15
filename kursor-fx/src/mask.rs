use kursor_core::{
    layout::{Direction, rect::Rect},
    render::{border::char_to_junction, cell::Cell},
};

/// selects which cells an effect applies to.
#[derive(Clone, Default)]
pub enum Mask {
    #[default]
    All,
    Text,
    NonEmpty,
    Inner(u16),
    Border,
    Chars(Vec<char>),
    Not(Box<Mask>),
}

impl Mask {
    pub fn all() -> Self {
        Self::All
    }

    pub fn text() -> Self {
        Self::Text
    }

    pub fn non_empty() -> Self {
        Self::NonEmpty
    }

    pub fn inner(margin: u16) -> Self {
        Self::Inner(margin)
    }

    pub fn border() -> Self {
        Self::Border
    }

    pub fn chars(chars: &str) -> Self {
        Self::Chars(chars.chars().collect())
    }

    pub fn not(mask: impl Into<Mask>) -> Self {
        Self::Not(Box::new(mask.into()))
    }

    pub fn includes(&self, cell: Cell, x: u16, y: u16, area: Rect) -> bool {
        match self {
            Self::All => true,
            Self::NonEmpty => cell.ch != ' ',
            Self::Text => cell.ch != ' ' && char_to_junction(cell.ch).is_none(),
            Self::Inner(margin) => {
                x >= *margin
                    && y >= *margin
                    && x.saturating_add(*margin) < area.width
                    && y.saturating_add(*margin) < area.height
            }
            Self::Border => {
                char_to_junction(cell.ch).is_some()
                    || x == 0
                    || y == 0
                    || x == area.width.saturating_sub(1)
                    || y == area.height.saturating_sub(1)
            }
            Self::Chars(chars) => chars.contains(&cell.ch),
            Self::Not(mask) => !mask.includes(cell, x, y, area),
        }
    }
}

/// how an effect progresses across the area over time.
#[derive(Clone, Copy)]
pub enum Spread {
    Uniform,
    Towards(Direction),
    Radial,
}

impl Spread {
    pub fn uniform() -> Self {
        Self::Uniform
    }

    pub fn towards(direction: Direction) -> Self {
        Self::Towards(direction)
    }

    pub fn radial() -> Self {
        Self::Radial
    }

    pub fn progress(self, progress: f32, x: u16, y: u16, area: Rect) -> f32 {
        if matches!(self, Self::Uniform) {
            return progress.clamp(0.0, 1.0);
        }

        let nx = if area.width > 1 {
            x as f32 / (area.width - 1) as f32
        } else {
            0.0
        };
        let ny = if area.height > 1 {
            y as f32 / (area.height - 1) as f32
        } else {
            0.0
        };
        let local = match self {
            Self::Uniform => 0.0,
            Self::Towards(Direction::Right) => nx,
            Self::Towards(Direction::Left) => 1.0 - nx,
            Self::Towards(Direction::Down) => ny,
            Self::Towards(Direction::Up) => 1.0 - ny,
            Self::Radial => {
                let rnx = x as f32 / area.width.max(1) as f32;
                let rny = y as f32 / area.height.max(1) as f32;
                let dx = rnx - 0.5;
                let dy = rny - 0.5;
                (dx * dx + dy * dy).sqrt().min(1.0)
            }
        };

        ((progress - local * 0.35) / 0.65).clamp(0.0, 1.0)
    }
}
