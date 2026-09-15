mod shared;

use kursor::{
    app::App,
    core::{
        component::{
            Component, MountChildren, Update, blueprint::Blueprint, context::Cx,
        },
        event::{Event, EventResult, Phase},
    },
    widgets::{Block, Flex, IntoBlueprintExt},
};

use crate::shared::{quit, themed};

struct Borders;

impl Component for Borders {
    type Props = ();

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self
    }

    fn mount(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        children: &mut MountChildren,
    ) {
        let content = Flex::row((
            Block::rounded(()).fill(1),
            Flex::column((
                Block::heavy(()).fill(1),
                Block::plain(()).margin_top(-1).fill(1),
            ))
            .margin_left(-1)
            .fill(1),
        ));

        children.replace(themed(content));
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
    App::builder(Blueprint::new::<Borders>(())).build()?.run()?;

    Ok(())
}
