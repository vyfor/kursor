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

impl IntoBlueprint for () {
    fn into_blueprint(self) -> Vec<Blueprint> {
        Vec::new()
    }
}

impl<T: IntoBlueprint> IntoBlueprint for Option<T> {
    fn into_blueprint(self) -> Vec<Blueprint> {
        self.map_or_else(Vec::new, IntoBlueprint::into_blueprint)
    }
}

impl<T: IntoBlueprint> IntoBlueprint for Vec<T> {
    fn into_blueprint(self) -> Vec<Blueprint> {
        self.into_iter()
            .flat_map(IntoBlueprint::into_blueprint)
            .collect()
    }
}

impl<T: IntoBlueprint, const N: usize> IntoBlueprint for [T; N] {
    fn into_blueprint(self) -> Vec<Blueprint> {
        self.into_iter()
            .flat_map(IntoBlueprint::into_blueprint)
            .collect()
    }
}

macro_rules! into_blueprint_tuple {
    ($($type:ident: $value:ident),+ $(,)?) => {
        impl<$($type: IntoBlueprint),+> IntoBlueprint for ($($type,)+) {
            fn into_blueprint(self) -> Vec<Blueprint> {
                let ($($value,)+) = self;
                let mut children = Vec::new();
                $(children.extend($value.into_blueprint());)+
                children
            }
        }
    };
}

into_blueprint_tuple!(A: a, B: b);
into_blueprint_tuple!(A: a, B: b, C: c);
into_blueprint_tuple!(A: a, B: b, C: c, D: d);
into_blueprint_tuple!(A: a, B: b, C: c, D: d, E: e);
into_blueprint_tuple!(A: a, B: b, C: c, D: d, E: e, F: f);
into_blueprint_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g);
into_blueprint_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h);
into_blueprint_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i);
into_blueprint_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i, J: j);
