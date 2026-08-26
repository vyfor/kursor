#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollDirection {
    #[default]
    Vertical,
    Horizontal,
    Both,
}
