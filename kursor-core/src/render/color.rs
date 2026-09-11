/// terminal colors.
///
/// see https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    #[default]
    /// inherit.
    Unset,
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    LightGray,
    White,
    Rgb(u8, u8, u8),
    Ansi(u8),
}

#[cfg(feature = "animate")]
impl animate::Interpolate for Color {
    fn lerp(start: &Self, end: &Self, t: f32) -> Self {
        match (rgb(*start), rgb(*end)) {
            (Some((sr, sg, sb)), Some((er, eg, eb))) => Color::Rgb(
                lerp_channel(sr, er, t),
                lerp_channel(sg, eg, t),
                lerp_channel(sb, eb, t),
            ),
            _ if t < 1.0 => *start,
            _ => *end,
        }
    }
}

#[cfg(feature = "animate")]
pub(crate) fn rgb(color: Color) -> Option<(u8, u8, u8)> {
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
        Color::Reset | Color::Unset => None,
    }
}

#[cfg(feature = "animate")]
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

#[cfg(feature = "animate")]
fn lerp_channel(start: u8, end: u8, t: f32) -> u8 {
    (start as f32 + (end as f32 - start as f32) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}

#[cfg(feature = "animate")]
pub(crate) fn to_u8(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

#[cfg(feature = "animate")]
impl animate::Distance for Color {
    fn distance(&self, other: &Self) -> f32 {
        match (rgb(*self), rgb(*other)) {
            (Some((sr, sg, sb)), Some((er, eg, eb))) => {
                let r = sr as f32 - er as f32;
                let g = sg as f32 - eg as f32;
                let b = sb as f32 - eb as f32;
                (r * r + g * g + b * b).sqrt()
            }
            _ => 0.0,
        }
    }
}

#[cfg(feature = "animate")]
impl animate::Integrate for Color {
    type Velocity = [f32; 3];

    fn integrate(
        &self,
        target: &Self,
        velocity: &Self::Velocity,
        params: animate::SpringSpec,
        dt: f32,
    ) -> (Self, Self::Velocity) {
        let (Some((cr, cg, cb)), Some((tr, tg, tb))) =
            (rgb(*self), rgb(*target))
        else {
            return (*target, [0.0; 3]);
        };
        let step = |pos: f32, tgt: f32, vel: f32| -> (f32, f32) {
            let displacement = pos - tgt;
            let accel = (-params.stiffness * displacement
                - params.damping * vel)
                / params.mass;
            let nvel = vel + accel * dt;
            (pos + nvel * dt, nvel)
        };
        let (r, vr) = step(cr as f32, tr as f32, velocity[0]);
        let (g, vg) = step(cg as f32, tg as f32, velocity[1]);
        let (b, vb) = step(cb as f32, tb as f32, velocity[2]);
        (Color::Rgb(to_u8(r), to_u8(g), to_u8(b)), [vr, vg, vb])
    }
}
