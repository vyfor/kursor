mod error;
mod format;
mod halfblocks;
mod iterm;
mod kitty;
mod placement;
mod sixel;
mod source;

pub use error::{Error, Result};
pub use format::{GraphicsProtocol, ImageFormat};
pub use halfblocks::Halfblocks;
pub use iterm::Iterm2;
pub use kitty::{Kitty, Transmission};
pub use placement::{ImageFit, ImageTarget, fit_cells};
pub use sixel::Sixel;
pub use source::{ImageData, ImageSource};

pub trait ImageEncoder {
    type Output;

    fn protocol(&self) -> GraphicsProtocol;

    fn encode(&self, source: &ImageSource, target: &ImageTarget) -> Result<Self::Output>;
}
