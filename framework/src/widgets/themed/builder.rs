use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    theme::Theme,
};

use super::Themed;

pub struct ThemedBuilder {
    theme: Theme,
    children: Vec<Blueprint>,
}

impl ThemedBuilder {
    pub(crate) fn new(child: impl IntoBlueprint) -> Self {
        Self {
            theme: Theme::default(),
            children: child.into_blueprint(),
        }
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl IntoBlueprint for ThemedBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Themed>(self.theme).children(self.children)]
    }
}
