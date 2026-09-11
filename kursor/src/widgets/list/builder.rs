use std::{ops::Range, rc::Rc};

use kursor_core::{
    component::{
        behavior::{Behavior, BehaviorBuilder},
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::Orientation,
    state::IntoValue,
};

use super::{
    IntoListSelection, List, ListData, ListFit, ListIntent, ListProps,
    ListState,
};

#[derive(Default)]
pub struct ListBuilder {
    props: ListProps,
}

impl ListBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn children(mut self, children: impl IntoBlueprint) -> Self {
        let mut items: Vec<Blueprint> = match self.props.data {
            ListData::Static(existing) => existing.as_ref().to_vec(),
            ListData::Lazy { .. } => Vec::new(),
        };
        items.extend(children.into_blueprint());
        self.props.data = ListData::Static(items.into());
        self
    }

    pub fn lazy(
        mut self,
        count: impl IntoValue<usize>,
        item_builder: impl Fn(usize) -> Blueprint + 'static,
    ) -> Self {
        self.props.data = ListData::Lazy {
            count: count.into_value(),
            builder: Rc::new(item_builder),
        };
        self
    }

    pub fn orientation(
        mut self,
        orientation: impl IntoValue<Orientation>,
    ) -> Self {
        self.props.orientation = orientation.into_value();
        self
    }

    pub fn vertical(mut self) -> Self {
        self.props.orientation = Orientation::Vertical.into_value();
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.props.orientation = Orientation::Horizontal.into_value();
        self
    }

    pub fn gap(mut self, gap: impl IntoValue<u16>) -> Self {
        self.props.gap = gap.into_value();
        self
    }

    pub fn scroll_margin(mut self, margin: impl IntoValue<u16>) -> Self {
        self.props.scroll_margin = margin.into_value();
        self
    }

    pub fn fit(mut self, fit: impl IntoValue<ListFit>) -> Self {
        self.props.fit = fit.into_value();
        self
    }

    pub fn overscan(mut self, overscan: impl IntoValue<usize>) -> Self {
        self.props.overscan = overscan.into_value();
        self
    }

    pub fn prefetch(mut self, prefetch: impl IntoValue<usize>) -> Self {
        self.props.prefetch = prefetch.into_value();
        self
    }

    pub fn wrap(mut self, wrap: impl IntoValue<bool>) -> Self {
        self.props.wrap = wrap.into_value();
        self
    }

    pub fn selection(mut self, selection: impl IntoListSelection) -> Self {
        self.props.selection = Some(selection.into_selection());
        self
    }

    pub fn on_select(
        mut self,
        on_select: impl Fn(&mut Cx, usize) + 'static,
    ) -> Self {
        self.props.on_select = Some(Rc::new(on_select));
        self
    }

    pub fn on_activate(
        mut self,
        on_activate: impl Fn(&mut Cx, usize) + 'static,
    ) -> Self {
        self.props.on_activate = Some(Rc::new(on_activate));
        self
    }

    pub fn on_visible_range(
        mut self,
        callback: impl Fn(Range<usize>) + 'static,
    ) -> Self {
        self.props.on_visible_range = Some(Rc::new(callback));
        self
    }

    pub fn on_request_range(
        mut self,
        callback: impl Fn(Range<usize>) + 'static,
    ) -> Self {
        self.props.on_request_range = Some(Rc::new(callback));
        self
    }

    pub fn build(self) -> Blueprint {
        List::with(self.props)
    }
}

impl BehaviorBuilder for ListBuilder {
    type State = ListState;
    type Intent = ListIntent;

    fn behavior<B>(mut self, behavior: B) -> Self
    where
        B: Behavior<State = Self::State, Intent = Self::Intent>,
    {
        self.props.behavior = Rc::new(behavior);
        self
    }
}

impl IntoBlueprint for ListBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}

impl From<ListBuilder> for Blueprint {
    fn from(builder: ListBuilder) -> Self {
        builder.build()
    }
}
