mod shared;

use kursor::{
    app::App,
    core::{
        component::{
            Component, MountChildren, Update, blueprint::Blueprint, context::Cx,
        },
        event::{Event, EventResult, Phase, key::KeyCode},
        state::{Memo, Signal},
    },
    widgets::{Block, Button, Column, IntoBlueprintExt, Row, Spacer, Text},
};

use shared::{quit, themed};

struct Counter {
    count: Signal<i32>,
    label: Memo<String>,
}

impl Component for Counter {
    type Props = ();

    fn create(cx: &mut Cx, _props: &Self::Props) -> Self {
        let c = cx.signal(0);
        let cc = c.clone();

        Self {
            count: c,
            label: Memo::new(move || format!("count: {}", cc.read())),
        }
    }

    fn mount(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        children: &mut MountChildren,
    ) {
        let content = Column::spaced(
            1,
            (
                Text::new(self.label.clone()).center(),
                Row::new((
                    Button::new(Text::new("+").center())
                        .on_press({
                            let count = self.count.clone();
                            move |_| count.update(|v| *v += 1)
                        })
                        .bounds(9, 3),
                    Spacer::new(2),
                    Button::new(Text::new("-").center())
                        .on_press({
                            let count = self.count.clone();
                            move |_| count.update(|v| *v -= 1)
                        })
                        .bounds(9, 3),
                ))
                .center(),
            ),
        );

        children.replace(themed(Block::rounded(content.center())));
    }

    fn update(&mut self, cx: &mut Cx, _props: &Self::Props) -> Update {
        cx.global_input(true);
        Update::NONE
    }

    fn event(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        event: &Event,
        phase: Phase,
    ) -> EventResult {
        if phase != Phase::Global {
            return EventResult::Ignored;
        }

        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    self.count.update(|v| *v += 1);
                    return EventResult::Consumed;
                }
                KeyCode::Char('-') => {
                    self.count.update(|v| *v -= 1);
                    return EventResult::Consumed;
                }
                _ => {}
            }
        }

        quit(event, phase)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::builder(Blueprint::new::<Counter>(())).build()?.run()?;

    Ok(())
}
