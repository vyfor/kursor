pub mod color;
mod colorize;
mod ink;
mod reveal;
mod shift;
mod tint;

pub use colorize::{Colorize, colorize};
pub use ink::{
    CharColor, Ink, InkClone, Lerp, Solid, Source, axis, char_color, constant,
    directional, edge, gradient, hue, lerp, perimeter, radial, random, solid,
    source,
};
pub use reveal::{Reveal, reveal};
pub use shift::{Shift, ShiftTarget, shift};
pub use tint::{Tint, tint_bg, tint_fg, tint_style};
