use std::cell::{Cell, UnsafeCell};

use crate::state::id::AtomId;

thread_local! {
    static SCOPES: UnsafeCell<Vec<Vec<AtomId>>> = const { UnsafeCell::new(Vec::new()) };
    static IN_SCOPE: Cell<bool> = const { Cell::new(false) };
}

struct Scope {
    active: bool,
    prev: bool,
}

impl Scope {
    fn new() -> Self {
        let prev = IN_SCOPE.get();
        IN_SCOPE.set(true);
        SCOPES.with(|scopes| unsafe { (*scopes.get()).push(Vec::new()) });
        Self { active: true, prev }
    }

    fn finish(mut self) -> Vec<AtomId> {
        self.active = false;
        IN_SCOPE.set(self.prev);
        SCOPES
            .with(|scopes| unsafe { (*scopes.get()).pop().unwrap_or_default() })
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        if self.active {
            IN_SCOPE.set(self.prev);
            SCOPES.with(|scopes| unsafe { (*scopes.get()).pop() });
        }
    }
}

pub fn collect<R>(f: impl FnOnce() -> R) -> (R, Vec<AtomId>) {
    let scope = Scope::new();
    let result = f();
    let reads = scope.finish();
    (result, reads)
}

#[inline(always)]
pub fn record(id: AtomId) {
    if !IN_SCOPE.get() {
        return;
    }
    SCOPES.with(|scopes| unsafe {
        let scopes = &mut *scopes.get();
        if let Some(scope) = scopes.last_mut() {
            scope.push(id);
        }
    });
}
