use std::time::Duration;

use kursor_core::{render::color::Color, term_info::TermInfo};

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

pub fn query(timeout: Duration) -> TermInfo {
    let mut info = TermInfo::from_env();
    if timeout.is_zero() {
        return info;
    }

    #[cfg(unix)]
    let _ = unix::query(timeout, &mut info);
    #[cfg(windows)]
    let _ = windows::query(timeout, &mut info);

    info
}

pub(super) fn direct_queries(info: &TermInfo, multiplexer: bool) -> String {
    let mut queries = String::new();
    if info.foreground.is_none() {
        queries.push_str("\x1b]10;?\x07");
    }
    if info.background.is_none() {
        queries.push_str("\x1b]11;?\x07");
    }

    let pal: Vec<usize> = (0..16)
        .filter(|index| info.palette[*index].is_none())
        .collect();
    if !pal.is_empty() {
        queries.push_str("\x1b]4;");
        for (position, index) in pal.iter().enumerate() {
            if position != 0 {
                queries.push(';');
            }
            queries.push_str(&format!("{index};?"));
        }
        queries.push('\x07');
    }

    if info.cell_size.is_none() {
        queries.push_str("\x1b[16t");
    }
    if info.window_pixels.is_none() {
        queries.push_str("\x1b[14t");
    }

    if multiplexer {
        if !info.synchronized_output {
            queries.push_str("\x1b[?2026$p");
        }
    } else {
        if !info.kitty_keyboard {
            queries.push_str("\x1b[?u");
        }
        if !info.synchronized_output {
            queries.push_str("\x1b[?2026$p");
        }
        if !info.graphics.kitty || !info.graphics.sixel {
            queries.push_str("\x1b[c");
        }
        if !info.graphics.kitty {
            queries.push_str("\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\");
        }
    }

    if !queries.is_empty() {
        queries.push_str("\x1b[6n");
    }
    queries
}

#[cfg(unix)]
pub(super) fn passthrough_queries(info: &TermInfo) -> String {
    let mut queries = String::new();
    if !info.kitty_keyboard {
        queries.push_str("\x1b[?u");
    }
    queries.push_str("\x1b[16t\x1b[14t");
    queries.push_str("\x1b[c");
    if !info.graphics.kitty {
        queries.push_str("\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\");
    }
    queries.push_str("\x1b[6n");
    match unix_route() {
        UnixRoute::Tmux => tmux_passthrough(&queries),
        UnixRoute::Screen => screen_passthrough(&queries),
        UnixRoute::Direct => queries,
    }
}

#[cfg(unix)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UnixRoute {
    Direct,
    Tmux,
    Screen,
}

#[cfg(unix)]
pub(super) fn unix_route() -> UnixRoute {
    let term = std::env::var("TERM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if std::env::var_os("TMUX").is_some() || term.starts_with("tmux") {
        UnixRoute::Tmux
    } else if std::env::var_os("STY").is_some() || term.starts_with("screen") {
        UnixRoute::Screen
    } else {
        UnixRoute::Direct
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProbePhase {
    Direct,
    Passthrough,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ProbeReply {
    Foreground(Color),
    Background(Color),
    PaletteEntry { index: usize, color: Color },
    CellSize { width: u16, height: u16 },
    WindowPixels { width: u16, height: u16 },
    KeyboardReport,
    GraphicsReport,
    DeviceAttributes { primary: bool, params: Vec<u16> },
    SyncReport { status: u8 },
    CursorPosition { row: u16, column: u16 },
}

pub(super) struct ProbeSession<'a> {
    info: &'a mut TermInfo,
    phase: ProbePhase,
    route: bool,
}

impl<'a> ProbeSession<'a> {
    pub fn new(info: &'a mut TermInfo) -> Self {
        Self {
            info,
            phase: ProbePhase::Direct,
            route: false,
        }
    }

    #[cfg(unix)]
    pub fn passthrough(&mut self) {
        self.phase = ProbePhase::Passthrough;
    }

    pub fn info(&self) -> &TermInfo {
        self.info
    }

    #[cfg(unix)]
    pub fn route(&self) -> bool {
        self.route
    }

    pub fn accept(&mut self, replies: impl IntoIterator<Item = ProbeReply>) -> bool {
        let mut fence = false;
        for reply in replies {
            match reply {
                ProbeReply::Foreground(color) => self.info.foreground = Some(color),
                ProbeReply::Background(color) => self.info.background = Some(color),
                ProbeReply::PaletteEntry { index, color } => {
                    if let Some(slot) = self.info.palette.get_mut(index) {
                        *slot = Some(color);
                    }
                }
                ProbeReply::CellSize { width, height } => {
                    if width != 0 && height != 0 {
                        self.info.cell_size = Some((width, height));
                    }
                }
                ProbeReply::WindowPixels { width, height } => {
                    if width != 0 && height != 0 {
                        self.info.window_pixels = Some((width, height));
                    }
                }
                ProbeReply::KeyboardReport => self.info.kitty_keyboard = true,
                ProbeReply::GraphicsReport => self.info.graphics.kitty = true,
                ProbeReply::DeviceAttributes { primary, params } => {
                    if primary && params.contains(&4) {
                        self.info.graphics.sixel = true;
                    }
                    if self.phase == ProbePhase::Passthrough {
                        self.route = true;
                    }
                }
                ProbeReply::SyncReport { status } => {
                    self.info.synchronized_output = status != 0;
                }
                ProbeReply::CursorPosition { .. } => fence = true,
            }
        }
        fence
    }
}

pub(super) struct ProbeParser {
    buffer: Vec<u8>,
}

impl Default for ProbeParser {
    fn default() -> Self {
        Self {
            buffer: Vec::with_capacity(256),
        }
    }
}

impl ProbeParser {
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<ProbeReply> {
        self.buffer.extend_from_slice(bytes);
        let mut replies = Vec::new();

        loop {
            let Some(start) = self.buffer.iter().position(|byte| {
                matches!(*byte, 0x1b | 0x90 | 0x9b | 0x9d | 0x9f)
            }) else {
                self.buffer.clear();
                break;
            };
            if start != 0 {
                self.buffer.drain(..start);
            }

            let Some((kind, body_start)) = self.sequence_start() else {
                if self.buffer.first() == Some(&0x1b) && self.buffer.len() < 2 {
                    break;
                }
                self.buffer.drain(..1);
                continue;
            };

            match kind {
                Sequence::Csi => {
                    let Some(relative_end) = self.buffer[body_start..]
                        .iter()
                        .position(|byte| (0x40..=0x7e).contains(byte))
                    else {
                        break;
                    };
                    let end = body_start + relative_end;
                    if let Some(reply) = parse_csi(
                        self.buffer[end],
                        &self.buffer[body_start..end],
                    ) {
                        replies.push(reply);
                    }
                    self.buffer.drain(..=end);
                }
                Sequence::Osc | Sequence::Dcs | Sequence::Apc => {
                    let Some((relative_end, terminator_length)) =
                        end(&self.buffer[body_start..])
                    else {
                        break;
                    };
                    let end = body_start + relative_end;
                    if kind == Sequence::Osc {
                        replies.extend(parse_osc(&self.buffer[body_start..end]));
                    } else if kind == Sequence::Apc {
                        if let Some(reply) = parse_apc(&self.buffer[body_start..end]) {
                            replies.push(reply);
                        }
                    }
                    self.buffer.drain(..end + terminator_length);
                }
            }
        }

        replies
    }

    fn sequence_start(&self) -> Option<(Sequence, usize)> {
        match self.buffer.first().copied()? {
            0x90 => Some((Sequence::Dcs, 1)),
            0x9b => Some((Sequence::Csi, 1)),
            0x9d => Some((Sequence::Osc, 1)),
            0x9f => Some((Sequence::Apc, 1)),
            0x1b => match self.buffer.get(1).copied()? {
                b'[' => Some((Sequence::Csi, 2)),
                b']' => Some((Sequence::Osc, 2)),
                b'P' => Some((Sequence::Dcs, 2)),
                b'_' => Some((Sequence::Apc, 2)),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Sequence {
    Csi,
    Osc,
    Dcs,
    Apc,
}

fn end(bytes: &[u8]) -> Option<(usize, usize)> {
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == 0x07 || *byte == 0x9c {
            return Some((index, 1));
        }
        if *byte == 0x1b && bytes.get(index + 1) == Some(&b'\\') {
            return Some((index, 2));
        }
    }
    None
}

fn parse_csi(final_byte: u8, params: &[u8]) -> Option<ProbeReply> {
    match final_byte {
        b'R' => {
            let mut values = params.split(|byte| *byte == b';');
            let row = parse_u16(values.next()?)?;
            let column = parse_u16(values.next()?)?;
            if values.next().is_some() {
                return None;
            }
            Some(ProbeReply::CursorPosition { row, column })
        }
        b'u' if params.first() == Some(&b'?') => Some(ProbeReply::KeyboardReport),
        b'y' => {
            let value = params.strip_prefix(b"?2026;")?.strip_suffix(b"$")?;
            let status = parse_u16(value)?.try_into().ok()?;
            Some(ProbeReply::SyncReport { status })
        }
        b't' => {
            let mut values = params.split(|byte| *byte == b';');
            let kind = parse_u16(values.next()?)?;
            let height = parse_u16(values.next()?)?;
            let width = parse_u16(values.next()?)?;
            if values.next().is_some() {
                return None;
            }
            match kind {
                4 => Some(ProbeReply::WindowPixels { width, height }),
                6 => Some(ProbeReply::CellSize { width, height }),
                _ => None,
            }
        }
        b'c' if params.first() == Some(&b'?') || params.first() == Some(&b'>') => {
            let primary = params.first() == Some(&b'?');
            let params = params[1..]
                .split(|byte| *byte == b';')
                .filter_map(parse_u16)
                .collect();
            Some(ProbeReply::DeviceAttributes { primary, params })
        }
        _ => None,
    }
}

fn parse_osc(payload: &[u8]) -> Vec<ProbeReply> {
    let mut fields = payload.split(|byte| *byte == b';');
    let Some(code) = fields.next().and_then(parse_u16) else {
        return Vec::new();
    };
    let mut replies = Vec::new();
    match code {
        10 => {
            if let Some(color) = fields.next().and_then(parse_color) {
                replies.push(ProbeReply::Foreground(color));
            }
        }
        11 => {
            if let Some(color) = fields.next().and_then(parse_color) {
                replies.push(ProbeReply::Background(color));
            }
        }
        4 => {
            while let (Some(index), Some(value)) = (fields.next(), fields.next()) {
                let Some(index) = parse_u16(index).map(usize::from).filter(|index| *index < 16)
                else {
                    continue;
                };
                if let Some(color) = parse_color(value) {
                    replies.push(ProbeReply::PaletteEntry { index, color });
                }
            }
        }
        _ => {}
    }
    replies
}

fn parse_apc(payload: &[u8]) -> Option<ProbeReply> {
    payload
        .starts_with(b"Gi=31;OK")
        .then_some(ProbeReply::GraphicsReport)
}

fn parse_u16(value: &[u8]) -> Option<u16> {
    if value.is_empty() {
        return None;
    }
    let mut result = 0u16;
    for byte in value {
        if !byte.is_ascii_digit() {
            return None;
        }
        result = result.checked_mul(10)?.checked_add(u16::from(byte - b'0'))?;
    }
    Some(result)
}

fn parse_hex(value: &[u8]) -> Option<u16> {
    if value.is_empty() || value.len() > 4 {
        return None;
    }
    let mut result = 0u16;
    for byte in value {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return None,
        };
        result = result.checked_mul(16)?.checked_add(u16::from(digit))?;
    }
    Some(result)
}

fn parse_color(value: &[u8]) -> Option<Color> {
    let value = value.strip_prefix(b"rgb:")?;
    let mut channels = value.split(|byte| *byte == b'/');
    let r = parse_channel(channels.next()?)?;
    let g = parse_channel(channels.next()?)?;
    let b = parse_channel(channels.next()?)?;
    if channels.next().is_some() {
        return None;
    }
    Some(Color::Rgb(r, g, b))
}

fn parse_channel(value: &[u8]) -> Option<u8> {
    let number = parse_hex(value)?;
    let maximum = (1u32 << (value.len() * 4)).saturating_sub(1).max(1);
    Some((u32::from(number) * 255 / maximum) as u8)
}

#[cfg(unix)]
pub(super) fn tmux_passthrough(sequence: &str) -> String {
    let mut output = String::from("\x1bPtmux;");
    for character in sequence.chars() {
        if character == '\x1b' {
            output.push('\x1b');
        }
        output.push(character);
    }
    output.push_str("\x1b\\");
    output
}

#[cfg(unix)]
pub(super) fn screen_passthrough(sequence: &str) -> String {
    let mut output = String::from("\x1bP");
    output.push_str(sequence);
    output.push_str("\x1b\\");
    output
}
