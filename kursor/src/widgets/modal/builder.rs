use std::rc::Rc;
use std::sync::Arc;

use kursor_core::{
    component::{
        behavior::Behavior,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::Alignment,
    render::color::Color,
    state::{IntoValue, Value},
};

use super::{Modal, ModalBehavior, ModalIntent, ModalProps, ModalState};

pub struct ModalBuilder {
    content: Rc<Blueprint>,
    backdrop: Option<Value<Color>>,
    alignment: Value<Alignment>,
    behavior: Arc<dyn Behavior<State = ModalState, Intent = ModalIntent>>,
    on_dismiss: Option<Rc<dyn Fn(&mut Cx)>>,
    trap_focus: bool,
}

impl ModalBuilder {
    pub(crate) fn new(content: impl IntoBlueprint) -> Self {
        let mut blueprints = content.into_blueprint();
        let content = blueprints.pop().expect("no child");

        Self {
            content: Rc::new(content),
            backdrop: None,
            alignment: Value::plain(Alignment::CENTER),
            behavior: Arc::new(ModalBehavior::new()),
            on_dismiss: None,
            trap_focus: true,
        }
    }

    pub fn backdrop(mut self, color: impl IntoValue<Color>) -> Self {
        self.backdrop = Some(color.into_value());
        self
    }

    pub fn alignment(mut self, alignment: impl IntoValue<Alignment>) -> Self {
        self.alignment = alignment.into_value();
        self
    }

    pub fn behavior<B>(mut self, behavior: B) -> Self
    where
        B: Behavior<State = ModalState, Intent = ModalIntent>,
    {
        self.behavior = Arc::new(behavior);
        self
    }

    pub fn on_dismiss(
        mut self,
        on_dismiss: impl Fn(&mut Cx) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Rc::new(on_dismiss));
        self
    }

    pub fn trap_focus(mut self, trap: bool) -> Self {
        self.trap_focus = trap;
        self
    }

    pub fn build(self) -> Blueprint {
        Blueprint::new::<Modal>(ModalProps {
            content: self.content,
            backdrop: self.backdrop,
            alignment: self.alignment,
            behavior: self.behavior,
            on_dismiss: self.on_dismiss,
            trap_focus: self.trap_focus,
        })
    }
}

impl IntoBlueprint for ModalBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}

impl From<ModalBuilder> for Blueprint {
    fn from(builder: ModalBuilder) -> Self {
        builder.build()
    }
}
