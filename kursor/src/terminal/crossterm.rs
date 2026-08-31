use std::{
    io::{self, Stdout, Write},
    time::Duration,
};

use crossterm::{
    cursor,
    event::{
        self, Event as CtEvent, KeyCode as CtKeyCode, KeyEventKind, KeyModifiers,
        MouseButton as CtMouseButton, MouseEventKind,
    },
    execute,
    style::{
        Attribute, Color as CtColor, Print, SetAttribute, SetAttributes, SetBackgroundColor,
        SetForegroundColor,
    },
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
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
        attrs::{Attrs, Blink, Underline},
        buffer::{CellDiff, GraphicsDiff},
        color::Color,
        style::Style,
    },
    runtime::Runtime,
};
use kursor_image::Kitty;

use super::Terminal;
use crate::app::App;

pub struct Crossterm {
    stdout: Stdout,
    active: bool,
}

impl Crossterm {
    pub fn new() -> Self {
        Self {
            stdout: io::stdout(),
            active: false,
        }
    }
}

impl App<Crossterm> {
    pub fn new(runtime: Runtime) -> Self {
        Self::with_terminal(runtime, Crossterm::new())
    }

    pub fn from(root: Blueprint) -> Result<Self, io::Error> {
        Self::builder(root).build()
    }
}

impl Default for Crossterm {
    fn default() -> Self {
        Self::new()
    }
}

// todo: make mouse opt-in?
impl Terminal for Crossterm {
    type Error = io::Error;

    fn size(&mut self) -> Result<Size, Self::Error> {
        let (width, height) = terminal::size()?;
        Ok(Size::new(width, height))
    }

    fn enter(&mut self) -> Result<(), Self::Error> {
        terminal::enable_raw_mode()?;
        if let Err(error) = execute!(
            self.stdout,
            EnterAlternateScreen,
            event::EnableFocusChange,
            event::EnableMouseCapture,
            cursor::Hide
        ) {
            let _ = terminal::disable_raw_mode();
            return Err(error);
        }
        self.active = true;
        Ok(())
    }

    fn leave(&mut self) {
        if !self.active {
            return;
        }
        let _ = execute!(
            self.stdout,
            event::DisableMouseCapture,
            event::DisableFocusChange,
            cursor::Show,
            LeaveAlternateScreen,
            SetAttribute(Attribute::Reset)
        );
        let _ = self.stdout.write_all(&Kitty::delete_all());
        let _ = self.stdout.flush();
        let _ = terminal::disable_raw_mode();
        self.active = false;
    }

    fn poll(&mut self, timeout: Duration) -> Result<bool, Self::Error> {
        event::poll(timeout)
    }

    fn read(&mut self) -> Result<Option<Event>, Self::Error> {
        translate(event::read()?).map_or(Ok(None), |event| Ok(Some(event)))
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        execute!(self.stdout, Clear(ClearType::All))?;
        self.stdout.flush()
    }

    fn present(
        &mut self,
        changes: &[CellDiff],
        graphics: &GraphicsDiff,
        cursor: Option<(u16, u16)>,
    ) -> Result<(), Self::Error> {
        let mut last_pos: Option<(u16, u16)> = None;
        let mut last_style: Option<Style> = None;

        for clear in &graphics.clears {
            self.stdout.write_all(&clear.prelude)?;
            for y in 0..clear.h {
                crossterm::queue!(
                    self.stdout,
                    cursor::MoveTo(clear.x, clear.y.saturating_add(y))
                )?;
                for _ in 0..clear.w {
                    crossterm::queue!(self.stdout, Print(' '))?;
                }
            }
            last_pos = None;
            last_style = None;
        }

        for change in changes {
            let need_move = match last_pos {
                Some((lx, ly)) => ly != change.y || lx.saturating_add(1) != change.x,
                None => true,
            };
            if need_move {
                crossterm::queue!(self.stdout, cursor::MoveTo(change.x, change.y))?;
            }
            last_pos = Some((change.x, change.y));

            if last_style != Some(change.cell.style) {
                apply_style(&mut self.stdout, change.cell.style)?;
                last_style = Some(change.cell.style);
            }
            crossterm::queue!(self.stdout, Print(change.cell.ch))?;
        }
        if !changes.is_empty() {
            crossterm::queue!(self.stdout, SetAttribute(Attribute::Reset))?;
        }
        for draw in &graphics.draws {
            crossterm::queue!(self.stdout, cursor::MoveTo(draw.x, draw.y))?;
            self.stdout.write_all(&draw.data)?;
        }
        match cursor {
            Some((x, y)) => crossterm::queue!(self.stdout, cursor::MoveTo(x, y), cursor::Show)?,
            None => crossterm::queue!(self.stdout, cursor::Hide)?,
        }
        self.stdout.flush()
    }
}

impl Drop for Crossterm {
    fn drop(&mut self) {
        self.leave();
    }
}

