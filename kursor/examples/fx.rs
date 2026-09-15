mod shared;

use std::time::Duration;

use kursor::{
    IntoBlueprintExt, IntoSpanExt,
    app::App,
    core::{
        component::{
            Component, MountChildren, Update, blueprint::Blueprint, context::Cx,
        },
        event::{Event, EventResult, Phase},
        render::style::Style,
    },
    fx::{self, Effect},
    widgets::{Block, Border},
};
use kursor_core::{layout::Direction, render::subcell::Subcell};
use kursor_fx::Spread;
use shared::{quit, themed};

use crate::shared::theme;

struct Fx;

impl Component for Fx {
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
        let theme = theme();

        let card = Block::new("kursor for the win!".bold().center())
            .border(Border::None)
            .style(Style::new().bg(theme.palette.error))
            .bounds(40, 5);

        let content = Effect::new(
            card,
            fx::infinite(
                fx::seq()
                    .then(
                        fx::reveal(Duration::from_millis(2000))
                            .spread(Spread::towards(Direction::Right))
                            .subcell(Subcell::EighthExt),
                    )
                    .then(
                        fx::reveal(Duration::from_millis(2000))
                            .spread(Spread::towards(Direction::Right))
                            .subcell(Subcell::EighthExt)
                            .out(),
                    ),
            )
            .delay(Duration::from_millis(500)),
        )
        .center();

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
    App::builder(Blueprint::new::<Fx>(())).build()?.run()?;

    Ok(())
}
