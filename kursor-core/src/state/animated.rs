use std::{
    cell::{Cell, UnsafeCell},
    rc::Rc,
};

use crate::state::{LocalState, Value};

pub struct Animated<T: LocalState> {
    pub(crate) inner: Rc<AnimatedInner<T>>,
}

pub(crate) struct AnimatedInner<T: LocalState> {
    target: Value<T>,
    driver: UnsafeCell<Box<dyn Driver<T>>>,
    current: UnsafeCell<T>,
    last_frame: Cell<u64>,
    running: Cell<bool>,
    advancing: Cell<bool>,
}

struct AdvanceGuard<'a>(&'a Cell<bool>);

impl Drop for AdvanceGuard<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

trait Driver<T: LocalState> {
    fn advance(&mut self, time: animate::Time, target: T) -> (T, animate::Activity);
}

struct DriverImpl<A>(A);

impl<T, A> Driver<T> for DriverImpl<A>
where
    T: LocalState,
    A: animate::Animation<Value = T>,
{
    fn advance(&mut self, time: animate::Time, target: T) -> (T, animate::Activity) {
        self.0.to(target);
        let activity = self.0.advance(time);
        (self.0.value().clone(), activity)
    }
}

impl<T: LocalState> Clone for Animated<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: LocalState> Animated<T> {
    pub fn with<A>(target: Value<T>, animation: A) -> Self
    where
        A: animate::Animation<Value = T> + 'static,
    {
        let initial = animation.value().clone();
        Self {
            inner: Rc::new(AnimatedInner {
                target,
                driver: UnsafeCell::new(Box::new(DriverImpl(animation))),
                current: UnsafeCell::new(initial),
                last_frame: Cell::new(u64::MAX),
                running: Cell::new(false),
                advancing: Cell::new(false),
            }),
        }
    }

    pub fn get(&self) -> T {
        let Some(context) = super::frame::current() else {
            return unsafe { (&*self.inner.current.get()).clone() };
        };

        if self.inner.last_frame.get() == context.frame_id {
            if self.inner.running.get() && context.phase == super::frame::Phase::Update {
                super::frame::request_frame();
            }
            return unsafe { (&*self.inner.current.get()).clone() };
        }

        if self.inner.advancing.replace(true) {
            panic!("animated value eval recursed into itself");
        }

        let _guard = AdvanceGuard(&self.inner.advancing);
        let target = self.inner.target.get();
        let (value, activity) = unsafe {
            (&mut *self.inner.driver.get())
                .advance(animate::Time::new(context.elapsed, context.delta), target)
        };

        unsafe { *self.inner.current.get() = value };
        self.inner.running.set(activity.running);
        self.inner.last_frame.set(context.frame_id);

        if activity.running && context.phase == super::frame::Phase::Update {
            super::frame::request_frame();
        }

        unsafe { (&*self.inner.current.get()).clone() }
    }
}
