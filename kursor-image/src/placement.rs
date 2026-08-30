use kursor_core::layout::size::Size;

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

    pub const fn cell_size(&self) -> Option<Size> {
        self.cell_size
    }
}

pub fn fit_cells(
    width: u32,
    height: u32,
    available: Size,
    cell_size: Option<Size>,
    fit: ImageFit,
) -> Option<Size> {
    if width == 0 || height == 0 || available.width == 0 || available.height == 0 {
        return None;
    }

    let cell = cell_size.unwrap_or(Size::new(1, 2));
    let pixel_width = width.min(u32::from(u16::MAX));
    let pixel_height = height.min(u32::from(u16::MAX));

    let natural = Size::new(
        pixel_width.div_ceil(u32::from(cell.width)) as u16,
        pixel_height.div_ceil(u32::from(cell.height)) as u16,
    );

    let size = match fit {
        ImageFit::None => Size::new(
            natural.width.min(available.width),
            natural.height.min(available.height),
        ),
        ImageFit::Fill => available,
        ImageFit::Contain => {
            let scale = f64::from(available.width) / f64::from(natural.width.max(1))
                .min(f64::from(available.height) / f64::from(natural.height.max(1)));
            Size::new(
                (f64::from(natural.width) * scale).floor().max(1.0) as u16,
                (f64::from(natural.height) * scale).floor().max(1.0) as u16,
            )
        }
        ImageFit::Cover => {
            let scale = f64::from(available.width) / f64::from(natural.width.max(1))
                .max(f64::from(available.height) / f64::from(natural.height.max(1)));
            Size::new(
                (f64::from(natural.width) * scale).floor().max(1.0) as u16,
                (f64::from(natural.height) * scale).floor().max(1.0) as u16,
            )
        }
    };

    (size.width > 0 && size.height > 0).then_some(size)
}
