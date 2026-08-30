use std::rc::Rc;

use crate::{Error, ImageFormat, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageData {
    width: u32,
    height: u32,
    rgba: Rc<[u8]>,
}

impl ImageData {
    pub fn rgba(width: u32, height: u32, rgba: impl Into<Rc<[u8]>>) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(Error::InvalidDimensions);
        }
        let rgba = rgba.into();
        let expected = width as usize * height as usize * 4;
        if rgba.len() != expected {
            return Err(Error::InvalidPixelData {
                expected,
                actual: rgba.len(),
            });
        }
        Ok(Self {
            width,
            height,
            rgba,
        })
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn rgba_bytes(&self) -> &[u8] {
        &self.rgba
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageSource {
    Data(ImageData),
    Encoded {
        format: ImageFormat,
        bytes: Rc<[u8]>,
    },
}

impl ImageSource {
    pub const fn data(image: ImageData) -> Self {
        Self::Data(image)
    }

    pub fn encoded(format: ImageFormat, bytes: impl Into<Rc<[u8]>>) -> Self {
        Self::Encoded {
            format,
            bytes: bytes.into(),
        }
    }

    pub fn data_ref(&self) -> Option<&ImageData> {
        match self {
            Self::Data(image) => Some(image),
            Self::Encoded { .. } => None,
        }
    }

    #[cfg(feature = "image")]
    pub fn to_data(&self) -> Result<ImageData> {
        match self {
            Self::Data(image) => Ok(image.clone()),
            Self::Encoded { bytes, .. } => {
                let image = image::load_from_memory(bytes)
                    .map_err(|error| crate::Error::Decode(error.to_string()))?
                    .into_rgba8();
                ImageData::rgba(image.width(), image.height(), image.into_raw())
            }
        }
    }

    #[cfg(not(feature = "image"))]
    pub fn to_data(&self) -> Result<ImageData> {
        match self {
            Self::Data(image) => Ok(image.clone()),
            Self::Encoded { format, .. } => Err(Error::UnsupportedFormat(*format)),
        }
    }
}
