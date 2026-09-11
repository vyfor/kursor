use std::{fmt, io};

use crate::format::ImageFormat;

#[derive(Debug)]
pub enum Error {
    InvalidDimensions,
    InvalidPixelData { expected: usize, actual: usize },
    UnsupportedFormat(ImageFormat),
    Io(io::Error),
    Decode(String),
    Encode(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => {
                f.write_str("image dimensions must be >0")
            }
            Self::InvalidPixelData { expected, actual } => {
                write!(f, "invalid rgba data length: {expected} vs {actual}")
            }
            Self::UnsupportedFormat(format) => {
                write!(f, "unsupported image format: {format:?}")
            }
            Self::Io(error) => error.fmt(f),
            Self::Decode(error) => f.write_str(error),
            Self::Encode(error) => f.write_str(error),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
