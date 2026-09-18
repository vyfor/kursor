use std::{error::Error, sync::Arc, time::Duration};

use kursor_core::{
    event::Event,
    layout::size::Size,
    render::buffer::{CellDiff, GraphicsDiff},
};

#[cfg(feature = "crossterm")]
pub mod crossterm;

#[cfg(feature = "termina")]
pub mod termina;

pub(crate) mod probe;

pub type TerminalWaker = Arc<dyn Fn() + Send + Sync + 'static>;

/// backend abstraction for a terminal.
pub trait Terminal {
    type Error: Error + Send + Sync + 'static;

    fn size(&mut self) -> Result<Size, Self::Error>;
    fn enter(&mut self) -> Result<(), Self::Error>;
    fn leave(&mut self);
    fn poll(&mut self, timeout: Duration) -> Result<bool, Self::Error>;
    fn read(&mut self) -> Result<Option<Event>, Self::Error>;
    fn clear(&mut self) -> Result<(), Self::Error>;
    fn present(
        &mut self,
        changes: &[CellDiff],
        graphics: &GraphicsDiff,
        cursor: Option<(u16, u16)>,
    ) -> Result<(), Self::Error>;

    fn waker(&self) -> Option<TerminalWaker> {
        None
    }
}
