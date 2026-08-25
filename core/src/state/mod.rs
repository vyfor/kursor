pub mod arena;
pub mod atom;
pub mod deps;
pub mod id;
pub mod memo;
pub mod queue;
pub mod scope;
pub mod signal;
pub mod slot;
pub mod spin;
pub mod value;

pub trait LocalState: Clone + PartialEq + 'static {}
impl<T: Clone + PartialEq + 'static> LocalState for T {}

pub trait SharedState: LocalState + Send + Sync {}
impl<T: LocalState + Send + Sync> SharedState for T {}

pub use atom::Atom;
pub use memo::Memo;
pub use signal::Signal;
pub use value::{IntoValue, Static, Value};
