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

use super::{LazyList, LazyListBehavior, LazyListProps, LazyListState};
use crate::widgets::list::{IntoListSelection, ListIntent};

pub struct LazyListBuilder {
    props: LazyListProps,
}

impl LazyListBuilder {
    pub fn new(
        count: impl IntoValue<usize>,
        item_builder: impl Fn(usize) -> Blueprint + 'static,
    ) -> Self {
        Self {
            props: LazyListProps {
                count: count.into_value(),
                item_extent: 1.into_value(),
                orientation: Orientation::Vertical.into_value(),
                gap: 0.into_value(),
                scroll_margin: 0.into_value(),
                overscan: 3.into_value(),
                prefetch: 10.into_value(),
                wrap: false.into_value(),
                selection: None,
                behavior: Rc::new(LazyListBehavior::default()),
                item_builder: Rc::new(item_builder),
                on_select: None,
                on_activate: None,
                on_visible_range: None,
                on_request_range: None,
            },
        }
    }

    pub fn item_extent(mut self, extent: impl IntoValue<u16>) -> Self {
        self.props.item_extent = extent.into_value();
        self
    }

    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
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

    pub fn on_select(mut self, on_select: impl Fn(&mut Cx, usize) + 'static) -> Self {
        self.props.on_select = Some(Rc::new(on_select));
        self
    }

    pub fn on_activate(mut self, on_activate: impl Fn(&mut Cx, usize) + 'static) -> Self {
        self.props.on_activate = Some(Rc::new(on_activate));
        self
    }

    pub fn on_visible_range(mut self, on_visible: impl Fn(Range<usize>) + 'static) -> Self {
        self.props.on_visible_range = Some(Rc::new(on_visible));
        self
    }

    pub fn on_request_range(mut self, on_request: impl Fn(Range<usize>) + 'static) -> Self {
        self.props.on_request_range = Some(Rc::new(on_request));
        self
    }

    pub fn build(self) -> Blueprint {
        LazyList::with(self.props)
    }
}

impl BehaviorBuilder for LazyListBuilder {
    type State = LazyListState;
    type Intent = ListIntent;

    fn behavior<B>(mut self, behavior: B) -> Self
    where
        B: Behavior<State = Self::State, Intent = Self::Intent>,
    {
        self.props.behavior = Rc::new(behavior);
        self
    }
}

impl IntoBlueprint for LazyListBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}

impl From<LazyListBuilder> for Blueprint {
    fn from(builder: LazyListBuilder) -> Self {
        builder.build()
    }
}
