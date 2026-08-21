use std::{
    any::{Any, TypeId},
    rc::Rc,
};

use crate::component::{AnyComponent, Component, context::Cx};

pub struct Blueprint {
    pub type_id: TypeId,
    pub props: Rc<dyn Any>,
    pub children: Rc<[Blueprint]>,
    pub create: fn(&mut Cx, &dyn Any) -> Box<dyn AnyComponent>,
}

impl Blueprint {
    pub fn new<C: Component>(props: C::Props) -> Self {
        fn create<C: Component>(cx: &mut Cx, props: &dyn Any) -> Box<dyn AnyComponent> {
            let props = props.downcast_ref::<C::Props>().unwrap();
            Box::new(C::create(cx, props))
        }
        Self {
            type_id: TypeId::of::<C>(),
            props: Rc::new(props),
            children: Rc::from([]),
            create: create::<C>,
        }
    }

    pub fn child(mut self, child: impl IntoBlueprint) -> Self {
        let mut children = self.children.as_ref().to_vec();
        children.extend(child.into_blueprint());
        self.children = children.into();
        self
    }

    pub fn children(mut self, children: impl IntoBlueprint) -> Self {
        self.children = children.into_blueprint().into();
        self
    }
}

impl Clone for Blueprint {
    fn clone(&self) -> Self {
        Self {
            type_id: self.type_id,
            props: self.props.clone(),
            children: self.children.clone(),
            create: self.create,
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
