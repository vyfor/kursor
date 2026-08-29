use kursor_core::{
    component::{
        Component, Update,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::{
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    render::canvas::Canvas,
};

use crate::{EffectCx, EffectLayer, Fx};

#[derive(Clone)]
pub struct EffectProps {
    pub(crate) fx: Box<dyn Fx>,
}

pub struct Effect {
    fx: Box<dyn Fx>,
    layer: EffectLayer,
    underlay_captured: bool,
}

impl Effect {
    pub fn new(child: impl IntoBlueprint, fx: impl Fx) -> Blueprint {
        Blueprint::new::<Self>(EffectProps { fx: Box::new(fx) }).children(child)
    }
}

impl Component for Effect {
    type Props = EffectProps;

    fn create(_cx: &mut Cx, props: &Self::Props) -> Self {
        Self {
            fx: props.fx.clone(),
            layer: EffectLayer::new(),
            underlay_captured: false,
        }
    }

    fn changed(&self, _old: &Self::Props, _new: &Self::Props) -> bool {
        false
    }

    fn update(&mut self, _cx: &mut Cx, _props: &Self::Props) -> Update {
        Update::NONE
    }

    fn pre_paint(&mut self, cx: &mut Cx, _props: &Self::Props, canvas: &mut Canvas) {
        if !self.underlay_captured {
            self.layer.capture_underlay(canvas, cx.rect);
            self.underlay_captured = true;
        }
        self.layer.write_underlay(canvas);
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        available: Size,
        children: &mut MeasureCx,
    ) -> Size {
        if children.is_empty() {
            Size::default()
        } else {
            children.measure(0, available)
        }
    }

    fn layout(&mut self, _cx: &mut Cx, _props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        if self.layer.area() != area {
            self.underlay_captured = false;
        }
        if !children.is_empty() {
            children.set(0, area);
        }
    }

    fn post_paint(&mut self, cx: &mut Cx, _props: &Self::Props, canvas: &mut Canvas) -> bool {
        self.layer.capture(canvas, cx.rect);

        let mut effect_cx = EffectCx {
            time: cx.time(),
            area: self.layer.area(),
            layer: &mut self.layer,
        };
        let activity = self.fx.apply(&mut effect_cx);

        self.layer.write(canvas);
        activity.running
    }
}
