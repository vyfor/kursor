use kursor_core::{
    component::{
        Component,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    theme::Theme,
};

pub struct Themed;

impl Themed {
    pub fn new(theme: Theme, child: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(theme).child(child)
    }

    pub fn default(child: impl IntoBlueprint) -> Blueprint {
        Self::new(Theme::default(), child)
    }
}

impl Component for Themed {
    type Props = Theme;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self
    }

    fn build(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        children: Vec<Blueprint>,
    ) -> Vec<Blueprint> {
        cx.provide(*props);
        children
    }
}
