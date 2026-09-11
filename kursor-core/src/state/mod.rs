//! ## state and reactivity
//!
//! kursor is reactive, meaning components can subscribe to the reactive
//! state they care about.
//!
//! currently, kursor provides three types of reactive state: [`Signal`]s,
//! [`Atom`]s and [`Memo`]s.
//!
//! ## the how
//!
//! 1. before calling a component's `update()` method, the runtime starts
//!    "recording" reactive state reads on the current thread.
//! 2. then inside `update()`, when a component reads from e.g. a signal, that
//!    signal's id is recorded.
//! 3. finally, after `update()` finishes, the runtime maps and keeps track of
//!    the signal ids that were read to the component that read them.

pub mod arena;
pub mod atom;
pub mod deps;
#[cfg(feature = "animate")]
pub(crate) mod frame;
pub mod id;
pub mod memo;
pub mod queue;
pub mod scope;
pub mod signal;
pub mod slot;
pub mod spin;
pub mod transition;
pub mod value;

pub trait LocalState: Clone + PartialEq + 'static {}
impl<T: Clone + PartialEq + 'static> LocalState for T {}

pub trait SharedState: LocalState + Send + Sync {}
impl<T: LocalState + Send + Sync> SharedState for T {}

pub use atom::Atom;
pub use memo::Memo;
pub use signal::Signal;
pub use transition::{Channel, IntoChannel, Transition};
pub use value::{IntoValue, Plain, Value};
