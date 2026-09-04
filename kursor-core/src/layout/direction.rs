use super::Orientation;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    #[default]
    Right,
    Left,
    Down,
    Up,
}

crate::into_value!(Direction);

impl From<Orientation> for Direction {
    fn from(orientation: Orientation) -> Self {
        match orientation {
            Orientation::Horizontal => Self::Right,
            Orientation::Vertical => Self::Up,
        }
    }
}
