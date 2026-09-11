use std::marker::PhantomData;

use kursor_core::{
    component::{
        behavior::{Behavior, BehaviorBuilder, BehaviorCx},
        blueprint::IntoBlueprint,
    },
    event::{
        Event, Modifiers, Phase,
        key::KeyCode,
        mouse::{MouseButton, MouseKind},
    },
};

/// a key or mouse input binding with optional modifier keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bind {
    Key {
        code: KeyCode,
        modifiers: Modifiers,
    },
    Mouse {
        kind: MouseKind,
        modifiers: Modifiers,
    },
}

impl Bind {
    pub fn key(code: KeyCode) -> Self {
        Self::Key {
            code,
            modifiers: Modifiers::default(),
        }
    }

    pub fn mouse(kind: MouseKind) -> Self {
        Self::Mouse {
            kind,
            modifiers: Modifiers::default(),
        }
    }

    pub fn click(button: MouseButton) -> Self {
        Self::mouse(MouseKind::Click(button))
    }

    pub fn double_click(button: MouseButton) -> Self {
        Self::mouse(MouseKind::DoubleClick(button))
    }

    pub fn scroll_up() -> Self {
        Self::mouse(MouseKind::ScrollUp)
    }

    pub fn scroll_down() -> Self {
        Self::mouse(MouseKind::ScrollDown)
    }

    pub fn scroll_left() -> Self {
        Self::mouse(MouseKind::ScrollLeft)
    }

    pub fn scroll_right() -> Self {
        Self::mouse(MouseKind::ScrollRight)
    }

    pub fn ctrl(mut self) -> Self {
        self.modifiers_mut().ctrl = true;
        self
    }

    pub fn shift(mut self) -> Self {
        self.modifiers_mut().shift = true;
        self
    }

    pub fn alt(mut self) -> Self {
        self.modifiers_mut().alt = true;
        self
    }

    pub fn system(mut self) -> Self {
        self.modifiers_mut().system = true;
        self
    }

    pub fn meta(mut self) -> Self {
        self.modifiers_mut().meta = true;
        self
    }

    pub fn hyper(mut self) -> Self {
        self.modifiers_mut().hyper = true;
        self
    }

    fn modifiers_mut(&mut self) -> &mut Modifiers {
        match self {
            Self::Key { modifiers, .. } | Self::Mouse { modifiers, .. } => {
                modifiers
            }
        }
    }

    pub fn matches(&self, event: &Event) -> bool {
        match (self, event) {
            (Self::Key { code, modifiers }, Event::Key(key)) => {
                *code == key.code && *modifiers == key.modifiers
            }
            (Self::Mouse { kind, modifiers }, Event::Mouse(mouse)) => {
                *kind == mouse.kind && *modifiers == mouse.modifiers
            }
            _ => false,
        }
    }
}

struct Binding<I> {
    bind: Bind,
    intent: I,
}

/// input -> intent mapper.
///
/// maps key and mouse inputs directly to intents for widgets that take in a
/// [`Behavior`]. this exists primarily to reduce boilerplate for trivial
/// [`Behavior`] implementations that only handle input.
///
/// ## example
///
/// ```rust,ignore
/// Input::builder("").behavior(
///     Bindings::new()
///         .bind(Bind::key(KeyCode::Left), InputIntent::CursorLeft)
///         .bind(Bind::key(KeyCode::Right), InputIntent::CursorRight)
///         .bind(Bind::key(KeyCode::Char('a')).ctrl(), InputIntent::SelectAll),
/// );
/// ```
pub struct Bindings<S, I> {
    entries: Vec<Binding<I>>,
    state: PhantomData<fn() -> S>,
}

pub struct Bound<B, S, I> {
    builder: B,
    bindings: Bindings<S, I>,
}

impl<S, I> Bindings<S, I> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            state: PhantomData,
        }
    }

    pub fn bind(mut self, bind: Bind, intent: I) -> Self {
        self.entries.push(Binding { bind, intent });
        self
    }

    pub fn with<B>(builder: B) -> Bound<B, S, I>
    where
        B: BehaviorBuilder<State = S, Intent = I>,
    {
        Bound {
            builder,
            bindings: Self::new(),
        }
    }
}

impl<B, S, I> Bound<B, S, I> {
    pub fn bind(mut self, bind: Bind, intent: I) -> Self {
        self.bindings = self.bindings.bind(bind, intent);
        self
    }
}

impl<B, S, I> IntoBlueprint for Bound<B, S, I>
where
    B: BehaviorBuilder<State = S, Intent = I> + IntoBlueprint,
    S: 'static,
    I: Clone + Send + Sync + 'static,
{
    fn into_blueprint(
        self,
    ) -> Vec<kursor_core::component::blueprint::Blueprint> {
        self.builder.behavior(self.bindings).into_blueprint()
    }
}

impl<S, I> Default for Bindings<S, I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, I> Behavior for Bindings<S, I>
where
    I: Clone + Send + Sync + 'static,
    S: 'static,
{
    type State = S;
    type Intent = I;

    fn event(&self, cx: &BehaviorCx, event: &Event, _state: &S) -> Option<I> {
        if cx.phase != Phase::Bubble {
            return None;
        }
        self.entries
            .iter()
            .find(|entry| entry.bind.matches(event))
            .map(|entry| entry.intent.clone())
    }
}
