use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    theme::Theme,
    state::{IntoValue, Value},
};

use super::Themed;

pub struct ThemedBuilder {
    theme: Value<Theme>,
    children: Vec<Blueprint>,
}

impl ThemedBuilder {
    pub(crate) fn new(child: impl IntoBlueprint) -> Self {
        Self {
            theme: Value::plain(Theme::default()),
            children: child.into_blueprint(),
        }
    }

    pub fn theme(mut self, theme: impl IntoValue<Theme>) -> Self {
        self.theme = theme.into_value();
        self
    }
}

impl IntoBlueprint for ThemedBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Themed>(self.theme).children(self.children)]
    }
}
