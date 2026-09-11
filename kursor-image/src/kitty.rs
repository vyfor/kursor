use std::io::Write as _;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

use kursor_core::{layout::size::Size, util::base64};

use crate::{
    Error, GraphicsProtocol, ImageEncoder, ImageFormat, ImageSource,
    ImageTarget, Result,
};

/// https://sw.kovidgoyal.net/kitty/graphics-protocol
pub struct Kitty {
    transmission: Transmission,
}

/// how image data is sent to the terminal.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub enum Transmission {
    #[default]
    Direct,
    TempFile,
}

const MAX_CHUNK: usize = 4096;

impl Kitty {
    pub const fn new() -> Self {
        Self {
            transmission: Transmission::Direct,
        }
    }

    pub const fn temp_file() -> Self {
        Self {
            transmission: Transmission::TempFile,
        }
    }

    pub const fn transmission(mut self, transmission: Transmission) -> Self {
        self.transmission = transmission;
        self
    }

    pub fn transmit(&self, source: &ImageSource, id: u32) -> Result<Vec<u8>> {
        let (control, payload) =
            self.transmit_parts(source, Some(id), false)?;
        Ok(write_chunks(&control, &payload))
    }

    pub fn display(&self, id: u32, size: Size) -> Vec<u8> {
        let mut out = Vec::new();
        let _ = write!(
            out,
            "\x1b_Ga=p,i={id},c={},r={},C=1;\x1b\\",
            size.width, size.height
        );
        out
    }

    pub fn delete(&self, id: u32) -> Vec<u8> {
        let mut out = Vec::new();
        let _ = write!(out, "\x1b_Ga=d,d=i,i={id};\x1b\\");
        out
    }

    pub fn undisplay(&self, id: u32) -> Vec<u8> {
        let mut out = Vec::new();
        let _ = write!(out, "\x1b_Ga=d,d=c,i={id};\x1b\\");
        out
    }

    pub fn delete_all() -> Vec<u8> {
        b"\x1b_Ga=d,d=a;\x1b\\".to_vec()
    }
}

impl Default for Kitty {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageEncoder for Kitty {
    type Output = Vec<u8>;

    fn protocol(&self) -> GraphicsProtocol {
        GraphicsProtocol::Kitty
    }

    fn encode(
        &self,
        source: &ImageSource,
        target: &ImageTarget,
    ) -> Result<Self::Output> {
        let (control, payload) = self.transmit_parts(source, None, true)?;
        let control = format!(
            "{control},c={},r={},C=1",
            target.size.width, target.size.height
        );
        Ok(write_chunks(&control, &payload))
    }
}

impl Kitty {
    fn transmit_parts(
        &self,
        source: &ImageSource,
        id: Option<u32>,
        display: bool,
    ) -> Result<(String, String)> {
        let action = if display { 'T' } else { 't' };
        let id_part = id.map(|id| format!(",i={id}")).unwrap_or_default();
        match source {
            ImageSource::File { path, format } => match format {
                ImageFormat::Png => {
                    let absolute =
                        std::fs::canonicalize(path).map_err(Error::Io)?;
                    let control = format!("a={action},t=f,f=100{id_part},q=2");
                    Ok((
                        control,
                        base64::encode(absolute.to_string_lossy().as_bytes()),
                    ))
                }
                _ => {
                    let image = source.to_data()?;
                    self.transmit_data(&image, action, &id_part)
                }
            },
            ImageSource::Data(image) => match self.transmission {
                Transmission::Direct => {
                    self.transmit_data(image, action, &id_part)
                }
                Transmission::TempFile => {
                    let path = write_temp_file(&image.rgba_bytes().to_vec())?;
                    let control = format!(
                        "a={action},t=t,f=32,s={},v={}{id_part},q=2",
                        image.width(),
                        image.height()
                    );
                    Ok((control, base64::encode(path.as_bytes())))
                }
            },
            ImageSource::Encoded {
                format: ImageFormat::Png,
                bytes,
            } => match self.transmission {
                Transmission::Direct => {
                    let control = format!("a={action},f=100{id_part},q=2");
                    Ok((control, base64::encode(bytes)))
                }
                Transmission::TempFile => {
                    let path = write_temp_file(bytes)?;
                    let control = format!("a={action},t=t,f=100{id_part},q=2");
                    Ok((control, base64::encode(path.as_bytes())))
                }
            },
            ImageSource::Encoded { .. } => {
                let image = source.to_data()?;
                match self.transmission {
                    Transmission::Direct => {
                        self.transmit_data(&image, action, &id_part)
                    }
                    Transmission::TempFile => {
                        let path =
                            write_temp_file(&image.rgba_bytes().to_vec())?;
                        let control = format!(
                            "a={action},t=t,f=32,s={},v={}{id_part},q=2",
                            image.width(),
                            image.height()
                        );
                        Ok((control, base64::encode(path.as_bytes())))
                    }
                }
            }
        }
    }

    fn transmit_data(
        &self,
        image: &crate::ImageData,
        action: char,
        id_part: &str,
    ) -> Result<(String, String)> {
        let control = format!(
            "a={action},f=32,s={},v={}{id_part},q=2",
            image.width(),
            image.height()
        );
        Ok((control, base64::encode(image.rgba_bytes())))
    }
}

static TEMP_COUNTER: AtomicU32 = AtomicU32::new(0);

fn write_temp_file(bytes: &[u8]) -> Result<String> {
    let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir()
        .join(format!("kursor-img-{}-{unique}.tmp", std::process::id()));
    std::fs::write(&path, bytes).map_err(Error::Io)?;
    Ok(path.to_string_lossy().into_owned())
}

pub(crate) fn file_format(path: &Path) -> ImageFormat {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => ImageFormat::Png,
        Some("jpg" | "jpeg") => ImageFormat::Jpeg,
        Some("gif") => ImageFormat::Gif,
        Some("webp") => ImageFormat::Webp,
        Some("bmp") => ImageFormat::Bmp,
        Some("tif" | "tiff") => ImageFormat::Tiff,
        _ => ImageFormat::Other,
    }
}

fn write_chunks(control: &str, payload: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 128);
    if payload.len() <= MAX_CHUNK {
        let _ = write!(out, "\x1b_G{control};{payload}\x1b\\");
        return out;
    }

    let chunks: Vec<&[u8]> = payload.as_bytes().chunks(MAX_CHUNK).collect();
    let last = chunks.len() - 1;
    for (index, chunk) in chunks.iter().enumerate() {
        if index == 0 {
            let _ = write!(out, "\x1b_G{control},m=1;");
        } else if index == last {
            let _ = write!(out, "\x1b_Gm=0;");
        } else {
            let _ = write!(out, "\x1b_Gm=1;");
        }
        out.extend_from_slice(chunk);
        out.extend_from_slice(b"\x1b\\");
    }
    out
}
