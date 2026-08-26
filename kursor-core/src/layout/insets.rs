#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Insets {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl Insets {
    pub const fn new(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub const fn all(value: u16) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(horizontal: u16, vertical: u16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub const fn horizontal(value: u16) -> Self {
        Self {
            top: 0,
            right: value,
            bottom: 0,
            left: value,
        }
    }

    pub const fn vertical(value: u16) -> Self {
        Self {
            top: value,
            right: 0,
            bottom: value,
            left: 0,
        }
    }

    pub const fn top(value: u16) -> Self {
        Self {
            top: value,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }

    pub const fn bottom(value: u16) -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: value,
            left: 0,
        }
    }

    pub const fn left(value: u16) -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: value,
        }
    }

    pub const fn right(value: u16) -> Self {
        Self {
            top: 0,
            right: value,
            bottom: 0,
            left: 0,
        }
    }

    pub const fn width(&self) -> u16 {
        self.left.saturating_add(self.right)
    }

    pub const fn height(&self) -> u16 {
        self.top.saturating_add(self.bottom)
    }
}
