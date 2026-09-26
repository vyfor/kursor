use kursor_core::component::context::Cx;

use crate::widgets::Layer;

use super::{LayerId, Overlays};

pub trait Closable {
    fn target(self, cx: &Cx) -> Option<LayerId>;
}

impl Closable for LayerId {
    fn target(self, _cx: &Cx) -> Option<LayerId> {
        Some(self)
    }
}

impl Closable for Option<LayerId> {
    fn target(self, cx: &Cx) -> Option<LayerId> {
        self.or_else(|| cx.get::<LayerId>().copied())
    }
}

impl Closable for () {
    fn target(self, cx: &Cx) -> Option<LayerId> {
        cx.get::<LayerId>().copied()
    }
}

pub trait CxOverlayExt {
    fn open_layer(&mut self, layer: Layer) -> Option<LayerId>;
    fn close_layer(&mut self, target: impl Closable) -> bool;
}

impl CxOverlayExt for Cx<'_> {
    fn open_layer(&mut self, layer: Layer) -> Option<LayerId> {
        let overlays = self.owned::<Overlays>()?;

        Some(overlays.open(self, layer))
    }

    fn close_layer(&mut self, target: impl Closable) -> bool {
        let Some(id) = target.target(self) else {
            return false;
        };

        let Some(overlays) = self.owned::<Overlays>() else {
            return false;
        };

        overlays.close(self, id);

        true
    }
}
