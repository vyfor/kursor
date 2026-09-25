mod shared;

use kursor::{
    IntoBlueprintExt,
    app::App,
    core::{
        component::{
            Component, MountChildren, Update, blueprint::Blueprint, context::Cx,
        },
        event::{Event, EventResult, Phase},
    },
    widgets::{
        Block, Button, Column, CxOverlayExt, IntoSpanExt, Layer, Modal,
        Overlay, Spacer, Text,
    },
};

use shared::{quit, themed};

struct Modals;

impl Component for Modals {
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
        let inner = Block::rounded(
            Column::new((
                "hello from kursor!".bold().center(),
                Spacer::new(1),
                Button::new(Text::new("close").center())
                    .on_press(|cx| {
                        cx.close_layer(());
                    })
                    .bounds(12, 3)
                    .center(),
            ))
            .padding(1),
        );

        // cloning the constructed blueprint is cheaper than cloning the
        // modal builder, hence, `.build()` is called
        let modal = Modal::new(inner).build();

        let button = Button::new(Text::new("open").center())
            .on_press(move |cx| {
                cx.open_layer(Layer::center(modal.clone()));
            })
            .bounds(16, 3)
            .center();

        children.replace(themed(button));
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
    let root = Blueprint::new::<Modals>(());

    App::builder(themed(Overlay::new(root).build()))
        .build()?
        .run()?;

    Ok(())
}
