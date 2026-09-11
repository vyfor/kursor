use kursor_core::util::base64;

use crate::{
    Error, GraphicsProtocol, ImageData, ImageEncoder, ImageFormat, ImageSource,
    ImageTarget, Result,
};

/// https://iterm2.com/documentation-images.html
pub struct Iterm2;

impl ImageEncoder for Iterm2 {
    type Output = Vec<u8>;

    fn protocol(&self) -> GraphicsProtocol {
        GraphicsProtocol::Iterm2
    }

    fn encode(
        &self,
        source: &ImageSource,
        target: &ImageTarget,
    ) -> Result<Self::Output> {
        let png = png_bytes(source)?;
        let b64 = base64::encode(&png);
        let mut hdr = format!("1337;File=inline=1;size={}", png.len());
        if target.size.width > 0 {
            hdr.push_str(&format!(";width={}", target.size.width));
        }
        if target.size.height > 0 {
            hdr.push_str(&format!(";height={}", target.size.height));
        }
        hdr.push_str(";preserveAspectRatio=0:");
        let mut out = Vec::with_capacity(hdr.len() + b64.len() + 8);
        out.extend_from_slice(b"\x1b]");
        out.extend_from_slice(hdr.as_bytes());
        out.extend_from_slice(b64.as_bytes());
        out.push(0x07);
        Ok(out)
    }
}

fn png_bytes(source: &ImageSource) -> Result<Vec<u8>> {
    match source {
        ImageSource::Encoded {
            format: ImageFormat::Png,
            bytes,
        } => Ok(bytes.to_vec()),
        ImageSource::File { path, .. } => {
            let bytes = std::fs::read(path).map_err(Error::Io)?;
            if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
                Ok(bytes)
            } else {
                encode_png(&source.to_data()?)
            }
        }
        _ => encode_png(&source.to_data()?),
    }
}

fn encode_png(image: &ImageData) -> Result<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        let mut buf = Vec::new();
        let rgba = image::RgbaImage::from_raw(
            image.width(),
            image.height(),
            image.rgba_bytes().to_vec(),
        )
        .ok_or(Error::InvalidDimensions)?;
        let enc = image::codecs::png::PngEncoder::new(&mut buf);
        image::ImageEncoder::write_image(
            enc,
            rgba.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| Error::Encode(e.to_string()))?;
        Ok(buf)
    }
    #[cfg(not(feature = "image"))]
    {
        let _ = image;
        Err(Error::Encode(
            "png encoding requires the `image` feature".into(),
        ))
    }
}
