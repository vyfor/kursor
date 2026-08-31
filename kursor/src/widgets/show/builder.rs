use std::rc::Rc;

use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    state::Value,
};

use super::{Show, ShowProps};

pub struct ShowBuilder {
    condition: Value<bool>,
    child: Option<Vec<Blueprint>>,
    fallback: Option<Vec<Blueprint>>,
}

impl ShowBuilder {
    pub fn new(condition: Value<bool>) -> Self {
        Self {
            condition,
            child: None,
            fallback: None,
        }
    }

    pub fn child(mut self, child: impl IntoBlueprint) -> Self {
        self.child = Some(child.into_blueprint());
        self
    }

    pub fn fallback(mut self, fallback: impl IntoBlueprint) -> Self {
        self.fallback = Some(fallback.into_blueprint());
        self
    }

    pub fn build(self) -> Blueprint {
        let condition = self.condition;
        let child = self.child.unwrap_or_default();
        let fallback = self.fallback.unwrap_or_default();

        Show::with(ShowProps {
            render: Rc::new(move || {
                if condition.get() {
                    child.clone()
                } else {
                    fallback.clone()
                }
            }),
        })
    }
}

impl IntoBlueprint for ShowBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}

impl From<ShowBuilder> for Blueprint {
    fn from(builder: ShowBuilder) -> Self {
        builder.build()
    }
}
