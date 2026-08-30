use kursor_core::layout::{size::Size};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFit {
    #[default]
    Contain,
    Cover,
    Fill,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageTarget {
    pub size: Size,
    pub cell_size: Option<Size>,
}

impl ImageTarget {
    pub const fn new(size: Size) -> Self {
        Self {
            size,
            cell_size: None,
        }
    }

    pub const fn with_cell_size(size: Size, cell_size: Size) -> Self {
        Self {
            size,
            cell_size: Some(cell_size),
        }
    }
}