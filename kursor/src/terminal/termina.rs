use std::{
    io::{self, Write},
    time::Duration,
};

use kursor_core::{
    component::blueprint::Blueprint,
    event::{
        Event, Modifiers,
        key::{KeyCode, KeyEvent},
        mouse::{MouseButton, MouseEvent, MouseKind},
    },
    layout::size::Size,
    render::{
        attrs::{Blink, Underline},
        buffer::{CellDiff, GraphicsDiff},
        color::Color,
        style::Style,
    },
    runtime::Runtime,
};
#[cfg(feature = "image")]
use kursor_image::Kitty;
use termina::{
    Event as TerminaEvent, OneBased, PlatformTerminal,
    Terminal as TerminaTerminal,
    escape::csi::{
        Csi, Cursor, DecPrivateMode, DecPrivateModeCode, Edit, EraseInDisplay,
        Mode, Sgr, SgrAttributes, SgrModifiers,
    },
    event::{
        KeyCode as TerminaKeyCode, KeyEventKind, Modifiers as TerminaModifiers,
        MouseButton as TerminaMouseButton, MouseEventKind,
    },
    style::{ColorSpec, RgbColor},
};

use super::Terminal;
use crate::app::App;

pub struct Termina {
    inner: PlatformTerminal,
    output: Vec<u8>,
    active: bool,
}

impl Termina {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Self {
            inner: PlatformTerminal::new()?,
            output: Vec::with_capacity(4096),
            active: false,
        })
    }

    fn mode(mode: DecPrivateModeCode, enabled: bool) -> Csi {
        let mode = DecPrivateMode::Code(mode);
        Csi::Mode(if enabled {
            Mode::SetDecPrivateMode(mode)
        } else {
            Mode::ResetDecPrivateMode(mode)
        })
    }

    fn cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        write!(
            self.output,
            "{}",
            Csi::Cursor(Cursor::Position {
                line: OneBased::from_zero_based(y),
                col: OneBased::from_zero_based(x),
            })
        )
    }
}

impl App<Termina> {
    pub fn new(runtime: Runtime) -> Result<Self, io::Error> {
        Ok(Self::with_terminal(runtime, Termina::new()?))
    }

    pub fn from(root: Blueprint) -> Result<Self, io::Error> {
        Self::builder(root).build()
    }
}

impl Terminal for Termina {
    type Error = io::Error;

    fn size(&mut self) -> Result<Size, Self::Error> {
        let size = self.inner.get_dimensions()?;
        Ok(Size::new(size.cols, size.rows))
    }

    fn enter(&mut self) -> Result<(), Self::Error> {
        self.inner.enter_raw_mode()?;
        if let Err(error) = (|| {
            write!(
                self.inner,
                "{}{}{}{}{}",
                Self::mode(
                    DecPrivateModeCode::ClearAndEnableAlternateScreen,
                    true
                ),
                Self::mode(DecPrivateModeCode::FocusTracking, true),
                Self::mode(DecPrivateModeCode::AnyEventMouse, true),
                Self::mode(DecPrivateModeCode::SGRMouse, true),
                Self::mode(DecPrivateModeCode::ShowCursor, false),
            )?;
            self.inner.flush()
        })() {
            let _ = self.inner.enter_cooked_mode();
            return Err(error);
        }
        self.active = true;
        Ok(())
    }

    fn leave(&mut self) {
        if !self.active {
            return;
        }
        let _ = write!(
            self.inner,
            "{}{}{}{}{}{}",
            Self::mode(DecPrivateModeCode::SGRMouse, false),
            Self::mode(DecPrivateModeCode::AnyEventMouse, false),
            Self::mode(DecPrivateModeCode::FocusTracking, false),
            Self::mode(DecPrivateModeCode::ShowCursor, true),
            Self::mode(
                DecPrivateModeCode::ClearAndEnableAlternateScreen,
                false
            ),
            Csi::Sgr(Sgr::Reset),
        );
        #[cfg(feature = "image")]
        {
            let _ = self.inner.write_all(&Kitty::delete_all());
            let _ = self.inner.flush();
        }
        let _ = self.inner.enter_cooked_mode();
        self.active = false;
    }

    fn poll(&mut self, timeout: Duration) -> Result<bool, Self::Error> {
        self.inner.poll(|_| true, Some(timeout))
    }

    fn read(&mut self) -> Result<Option<Event>, Self::Error> {
        translate(self.inner.read(|_| true)?)
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        write!(
            self.inner,
            "{}",
            Csi::Edit(Edit::EraseInDisplay(EraseInDisplay::EraseDisplay))
        )?;
        self.inner.flush()
    }

