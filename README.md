# kursor

a retained-mode terminal ui library for rust.

## motivation

kursor emerged from a personal need to build tui applications where components would stay self-contained.

application state should not need to know about widget handles, and widgets should not need to push state through every intermediate parent. there should be a shared state model that both the ui and application logic can observe and mutate effortlessly.

kursor is an attempt to make that model practical without delegating much of that plumbing onto the user.

the project is still early and the api is subject to change.

## usage

basic counter example:

```rs
use kursor::{
    app::{self, App},
    core::{
        component::{Component, blueprint::Blueprint, context::Cx},
        event::{Event, EventResult, Phase, key::KeyCode},
        render::{canvas::Canvas, style::Style},
    },
    terminal::crossterm::Crossterm,
};

struct Counter {
    count: i64,
}

impl Component for Counter {
    type Props = ();

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self { count: 0 }
    }

    fn event(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        event: &Event,
        phase: Phase,
    ) -> EventResult {
        if phase == Phase::Bubble
            && let Event::Key(key) = event
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    app::quit();
                    return EventResult::Stop;
                }
                KeyCode::Char('+') => {
                    self.count += 1;
                    return EventResult::Consumed;
                }
                KeyCode::Char('-') => {
                    self.count -= 1;
                    return EventResult::Consumed;
                }
                _ => {}
            }
        }

        EventResult::Ignored
    }

    fn paint(&self, cx: &mut Cx, _props: &Self::Props, canvas: &mut Canvas) {
        canvas.clear();

        let text = self.count.to_string();
        let width = text.chars().count() as u16;
        let x = cx.rect.x + cx.rect.width.saturating_sub(width) / 2;
        let y = cx.rect.y + cx.rect.height / 2;

        canvas.set_str(x, y, &text, Style::default());
    }
}

fn main() -> std::io::Result<()> {
    App::<Crossterm>::from(Blueprint::new::<Counter>(()))?.run()
}
```