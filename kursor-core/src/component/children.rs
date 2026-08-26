use std::rc::Rc;

use super::blueprint::{Blueprint, IntoBlueprint};

pub struct Children {
    existing: Rc<[Blueprint]>,
    replacement: Option<Vec<Blueprint>>,
}

impl Children {
    pub(crate) fn new(existing: Rc<[Blueprint]>) -> Self {
        Self {
            existing,
            replacement: None,
        }
    }

    pub fn len(&self) -> usize {
        self.replacement
            .as_ref()
            .map_or(self.existing.len(), Vec::len)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn replace(&mut self, children: impl IntoBlueprint) {
        self.replacement = Some(children.into_blueprint());
    }

    pub fn clear(&mut self) {
        self.replacement = Some(Vec::new());
    }

    pub(crate) fn finish(self) -> Option<Vec<Blueprint>> {
        self.replacement
    }
}
