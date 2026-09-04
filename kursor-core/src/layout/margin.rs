use crate::layout::insets::Insets;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Margin {
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
    pub left: i16,
}

impl Margin {
    pub const fn new(top: i16, right: i16, bottom: i16, left: i16) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub const fn all(value: i16) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(horizontal: i16, vertical: i16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub const fn horizontal(value: i16) -> Self {
        Self {
            top: 0,
            right: value,
            bottom: 0,
            left: value,
        }
    }

    pub const fn vertical(value: i16) -> Self {
        Self {
            top: value,
            right: 0,
            bottom: value,
            left: 0,
        }
    }

    pub const fn top(value: i16) -> Self {
        Self {
            top: value,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }

    pub const fn bottom(value: i16) -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: value,
            left: 0,
        }
    }

    pub const fn left(value: i16) -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: value,
        }
    }

    pub const fn right(value: i16) -> Self {
        Self {
            top: 0,
            right: value,
            bottom: 0,
            left: 0,
        }
    }

    pub const fn horizontal_total(&self) -> i16 {
        self.left.saturating_add(self.right)
    }

    pub const fn vertical_total(&self) -> i16 {
        self.top.saturating_add(self.bottom)
    }
}

impl From<i16> for Margin {
    fn from(val: i16) -> Self {
        Self::all(val)
    }
}

impl From<(i16, i16)> for Margin {
    fn from((h, v): (i16, i16)) -> Self {
        Self::symmetric(h, v)
    }
}

impl From<(i16, i16, i16, i16)> for Margin {
    fn from((top, right, bottom, left): (i16, i16, i16, i16)) -> Self {
        Self::new(top, right, bottom, left)
    }
}

impl From<Insets> for Margin {
    fn from(insets: Insets) -> Self {
        Self::new(
            insets.top as i16,
            insets.right as i16,
            insets.bottom as i16,
            insets.left as i16,
        )
    }
}