    fn present(
        &mut self,
        changes: &[CellDiff],
        graphics: &GraphicsDiff,
        cursor: Option<(u16, u16)>,
    ) -> Result<(), Self::Error> {
        self.output.clear();
        let mut last_pos: Option<(u16, u16)> = None;
        let mut last_style: Option<Style> = None;

        for clear in &graphics.clears {
            self.output.extend_from_slice(&clear.prelude);
            for y in 0..clear.h {
                self.cursor(clear.x, clear.y.saturating_add(y))?;
                for _ in 0..clear.w {
                    self.output.push(b' ');
                }
            }
            last_pos = None;
            last_style = None;
        }

        for change in changes {
            let need_move = match last_pos {
                Some((lx, ly)) => {
                    ly != change.y || lx.saturating_add(1) != change.x
                }
                None => true,
            };
            if need_move {
                self.cursor(change.x, change.y)?;
            }
            last_pos = Some((change.x, change.y));

            if last_style != Some(change.cell.style) {
                apply_style(&mut self.output, change.cell.style)?;
                last_style = Some(change.cell.style);
            }
            write!(self.output, "{}", change.cell.ch)?;
        }
        if !changes.is_empty() {
            write!(self.output, "{}", Csi::Sgr(Sgr::Reset))?;
        }
        for draw in &graphics.draws {
            self.cursor(draw.x, draw.y)?;
            self.output.extend_from_slice(&draw.data);
        }
        match cursor {
            Some((x, y)) => {
                self.cursor(x, y)?;
                write!(
                    self.output,
                    "{}",
                    Self::mode(DecPrivateModeCode::ShowCursor, true)
                )?;
            }
            None => write!(
                self.output,
                "{}",
                Self::mode(DecPrivateModeCode::ShowCursor, false)
            )?,
        }
        self.inner.write_all(&self.output)?;
        self.inner.flush()
    }
}

impl Drop for Termina {
    fn drop(&mut self) {
        self.leave();
    }
}

fn translate(event: TerminaEvent) -> Result<Option<Event>, io::Error> {
    Ok(match event {
        TerminaEvent::Key(key) if key.kind != KeyEventKind::Release => {
            translate_code(key.code).map(|code| {
                Event::Key(KeyEvent {
                    code,
                    modifiers: translate_modifiers(key.modifiers),
                })
            })
        }
        TerminaEvent::Key(_) => None,
        TerminaEvent::Mouse(mouse) => {
            let kind = match mouse.kind {
                MouseEventKind::Down(button) => {
                    MouseKind::Down(translate_button(button))
                }
                MouseEventKind::Up(button) => {
                    MouseKind::Up(translate_button(button))
                }
                MouseEventKind::Drag(button) => {
                    MouseKind::Drag(translate_button(button))
                }
                MouseEventKind::Moved => MouseKind::Move,
                MouseEventKind::ScrollDown => MouseKind::ScrollDown,
                MouseEventKind::ScrollUp => MouseKind::ScrollUp,
                MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight => {
                    return Ok(None);
                }
            };
            Some(Event::Mouse(MouseEvent {
                kind,
                column: mouse.column,
                row: mouse.row,
                modifiers: translate_modifiers(mouse.modifiers),
            }))
        }
        TerminaEvent::WindowResized(size) => {
            Some(Event::Resize(size.cols, size.rows))
        }
        TerminaEvent::Paste(text) => Some(Event::Paste(text)),
        TerminaEvent::FocusIn => Some(Event::WindowFocus(true)),
        TerminaEvent::FocusOut => Some(Event::WindowFocus(false)),
        TerminaEvent::Csi(_) | TerminaEvent::Osc(_) | TerminaEvent::Dcs(_) => {
            None
        }
    })
}

fn translate_modifiers(modifiers: TerminaModifiers) -> Modifiers {
    Modifiers {
        ctrl: modifiers.contains(TerminaModifiers::CONTROL),
        alt: modifiers.contains(TerminaModifiers::ALT),
        shift: modifiers.contains(TerminaModifiers::SHIFT),
        system: modifiers.contains(TerminaModifiers::SUPER),
        meta: modifiers.contains(TerminaModifiers::META),
        hyper: modifiers.contains(TerminaModifiers::HYPER),
    }
}

