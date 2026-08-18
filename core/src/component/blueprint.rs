use std::any::{Any, TypeId};

use crate::component::{AnyComponent, Component, context::Cx};

pub struct Blueprint {
    pub type_id: TypeId,
    pub props: Box<dyn Any>,
    pub children: Vec<Blueprint>,
    pub create: fn(&mut Cx, &dyn Any) -> Box<dyn AnyComponent>,
    pub clone: fn(&dyn Any) -> Box<dyn Any>,
}

impl Blueprint {
    pub fn new<C: Component>(props: C::Props) -> Self {
        fn create<C: Component>(cx: &mut Cx, props: &dyn Any) -> Box<dyn AnyComponent> {
            let props = props.downcast_ref::<C::Props>().unwrap();
            Box::new(C::create(cx, props))
        }
        fn clone<C: Component>(props: &dyn Any) -> Box<dyn Any> {
            let props = props.downcast_ref::<C::Props>().unwrap();
            Box::new(props.clone())
        }
        Self {
            type_id: TypeId::of::<C>(),
            props: Box::new(props),
            children: Vec::new(),
            create: create::<C>,
            clone: clone::<C>,
        }
    }

    pub fn child(mut self, child: impl IntoBlueprint) -> Self {
        self.children.extend(child.into_blueprint());
        self
    }

    pub fn children(mut self, children: impl IntoBlueprint) -> Self {
        self.children = children.into_blueprint();
        self
    }
}

impl Clone for Blueprint {
    fn clone(&self) -> Self {
        Self {
            type_id: self.type_id,
            props: (self.clone)(self.props.as_ref()),
            children: self.children.clone(),
            create: self.create,
            clone: self.clone,
        }
    }
}

pub trait IntoBlueprint {
    fn into_blueprint(self) -> Vec<Blueprint>;
}

impl IntoBlueprint for Blueprint {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self]
    }
}

impl IntoBlueprint for Vec<Blueprint> {
    fn into_blueprint(self) -> Vec<Blueprint> {
        self
    }
}
