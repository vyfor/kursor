mod shared;

use std::time::Duration;

use kursor::{
    app::App,
    core::{
        component::{
            Component, MountChildren, Update, blueprint::Blueprint, context::Cx,
        },
        event::{Event, EventResult, Phase},
        state::Signal,
    },
    widgets::{
        Block, Button, CxToastExt, IntoBlueprintExt, IntoSpanExt, Overlay,
        Text, Toaster,
    },
};

use shared::{quit, themed};

struct Toast {
    count: Signal<u32>,
}

impl Component for Toast {
    type Props = ();

    fn create(cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            count: cx.signal(0),
        }
    }

    fn mount(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        children: &mut MountChildren,
    ) {
        let button = Button::new(Text::new("click!").center())
            .on_press({
                let count = self.count.clone();
                move |cx| {
                    count.update(|v| *v += 1);

                    cx.toast(
                        Block::new(format!("toast #{}", count.read()).bold()),
                        Duration::from_secs(3),
                    );
                }
            })
            .bounds(14, 3);

        children.replace(themed(button.center()));
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
        quit(event, phase)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Toaster::new(Blueprint::new::<Toast>(())).soft_limit(4);

    App::builder(themed(Overlay::new(app).build()))
        .build()?
        .run()?;

    Ok(())
}
