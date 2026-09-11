use std::io::Write as _;

use crate::{
    Error, GraphicsProtocol, ImageData, ImageEncoder, ImageSource, ImageTarget,
    Result,
};

/// https://en.wikipedia.org/wiki/Sixel
pub struct Sixel {
    pub colors: usize,
}

impl Default for Sixel {
    fn default() -> Self {
        Self { colors: 256 }
    }
}

impl Sixel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn colors(mut self, colors: usize) -> Self {
        self.colors = colors.clamp(2, 256);
        self
    }
}

impl ImageEncoder for Sixel {
    type Output = Vec<u8>;

    fn protocol(&self) -> GraphicsProtocol {
        GraphicsProtocol::Sixel
    }

    fn encode(
        &self,
        source: &ImageSource,
        target: &ImageTarget,
    ) -> Result<Self::Output> {
        let image = source.to_data()?;
        let (mut width, mut height) = image.size();
        if width == 0 || height == 0 {
            return Err(Error::InvalidDimensions);
        }
        let image = if let Some(cs) = target.cell_size() {
            let pw = target.size.width as u32 * u32::from(cs.width);
            let ph = target.size.height as u32 * u32::from(cs.height);
            if pw > 0 && ph > 0 && (pw != width || ph != height) {
                let scaled =
                    scale_rgba(image.rgba_bytes(), width, height, pw, ph);
                width = pw;
                height = ph;
                ImageData::rgba(width, height, scaled)?
            } else {
                image
            }
        } else {
            image
        };
        let width = width as usize;
        let height = height as usize;
        let pixels = image.rgba_bytes();

        let mut pal = [false; 216];
        let mut idx: Vec<u8> = Vec::with_capacity(width * height);
        for px in pixels.chunks_exact(4) {
            if px[3] < 128 {
                idx.push(255);
                continue;
            }
            let r = px[0] as usize * 5 / 255;
            let g = px[1] as usize * 5 / 255;
            let b = px[2] as usize * 5 / 255;
            let id = (r * 36 + g * 6 + b) as u8;
            pal[id as usize] = true;
            idx.push(id);
        }

        let mut out = Vec::with_capacity(width * height / 2);
        out.extend_from_slice(b"\x1bPq\"1;1");
        for (id, &present) in pal.iter().enumerate() {
            if present {
                let id = id as u8;
                let r = (id / 36) * 20;
                let g = ((id % 36) / 6) * 20;
                let b = (id % 6) * 20;
                let _ = write!(out, "#{id};2;{r};{g};{b}");
            }
        }

        let bands = height.div_ceil(6);
        let mut band_bits = vec![0u8; 216 * width];
        let mut active = Vec::with_capacity(216);
        let mut active2 = [false; 216];

        for band in 0..bands {
            let y0 = band * 6;
            active.clear();
            active2.fill(false);

            for dy in 0..6 {
                let y = y0 + dy;
                if y >= height {
                    break;
                }
                let row_start = y * width;
                let bit = 1u8 << dy;
                for x in 0..width {
                    let id = idx[row_start + x];
                    if id < 216 {
                        let offset = id as usize * width + x;
                        band_bits[offset] |= bit;
                        if !active2[id as usize] {
                            active2[id as usize] = true;
                            active.push(id);
                        }
                    }
                }
            }

            if active.is_empty() {
                out.push(b'-');
                continue;
            }

            active.sort_unstable();
            let mut first = true;
            for &id in &active {
                let offset = id as usize * width;
                let slice = &mut band_bits[offset..offset + width];

                let mut last = 0;
                for (x, bits) in slice.iter_mut().enumerate() {
                    let b = *bits;
                    if b != 0 {
                        last = x + 1;
                    }
                    *bits = 0x3f + b;
                }

                if last > 0 {
                    if !first {
                        out.push(b'$');
                    }
                    first = false;
                    let _ = write!(out, "#{id}");
                    write_rle(&mut out, &slice[..last]);
                }
                slice.fill(0);
            }

            if band + 1 < bands {
                out.push(b'-');
            }
        }

        out.extend_from_slice(b"\x1b\\");
        Ok(out)
    }
}

fn write_rle(out: &mut Vec<u8>, chars: &[u8]) {
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        let mut count = 1;
        while i + count < chars.len() && chars[i + count] == ch {
            count += 1;
        }
        if count > 3 {
            let _ = write!(out, "!{count}{}", ch as char);
        } else {
            for _ in 0..count {
                out.push(ch);
            }
        }
        i += count;
    }
}

fn scale_rgba(src: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<u8> {
    let mut out = vec![0; (dw * dh * 4) as usize];
    for y in 0..dh {
        let sy = y * sh / dh;
        for x in 0..dw {
            let sx = x * sw / dw;
            let si = ((sy * sw + sx) * 4) as usize;
            let di = ((y * dw + x) * 4) as usize;
            out[di..di + 4].copy_from_slice(&src[si..si + 4]);
        }
    }
    out
}
