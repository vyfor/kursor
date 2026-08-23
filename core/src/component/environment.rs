use std::{
    any::{Any, TypeId},
    rc::Rc,
};

type Entry = (TypeId, Rc<dyn Any>);

#[derive(Clone, Default)]
pub struct Environment {
    values: Rc<[Entry]>,
}

impl Environment {
    pub fn get<T: 'static>(&self) -> Option<&T> {
        let id = TypeId::of::<T>();
        self.values
            .iter()
            .find_map(|(tid, val)| (*tid == id).then(|| val.downcast_ref::<T>().unwrap()))
    }

    pub fn set<T: 'static>(&mut self, value: T) {
        let id = TypeId::of::<T>();
        let mut existing = self.values.to_vec();
        if let Some(pos) = existing.iter().position(|(tid, _)| *tid == id) {
            existing[pos].1 = Rc::new(value);
        } else {
            existing.push((id, Rc::new(value)));
        }
        self.values = existing.into();
    }

    pub fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.values, &other.values)
    }
}
