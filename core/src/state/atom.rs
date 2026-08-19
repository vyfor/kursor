use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::state::{
    arena::arena,
    id::AtomId,
    slot::{Guard, Slot, State},
};

pub struct Atom<T: State> {
    slot: NonNull<Slot<T>>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: State> Atom<T> {
    pub fn new(value: T) -> Self {
        let id = AtomId::next();
        let slot_index = arena().insert(id, value);
        let slot = arena().get_ptr::<T>(slot_index);
        Self {
            slot: unsafe { NonNull::new_unchecked(slot as *mut Slot<T>) },
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    fn slot(&self) -> &Slot<T> {
        unsafe { self.slot.as_ref() }
    }

    pub fn id(&self) -> AtomId {
        self.slot().id()
    }

    #[inline(always)]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.slot().with(f)
    }

    #[inline(always)]
    pub fn borrow(&self) -> Guard<'_, T> {
        self.slot().borrow()
    }

    #[inline(always)]
    pub fn read(&self) -> T {
        self.slot().read()
    }

    #[inline(always)]
    pub fn peek(&self) -> T {
        self.slot().peek()
    }

    #[inline(always)]
    pub fn set(&self, value: T) {
        self.slot().set(value);
    }

    #[inline(always)]
    pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.slot().update(f)
    }
}

impl<T: State> Clone for Atom<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: State> Copy for Atom<T> {}

impl<T: State> PartialEq for Atom<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.slot == other.slot
    }
}

impl<T: State> Eq for Atom<T> {}

unsafe impl<T: State> Send for Atom<T> {}
unsafe impl<T: State> Sync for Atom<T> {}
