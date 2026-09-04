mod compose;
mod context;
mod feather;
mod fx;
mod layer;
mod mask;
mod widget;

pub use animate::{Activity, Time};
pub use compose::{Infinite, Parallel, Sequence, infinite, par, seq};
pub use context::{EffectCx, Fx, FxClone};
pub use feather::Feather;
pub use fx::*;
pub use layer::{EffectLayer, LayerCell};
pub use mask::{Mask, Spread};
pub use widget::{Effect, EffectProps};
