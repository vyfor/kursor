mod error;
mod format;
mod halfblocks;
mod placement;
mod source;

pub use error::{Error, Result};
pub use format::{GraphicsProtocol, ImageFormat};
pub use halfblocks::Halfblocks;
pub use placement::{ImageFit, ImageTarget, fit_cells};
pub use source::{ImageData, ImageSource};

pub trait ImageEncoder {
    type Output;

    fn protocol(&self) -> GraphicsProtocol;

    fn encode(
        &self,
        image: &ImageData,
        target: &ImageTarget,
    ) -> Result<Self::Output>;
}