fn translate_code(code: TerminaKeyCode) -> Option<KeyCode> {
    Some(match code {
        TerminaKeyCode::Backspace => KeyCode::Backspace,
        TerminaKeyCode::Enter => KeyCode::Enter,
        TerminaKeyCode::Left => KeyCode::Left,
        TerminaKeyCode::Right => KeyCode::Right,
        TerminaKeyCode::Up => KeyCode::Up,
        TerminaKeyCode::Down => KeyCode::Down,
        TerminaKeyCode::Home => KeyCode::Home,
        TerminaKeyCode::End => KeyCode::End,
        TerminaKeyCode::PageUp => KeyCode::PageUp,
        TerminaKeyCode::PageDown => KeyCode::PageDown,
        TerminaKeyCode::Tab => KeyCode::Tab,
        TerminaKeyCode::BackTab => KeyCode::BackTab,
        TerminaKeyCode::Delete => KeyCode::Delete,
        TerminaKeyCode::Insert => KeyCode::Insert,
        TerminaKeyCode::Function(number) => KeyCode::F(number),
        TerminaKeyCode::Char(character) => KeyCode::Char(character),
        TerminaKeyCode::Escape => KeyCode::Esc,
        TerminaKeyCode::CapsLock => KeyCode::CapsLock,
        TerminaKeyCode::ScrollLock => KeyCode::ScrollLock,
        TerminaKeyCode::NumLock => KeyCode::NumLock,
        TerminaKeyCode::PrintScreen => KeyCode::PrintScreen,
        TerminaKeyCode::Pause => KeyCode::Pause,
        TerminaKeyCode::Menu => KeyCode::Menu,
        _ => return None,
    })
}

fn translate_button(button: TerminaMouseButton) -> MouseButton {
    match button {
        TerminaMouseButton::Left => MouseButton::Left,
        TerminaMouseButton::Right => MouseButton::Right,
        TerminaMouseButton::Middle => MouseButton::Middle,
    }
}

fn apply_style(out: &mut impl Write, style: Style) -> io::Result<()> {
    let mut modifiers = SgrModifiers::NONE;
    if style.attrs.bold {
        modifiers |= SgrModifiers::INTENSITY_BOLD;
    }
    if style.attrs.dim {
        modifiers |= SgrModifiers::INTENSITY_DIM;
    }
    if style.attrs.italic {
        modifiers |= SgrModifiers::ITALIC;
    }
    match style.attrs.underline {
        Underline::None => {}
        Underline::Single => modifiers |= SgrModifiers::UNDERLINE_SINGLE,
        Underline::Double => modifiers |= SgrModifiers::UNDERLINE_DOUBLE,
    }
    match style.attrs.blink {
        Blink::None => {}
        Blink::Slow => modifiers |= SgrModifiers::BLINK_SLOW,
        Blink::Rapid => modifiers |= SgrModifiers::BLINK_RAPID,
    }
    if style.attrs.reverse {
        modifiers |= SgrModifiers::REVERSE;
    }
    if style.attrs.hidden {
        modifiers |= SgrModifiers::INVISIBLE;
    }
    if style.attrs.strikethrough {
        modifiers |= SgrModifiers::STRIKE_THROUGH;
    }

    write!(
        out,
        "{}{}",
        Csi::Sgr(Sgr::Reset),
        Csi::Sgr(Sgr::Attributes(SgrAttributes {
            foreground: Some(map_color(style.fg)),
            background: Some(map_color(style.bg)),
            modifiers,
            ..Default::default()
        })),
    )?;
    if style.attrs.overline {
        write!(out, "{}", Csi::Sgr(Sgr::Overline(true)))?;
    }
    Ok(())
}

fn map_color(color: Color) -> ColorSpec {
    match color {
        Color::Reset | Color::Unset => ColorSpec::Reset,
        Color::Black => ColorSpec::BLACK,
        Color::Red => ColorSpec::RED,
        Color::Green => ColorSpec::GREEN,
        Color::Yellow => ColorSpec::YELLOW,
        Color::Blue => ColorSpec::BLUE,
        Color::Magenta => ColorSpec::MAGENTA,
        Color::Cyan => ColorSpec::CYAN,
        Color::Gray => ColorSpec::BRIGHT_BLACK,
        Color::LightRed => ColorSpec::BRIGHT_RED,
        Color::LightGreen => ColorSpec::BRIGHT_GREEN,
        Color::LightYellow => ColorSpec::BRIGHT_YELLOW,
        Color::LightBlue => ColorSpec::BRIGHT_BLUE,
        Color::LightMagenta => ColorSpec::BRIGHT_MAGENTA,
        Color::LightCyan => ColorSpec::BRIGHT_CYAN,
        Color::LightGray => ColorSpec::WHITE,
        Color::White => ColorSpec::BRIGHT_WHITE,
        Color::Rgb(red, green, blue) => {
            ColorSpec::TrueColor(RgbColor::new(red, green, blue).into())
        }
        Color::Ansi(index) => ColorSpec::PaletteIndex(index),
    }
}