pub fn translate(event: CtEvent) -> Option<Event> {
    match event {
        CtEvent::Key(key) if key.kind != KeyEventKind::Release => Some(Event::Key(KeyEvent {
            code: translate_code(key.code)?,
            modifiers: translate_modifiers(key.modifiers),
        })),
        CtEvent::Key(_) => None,
        CtEvent::Mouse(mouse) => {
            let kind = match mouse.kind {
                MouseEventKind::Down(button) => MouseKind::Down(translate_button(button)),
                MouseEventKind::Up(button) => MouseKind::Up(translate_button(button)),
                MouseEventKind::Drag(button) => MouseKind::Drag(translate_button(button)),
                MouseEventKind::Moved => MouseKind::Move,
                MouseEventKind::ScrollDown => MouseKind::ScrollDown,
                MouseEventKind::ScrollUp => MouseKind::ScrollUp,
                _ => return None,
            };
            Some(Event::Mouse(MouseEvent {
                kind,
                column: mouse.column,
                row: mouse.row,
                modifiers: translate_modifiers(mouse.modifiers),
            }))
        }
        CtEvent::Resize(width, height) => Some(Event::Resize(width, height)),
        CtEvent::Paste(text) => Some(Event::Paste(text)),
        CtEvent::FocusGained => Some(Event::WindowFocus(true)),
        CtEvent::FocusLost => Some(Event::WindowFocus(false)),
    }
}

fn translate_modifiers(modifiers: KeyModifiers) -> Modifiers {
    Modifiers {
        ctrl: modifiers.contains(KeyModifiers::CONTROL),
        alt: modifiers.contains(KeyModifiers::ALT),
        shift: modifiers.contains(KeyModifiers::SHIFT),
        system: modifiers.contains(KeyModifiers::SUPER),
        meta: modifiers.contains(KeyModifiers::META),
        hyper: modifiers.contains(KeyModifiers::HYPER),
    }
}

fn translate_code(code: CtKeyCode) -> Option<KeyCode> {
    Some(match code {
        CtKeyCode::Backspace => KeyCode::Backspace,
        CtKeyCode::Enter => KeyCode::Enter,
        CtKeyCode::Left => KeyCode::Left,
        CtKeyCode::Right => KeyCode::Right,
        CtKeyCode::Up => KeyCode::Up,
        CtKeyCode::Down => KeyCode::Down,
        CtKeyCode::Home => KeyCode::Home,
        CtKeyCode::End => KeyCode::End,
        CtKeyCode::PageUp => KeyCode::PageUp,
        CtKeyCode::PageDown => KeyCode::PageDown,
        CtKeyCode::Tab => KeyCode::Tab,
        CtKeyCode::BackTab => KeyCode::BackTab,
        CtKeyCode::Delete => KeyCode::Delete,
        CtKeyCode::Insert => KeyCode::Insert,
        CtKeyCode::F(number) => KeyCode::F(number),
        CtKeyCode::Char(character) => KeyCode::Char(character),
        CtKeyCode::Esc => KeyCode::Esc,
        CtKeyCode::CapsLock => KeyCode::CapsLock,
        CtKeyCode::ScrollLock => KeyCode::ScrollLock,
        CtKeyCode::NumLock => KeyCode::NumLock,
        CtKeyCode::PrintScreen => KeyCode::PrintScreen,
        CtKeyCode::Pause => KeyCode::Pause,
        CtKeyCode::Menu => KeyCode::Menu,
        _ => return None,
    })
}

fn translate_button(button: CtMouseButton) -> MouseButton {
    match button {
        CtMouseButton::Left => MouseButton::Left,
        CtMouseButton::Right => MouseButton::Right,
        CtMouseButton::Middle => MouseButton::Middle,
    }
}

fn apply_style(out: &mut impl Write, style: Style) -> io::Result<()> {
    crossterm::queue!(
        out,
        SetAttribute(Attribute::Reset),
        SetAttributes(map_attrs(style.attrs)),
        SetForegroundColor(map_color(style.fg)),
        SetBackgroundColor(map_color(style.bg))
    )?;
    Ok(())
}

fn map_color(color: Color) -> CtColor {
    match color {
        Color::Reset => CtColor::Reset,
        Color::Black => CtColor::Black,
        Color::Red => CtColor::DarkRed,
        Color::Green => CtColor::DarkGreen,
        Color::Yellow => CtColor::DarkYellow,
        Color::Blue => CtColor::DarkBlue,
        Color::Magenta => CtColor::DarkMagenta,
        Color::Cyan => CtColor::DarkCyan,
        Color::Gray => CtColor::DarkGrey,
        Color::LightRed => CtColor::Red,
        Color::LightGreen => CtColor::Green,
        Color::LightYellow => CtColor::Yellow,
        Color::LightBlue => CtColor::Blue,
        Color::LightMagenta => CtColor::Magenta,
        Color::LightCyan => CtColor::Cyan,
        Color::LightGray => CtColor::Grey,
        Color::White => CtColor::White,
        Color::Rgb(r, g, b) => CtColor::Rgb { r, g, b },
        Color::Ansi(value) => CtColor::AnsiValue(value),
    }
}

fn map_attrs(attrs: Attrs) -> crossterm::style::Attributes {
    let mut res = crossterm::style::Attributes::default();
    if attrs.bold {
        res.set(Attribute::Bold);
    }
    if attrs.dim {
        res.set(Attribute::Dim);
    }
    if attrs.italic {
        res.set(Attribute::Italic);
    }
    match attrs.underline {
        Underline::None => {}
        Underline::Single => res.set(Attribute::Underlined),
        Underline::Double => res.set(Attribute::DoubleUnderlined),
    }
    match attrs.blink {
        Blink::None => {}
        Blink::Slow => res.set(Attribute::SlowBlink),
        Blink::Rapid => res.set(Attribute::RapidBlink),
    }
    if attrs.reverse {
        res.set(Attribute::Reverse);
    }
    if attrs.hidden {
        res.set(Attribute::Hidden);
    }
    if attrs.strikethrough {
        res.set(Attribute::CrossedOut);
    }
    if attrs.overline {
        res.set(Attribute::OverLined);
    }

    res
}
