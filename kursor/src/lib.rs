pub mod app;
pub mod bindings;
pub mod focus;
pub mod layout;
pub mod terminal;
pub mod widgets;

pub use bindings::{BehaviorBuilderExt, Bind, Bindings, Bound};
#[doc(inline)]
pub use kursor_core as core;
pub use widgets::{IntoBlueprintExt, IntoSpan, IntoSpanExt};

#[cfg(feature = "crossterm")]
pub use crossterm;

#[cfg(feature = "termina")]
pub use termina;

#[cfg(feature = "animate")]
pub use animate;

#[cfg(feature = "fx")]
pub use kursor_fx as fx;
