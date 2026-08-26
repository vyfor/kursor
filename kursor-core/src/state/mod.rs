#[cfg(feature = "animate")]
pub mod animated;
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
pub mod value;

pub trait LocalState: Clone + PartialEq + 'static {}
impl<T: Clone + PartialEq + 'static> LocalState for T {}

pub trait SharedState: LocalState + Send + Sync {}
impl<T: LocalState + Send + Sync> SharedState for T {}

pub use atom::Atom;
pub use memo::Memo;
pub use signal::Signal;
pub use value::{IntoValue, Plain, Value};

#[cfg(feature = "animate")]
pub use animated::Animated;
