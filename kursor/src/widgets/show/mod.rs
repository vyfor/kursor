pub mod builder;
pub use builder::ShowBuilder;

use std::rc::Rc;

use kursor_core::{
    component::{
        Component, Update,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::{
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    state::IntoValue,
};

#[derive(Clone)]
pub struct ShowProps {
    pub render: Rc<dyn Fn() -> Vec<Blueprint>>,
}

impl PartialEq for ShowProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.render, &other.render)
    }
}

/// conditionally mounts or unmounts its children.
pub struct Show;

impl Show {
    pub fn new(condition: impl IntoValue<bool>) -> ShowBuilder {
        ShowBuilder::new(condition.into_value())
    }

    pub fn when(
        condition: impl IntoValue<bool>,
        child: impl IntoBlueprint,
    ) -> Blueprint {
        let condition = condition.into_value();
        let child = child.into_blueprint();
        Self::with(ShowProps {
            render: Rc::new(move || {
                if condition.get() {
                    child.clone()
                } else {
                    Vec::new()
                }
            }),
        })
    }

    pub fn matching<T, B>(
        value: impl IntoValue<T>,
        match_fn: impl Fn(&T) -> B + 'static,
    ) -> Blueprint
    where
        T: Clone + PartialEq + 'static,
        B: IntoBlueprint,
    {
        let value = value.into_value();
        Self::with(ShowProps {
            render: Rc::new(move || match_fn(&value.get()).into_blueprint()),
        })
    }

    pub fn with(props: ShowProps) -> Blueprint {
        Blueprint::new::<Self>(props)
    }
}

impl Component for Show {
    type Props = ShowProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        Update::children((props.render)())
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        available: Size,
        children: &mut MeasureCx,
    ) -> Size {
        if children.is_empty() {
            Size::default()
        } else {
            children.measure(0, available)
        }
    }

    fn layout(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        area: Rect,
        children: &mut LayoutCx,
    ) {
        if !children.is_empty() {
            children.set(0, area);
        }
    }
}
