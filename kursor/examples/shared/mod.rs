#![allow(dead_code)]

use kursor::{
    IntoBlueprintExt, app,
    core::{
        event::{Event, EventResult, Phase, key::KeyCode},
        render::color::Color,
        theme::{Palette, Theme},
    },
    widgets::{Block, Border},
};
use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    render::style::Style,
};

pub fn quit(event: &Event, phase: Phase) -> EventResult {
    if phase != Phase::Global {
        return EventResult::Ignored;
    }

    if let Event::Key(key) = event
        && key.modifiers.ctrl
        && matches!(key.code, KeyCode::Char('c'))
    {
        app::quit();
        return EventResult::Consumed;
    }

    EventResult::Ignored
}

pub fn themed(child: impl IntoBlueprint) -> Blueprint {
    let theme = theme();

    Block::new(child)
        .border(Border::None)
        .style(Style::new().bg(theme.surface.bg))
        .build()
        .themed(theme)
}

// moonfly @ https://github.com/bluz71/vim-moonfly-colors
pub fn theme() -> Theme {
    Theme::from_palette(Palette {
        fg: Color::Rgb(0xBD, 0xBD, 0xBD),
        bg: Color::Rgb(0x08, 0x08, 0x08),
        primary: Color::Rgb(0x80, 0xA0, 0xFF),
        accent: Color::Rgb(0x79, 0xDA, 0xC8),
        muted: Color::Rgb(0x94, 0x94, 0x94),
        success: Color::Rgb(0x8C, 0xC8, 0x5F),
        warning: Color::Rgb(0xE3, 0xC7, 0x8A),
        error: Color::Rgb(0xFF, 0x5D, 0x5D),
    })
}
