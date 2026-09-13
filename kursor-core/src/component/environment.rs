use std::{
    any::{Any, TypeId},
    rc::Rc,
};

type Compare = fn(&Rc<dyn Any>, &Rc<dyn Any>) -> bool;

#[derive(Clone)]
struct Entry {
    type_id: TypeId,
    value: Rc<dyn Any>,
    compare: Compare,
}

#[derive(Clone, Default)]
pub struct Environment {
    values: Rc<[Entry]>,
}

impl Environment {
    pub fn get<T: 'static>(&self) -> Option<&T> {
        let id = TypeId::of::<T>();
        self.values.iter().find_map(|entry| {
            (entry.type_id == id)
                .then(|| entry.value.downcast_ref::<T>().unwrap())
        })
    }

    pub fn set<T: PartialEq + 'static>(&mut self, value: T) {
        self.set_entry(TypeId::of::<T>(), Rc::new(value), |a, b| {
            Rc::ptr_eq(a, b) || a.downcast_ref::<T>() == b.downcast_ref::<T>()
        });
    }

    fn set_entry(
        &mut self,
        type_id: TypeId,
        value: Rc<dyn Any>,
        compare: Compare,
    ) {
        let mut existing = self.values.to_vec();
        if let Some(pos) =
            existing.iter().position(|entry| entry.type_id == type_id)
        {
            existing[pos] = Entry {
                type_id,
                value,
                compare,
            };
        } else {
            existing.push(Entry {
                type_id,
                value,
                compare,
            });
        }
        self.values = existing.into();
    }

    pub fn same(&self, other: &Self) -> bool {
        if Rc::ptr_eq(&self.values, &other.values) {
            return true;
        }
        if self.values.len() != other.values.len() {
            return false;
        }
        self.values.iter().zip(other.values.iter()).all(|(a, b)| {
            a.type_id == b.type_id && (a.compare)(&a.value, &b.value)
        })
    }
}
