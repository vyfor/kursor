use crate::render::color::Color;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Notifications {
    pub osc_9: bool,
    pub osc_99: bool,
    pub osc_777: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Graphics {
    pub kitty: bool,
    pub sixel: bool,
    pub iterm2: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TermInfo {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub palette: [Option<Color>; 16],
    pub color_256: bool,
    pub true_color: bool,
    pub hyperlinks: bool,
    pub clipboard: bool,
    pub kitty_keyboard: bool,
    pub graphics: Graphics,
    pub bracketed_paste: bool,
    pub mouse: bool,
    pub synchronized_output: bool,
    pub notifications: Notifications,
    pub cell_size: Option<(u16, u16)>,
    pub window_pixels: Option<(u16, u16)>,
}

impl Default for TermInfo {
    fn default() -> Self {
        Self {
            foreground: None,
            background: None,
            palette: [None; 16],
            color_256: false,
            true_color: false,
            hyperlinks: false,
            clipboard: false,
            kitty_keyboard: false,
            graphics: Graphics::default(),
            bracketed_paste: false,
            mouse: false,
            synchronized_output: false,
            notifications: Notifications::default(),
            cell_size: None,
            window_pixels: None,
        }
    }
}

impl TermInfo {
    pub fn cell_size(&self, columns: u16, rows: u16) -> Option<(u16, u16)> {
        if let Some(size) = self.cell_size {
            return Some(size);
        }
        let (width, height) = self.window_pixels?;
        if columns == 0 || rows == 0 {
            return None;
        }
        let size = (width / columns, height / rows);
        (size.0 != 0 && size.1 != 0).then_some(size)
    }

    pub fn from_env() -> Self {
        let mut info = Self::default();
        let term = std::env::var("TERM")
            .unwrap_or_default()
            .to_ascii_lowercase();
        let color_term = std::env::var("COLORTERM")
            .unwrap_or_default()
            .to_ascii_lowercase();

        if term == "dumb" {
            info.bracketed_paste = true;
            return info;
        }

        if term.contains("256color")
            || term.contains("kitty")
            || term.contains("foot")
        {
            info.color_256 = true;
        }

        if color_term.contains("truecolor")
            || color_term.contains("24bit")
            || term.contains("truecolor")
            || term.contains("direct")
        {
            info.color_256 = true;
            info.true_color = true;
        }

        let program = std::env::var("TERM_PROGRAM")
            .unwrap_or_default()
            .to_ascii_lowercase();
        let multiplexed = std::env::var_os("TMUX").is_some()
            || std::env::var_os("STY").is_some()
            || term.starts_with("tmux")
            || term.starts_with("screen");

        if !multiplexed {
            match () {
                _ if std::env::var_os("KITTY_WINDOW_ID").is_some()
                    || std::env::var_os("KITTY_PID").is_some()
                    || term == "xterm-kitty"
                    || program == "kitty" =>
                {
                    info.color_256 = true;
                    info.true_color = true;
                    info.hyperlinks = true;
                    info.clipboard = true;
                    info.kitty_keyboard = true;
                    info.graphics.kitty = true;
                    info.bracketed_paste = true;
                    info.mouse = true;
                    info.synchronized_output = true;
                    info.notifications.osc_99 = true;
                }
                _ if program == "ghostty"
                    || std::env::var_os("GHOSTTY_RESOURCES_DIR").is_some() =>
                {
                    info.color_256 = true;
                    info.true_color = true;
                    info.hyperlinks = true;
                    info.clipboard = true;
                    info.kitty_keyboard = true;
                    info.graphics.kitty = true;
                    info.bracketed_paste = true;
                    info.mouse = true;
                    info.synchronized_output = true;
                    info.notifications.osc_777 = true;
                }
                _ if program == "wezterm"
                    || std::env::var_os("WEZTERM_EXECUTABLE").is_some() =>
                {
                    info.color_256 = true;
                    info.true_color = true;
                    info.hyperlinks = true;
                    info.clipboard = true;
                    info.kitty_keyboard = true;
                    info.graphics.kitty = true;
                    info.graphics.sixel = true;
                    info.graphics.iterm2 = true;
                    info.bracketed_paste = true;
                    info.mouse = true;
                    info.synchronized_output = true;
                    info.notifications.osc_777 = true;
                }
                _ if program == "iterm.app" => {
                    info.color_256 = true;
                    info.true_color = true;
                    info.hyperlinks = true;
                    info.clipboard = true;
                    info.graphics.iterm2 = true;
                    info.bracketed_paste = true;
                    info.mouse = true;
                    info.synchronized_output = true;
                    info.notifications.osc_9 = true;
                }
                _ if program == "apple_terminal" => {
                    info.color_256 = true;
                    info.bracketed_paste = true;
                    info.mouse = true;
                }
                _ if program == "foot"
                    || term == "foot"
                    || term.starts_with("foot-") =>
                {
                    info.color_256 = true;
                    info.true_color = true;
                    info.hyperlinks = true;
                    info.clipboard = true;
                    info.kitty_keyboard = true;
                    info.graphics.sixel = true;
                    info.bracketed_paste = true;
                    info.mouse = true;
                    info.synchronized_output = true;
                    info.notifications.osc_777 = true;
                }
                _ if program == "alacritty" => {
                    info.color_256 = true;
                    info.true_color = true;
                    info.hyperlinks = true;
                    info.clipboard = true;
                    info.bracketed_paste = true;
                    info.mouse = true;
                    info.synchronized_output = true;
                }
                _ if program == "vscode" => {
                    info.bracketed_paste = true;
                    info.mouse = true;
                }
                _ if std::env::var_os("WT_SESSION").is_some() => {
                    info.color_256 = true;
                    info.true_color = true;
                    info.graphics.sixel;
                    info.bracketed_paste = true;
                    info.mouse = true;
                }
                _ => {}
            }
        } else {
            info.bracketed_paste = true;
            info.mouse = true;
        }

        if !multiplexed {
            if std::env::var_os("VTE_VERSION").is_some() {
                info.notifications.osc_777 = true;
            } else if std::env::var_os("WT_SESSION").is_some() && !is_conemu() {
                info.notifications.osc_9 = true;
            }
        }

        info
    }
}

fn is_conemu() -> bool {
    std::env::var_os("ConEmuPID").is_some()
        || std::env::var_os("ConEmuANSI").is_some()
        || std::env::var_os("ConEmuBuild").is_some()
}
