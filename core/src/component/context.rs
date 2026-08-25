use crate::{
    component::{action::Action, environment::Environment},
    layout::rect::Rect,
    state::{LocalState, Signal},
    theme::Theme,
    tree::id::NodeId,
};

pub struct Cx<'a> {
    pub rect: Rect,
    pub node: Option<NodeId>,
    pub env: Environment,
    pub(crate) actions: Option<&'a mut Vec<Action>>,
    pub(crate) global_input: Option<&'a mut Option<bool>>,
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
