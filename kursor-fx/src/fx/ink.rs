use std::rc::Rc;

use animate::Time;
use kursor_core::{
    layout::size::Size,
    render::{cell::Cell, color::Color},
};

use crate::color::{hsv, scale};
use crate::{Axis, Direction, fx::color};

pub trait Ink: InkClone + 'static {
    fn color(&self, x: u16, y: u16, cell: Cell, size: Size, time: Time) -> Color;
}

pub trait InkClone {
    fn clone_box(&self) -> Box<dyn Ink>;
}

impl<T> InkClone for T
where
    T: Ink + Clone,
{
    fn clone_box(&self) -> Box<dyn Ink> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Ink> {
    fn clone(&self) -> Self {
        self.as_ref().clone_box()
    }
}

impl Ink for Box<dyn Ink> {
    // todo: consider InkCx?
    fn color(&self, x: u16, y: u16, cell: Cell, size: Size, time: Time) -> Color {
        self.as_ref().color(x, y, cell, size, time)
    }
}

pub fn source(field: impl Fn(u16, u16, Size) -> f32 + 'static) -> Source {
    Source {
        field: Rc::new(field),
        motion: Motion::None,
        shaping: None,
        colormap: None,
    }
}

#[derive(Clone)]
pub struct Source {
    field: Rc<dyn Fn(u16, u16, Size) -> f32>,
    motion: Motion,
    shaping: Option<(f32, Shaping)>,
    colormap: Option<ColorMap>,
}

#[derive(Clone)]
enum Motion {
    None,
    Flow { speed: f32 },
    Bounce { speed: f32 },
    Oscillate { freq: f32, speed: f32 },
}

#[derive(Clone, Copy)]
enum Shaping {
    Falloff,
}

#[derive(Clone)]
enum ColorMap {
    Hue { sat: f32, val: f32 },
    Shade { color: Color },
    Gradient { from: Color, to: Color },
    Cycle { colors: Vec<Color> },
}

impl Source {
    pub fn flow(mut self, speed: f32) -> Self {
        self.motion = Motion::Flow { speed };
        self
    }

    pub fn bounce(mut self, speed: f32) -> Self {
        self.motion = Motion::Bounce { speed };
        self
    }

    pub fn oscillate(mut self, freq: f32, speed: f32) -> Self {
        self.motion = Motion::Oscillate { freq, speed };
        self
    }

    pub fn falloff(mut self, width: f32) -> Self {
        self.shaping = Some((width, Shaping::Falloff));
        self
    }

    pub fn hue(mut self, sat: f32, val: f32) -> Self {
        self.colormap = Some(ColorMap::Hue { sat, val });
        self
    }

    pub fn shade(mut self, color: Color) -> Self {
        self.colormap = Some(ColorMap::Shade { color });
        self
    }

    pub fn gradient(mut self, from: Color, to: Color) -> Self {
        self.colormap = Some(ColorMap::Gradient { from, to });
        self
    }

    pub fn cycle(mut self, colors: Vec<Color>) -> Self {
        self.colormap = Some(ColorMap::Cycle { colors });
        self
    }

    fn scalar(&self, x: u16, y: u16, size: Size, time: Time) -> f32 {
        let base = (self.field)(x, y, size);
        let t = time.elapsed.as_secs_f32();
        let v = match &self.motion {
            Motion::None => base,
            Motion::Flow { speed } => base + t * speed,
            Motion::Bounce { speed } => {
                let phase = (t * speed).rem_euclid(2.0);
                let phase = if phase > 1.0 { 2.0 - phase } else { phase };
                base + phase
            }
            Motion::Oscillate { freq, speed } => {
                0.5 + 0.5 * ((base * freq - t * speed) * std::f32::consts::TAU).sin()
            }
        };
        match &self.shaping {
            Some((width, Shaping::Falloff)) => (-(v / width.max(1e-3)) * 7.0).exp(),
            None => v,
        }
    }
}

impl Ink for Source {
    fn color(&self, x: u16, y: u16, _cell: Cell, size: Size, time: Time) -> Color {
        let v = self.scalar(x, y, size, time);
        match &self.colormap {
            Some(ColorMap::Hue { sat, val }) => hsv(v, *sat, *val),
            Some(ColorMap::Shade { color }) => scale(*color, 0.25 + 0.75 * v.clamp(0.0, 1.0)),
            Some(ColorMap::Gradient { from, to }) => {
                color::mix(*from, *to, v.clamp(0.0, 1.0), Color::Black)
            }
            Some(ColorMap::Cycle { colors }) => {
                if colors.is_empty() {
                    return Color::Reset;
                }
                let idx = v.rem_euclid(colors.len() as f32) as usize % colors.len();
                colors[idx]
            }
            None => hsv(v, 0.85, 1.0),
        }
    }
}

pub fn axis(axis: Axis) -> Source {
    match axis {
        Axis::Vertical => source(|_x, y, size| {
            if size.height > 1 {
                y as f32 / (size.height - 1) as f32
            } else {
                0.0
            }
        }),
        Axis::Horizontal => source(|x, _y, size| {
            if size.width > 1 {
                x as f32 / (size.width - 1) as f32
            } else {
                0.0
            }
        }),
    }
}

pub fn directional(dir: Direction) -> Source {
    match dir {
        Direction::Right => axis(Axis::Horizontal),
        Direction::Left => axis(Axis::Horizontal).map(|v| 1.0 - v),
        Direction::Down => axis(Axis::Vertical),
        Direction::Up => axis(Axis::Vertical).map(|v| 1.0 - v),
    }
}

pub fn radial() -> Source {
    source(|x, y, size| {
        let w = size.width.max(1) as f32;
        let h = size.height.max(1) as f32;
        let dx = (x as f32 + 0.5) / w - 0.5;
        let dy = (y as f32 + 0.5) / h - 0.5;
        (dx * dx + dy * dy).sqrt() / 0.5_f32.sqrt()
    })
}

pub fn perimeter() -> Source {
    source(|x, y, size| {
        let (w, h) = (size.width as u32, size.height as u32);
        if w < 2 || h < 2 {
            return 0.0;
        }
        let total = 2 * w + 2 * h - 4;
        let idx = if y == 0 {
            x as u32
        } else if x as u32 == w - 1 {
            w + y as u32 - 1
        } else if y as u32 == h - 1 {
            w + h - 2 + (w - 1 - x as u32)
        } else {
            2 * w + 2 * h - 4 - y as u32
        };
        idx as f32 / total as f32
    })
}

pub fn edge() -> Source {
    source(|x, y, size| {
        let w = size.width.max(2) as f32;
        let h = size.height.max(2) as f32;
        let dx = (x as f32).min(w - 1.0 - x as f32);
        let dy = (y as f32).min(h - 1.0 - y as f32);
        dx.min(dy)
    })
}

pub fn constant(v: f32) -> Source {
    source(move |_x, _y, _size| v)
}

// good enough
pub fn random(seed: u64) -> Source {
    source(move |x, y, _size| {
        let mut h = (x as u64)
            .wrapping_add((y as u64).rotate_left(24))
            .wrapping_add(seed.rotate_left(48));
        h = h.wrapping_mul(1103515245).wrapping_add(12345);
        h ^= h >> 16;
        h = h.wrapping_mul(1103515245).wrapping_add(12345);
        h ^= h >> 16;
        (h >> 11) as f32 / (1u64 << 53) as f32
    })
}

pub fn solid(color: Color) -> Solid {
    Solid { color }
}

#[derive(Clone, Copy)]
pub struct Solid {
    color: Color,
}

impl Ink for Solid {
    fn color(&self, _x: u16, _y: u16, _cell: Cell, _size: Size, _time: Time) -> Color {
        self.color
    }
}

pub fn char_color(f: impl Fn(char) -> Color + 'static) -> CharColor {
    CharColor { f: Rc::new(f) }
}

#[derive(Clone)]
pub struct CharColor {
    f: Rc<dyn Fn(char) -> Color>,
}

impl Ink for CharColor {
    fn color(&self, _x: u16, _y: u16, cell: Cell, _size: Size, _time: Time) -> Color {
        (self.f)(cell.ch)
    }
}

pub fn gradient(from: Color, to: Color, dir: Direction) -> Source {
    directional(dir).gradient(from, to)
}

pub fn hue(speed: f32) -> Source {
    axis(Axis::Horizontal).flow(speed).hue(0.85, 1.0)
}

impl Source {
    pub fn map(mut self, f: impl Fn(f32) -> f32 + 'static) -> Self {
        let inner = self.field.clone();
        self.field = Rc::new(move |x, y, size| f(inner(x, y, size)));
        self
    }
}

pub fn lerp(a: impl Ink + Clone, b: impl Ink + Clone, amount: f32) -> Lerp {
    Lerp {
        a: Box::new(a),
        b: Box::new(b),
        amount,
    }
}

#[derive(Clone)]
pub struct Lerp {
    a: Box<dyn Ink>,
    b: Box<dyn Ink>,
    amount: f32,
}

impl Ink for Lerp {
    fn color(&self, x: u16, y: u16, cell: Cell, size: Size, time: Time) -> Color {
        let ca = self.a.color(x, y, cell, size, time);
        let cb = self.b.color(x, y, cell, size, time);
        color::mix(ca, cb, self.amount, Color::Black)
    }
}

// todo: possibly modularize further
