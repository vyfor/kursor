use crate::{
    component::{action::Action, environment::Environment},
    layout::rect::Rect,
    state::{IntoChannel, LocalState, Signal, Transition, Value},
    theme::Theme,
    tree::id::NodeId,
};

#[cfg(feature = "animate")]
use crate::state::Channel;

#[cfg(feature = "animate")]
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[cfg(feature = "animate")]
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct ChannelKey {
    channel: Channel,
    type_id: TypeId,
}

#[cfg(feature = "animate")]
pub(crate) struct Animations {
    values: HashMap<ChannelKey, Box<dyn Any>>,
}

#[cfg(feature = "animate")]
impl Default for Animations {
    fn default() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

#[cfg(feature = "animate")]
pub trait Transitionable:
    LocalState
    + animate::Integrate<Velocity: animate::Settled>
    + animate::Interpolate
    + animate::Distance
{
}

#[cfg(feature = "animate")]
impl<T> Transitionable for T where
    T: LocalState
        + animate::Integrate<Velocity: animate::Settled>
        + animate::Interpolate
        + animate::Distance
{
}

#[cfg(not(feature = "animate"))]
pub trait Transitionable: LocalState {}

#[cfg(not(feature = "animate"))]
impl<T: LocalState> Transitionable for T {}

#[cfg(feature = "animate")]
struct TransitionItem<T: Transitionable> {
    transition: Transition,
    driver: animate::Driver<T>,
}

pub struct Cx<'a> {
    pub rect: Rect,
    pub node: Option<NodeId>,
    pub env: Environment,
    pub(crate) actions: Option<&'a mut Vec<Action>>,
    pub(crate) global_input: Option<&'a mut Option<bool>>,
    #[cfg(feature = "animate")]
    pub(crate) animations: Option<&'a mut Animations>,
}

impl<'a> Cx<'a> {
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.env.get()
    }

    pub fn provide<T: 'static>(&mut self, value: T) {
        self.env.set(value);
    }

    pub fn owned<T: Clone + 'static>(&self) -> Option<T> {
        self.get::<T>().cloned()
    }

    pub fn signal<T: LocalState>(&self, value: T) -> Signal<T> {
        Signal::new(value)
    }

    #[cfg(feature = "animate")]
    pub fn time(&self) -> animate::Time {
        let context = crate::state::frame::current()
            .expect("animation time is only available within runtime");
        animate::Time::new(context.elapsed, context.delta)
    }

    #[cfg(feature = "animate")]
    pub fn animate<A: animate::Animation>(&mut self, animation: &mut A) -> animate::Activity {
        let activity = animation.advance(self.time());
        if activity.running {
            crate::state::frame::request_frame();
        }
        activity
    }

    #[cfg(feature = "animate")]
    pub fn transition<K: IntoChannel, T: Transitionable>(
        &mut self,
        channel: K,
        target: T,
        transition: Option<Transition>,
    ) -> T {
        let channel = channel.into_channel();
        let channel_key = ChannelKey {
            channel,
            type_id: TypeId::of::<T>(),
        };
        let Some(transition) = transition else {
            if let Some(store) = self.animations.as_deref_mut() {
                store.values.remove(&channel_key);
            }
            return target;
        };
        let time = self.time();
        let Some(store) = self.animations.as_deref_mut() else {
            return target;
        };
        let slot = store.values.entry(channel_key).or_insert_with(|| {
            Box::new(TransitionItem {
                transition,
                driver: transition.build(target.clone()),
            })
        });
        let slot = slot
            .downcast_mut::<TransitionItem<T>>()
            .expect("type mismatch !!");
        if slot.transition != transition {
            slot.transition = transition;
            slot.driver.transition(transition);
        }
        slot.driver.to(target);
        let activity = slot.driver.advance(time);
        if activity.running {
            crate::state::frame::request_frame();
        }
        slot.driver.value().clone()
    }

    #[cfg(not(feature = "animate"))]
    pub fn transition<K: IntoChannel, T: Transitionable>(
        &mut self,
        _channel: K,
        target: T,
        _transition: Option<Transition>,
    ) -> T {
        target
    }

    pub fn resolve<K: IntoChannel, T: Transitionable>(
        &mut self,
        channel: K,
        value: &Value<T>,
    ) -> T {
        let target = value.get();
        let transition = value.get_transition();
        self.transition(channel, target, transition)
    }

    pub fn resolve_or<K: IntoChannel, T: Transitionable, V: ResolveValue<T>>(
        &mut self,
        channel: K,
        value: V,
        fallback: T,
        default_transition: Option<Transition>,
    ) -> T {
        value.resolve_or(self, channel, fallback, default_transition)
    }

    pub fn theme(&self) -> &Theme {
        match self.get() {
            Some(theme) => theme,
            None => Theme::default_ref(),
        }
    }

    pub fn focus(&mut self, focused: bool) {
        if let Some(actions) = self.actions.as_deref_mut() {
            match (focused, self.node) {
                (true, Some(node)) => actions.push(Action::Focus(Some(node))),
                (false, _) => actions.push(Action::Focus(None)),
                (true, None) => {}
            }
        }
    }

    pub fn capture(&mut self, captured: bool) {
        if let Some(actions) = self.actions.as_deref_mut() {
            match (captured, self.node) {
                (true, Some(node)) => actions.push(Action::Capture(node)),
                (false, _) => actions.push(Action::Release),
                (true, None) => {}
            }
        }
    }

    pub fn remeasure(&mut self, node: NodeId) {
        if let Some(actions) = self.actions.as_deref_mut() {
            actions.push(Action::Remeasure(node));
        }
    }

    pub fn remeasure_self(&mut self) {
        if let Some(node) = self.node {
            self.remeasure(node);
        }
    }

    pub fn repaint(&mut self, node: NodeId) {
        if let Some(actions) = self.actions.as_deref_mut() {
            actions.push(Action::Repaint(node));
        }
    }

    pub fn repaint_self(&mut self) {
        if let Some(node) = self.node {
            self.repaint(node);
        }
    }

    pub fn relayout(&mut self, node: NodeId) {
        if let Some(actions) = self.actions.as_deref_mut() {
            actions.push(Action::Relayout(node));
        }
    }

    pub fn relayout_self(&mut self) {
        if let Some(node) = self.node {
            self.relayout(node);
        }
    }

    pub fn cursor(&mut self, position: Option<(u16, u16)>) {
        if let (Some(actions), Some(node)) = (self.actions.as_deref_mut(), self.node) {
            actions.push(Action::Cursor(node, position));
        }
    }

    pub fn global_input(&mut self, enabled: bool) {
        if let Some(global_input) = self.global_input.as_deref_mut() {
            *global_input = Some(enabled);
        }
    }
}

