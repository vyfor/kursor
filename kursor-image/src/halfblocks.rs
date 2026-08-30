use kursor_core::render::{cell::Cell, color::Color, style::Style};

use crate::{
    GraphicsProtocol, ImageData, ImageEncoder, ImageTarget, Result,
};

pub struct Halfblocks;

impl ImageEncoder for Halfblocks {
    type Output = Vec<Cell>;

    fn protocol(&self) -> GraphicsProtocol {
        GraphicsProtocol::Halfblocks
    }

    fn encode(&self, image: &ImageData, target: &ImageTarget) -> Result<Self::Output> {
        let width = u32::from(target.size.width);
        let height = u32::from(target.size.height);
        if width == 0 || height == 0 {
            return Ok(Vec::new());
        }

        let (source_width, source_height) = image.size();
        let pixels = image.rgba_bytes();
        let mut cells = Vec::with_capacity((width * height) as usize);

        for row in 0..height {
            for column in 0..width {
                let top = sample(
                    pixels,
                    source_width,
                    source_height,
                    column,
                    row * 2,
                    width,
                    height * 2,
                );
                let bottom = sample(
                    pixels,
                    source_width,
                    source_height,
                    column,
                    row * 2 + 1,
                    width,
                    height * 2,
                );
                cells.push(half_cell(top, bottom));
            }
        }

        Ok(cells)
    }
}

fn sample(
    pixels: &[u8],
    source_width: u32,
    source_height: u32,
    x: u32,
    y: u32,
    target_width: u32,
    target_height: u32,
) -> [u8; 4] {
    let sx = ((u64::from(x) * u64::from(source_width)) / u64::from(target_width))
        .min(u64::from(source_width - 1)) as u32;
    let sy = ((u64::from(y) * u64::from(source_height)) / u64::from(target_height))
        .min(u64::from(source_height - 1)) as u32;
    let index = ((sy * source_width + sx) * 4) as usize;
    [
        pixels[index],
        pixels[index + 1],
        pixels[index + 2],
        pixels[index + 3],
    ]
}

fn half_cell(top: [u8; 4], bottom: [u8; 4]) -> Cell {
    let top_opaque = top[3] >= 128;
    let bottom_opaque = bottom[3] >= 128;
    let top_color = Color::Rgb(top[0], top[1], top[2]);
    let bottom_color = Color::Rgb(bottom[0], bottom[1], bottom[2]);

    match (top_opaque, bottom_opaque) {
        (true, true) if top[..3] == bottom[..3] => Cell::new(' ', Style::new().bg(top_color)),
        (true, true) => Cell::new('▀', Style::new().fg(top_color).bg(bottom_color)),
        (true, false) => Cell::new('▀', Style::new().fg(top_color)),
        (false, true) => Cell::new('▄', Style::new().fg(bottom_color)),
        (false, false) => Cell::new(' ', Style::new()),
    }
}
