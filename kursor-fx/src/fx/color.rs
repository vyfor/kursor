use kursor_core::render::color::Color;

pub fn hsv(h: f32, s: f32, v: f32) -> Color {
    let h = (h - h.floor()).rem_euclid(1.0) * 6.0;
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match h as u8 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    Color::Rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

pub fn scale(c: Color, k: f32) -> Color {
    match c {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f32 * k) as u8,
            (g as f32 * k) as u8,
            (b as f32 * k) as u8,
        ),
        other => other,
    }
}

pub fn mix(from: Color, to: Color, amount: f32, default: Color) -> Color {
    if from == to {
        return from;
    }
    if amount <= 0.0 {
        return from;
    }
    if amount >= 1.0 {
        return to;
    }

    let from = rgb(from).unwrap_or(rgb(default).unwrap());
    let to = rgb(to).unwrap_or(rgb(default).unwrap());
    Color::Rgb(
        channel(from.0, to.0, amount),
        channel(from.1, to.1, amount),
        channel(from.2, to.2, amount),
    )
}

fn channel(from: u8, to: u8, amount: f32) -> u8 {
    (from as f32 + (to as f32 - from as f32) * amount)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn rgb(color: Color) -> Option<(u8, u8, u8)> {
    match color {
        Color::Black => Some(ansi_to_rgb(0)),
        Color::Red => Some(ansi_to_rgb(1)),
        Color::Green => Some(ansi_to_rgb(2)),
        Color::Yellow => Some(ansi_to_rgb(3)),
        Color::Blue => Some(ansi_to_rgb(4)),
        Color::Magenta => Some(ansi_to_rgb(5)),
        Color::Cyan => Some(ansi_to_rgb(6)),
        Color::Gray => Some(ansi_to_rgb(8)),
        Color::LightRed => Some(ansi_to_rgb(9)),
        Color::LightGreen => Some(ansi_to_rgb(10)),
        Color::LightYellow => Some(ansi_to_rgb(11)),
        Color::LightBlue => Some(ansi_to_rgb(12)),
        Color::LightMagenta => Some(ansi_to_rgb(13)),
        Color::LightCyan => Some(ansi_to_rgb(14)),
        Color::LightGray => Some(ansi_to_rgb(7)),
        Color::White => Some(ansi_to_rgb(15)),
        Color::Rgb(r, g, b) => Some((r, g, b)),
        Color::Ansi(value) => Some(ansi_to_rgb(value)),
        Color::Reset => None,
    }
}

fn ansi_to_rgb(value: u8) -> (u8, u8, u8) {
    match value {
        0 => (0, 0, 0),
        1 => (128, 0, 0),
        2 => (0, 128, 0),
        3 => (128, 128, 0),
        4 => (0, 0, 128),
        5 => (128, 0, 128),
        6 => (0, 128, 128),
        7 => (192, 192, 192),
        8 => (128, 128, 128),
        9 => (255, 0, 0),
        10 => (0, 255, 0),
        11 => (255, 255, 0),
        12 => (0, 0, 255),
        13 => (255, 0, 255),
        14 => (0, 255, 255),
        15 => (255, 255, 255),
        16..=231 => {
            let v = value - 16;
            let r = (v / 36) % 6;
            let g = (v / 6) % 6;
            let b = v % 6;

            let to_channel = |c: u8| if c > 0 { c * 40 + 55 } else { 0 };
            (to_channel(r), to_channel(g), to_channel(b))
        }
        232..=255 => {
            let gray = (value - 232) * 10 + 8;
            (gray, gray, gray)
        }
    }
}
