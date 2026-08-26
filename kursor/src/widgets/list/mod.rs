pub mod builder;
pub use builder::ListBuilder;

use std::rc::Rc;

use kursor_core::{
    component::{
        Component, Focus, Update,
        behavior::{Behavior, BehaviorCx},
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    event::{
        Event, EventResult, Phase,
        key::KeyCode,
        mouse::{MouseButton, MouseKind},
    },
    layout::{
        Offset, Orientation,
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    state::{Signal, Value},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListState {
    pub selected: Option<usize>,
    pub count: usize,
    pub scroll_offset: u32,
    pub content_size: u32,
    pub viewport_size: u32,
    pub item_offsets: Vec<u32>,
    pub item_sizes: Vec<u32>,
}

impl ListState {
    pub fn item_at(&self, position: u32) -> Option<usize> {
        for (i, (&offset, &size)) in self.item_offsets.iter().zip(&self.item_sizes).enumerate() {
            if position >= offset && position < offset.saturating_add(size) {
                return Some(i);
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListFit {
    #[default]
    Whole,
    Partial,
}

crate::core::into_value!(ListFit);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListIntent {
    Prev,
    Next,
    First,
    Last,
    PageUp(usize),
    PageDown(usize),
    Select(usize),
    ScrollBy(i32),
    Activate,
}

#[derive(Clone, Debug)]
pub struct ListBehavior {
    pub wrap: bool,
    pub page_step: usize,
    pub wheel_step: u16,
}

impl Default for ListBehavior {
    fn default() -> Self {
        Self {
            wrap: false,
            page_step: 5,
            wheel_step: 2,
        }
    }
}

impl Behavior for ListBehavior {
    type State = ListState;
    type Intent = ListIntent;

    fn event(&self, cx: &BehaviorCx, event: &Event, state: &Self::State) -> Option<Self::Intent> {
        if cx.phase != Phase::Bubble {
            return None;
        }

        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(ListIntent::Prev),
                KeyCode::Down | KeyCode::Char('j') => Some(ListIntent::Next),
                KeyCode::Home | KeyCode::Char('g') => Some(ListIntent::First),
                KeyCode::End | KeyCode::Char('G') => Some(ListIntent::Last),
                KeyCode::PageUp => Some(ListIntent::PageUp(self.page_step)),
                KeyCode::PageDown => Some(ListIntent::PageDown(self.page_step)),
                KeyCode::Enter => Some(ListIntent::Activate),
                _ => None,
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseKind::ScrollUp => Some(ListIntent::ScrollBy(-i32::from(self.wheel_step))),
                MouseKind::ScrollDown => Some(ListIntent::ScrollBy(i32::from(self.wheel_step))),
                MouseKind::Down(MouseButton::Left) => {
                    let rel_pos = mouse.row.saturating_sub(cx.rect.y);
                    let absolute_pos = u32::from(rel_pos).saturating_add(state.scroll_offset);
                    state.item_at(absolute_pos).map(ListIntent::Select)
                }
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum ListSelection {
    Optional(Signal<Option<usize>>),
    Exact(Signal<usize>),
}

pub trait IntoListSelection {
    fn into_selection(self) -> ListSelection;
}

impl IntoListSelection for Signal<Option<usize>> {
    fn into_selection(self) -> ListSelection {
        ListSelection::Optional(self)
    }
}

impl IntoListSelection for Signal<usize> {
    fn into_selection(self) -> ListSelection {
        ListSelection::Exact(self)
    }
}

impl IntoListSelection for ListSelection {
    fn into_selection(self) -> ListSelection {
        self
    }
}

#[derive(Clone)]
pub struct ListProps {
    pub orientation: Value<Orientation>,
    pub gap: Value<u16>,
    pub scroll_margin: Value<u16>,
    pub fit: Value<ListFit>,
    pub wrap: Value<bool>,
    pub selection: Option<ListSelection>,
    pub behavior: Rc<dyn Behavior<State = ListState, Intent = ListIntent>>,
    pub on_select: Option<Rc<dyn Fn(&mut Cx, usize)>>,
    pub on_activate: Option<Rc<dyn Fn(&mut Cx, usize)>>,
}

impl Default for ListProps {
    fn default() -> Self {
        Self {
            orientation: Value::plain(Orientation::Vertical),
            gap: Value::plain(0),
            scroll_margin: Value::plain(0),
            fit: Value::plain(ListFit::Partial),
            wrap: Value::plain(false),
            selection: None,
            behavior: Rc::new(ListBehavior::default()),
            on_select: None,
            on_activate: None,
        }
    }
}

pub struct List {
    state: ListState,
    orientation: Orientation,
    gap: u16,
    scroll_margin: u16,
    fit: ListFit,
    wrap: bool,
    selection: Option<Option<usize>>,
}

impl List {
    pub fn builder() -> ListBuilder {
        ListBuilder::new()
    }

    pub fn new(children: impl IntoBlueprint) -> Blueprint {
        Self::vertical(children)
    }

    pub fn vertical(children: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ListProps {
                orientation: Value::plain(Orientation::Vertical),
                ..Default::default()
            },
            children,
        )
    }

    pub fn horizontal(children: impl IntoBlueprint) -> Blueprint {
        Self::with(
            ListProps {
                orientation: Value::plain(Orientation::Horizontal),
                ..Default::default()
            },
            children,
        )
    }

    pub fn with(props: ListProps, children: impl IntoBlueprint) -> Blueprint {
        Blueprint::new::<Self>(props).children(children)
    }

    fn adjust_scroll(&mut self, index: usize) {
        if index >= self.state.item_offsets.len() {
            return;
        }

        let item_start = self.state.item_offsets[index];
        let item_size = self.state.item_sizes.get(index).copied().unwrap_or(1);
        let item_end = item_start.saturating_add(item_size);
        let margin = u32::from(self.scroll_margin);
        let viewport = self.state.viewport_size;
        let max_scroll = self.state.content_size.saturating_sub(viewport);

        let target_min = item_start.saturating_sub(margin);
        let target_max = item_end.saturating_add(margin);

        if target_min < self.state.scroll_offset {
            self.state.scroll_offset = target_min;
        } else if target_max > self.state.scroll_offset.saturating_add(viewport) {
            self.state.scroll_offset = target_max.saturating_sub(viewport);
        }

        self.state.scroll_offset = self.state.scroll_offset.min(max_scroll);
    }

    fn set_selected(
        &mut self,
        cx: &mut Cx,
        props: &ListProps,
        index: Option<usize>,
    ) -> bool {
        let prev = self.state.selected;
        self.state.selected = index;

        if let Some(selection) = &props.selection {
            match selection {
                ListSelection::Optional(sig) => sig.set(index),
                ListSelection::Exact(sig) => {
                    if let Some(idx) = index {
                        sig.set(idx);
                    }
                }
            }
            self.selection = Some(index);
        }

        if let Some(idx) = index {
            self.adjust_scroll(idx);
            if let Some(on_select) = &props.on_select {
                (on_select)(cx, idx);
            }
        }

        self.state.selected != prev
    }

    fn apply(
        &mut self,
        cx: &mut Cx,
        props: &ListProps,
        intent: ListIntent,
    ) -> bool {
        let count = self.state.count;
        if count == 0 {
            return false;
        }

        match intent {
            ListIntent::Prev => {
                let curr = self.state.selected.unwrap_or(0);
                let next = if curr == 0 {
                    if self.wrap {
                        count.saturating_sub(1)
                    } else {
                        0
                    }
                } else {
                    curr - 1
                };
                self.set_selected(cx, props, Some(next))
            }
            ListIntent::Next => {
                let curr = self.state.selected.unwrap_or(0);
                let next = if curr + 1 >= count {
                    if self.wrap {
                        0
                    } else {
                        count.saturating_sub(1)
                    }
                } else {
                    curr + 1
                };
                self.set_selected(cx, props, Some(next))
            }
            ListIntent::First => self.set_selected(cx, props, Some(0)),
            ListIntent::Last => self.set_selected(cx, props, Some(count.saturating_sub(1))),
            ListIntent::PageUp(step) => {
                let curr = self.state.selected.unwrap_or(0);
                let next = curr.saturating_sub(step);
                self.set_selected(cx, props, Some(next))
            }
            ListIntent::PageDown(step) => {
                let curr = self.state.selected.unwrap_or(0);
                let next = (curr + step).min(count.saturating_sub(1));
                self.set_selected(cx, props, Some(next))
            }
            ListIntent::Select(idx) => {
                let clamped = idx.min(count.saturating_sub(1));
                self.set_selected(cx, props, Some(clamped))
            }
            ListIntent::ScrollBy(delta) => {
                let max_scroll = self.state.content_size.saturating_sub(self.state.viewport_size);
                let prev = self.state.scroll_offset;
                if delta > 0 {
                    self.state.scroll_offset = self
                        .state
                        .scroll_offset
                        .saturating_add(delta as u32)
                        .min(max_scroll);
                } else {
                    self.state.scroll_offset =
                        self.state.scroll_offset.saturating_sub((-delta) as u32);
                }
                self.state.scroll_offset != prev
            }
            ListIntent::Activate => {
                if let Some(selected) = self.state.selected
                    && let Some(on_activate) = &props.on_activate
                {
                    (on_activate)(cx, selected);
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl Component for List {
    type Props = ListProps;

    fn create(_cx: &mut Cx, props: &Self::Props) -> Self {
        let init_sel = props.selection.as_ref().and_then(|sel| match sel {
            ListSelection::Optional(sig) => sig.peek(),
            ListSelection::Exact(sig) => Some(sig.peek()),
        });
        let ext_sel = props.selection.as_ref().map(|selection| match selection {
            ListSelection::Optional(signal) => signal.peek(),
            ListSelection::Exact(signal) => Some(signal.peek()),
        });

        Self {
            state: ListState {
                selected: init_sel,
                ..Default::default()
            },
            orientation: Orientation::Vertical,
            gap: 0,
            scroll_margin: 0,
            fit: ListFit::Partial,
            wrap: false,
            selection: ext_sel,
        }
    }

    fn focus(&self, _props: &Self::Props) -> Focus {
        Focus {
            focusable: true,
            trap: false,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old.orientation != new.orientation
            || old.gap != new.gap
            || old.scroll_margin != new.scroll_margin
            || old.fit != new.fit
            || old.wrap != new.wrap
            || old.selection != new.selection
            || !Rc::ptr_eq(&old.behavior, &new.behavior)
            || old.on_select.as_ref().map(Rc::as_ptr) != new.on_select.as_ref().map(Rc::as_ptr)
            || old.on_activate.as_ref().map(Rc::as_ptr) != new.on_activate.as_ref().map(Rc::as_ptr)
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let orientation = props.orientation.get();
        let gap = props.gap.get();
        let scroll_margin = props.scroll_margin.get();
        let fit = props.fit.get();
        let wrap = props.wrap.get();

        let mut measure_needed = false;
        let mut layout_needed = false;

        if self.orientation != orientation || self.gap != gap {
            self.orientation = orientation;
            self.gap = gap;
            measure_needed = true;
        }

        if self.scroll_margin != scroll_margin || self.wrap != wrap || self.fit != fit {
            self.scroll_margin = scroll_margin;
            self.wrap = wrap;
            self.fit = fit;
            layout_needed = true;
        }

        if let Some(selection) = &props.selection {
            let ext_selected = match selection {
                ListSelection::Optional(sig) => sig.read(),
                ListSelection::Exact(sig) => Some(sig.read()),
            };
            if self.selection != Some(ext_selected) {
                self.state.selected = ext_selected;
                self.selection = Some(ext_selected);
                layout_needed = true;
            }
        }

        if measure_needed {
            Update::MEASURE
        } else if layout_needed {
            Update::LAYOUT
        } else {
            Update::NONE
        }
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        available: Size,
        children: &mut MeasureCx,
    ) -> Size {
        let count = children.len();
        self.state.count = count;
        if count == 0 {
            self.state.item_offsets.clear();
            self.state.item_sizes.clear();
            self.state.content_size = 0;
            return Size::default();
        }

        self.state.item_offsets.clear();
        self.state.item_sizes.clear();
        self.state.item_offsets.reserve(count);
        self.state.item_sizes.reserve(count);

        let mut main_cursor: u32 = 0;
        let mut max_cross: u16 = 0;

        for index in 0..count {
            let child_available = match self.orientation {
                Orientation::Vertical => Size::new(available.width, u16::MAX),
                Orientation::Horizontal => Size::new(u16::MAX, available.height),
            };
            let child_size = children.measure(index, child_available);

            let (main_size, cross_size) = match self.orientation {
                Orientation::Vertical => (child_size.height, child_size.width),
                Orientation::Horizontal => (child_size.width, child_size.height),
            };

            self.state.item_offsets.push(u32::from(main_cursor));
            self.state.item_sizes.push(u32::from(main_size));

            main_cursor = main_cursor.saturating_add(u32::from(main_size));
            if index + 1 < count {
                main_cursor = main_cursor.saturating_add(u32::from(self.gap));
            }
            max_cross = max_cross.max(cross_size);
        }

        self.state.content_size = main_cursor;

        match self.orientation {
            Orientation::Vertical => Size::new(
                max_cross.min(available.width),
                main_cursor.min(u32::from(available.height)) as u16,
            ),
            Orientation::Horizontal => Size::new(
                main_cursor.min(u32::from(available.width)) as u16,
                max_cross.min(available.height),
            ),
        }
    }

    fn layout(&mut self, _cx: &mut Cx, _props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        let count = children.len();
        self.state.count = count;
        if count == 0 {
            return;
        }

        let (viewport_size, cross_size) = match self.orientation {
            Orientation::Vertical => (u32::from(area.height), area.width),
            Orientation::Horizontal => (u32::from(area.width), area.height),
        };
        self.state.viewport_size = viewport_size;

        let scroll_offset = self.state.scroll_offset;

        for index in 0..count {
            let item_offset = self.state.item_offsets.get(index).copied().unwrap_or(0);
            let item_size = self.state.item_sizes.get(index).copied().unwrap_or(0);

            match self.orientation {
                Orientation::Vertical => {
                    let visible_start = i64::from(item_offset) - i64::from(scroll_offset);
                    let visible_end = visible_start + i64::from(item_size);
                    let hidden = self.fit == ListFit::Whole
                        && (visible_start < 0 || visible_end > i64::from(viewport_size));
                    let offset = visible_start.min(0).max(i64::from(i32::MIN)) as i32;
                    let position = visible_start.max(0).min(i64::from(u16::MAX)) as u16;
                    children.set(
                        index,
                        Rect::new(
                            area.x,
                            area.y.saturating_add(position),
                            cross_size,
                            if hidden { 0 } else { item_size.min(u32::from(u16::MAX)) as u16 },
                        ),
                    );
                    children.translate(index, Offset::new(0, offset));
                }
                Orientation::Horizontal => {
                    let visible_start = i64::from(item_offset) - i64::from(scroll_offset);
                    let visible_end = visible_start + i64::from(item_size);
                    let hidden = self.fit == ListFit::Whole
                        && (visible_start < 0 || visible_end > i64::from(viewport_size));
                    let offset = visible_start.min(0).max(i64::from(i32::MIN)) as i32;
                    let position = visible_start.max(0).min(i64::from(u16::MAX)) as u16;
                    children.set(
                        index,
                        Rect::new(
                            area.x.saturating_add(position),
                            area.y,
                            if hidden { 0 } else { item_size.min(u32::from(u16::MAX)) as u16 },
                            cross_size,
                        ),
                    );
                    children.translate(index, Offset::new(offset, 0));
                }
            }
        }
    }

    fn event(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        event: &Event,
        phase: Phase,
    ) -> EventResult {
        let bcx = BehaviorCx {
            phase,
            rect: cx.rect,
        };

        if let Some(intent) = props.behavior.event(&bcx, event, &self.state) {
            if self.apply(cx, props, intent) {
                cx.relayout_self();
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        } else {
            EventResult::Ignored
        }
    }
}
