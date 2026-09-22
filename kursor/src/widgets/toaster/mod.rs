pub mod builder;
pub mod entry;
pub mod ext;
pub mod handle;
pub mod stack;

use std::{rc::Rc, time::Duration};

#[cfg(feature = "fx")]
use crate::fx::Fx;
use crate::widgets::{Layer, Overlays, overlay::LayerId};
use kursor_core::{
    component::{
        Component, MountChildren, Update,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::{
        Alignment,
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
};

pub use builder::ToasterBuilder;
pub use ext::CxToastExt;
pub use handle::{ToastId, Toasts};
use stack::ToastStack;

#[derive(Clone)]
pub struct ToasterProps {
    pub(crate) gap: u16,
    // when reached, will prematurely trigger exit fx of oldest toast
    pub(crate) soft_limit: Option<usize>,
    // when reached, will enqueue rest of the new toasts
    pub(crate) hard_limit: Option<usize>,
    pub(crate) pace: Duration,
    pub(crate) delay: Duration,
    pub(crate) anchor: Alignment,
    #[cfg(feature = "fx")]
    pub(crate) enter: Box<dyn Fx>,
    #[cfg(feature = "fx")]
    pub(crate) exit: Box<dyn Fx>,
    pub(crate) content: Rc<Blueprint>,
}

impl PartialEq for ToasterProps {
    fn eq(&self, other: &Self) -> bool {
        self.gap == other.gap
            && self.soft_limit == other.soft_limit
            && self.hard_limit == other.hard_limit
            && self.pace == other.pace
            && self.delay == other.delay
            && self.anchor == other.anchor
            && Rc::ptr_eq(&self.content, &other.content)
    }
}

pub struct Toaster {
    handle: Toasts,
    overlays: Option<Overlays>,
    layer: Option<LayerId>,
    content: Rc<Blueprint>,
    config: ToasterProps,
}

impl Toaster {
    pub fn new(child: impl IntoBlueprint) -> ToasterBuilder {
        ToasterBuilder::new(child)
    }

    pub fn with(props: ToasterProps) -> Blueprint {
        Blueprint::new::<Self>(props)
    }
}

impl Component for Toaster {
    type Props = ToasterProps;

    fn create(_cx: &mut Cx, props: &Self::Props) -> Self {
        Self {
            handle: Toasts::new(),
            overlays: None,
            layer: None,
            content: props.content.clone(),
            config: props.clone(),
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn mount(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        children: &mut MountChildren,
    ) {
        self.config = props.clone();
        self.content = props.content.clone();

        if cx.get::<Toasts>() != Some(&self.handle) {
            cx.provide(self.handle.clone());
        }

        if let Some(overlays) = cx.owned::<Overlays>() {
            let stack =
                ToastStack::blueprint(self.handle.clone(), self.config.clone());
            let layer =
                overlays.open(cx, Layer::viewport(self.config.anchor, stack));
            self.overlays = Some(overlays);
            self.layer = Some(layer);
        }

        children.replace(vec![(*self.content).clone()]);
    }

    fn update(&mut self, cx: &mut Cx, props: &Self::Props) -> Update {
        self.config = props.clone();
        let content_changed = !Rc::ptr_eq(&self.content, &props.content);
        if content_changed {
            self.content = props.content.clone();
        }

        if cx.get::<Toasts>() != Some(&self.handle) {
            cx.provide(self.handle.clone());
        }

        if content_changed {
            Update::children(vec![(*self.content).clone()])
        } else {
            Update::NONE
        }
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

    fn drop(&mut self, cx: &mut Cx) {
        if let (Some(overlays), Some(layer)) = (&self.overlays, self.layer) {
            overlays.close(cx, layer);
        }
    }
}
