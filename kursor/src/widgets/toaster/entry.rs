use std::time::Duration;

use kursor_core::{component::blueprint::Blueprint, layout::rect::Rect};

#[cfg(feature = "fx")]
use crate::fx::FxCell;

use super::{ToasterProps, handle::ToastId};

pub(crate) struct Entry {
    pub(crate) id: ToastId,
    pub(crate) content: Blueprint,
    pub(crate) deadline: Option<Duration>,
    #[cfg(feature = "fx")]
    pub(crate) cell: FxCell,
    pub(crate) exiting: bool,
    pub(crate) rect: Rect,
    pub(crate) motion: Option<Motion>,
}

pub(crate) struct Motion {
    pub(crate) y: u16,
    pub(crate) distance: i32,
    pub(crate) height: u16,
    pub(crate) start: Duration,
}

impl Entry {
    pub(crate) fn finished(&self) -> bool {
        self.exiting && self.cell_finished()
    }

    pub(crate) fn exit(&mut self, config: &ToasterProps) {
        self.exiting = true;
        #[cfg(feature = "fx")]
        self.cell.set(config.exit.clone());
    }

    #[cfg(feature = "fx")]
    fn cell_finished(&self) -> bool {
        self.cell.finished()
    }

    #[cfg(not(feature = "fx"))]
    fn cell_finished(&self) -> bool {
        true
    }
}
