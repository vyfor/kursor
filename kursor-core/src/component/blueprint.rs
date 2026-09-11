use std::{
    any::{Any, TypeId},
    rc::Rc,
};

use crate::{
    component::{AnyComponent, Component, context::Cx, key::Key},
    layout::{Margin, offset::Offset},
};

/// a description of a component that hasn't been mounted yet.
pub struct Blueprint {
    pub key: Option<Key>,
    pub type_id: TypeId,
    pub props: Rc<dyn Any>,
    pub children: Rc<[Blueprint]>,
    pub create: fn(&mut Cx, &dyn Any) -> Box<dyn AnyComponent>,
    pub offset: Offset,
    pub margin: Margin,
}

impl Blueprint {
    pub fn new<C: Component>(props: C::Props) -> Self {
        fn create<C: Component>(
            cx: &mut Cx,
            props: &dyn Any,
        ) -> Box<dyn AnyComponent> {
            let props = props.downcast_ref::<C::Props>().unwrap();
            Box::new(C::create(cx, props))
        }
        let props = if TypeId::of::<C::Props>() == TypeId::of::<()>() {
            drop(props);
            empty_props()
        } else {
            Rc::new(props)
        };
        Self {
            key: None,
            type_id: TypeId::of::<C>(),
            props,
            children: empty_children(),
            create: create::<C>,
            offset: Offset::ZERO,
            margin: Margin::default(),
        }
    }

    /// attaches a key/id to the blueprint so that it can retain its identity
    /// when reconciled/rebuilt.
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// moves the component visually without affecting the layout.
    pub fn offset(mut self, offset: impl Into<Offset>) -> Self {
        self.offset = offset.into();
        self
    }

    /// applies given margin around the component that affects the layout.
    ///
    /// often used for merging borders.
    pub fn margin(mut self, margin: impl Into<Margin>) -> Self {
        self.margin = margin.into();
        self
    }

    pub fn margin_all(mut self, value: i16) -> Self {
        self.margin = Margin::all(value);
        self
    }

    pub fn margin_symmetric(mut self, horizontal: i16, vertical: i16) -> Self {
        self.margin = Margin::symmetric(horizontal, vertical);
        self
    }

    pub fn margin_horizontal(mut self, value: i16) -> Self {
        self.margin.left = value;
        self.margin.right = value;
        self
    }

    pub fn margin_vertical(mut self, value: i16) -> Self {
        self.margin.top = value;
        self.margin.bottom = value;
        self
    }

    pub fn margin_left(mut self, value: i16) -> Self {
        self.margin.left = value;
        self
    }

    pub fn margin_right(mut self, value: i16) -> Self {
        self.margin.right = value;
        self
    }

    pub fn margin_top(mut self, value: i16) -> Self {
        self.margin.top = value;
        self
    }

    pub fn margin_bottom(mut self, value: i16) -> Self {
        self.margin.bottom = value;
        self
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

thread_local! {
    static EMPTY_CHILDREN: Rc<[Blueprint]> = Rc::from(Vec::new());
    static EMPTY_PROPS: Rc<dyn Any> = Rc::new(());
}

pub(crate) fn empty_children() -> Rc<[Blueprint]> {
    EMPTY_CHILDREN.with(|children| children.clone())
}

pub(crate) fn empty_props() -> Rc<dyn Any> {
    EMPTY_PROPS.with(|props| props.clone())
}

impl Clone for Blueprint {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            type_id: self.type_id,
            props: self.props.clone(),
            children: self.children.clone(),
            create: self.create,
            offset: self.offset,
            margin: self.margin,
        }
    }
}

pub trait IntoBlueprint {
    fn into_blueprint(self) -> Vec<Blueprint>;

    fn offset(self, offset: impl Into<Offset>) -> Vec<Blueprint>
    where
        Self: Sized,
    {
        let off = offset.into();
        let mut blueprints = self.into_blueprint();
        for bp in &mut blueprints {
            bp.offset = off;
        }
        blueprints
    }

    fn margin(self, margin: impl Into<Margin>) -> Vec<Blueprint>
    where
        Self: Sized,
    {
        let m = margin.into();
        let mut blueprints = self.into_blueprint();
        for bp in &mut blueprints {
            bp.margin = m;
        }
        blueprints
    }

    fn margin_all(self, value: i16) -> Vec<Blueprint>
    where
        Self: Sized,
    {
        self.margin(Margin::all(value))
    }

    fn margin_symmetric(self, horizontal: i16, vertical: i16) -> Vec<Blueprint>
    where
        Self: Sized,
    {
        self.margin(Margin::symmetric(horizontal, vertical))
    }

    fn margin_horizontal(self, value: i16) -> Vec<Blueprint>
    where
        Self: Sized,
    {
        let mut blueprints = self.into_blueprint();
        for bp in &mut blueprints {
            bp.margin.left = value;
            bp.margin.right = value;
        }
        blueprints
    }

    fn margin_vertical(self, value: i16) -> Vec<Blueprint>
    where
        Self: Sized,
    {
        let mut blueprints = self.into_blueprint();
        for bp in &mut blueprints {
            bp.margin.top = value;
            bp.margin.bottom = value;
        }
        blueprints
    }

    fn margin_left(self, value: i16) -> Vec<Blueprint>
    where
        Self: Sized,
    {
        let mut blueprints = self.into_blueprint();
        for bp in &mut blueprints {
            bp.margin.left = value;
        }
        blueprints
    }

    fn margin_right(self, value: i16) -> Vec<Blueprint>
    where
        Self: Sized,
    {
        let mut blueprints = self.into_blueprint();
        for bp in &mut blueprints {
            bp.margin.right = value;
        }
        blueprints
    }

    fn margin_top(self, value: i16) -> Vec<Blueprint>
    where
        Self: Sized,
    {
        let mut blueprints = self.into_blueprint();
        for bp in &mut blueprints {
            bp.margin.top = value;
        }
        blueprints
    }

    fn margin_bottom(self, value: i16) -> Vec<Blueprint>
    where
        Self: Sized,
    {
        let mut blueprints = self.into_blueprint();
        for bp in &mut blueprints {
            bp.margin.bottom = value;
        }
        blueprints
    }
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
