use std::{
    cell::{Cell, UnsafeCell},
    collections::HashMap,
    rc::{Rc, Weak},
};

use super::{LocalState, deps, id::AtomId, scope};

trait MemoNode {
    fn id(&self) -> AtomId;
    fn depth(&self) -> usize;
    fn refresh(self: Rc<Self>) -> bool;
}

struct Registry {
    dependents: HashMap<AtomId, Vec<Weak<dyn MemoNode>>>,
    depths: HashMap<AtomId, usize>,
}

thread_local! {
    static REGISTRY: UnsafeCell<Registry> = UnsafeCell::new(Registry {
        dependents: HashMap::new(),
        depths: HashMap::new(),
    });
}

pub(crate) fn refresh_dependents(sources: &[AtomId]) -> Vec<AtomId> {
    if sources.is_empty() {
        return Vec::new();
    }

    let mut nodes: Vec<Rc<dyn MemoNode>> = Vec::new();
    let mut seen: Vec<AtomId> = Vec::new();

    REGISTRY.with(|reg| unsafe {
        let registry = &mut *reg.get();
        for source in sources {
            let Some(entries) = registry.dependents.get_mut(source) else {
                continue;
            };
            entries.retain(|node| node.strong_count() != 0);
            if entries.is_empty() {
                registry.dependents.remove(source);
                continue;
            }
            for node in entries.iter().filter_map(Weak::upgrade) {
                let id = node.id();
                if !seen.contains(&id) {
                    seen.push(id);
                    nodes.push(node);
                }
            }
        }
    });

    if nodes.is_empty() {
        return Vec::new();
    }

    nodes.sort_unstable_by_key(|node| node.depth());

    nodes
        .into_iter()
        .filter_map(|node| {
            let id = node.id();
            node.refresh().then_some(id)
        })
        .collect()
}

pub struct Memo<T: LocalState> {
    inner: Rc<Inner<T>>,
}

struct Inner<T: LocalState> {
    id: AtomId,
    value: UnsafeCell<Option<T>>,
    compute: Box<dyn Fn() -> T>,
    dirty: Cell<bool>,
    computing: Cell<bool>,
    depth: Cell<usize>,
    dependencies: UnsafeCell<Vec<AtomId>>,
}

struct ComputeGuard<'a>(&'a Cell<bool>);

impl<'a> Drop for ComputeGuard<'a> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

impl<T: LocalState> Memo<T> {
    pub fn new(compute: impl Fn() -> T + 'static) -> Self {
        let id = AtomId::next();
        let inner = Rc::new(Inner {
            id,
            value: UnsafeCell::new(None),
            compute: Box::new(compute),
            dirty: Cell::new(true),
            computing: Cell::new(false),
            depth: Cell::new(0),
            dependencies: UnsafeCell::new(Vec::new()),
        });
        Self { inner }
    }

    pub fn id(&self) -> AtomId {
        self.inner.id
    }

    pub fn get(&self) -> T {
        self.with(|val| val.clone())
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        deps::record(self.id());
        if scope::is_active() {
            let node: Rc<dyn MemoNode> = self.inner.clone();
            Inner::refresh(&self.inner, &node);
        }
        let val = unsafe {
            (*self.inner.value.get())
                .as_ref()
                .unwrap()
        };

        f(val)
    }
}

impl<T: LocalState> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: LocalState> PartialEq for Memo<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T: LocalState> Eq for Memo<T> {}

impl<T: LocalState> MemoNode for Inner<T> {
    fn id(&self) -> AtomId {
        self.id
    }

    fn depth(&self) -> usize {
        self.depth.get()
    }

    fn refresh(self: Rc<Self>) -> bool {
        let node: Rc<dyn MemoNode> = self.clone();
        Inner::refresh(&self, &node)
    }
}

impl<T: LocalState> Inner<T> {
    fn refresh(&self, node: &Rc<dyn MemoNode>) -> bool {
        if !self.dirty.get() && unsafe { (*self.value.get()).is_some() } {
            return false;
        }
        self.dirty.set(false);

        if self.computing.get() {
            panic!("memo circular dependency");
        }

        self.computing.set(true);
        let _guard = ComputeGuard(&self.computing);

        let (new_val, mut dependencies) = deps::collect(|| (self.compute)());

        dependencies.sort_unstable();
        dependencies.dedup();

        let mut max_dep_depth = 0;
        REGISTRY.with(|reg| unsafe {
            let registry = &*reg.get();
            for &dep in &dependencies {
                if let Some(&d) = registry.depths.get(&dep) {
                    max_dep_depth = max_dep_depth.max(d);
                }
            }
        });
        let depth = max_dep_depth + 1;
        self.depth.set(depth);

        let old_deps = unsafe { &mut *self.dependencies.get() };
        REGISTRY.with(|reg| unsafe {
            let registry = &mut *reg.get();
            registry.depths.insert(self.id, depth);
            for dep in old_deps.drain(..) {
                if let Some(entries) = registry.dependents.get_mut(&dep) {
                    entries.retain(|entry| {
                        entry
                            .upgrade()
                            .is_some_and(|entry| entry.id() != self.id)
                    });
                    if entries.is_empty() {
                        registry.dependents.remove(&dep);
                    }
                }
            }
            for &dep in &dependencies {
                registry
                    .dependents
                    .entry(dep)
                    .or_default()
                    .push(Rc::downgrade(node));
            }
        });
        *old_deps = dependencies;

        let changed = unsafe {
            let val_ptr = &mut *self.value.get();
            let changed = val_ptr.as_ref() != Some(&new_val);
            *val_ptr = Some(new_val);
            changed
        };

        changed
    }
}
