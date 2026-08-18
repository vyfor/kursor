use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::{BuildHasherDefault, Hasher},
    rc::Rc,
};

#[derive(Default)]
struct TypeIdHasher(u64);

impl Hasher for TypeIdHasher {
    fn write(&mut self, _bytes: &[u8]) {
        unreachable!("it's over");
    }

    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

type TypeMap = HashMap<TypeId, Rc<dyn Any>, BuildHasherDefault<TypeIdHasher>>;

#[derive(Clone, Default)]
pub struct Environment {
    values: Rc<TypeMap>,
}

impl Environment {
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.values.get(&TypeId::of::<T>())?.downcast_ref()
    }

    pub fn set<T: 'static>(&mut self, value: T) {
        Rc::make_mut(&mut self.values).insert(TypeId::of::<T>(), Rc::new(value));
    }

    pub fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.values, &other.values)
    }
}
