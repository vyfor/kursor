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
    render::{attrs::Attrs, buffer::CellDiff, color::Color, style::Style},
    runtime::Runtime,
};

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
        let mut terminal = Crossterm::new();
        let size = terminal.size()?;
        let mut runtime = Runtime::new(size);
        runtime.mount(root);
        let mut app = Self::with_terminal(runtime, terminal);
        app.init_focus();

        Ok(app)
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
            cursor::Show,
            LeaveAlternateScreen,
            SetAttribute(Attribute::Reset)
        );
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
        cursor: Option<(u16, u16)>,
    ) -> Result<(), Self::Error> {
        for change in changes {
            crossterm::queue!(self.stdout, cursor::MoveTo(change.x, change.y))?;
            apply_style(&mut self.stdout, change.cell.style)?;
            crossterm::queue!(self.stdout, Print(change.cell.ch))?;
        }
        if !changes.is_empty() {
            crossterm::queue!(self.stdout, SetAttribute(Attribute::Reset))?;
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
            modifiers: Modifiers {
                shift: key.modifiers.contains(KeyModifiers::SHIFT),
                ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
                alt: key.modifiers.contains(KeyModifiers::ALT),
            },
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
                modifiers: Modifiers {
                    shift: mouse.modifiers.contains(KeyModifiers::SHIFT),
                    ctrl: mouse.modifiers.contains(KeyModifiers::CONTROL),
                    alt: mouse.modifiers.contains(KeyModifiers::ALT),
                },
            }))
        }
        CtEvent::Resize(width, height) => Some(Event::Resize(width, height)),
        CtEvent::Paste(text) => Some(Event::Paste(text)),
        _ => None,
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
    if attrs.underline {
        res.set(Attribute::Underlined);
    }

    res
}
