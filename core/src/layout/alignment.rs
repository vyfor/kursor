#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VAlign {
    #[default]
    Top,
    Center,
    Bottom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Alignment {
    pub horizontal: HAlign,
    pub vertical: VAlign,
}

impl Alignment {
    pub const TOP_LEFT: Self = Self::new(HAlign::Left, VAlign::Top);
    pub const TOP_CENTER: Self = Self::new(HAlign::Center, VAlign::Top);
    pub const TOP_RIGHT: Self = Self::new(HAlign::Right, VAlign::Top);
    pub const CENTER_LEFT: Self = Self::new(HAlign::Left, VAlign::Center);
    pub const CENTER: Self = Self::new(HAlign::Center, VAlign::Center);
    pub const CENTER_RIGHT: Self = Self::new(HAlign::Right, VAlign::Center);
    pub const BOTTOM_LEFT: Self = Self::new(HAlign::Left, VAlign::Bottom);
    pub const BOTTOM_CENTER: Self = Self::new(HAlign::Center, VAlign::Bottom);
    pub const BOTTOM_RIGHT: Self = Self::new(HAlign::Right, VAlign::Bottom);

    pub const fn new(horizontal: HAlign, vertical: VAlign) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
}
