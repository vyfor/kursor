use std::{cell::UnsafeCell, time::Duration};

use crate::tree::id::NodeId;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    Update,
    Passive,
}

#[derive(Clone, Copy)]
pub(crate) struct FrameContext {
    pub frame_id: u64,
    pub elapsed: Duration,
    pub delta: Duration,
    pub phase: Phase,
    pub node: Option<NodeId>,
    pub runtime: *mut (),
    pub request_frame: unsafe fn(*mut (), NodeId),
}

thread_local! {
    static ACTIVE: UnsafeCell<Option<FrameContext>> = const { UnsafeCell::new(None) };
}

pub(crate) struct Scope {
    previous: Option<FrameContext>,
}

pub(crate) fn enter(context: FrameContext) -> Scope {
    let previous = ACTIVE.with(|active| unsafe { (*active.get()).replace(context) });
    Scope { previous }
}

pub(crate) fn enter_node(node: Option<NodeId>, phase: Phase) -> Scope {
    ACTIVE.with(|active| unsafe {
        let active = &mut *active.get();
        let previous = *active;
        if let Some(context) = active.as_mut() {
            context.node = node;
            context.phase = phase;
        }
        Scope { previous }
    })
}

impl Drop for Scope {
    fn drop(&mut self) {
        ACTIVE.with(|active| unsafe {
            *active.get() = self.previous.take();
        });
    }
}

pub(crate) fn current() -> Option<FrameContext> {
    ACTIVE.with(|active| unsafe { (*active.get()).as_ref().copied() })
}

pub(crate) fn request_frame() {
    ACTIVE.with(|active| unsafe {
        if let Some(context) = (*active.get()).as_ref()
            && let Some(node) = context.node
        {
            (context.request_frame)(context.runtime, node);
        }
    });
}
