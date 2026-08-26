pub mod builder;
pub use builder::ThemedBuilder;

use kursor_core::{
    component::{
        Component, Update,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    state::{IntoValue, Value},
    theme::Theme,
};

pub struct Themed;

impl Themed {
    pub fn builder(child: impl IntoBlueprint) -> ThemedBuilder {
        ThemedBuilder::new(child)
    }

    pub fn new(theme: impl IntoValue<Theme>, child: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(theme.into_value()).child(child)
    }

    pub fn default(child: impl IntoBlueprint) -> Blueprint {
        Self::new(Theme::default(), child)
    }
}

impl Component for Themed {
    type Props = Value<Theme>;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn mount(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        _children: &mut kursor_core::component::MountChildren,
    ) {
        cx.provide(props.get());
    }

    fn update(&mut self, cx: &mut Cx, props: &Self::Props) -> Update {
        let theme = props.get();
        if cx.get::<Theme>() != Some(&theme) {
            cx.provide(theme);
            Update::PAINT
        } else {
            Update::NONE
        }
    }
}
