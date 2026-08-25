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
    pub(crate) global_input: Option<&'a mut bool>,
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

    pub fn focus(&mut self) {
        if let (Some(actions), Some(node)) = (self.actions.as_deref_mut(), self.node) {
            actions.push(Action::Focus(Some(node)));
        }
    }

    pub fn unfocus(&mut self) {
        if let Some(actions) = self.actions.as_deref_mut() {
            actions.push(Action::Focus(None));
        }
    }

    pub fn capture(&mut self) {
        if let (Some(actions), Some(node)) = (self.actions.as_deref_mut(), self.node) {
            actions.push(Action::Capture(node));
        }
    }

    pub fn release_capture(&mut self) {
        if let Some(actions) = self.actions.as_deref_mut() {
            actions.push(Action::Release);
        }
    }

    pub fn invalidate(&mut self, node: NodeId) {
        if let Some(actions) = self.actions.as_deref_mut() {
            actions.push(Action::Invalidate(node));
        }
    }

    pub fn cursor(&mut self, x: u16, y: u16) {
        if let (Some(actions), Some(node)) = (self.actions.as_deref_mut(), self.node) {
            actions.push(Action::Cursor(node, Some((x, y))));
        }
    }

    pub fn clear_cursor(&mut self) {
        if let (Some(actions), Some(node)) = (self.actions.as_deref_mut(), self.node) {
            actions.push(Action::Cursor(node, None));
        }
    }

    pub fn global_input(&mut self) {
        if let Some(global_input) = self.global_input.as_deref_mut() {
            *global_input = true;
        }
    }
}
