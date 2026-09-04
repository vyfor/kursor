#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Offset {
    pub x: i32,
    pub y: i32,
}

impl Offset {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl From<(i32, i32)> for Offset {
    fn from((x, y): (i32, i32)) -> Self {
        Self::new(x, y)
    }
}
