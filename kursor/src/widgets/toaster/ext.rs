use kursor_core::{
    component::{blueprint::IntoBlueprint, context::Cx},
    util::IntoDuration,
};

use super::handle::{ToastId, Toasts};

pub trait CxToastExt {
    fn toast(
        &mut self,
        toast: impl IntoBlueprint,
        duration: impl IntoDuration,
    ) -> Option<ToastId>;

    fn dismiss_toast(&mut self, id: ToastId);

    fn dismiss_all_toasts(&mut self);

    fn replace_toast(&mut self, id: ToastId, toast: impl IntoBlueprint);
}

impl CxToastExt for Cx<'_> {
    fn toast(
        &mut self,
        toast: impl IntoBlueprint,
        duration: impl IntoDuration,
    ) -> Option<ToastId> {
        let handle = self.owned::<Toasts>();
        handle.map(|handle| handle.push(self, toast, duration))
    }

    fn dismiss_toast(&mut self, id: ToastId) {
        if let Some(handle) = self.owned::<Toasts>() {
            handle.dismiss(self, id);
        }
    }

    fn dismiss_all_toasts(&mut self) {
        if let Some(handle) = self.owned::<Toasts>() {
            handle.dismiss_all(self);
        }
    }

    fn replace_toast(&mut self, id: ToastId, toast: impl IntoBlueprint) {
        if let Some(handle) = self.owned::<Toasts>() {
            handle.replace(self, id, toast);
        }
    }
}
