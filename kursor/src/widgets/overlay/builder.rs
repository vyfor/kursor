use std::rc::Rc;

use kursor_core::component::blueprint::{Blueprint, IntoBlueprint};

use super::{Overlay, OverlayProps, Overlays};

pub struct OverlayBuilder {
    overlays: Overlays,
    base: Rc<Blueprint>,
}

impl OverlayBuilder {
    pub(crate) fn new(content: impl IntoBlueprint) -> Self {
        let mut blueprints = content.into_blueprint();
        let content = blueprints.pop().unwrap();

        Self {
            overlays: Overlays::new(),
            base: Rc::new(content),
        }
    }

    pub fn overlays(mut self, overlays: Overlays) -> Self {
        self.overlays = overlays;
        self
    }
}

impl IntoBlueprint for OverlayBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Overlay>(OverlayProps {
            overlays: self.overlays,
            base: self.base,
        })]
    }
}
