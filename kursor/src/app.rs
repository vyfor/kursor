use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use kursor_core::{
    term_info::TermInfo,
    component::blueprint::Blueprint,
    event::{Event, EventResult, mouse::MouseKind},
    render::buffer::{Buffer, CellDiff},
    runtime::Runtime,
};

use crate::{focus, terminal::Terminal};

#[cfg(feature = "termina")]
use crate::terminal::termina::Termina;
#[cfg(all(not(feature = "termina"), feature = "crossterm"))]
use crate::terminal::crossterm::Crossterm;

#[cfg(feature = "termina")]
type DefaultTerminal = Termina;
#[cfg(all(not(feature = "termina"), feature = "crossterm"))]
type DefaultTerminal = Crossterm;

static QUIT: AtomicBool = AtomicBool::new(false);

pub fn quit() {
    QUIT.store(true, Ordering::Relaxed);
}

#[cfg(any(feature = "termina", feature = "crossterm"))]
pub struct AppBuilder {
    root: Blueprint,
    query_timeout: Option<Duration>,
}

#[cfg(any(feature = "termina", feature = "crossterm"))]
impl AppBuilder {
    pub fn query_terminal(mut self, timeout: Duration) -> Self {
        self.query_timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Result<App<DefaultTerminal>, std::io::Error> {
        let capabilities = match self.query_timeout {
            Some(timeout) => crate::terminal::probe::query(timeout),
            None => TermInfo::from_env(),
        };
        let mut terminal = default_terminal()?;
        let size = terminal.size()?;
        let mut runtime = Runtime::new(size);
        runtime.provide(capabilities);
        runtime.mount(self.root);
        let mut app = App::with_terminal(runtime, terminal);
        app.init_focus();
        Ok(app)
    }
}

#[cfg(feature = "termina")]
fn default_terminal() -> Result<DefaultTerminal, std::io::Error> {
    Termina::new()
}

#[cfg(all(not(feature = "termina"), feature = "crossterm"))]
fn default_terminal() -> Result<DefaultTerminal, std::io::Error> {
    Ok(Crossterm::new())
}

pub struct App<T: Terminal> {
    runtime: Runtime,
    terminal: T,
    active: bool,
}

impl<T: Terminal> App<T> {
    pub fn with_terminal(runtime: Runtime, terminal: T) -> Self {
        let mut app = Self {
            runtime,
            terminal,
            active: false,
        };
        app.init_focus();
        app
    }

    pub fn mount(&mut self, blueprint: Blueprint) {
        self.runtime.mount(blueprint);
        self.init_focus();
    }

    pub(crate) fn init_focus(&mut self) {
        if self.runtime.focus().is_none() {
            focus::next(&mut self.runtime);
        }
    }

    pub fn handle_event(&mut self, event: Event) -> EventResult {
        self.runtime.handle_event(event)
    }

    pub fn render(&mut self) -> Vec<CellDiff> {
        let changes = self.runtime.render();
        self.runtime.commit(&changes);
        changes
    }

    pub fn focus_next(&mut self) -> bool {
        focus::next(&mut self.runtime)
    }

    pub fn focus_previous(&mut self) -> bool {
        focus::previous(&mut self.runtime)
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut Runtime {
        &mut self.runtime
    }

    pub fn terminal(&self) -> &T {
        &self.terminal
    }

    pub fn terminal_mut(&mut self) -> &mut T {
        &mut self.terminal
    }

    pub fn front_buffer(&self) -> &Buffer {
        self.runtime.front_buffer()
    }

    pub fn run(&mut self) -> Result<(), T::Error> {
        QUIT.store(false, Ordering::Relaxed);
        self.terminal.enter()?;
        self.active = true;

        let res = self.event_loop();

        self.terminal.leave();
        self.active = false;

        res
    }

    fn event_loop(&mut self) -> Result<(), T::Error> {
        loop {
            let changes = self.runtime.render();
            self.terminal.present(&changes, self.runtime.cursor())?;
            self.runtime.commit(&changes);
            if QUIT.load(Ordering::Relaxed) {
                return Ok(());
            }

            #[cfg(feature = "animate")]
            let timeout = self
                .runtime
                .next_deadline()
                .map_or(Duration::from_millis(250), |deadline| {
                    deadline.saturating_duration_since(std::time::Instant::now())
                })
                .min(Duration::from_millis(250));
            #[cfg(not(feature = "animate"))]
            let timeout = Duration::from_millis(250);
            if !self.terminal.poll(timeout)? {
                continue;
            }

            let mut events = Vec::new();
            if let Some(event) = self.terminal.read()? {
                coalesce(&mut events, event);
            }

            while self.terminal.poll(Duration::ZERO)? {
                if let Some(event) = self.terminal.read()? {
                    coalesce(&mut events, event);
                }
            }

            for event in events {
                if matches!(event, Event::Resize(_, _)) {
                    self.terminal.clear()?;
                }

                self.handle_event(event);
            }
        }
    }
}

#[cfg(any(feature = "termina", feature = "crossterm"))]
impl App<DefaultTerminal> {
    pub fn builder(root: Blueprint) -> AppBuilder {
        AppBuilder {
            root,
            query_timeout: None,
        }
    }
}

fn coalesce(events: &mut Vec<Event>, event: Event) {
    match event {
        Event::Resize(next_width, next_height) => {
            if let Some(Event::Resize(width, height)) = events.last_mut() {
                *width = next_width;
                *height = next_height;
            } else {
                events.push(Event::Resize(next_width, next_height));
            }
        }
        Event::Mouse(next) => {
            let moved = next.kind == MouseKind::Move;
            if moved
                && let Some(Event::Mouse(last)) = events.last_mut()
                && last.kind == MouseKind::Move
            {
                *last = next;
                return;
            }

            events.push(Event::Mouse(next));
        }
        event => events.push(event),
    }
}

impl<T: Terminal> Drop for App<T> {
    fn drop(&mut self) {
        if self.active {
            self.terminal.leave();
            self.active = false;
        }
    }
}