pub trait ResolveValue<T: Transitionable> {
    fn resolve_or<K: IntoChannel>(
        self,
        cx: &mut Cx,
        channel: K,
        fallback: T,
        default_transition: Option<Transition>,
    ) -> T;
}

impl<'a, T: Transitionable> ResolveValue<T> for Option<&'a Value<T>> {
    fn resolve_or<K: IntoChannel>(
        self,
        cx: &mut Cx,
        channel: K,
        fallback: T,
        default_transition: Option<Transition>,
    ) -> T {
        let (target, transition) = match self {
            Some(val) => {
                let t = val.get_transition().or(default_transition);
                (val.get(), t)
            }
            None => (fallback, default_transition),
        };
        cx.transition(channel, target, transition)
    }
}

impl<'a, T: Transitionable> ResolveValue<T> for &'a Option<Value<T>> {
    fn resolve_or<K: IntoChannel>(
        self,
        cx: &mut Cx,
        channel: K,
        fallback: T,
        default_transition: Option<Transition>,
    ) -> T {
        self.as_ref()
            .resolve_or(cx, channel, fallback, default_transition)
    }
}

impl<'a, T: Transitionable> ResolveValue<T> for &'a Value<T> {
    fn resolve_or<K: IntoChannel>(
        self,
        cx: &mut Cx,
        channel: K,
        _fallback: T,
        default_transition: Option<Transition>,
    ) -> T {
        let transition = self.get_transition().or(default_transition);
        cx.transition(channel, self.get(), transition)
    }
}

impl<T: Transitionable> ResolveValue<T> for Option<T> {
    fn resolve_or<K: IntoChannel>(
        self,
        cx: &mut Cx,
        channel: K,
        fallback: T,
        default_transition: Option<Transition>,
    ) -> T {
        let target = self.unwrap_or(fallback);
        cx.transition(channel, target, default_transition)
    }
}

impl<'a, T: Transitionable> ResolveValue<T> for &'a Value<Option<T>> {
    fn resolve_or<K: IntoChannel>(
        self,
        cx: &mut Cx,
        channel: K,
        fallback: T,
        default_transition: Option<Transition>,
    ) -> T {
        let target = self.get().unwrap_or(fallback);
        cx.transition(channel, target, default_transition)
    }
}
