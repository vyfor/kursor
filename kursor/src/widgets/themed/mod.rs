pub mod builder;
pub use builder::ThemedBuilder;

use kursor_core::{
    component::{Component, Update, blueprint::IntoBlueprint, context::Cx},
    state::Value,
    theme::Theme,
};

/// a plain convenience wrapper that
/// [`provides`](crate::core::component::context::Cx::provide) the given
/// [`Theme`](crate::core::theme::Theme) to all descendant components via the
/// [`Environment`](crate::core::component::environment::Environment).
pub struct Themed;

impl Themed {
    pub fn new(child: impl IntoBlueprint) -> ThemedBuilder {
        ThemedBuilder::new(child)
    }

    pub fn default(child: impl IntoBlueprint) -> ThemedBuilder {
        Self::new(child)
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
