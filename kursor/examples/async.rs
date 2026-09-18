mod shared;

use std::time::Duration;

use kursor::{
    CxAsyncExt, IntoSpanExt,
    app::App,
    core::{
        component::{
            Component, MountChildren, Update, blueprint::Blueprint, context::Cx,
        },
        event::{Event, EventResult, Phase},
        state::Memo,
    },
    task::Task,
    widgets::{IntoBlueprintExt, Line},
};

use shared::{quit, themed};

struct Async {
    task: Task<String>,
}

impl Component for Async {
    type Props = ();

    fn create(cx: &mut Cx, _props: &Self::Props) -> Self {
        let task = cx.task(async {
            tokio::time::sleep(Duration::from_secs(2)).await;
            "done".to_string()
        });

        Self { task }
    }

    fn mount(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        children: &mut MountChildren,
    ) {
        let status = Memo::new({
            let task = self.task.clone();
            move || match task.read() {
                None => "loading...".to_string(),
                Some(msg) => msg,
            }
        });

        let content = Line::new().span("status: ".dim()).span(status.bold());

        children.replace(themed(content.center()));
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::builder(Blueprint::new::<Async>(())).build()?.run()?;

    Ok(())
}
