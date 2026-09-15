mod shared;

use std::time::Duration;

use kursor::IntoBlueprintExt;
use kursor::image::{ImageData, ImageSource};
use kursor::{
    app::App,
    core::{
        component::{
            Component, MountChildren, Update, blueprint::Blueprint, context::Cx,
        },
        event::{Event, EventResult, Phase},
    },
    widgets::{Block, Border, Image},
};
use shared::{quit, themed};

fn gradient(width: u32, height: u32) -> ImageData {
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            data.push((x * 255 / width.saturating_sub(1).max(1)) as u8);
            data.push((y * 255 / height.saturating_sub(1).max(1)) as u8);
            data.push(180);
            data.push(255);
        }
    }

    ImageData::rgba(width, height, data).unwrap()
}

struct Graphics;

impl Component for Graphics {
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
        let img = Image::new(ImageSource::Data(gradient(80, 20)))
            .build()
            .bounds(80, 20);

        children.replace(themed(
            Block::new(img.padding(1).center()).border(Border::None),
        ));
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
    App::builder(Blueprint::new::<Graphics>(()))
        .query_terminal(Duration::from_millis(100))
        .build()?
        .run()?;

    Ok(())
}
