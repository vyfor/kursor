use std::rc::Rc;

use super::blueprint::{Blueprint, IntoBlueprint};

pub struct Children {
    existing: Rc<[Blueprint]>,
    replacement: Option<Vec<Blueprint>>,
    rev: Option<u64>,
    hit: bool,
}

impl Children {
    pub(crate) fn new(existing: Rc<[Blueprint]>, rev: Option<u64>) -> Self {
        Self {
            existing,
            replacement: None,
            rev,
            hit: false,
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
        self.rev = None;
        self.replacement = Some(children.into_blueprint());
    }

    pub fn clear(&mut self) {
        self.rev = None;
        self.replacement = Some(Vec::new());
    }

    pub fn memo<T>(&mut self, key: u64, build: impl FnOnce() -> T)
    where
        T: IntoBlueprint,
    {
        if self.rev == Some(key) {
            self.hit = true;
            return;
        }
        self.rev = Some(key);
        self.replacement = Some(build().into_blueprint());
    }

    pub(crate) fn finish(self) -> (Option<Vec<Blueprint>>, Option<u64>, bool) {
        (self.replacement, self.rev, self.hit)
    }
}
