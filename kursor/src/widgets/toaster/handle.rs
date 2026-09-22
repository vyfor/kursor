use std::{cell::UnsafeCell, mem, rc::Rc, time::Duration};

use kursor_core::{
    component::{
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    tree::id::NodeId,
    util::IntoDuration,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ToastId(pub u64);

pub(crate) struct ToasterState {
    pub(crate) host: Option<NodeId>,
    pub(crate) pending: Vec<ToasterCommand>,
    pub(crate) next_id: u64,
}

pub(crate) enum ToasterCommand {
    Push {
        id: ToastId,
        content: Blueprint,
        duration: Option<Duration>,
    },
    Dismiss(ToastId),
    DismissAll,
    Replace(ToastId, Blueprint),
}

#[derive(Clone)]
pub struct Toasts {
    inner: Rc<UnsafeCell<ToasterState>>,
}

impl Toasts {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(UnsafeCell::new(ToasterState {
                host: None,
                pending: Vec::new(),
                next_id: 0,
            })),
        }
    }

    pub fn push(
        &self,
        cx: &mut Cx,
        toast: impl IntoBlueprint,
        duration: impl IntoDuration,
    ) -> ToastId {
        let id = self.next_id();
        let duration = duration.into_duration();
        let mut blueprints = toast.into_blueprint();
        let content = blueprints.pop().expect("no child");

        self.command(
            cx,
            ToasterCommand::Push {
                id,
                content,
                duration,
            },
        );

        id
    }

    pub fn dismiss(&self, cx: &mut Cx, id: ToastId) {
        self.command(cx, ToasterCommand::Dismiss(id));
    }

    pub fn dismiss_all(&self, cx: &mut Cx) {
        self.command(cx, ToasterCommand::DismissAll);
    }

    pub fn replace(&self, cx: &mut Cx, id: ToastId, toast: impl IntoBlueprint) {
        let mut blueprints = toast.into_blueprint();
        let content = blueprints.pop().expect("no child");
        self.command(cx, ToasterCommand::Replace(id, content));
    }

    pub(crate) fn drain(&self) -> Vec<ToasterCommand> {
        unsafe { mem::take(&mut (*self.inner.get()).pending) }
    }

    pub(crate) fn attach(&self, host: NodeId) {
        unsafe { (*self.inner.get()).host = Some(host) };
    }

    pub(crate) fn detach(&self, host: NodeId) {
        let core = unsafe { &mut *self.inner.get() };
        if core.host == Some(host) {
            core.host = None;
        }
    }

    fn next_id(&self) -> ToastId {
        let core = unsafe { &mut *self.inner.get() };
        core.next_id += 1;

        ToastId(core.next_id)
    }

    fn command(&self, cx: &mut Cx, command: ToasterCommand) {
        let host = {
            let core = unsafe { &mut *self.inner.get() };
            core.pending.push(command);
            core.host
        };

        if let Some(host) = host {
            cx.remeasure(host);
        }
    }
}

impl Default for Toasts {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Toasts {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}
