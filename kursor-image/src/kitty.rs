use std::io::Write as _;

use kursor_core::{layout::size::Size, util::base64};

use crate::{
    GraphicsProtocol, ImageEncoder, ImageFormat, ImageSource, ImageTarget, Result,
};

pub struct Kitty;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Transmission {
    #[default]
    Direct,
}

const MAX_CHUNK: usize = 4096;

impl Kitty {
    pub fn transmit(&self, source: &ImageSource, id: u32) -> Result<Vec<u8>> {
        let (control, payload) = transmit_parts(source, Some(id), false)?;
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
}

impl ImageEncoder for Kitty {
    type Output = Vec<u8>;

    fn protocol(&self) -> GraphicsProtocol {
        GraphicsProtocol::Kitty
    }

    fn encode(&self, source: &ImageSource, target: &ImageTarget) -> Result<Self::Output> {
        let (control, payload) = transmit_parts(source, None, true)?;
        let control = format!(
            "{control},c={},r={},C=1",
            target.size.width, target.size.height
        );
        Ok(write_chunks(&control, &payload))
    }
}

fn transmit_parts(
    source: &ImageSource,
    id: Option<u32>,
    display: bool,
) -> Result<(String, String)> {
    let action = if display { 'T' } else { 't' };
    let id_part = id.map(|id| format!(",i={id}")).unwrap_or_default();
    match source {
        ImageSource::Data(image) => {
            let control = format!(
                "a={action},f=32,s={},v={}{id_part},q=2",
                image.width(),
                image.height()
            );
            Ok((control, base64::encode(image.rgba_bytes())))
        }
        ImageSource::Encoded {
            format: ImageFormat::Png,
            bytes,
        } => {
            let control = format!("a={action},f=100{id_part},q=2");
            Ok((control, base64::encode(bytes)))
        }
        ImageSource::Encoded { .. } => {
            let image = source.to_data()?;
            let control = format!(
                "a={action},f=32,s={},v={}{id_part},q=2",
                image.width(),
                image.height()
            );
            Ok((control, base64::encode(image.rgba_bytes())))
        }
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
