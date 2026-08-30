use std::collections::{BTreeMap, BTreeSet};

use crate::{Error, GraphicsProtocol, ImageEncoder, ImageSource, ImageTarget, Result};

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

    fn encode(&self, source: &ImageSource, _target: &ImageTarget) -> Result<Self::Output> {
        let image = source.to_data()?;
        let (width, height) = image.size();
        if width == 0 || height == 0 {
            return Err(Error::InvalidDimensions);
        }
        let width = width as usize;
        let height = height as usize;
        let pixels = image.rgba_bytes();

        let mut pal: BTreeMap<u8, (u8, u8, u8)> = BTreeMap::new();
        let mut idx: Vec<Option<u8>> = Vec::with_capacity(width * height);
        for px in pixels.chunks_exact(4) {
            if px[3] < 128 {
                idx.push(None);
                continue;
            }
            let r = px[0] as usize * 5 / 255;
            let g = px[1] as usize * 5 / 255;
            let b = px[2] as usize * 5 / 255;
            let id = (r * 36 + g * 6 + b) as u8;
            pal.entry(id)
                .or_insert(((r * 100 / 5) as u8, (g * 100 / 5) as u8, (b * 100 / 5) as u8));
            idx.push(Some(id));
        }

        let mut out = Vec::new();
        out.extend_from_slice(b"\x1bPq\"1;1");
        for (id, (r, g, b)) in &pal {
            out.extend_from_slice(format!("#{id};2;{r};{g};{b}").as_bytes());
        }

        let bands = height.div_ceil(6);
        for band in 0..bands {
            let y0 = band * 6;
            let mut set: BTreeSet<u8> = BTreeSet::new();
            for y in y0..(y0 + 6).min(height) {
                for x in 0..width {
                    if let Some(id) = idx[y * width + x] {
                        set.insert(id);
                    }
                }
            }
            if set.is_empty() {
                out.push(b'-');
                continue;
            }
            let mut first = true;
            for id in set {
                if !first {
                    out.push(b'$');
                }
                first = false;
                out.extend_from_slice(format!("#{id}").as_bytes());
                for x in 0..width {
                    let mut bits: u8 = 0;
                    for dy in 0..6 {
                        let y = y0 + dy;
                        if y >= height {
                            continue;
                        }
                        if idx[y * width + x] == Some(id) {
                            bits |= 1 << dy;
                        }
                    }
                    out.push(0x3f + bits);
                }
            }
            if band + 1 < bands {
                out.push(b'-');
            }
        }

        out.extend_from_slice(b"\x1b\\");
        Ok(out)
    }
}
