pub mod builder;
pub use builder::OverlayBuilder;

use std::{cell::UnsafeCell, mem, rc::Rc};

use kursor_core::{
    component::{
        Component, Update,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::{
        Alignment, HAlign, VAlign,
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    tree::id::NodeId,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Point(u16, u16),
    Rect(Rect),
    Viewport(Alignment),
}

impl Default for Anchor {
    fn default() -> Self {
        Self::Viewport(Alignment::CENTER)
    }
}

#[derive(Clone)]
pub struct Layer {
    anchor: Anchor,
    content: Blueprint,
}

impl Layer {
    pub fn new(anchor: Anchor, content: impl IntoBlueprint) -> Self {
        let mut blueprints = content.into_blueprint();
        let content = blueprints.pop().unwrap();

        Self { anchor, content }
    }

    pub fn center(content: impl IntoBlueprint) -> Self {
        Self::new(Anchor::Viewport(Alignment::CENTER), content)
    }

    pub fn viewport(alignment: Alignment, content: impl IntoBlueprint) -> Self {
        Self::new(Anchor::Viewport(alignment), content)
    }

    pub fn at(x: u16, y: u16, content: impl IntoBlueprint) -> Self {
        Self::new(Anchor::Point(x, y), content)
    }

    pub fn anchored(rect: Rect, content: impl IntoBlueprint) -> Self {
        Self::new(Anchor::Rect(rect), content)
    }
}

#[derive(Clone)]
pub struct Overlays {
    state: Rc<UnsafeCell<State>>,
}

struct State {
    host: Option<NodeId>,
    pending: Vec<Command>,
}

enum Command {
    Open(Layer),
    CloseTop,
    CloseAll,
}

impl Overlays {
    pub fn new() -> Self {
        Self {
            state: Rc::new(UnsafeCell::new(State {
                host: None,
                pending: Vec::new(),
            })),
        }
    }

    pub fn open(&self, cx: &mut Cx, layer: Layer) {
        self.command(cx, Command::Open(layer));
    }

    // todo: possibly add means to close by id (whenever component ids are
    // added)
    pub fn close_top(&self, cx: &mut Cx) {
        self.command(cx, Command::CloseTop);
    }

    pub fn close_all(&self, cx: &mut Cx) {
        self.command(cx, Command::CloseAll);
    }

    fn command(&self, cx: &mut Cx, command: Command) {
        let host = {
            let state = unsafe { &mut *self.state.get() };
            state.pending.push(command);
            state.host
        };
        if let Some(host) = host {
            cx.remeasure(host);
        }
    }

    fn attach(&self, host: NodeId) {
        unsafe { (*self.state.get()).host = Some(host) };
    }

    fn detach(&self, host: NodeId) {
        let state = unsafe { &mut *self.state.get() };
        if state.host == Some(host) {
            state.host = None;
        }
    }

    fn drain(&self) -> Vec<Command> {
        unsafe { mem::take(&mut (*self.state.get()).pending) }
    }
}

impl Default for Overlays {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Overlays {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }
}

impl Eq for Overlays {}

#[derive(Clone)]
pub struct OverlayProps {
    overlays: Overlays,
    base: Rc<Blueprint>,
}

impl PartialEq for OverlayProps {
    fn eq(&self, other: &Self) -> bool {
        self.overlays == other.overlays && Rc::ptr_eq(&self.base, &other.base)
    }
}

struct ActiveLayer {
    layer: Layer,
    max_size: Size,
}

/// draws floating layers above the underlying content.
///
/// manages a stack of [`Layer`]s, which can be opened and
/// closed through [`Overlays`]. layers can be positioned relative to the
/// viewport, a point, or a rect.
///
/// an `Overlay` is typically mounted near the root of the application.
/// if your application uses overlays, it is recommended to have a single
/// `Overlay` near the root and refer to [`Overlays`] to manage its layers.
pub struct Overlay {
    overlays: Overlays,
    base: Rc<Blueprint>,
    layers: Vec<ActiveLayer>,
}

impl Overlay {
    pub fn new(content: impl IntoBlueprint) -> OverlayBuilder {
        OverlayBuilder::new(content)
    }

    pub fn with(overlays: Overlays, content: impl IntoBlueprint) -> Blueprint {
        let mut blueprints = content.into_blueprint();
        let content = blueprints.pop().unwrap();

        Blueprint::new::<Self>(OverlayProps {
            overlays,
            base: Rc::new(content),
        })
    }

    fn pos(anchor: Anchor, natural: Size, viewport: Rect) -> Rect {
        let width = natural.width.min(viewport.width);
        let height = natural.height.min(viewport.height);
        let max_x = viewport.right().saturating_sub(width);
        let max_y = viewport.bottom().saturating_sub(height);

        let (x, y) = match anchor {
            Anchor::Point(x, y) => (x, y),
            Anchor::Rect(rect) => (rect.x, rect.y),
            Anchor::Viewport(alignment) => {
                let x = match alignment.horizontal {
                    HAlign::Left => viewport.x,
                    HAlign::Center => viewport.x.saturating_add(
                        viewport.width.saturating_sub(width) / 2,
                    ),
                    HAlign::Right => max_x,
                };
                let y = match alignment.vertical {
                    VAlign::Top => viewport.y,
                    VAlign::Center => viewport.y.saturating_add(
                        viewport.height.saturating_sub(height) / 2,
                    ),
                    VAlign::Bottom => max_y,
                };
                (x, y)
            }
        };

        Rect::new(x.min(max_x), y.min(max_y), width, height)
    }

    fn blueprints(&self) -> Vec<Blueprint> {
        let mut children = Vec::with_capacity(self.layers.len() + 1);
        children.push((*self.base).clone());
        children.extend(
            self.layers.iter().map(|layer| layer.layer.content.clone()),
        );
        children
    }
}

impl Component for Overlay {
    type Props = OverlayProps;

    fn create(_cx: &mut Cx, props: &Self::Props) -> Self {
        Self {
            overlays: props.overlays.clone(),
            base: props.base.clone(),
            layers: Vec::new(),
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn mount(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        children: &mut kursor_core::component::MountChildren,
    ) {
        self.overlays = props.overlays.clone();
        self.base = props.base.clone();
        if cx.get::<Overlays>() != Some(&self.overlays) {
            cx.provide(self.overlays.clone());
        }
        if let Some(host) = cx.node {
            self.overlays.attach(host);
        }
        children.replace(self.blueprints());
    }

    fn update(&mut self, cx: &mut Cx, props: &Self::Props) -> Update {
        let mut changed = false;
        if self.overlays != props.overlays {
            if let Some(host) = cx.node {
                self.overlays.detach(host);
            }
            self.overlays = props.overlays.clone();
            self.layers.clear();
            changed = true;
        }
        if !Rc::ptr_eq(&self.base, &props.base) {
            self.base = props.base.clone();
            changed = true;
        }

        if cx.get::<Overlays>() != Some(&self.overlays) {
            cx.provide(self.overlays.clone());
        }
        if let Some(host) = cx.node {
            self.overlays.attach(host);
        }

        for command in self.overlays.drain() {
            match command {
                Command::Open(layer) => self.layers.push(ActiveLayer {
                    layer,
                    max_size: Size::default(),
                }),
                Command::CloseTop => {
                    self.layers.pop();
                }
                Command::CloseAll => self.layers.clear(),
            }
            changed = true;
        }

        if changed {
            Update::children(self.blueprints())
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
        if !children.is_empty() {
            children.measure(0, available);
        }
        for idx in 1..children.len() {
            let max_size = children.measure(idx, Size::new(u16::MAX, u16::MAX));
            if let Some(layer) = self.layers.get_mut(idx - 1) {
                layer.max_size = max_size;
            }
        }

        available
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
        for (index, layer) in self.layers.iter().enumerate() {
            let child = index + 1;
            if child < children.len() {
                children.set(
                    child,
                    Self::pos(layer.layer.anchor, layer.max_size, area),
                );
            }
        }
    }

    fn drop(&mut self, cx: &mut Cx) {
        if let Some(host) = cx.node {
            self.overlays.detach(host);
        }
    }
}
