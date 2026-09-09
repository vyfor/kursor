pub mod builder;
pub use builder::ListBuilder;

use std::{ops::Range, rc::Rc};

use kursor_core::{
    component::{
        Component, Focus, MountChildren, Update,
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

use crate::layout::Virtualizer;

#[derive(Clone, Debug, Default)]
pub struct ListState {
    pub selected: Option<usize>,
    pub count: usize,
    pub scroll_offset: u32,
    pub content_size: u32,
    pub viewport_size: u32,
    pub offsets: Rc<[u32]>,
    pub extents: Rc<[u32]>,
}

impl PartialEq for ListState {
    fn eq(&self, other: &Self) -> bool {
        self.selected == other.selected
            && self.count == other.count
            && self.scroll_offset == other.scroll_offset
            && self.content_size == other.content_size
            && self.viewport_size == other.viewport_size
            && Rc::ptr_eq(&self.offsets, &other.offsets)
            && Rc::ptr_eq(&self.extents, &other.extents)
    }
}

impl ListState {
    pub fn item_at(&self, position: u32) -> Option<usize> {
        if self.count == 0 || self.offsets.len() < 2 {
            return None;
        }
        let idx = match self.offsets[1..].binary_search(&position) {
            Ok(i) => (i + 1).min(self.count - 1),
            Err(i) => i.min(self.count - 1),
        };
        let start = self.offsets[idx];
        let end = start.saturating_add(self.extents.get(idx).copied().unwrap_or(0));
        if position >= start && position < end {
            Some(idx)
        } else {
            None
        }
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
            page_step: 10,
            wheel_step: 3,
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
                    let rel_pos = match cx.rect {
                        rect if rect.height >= rect.width => mouse.row.saturating_sub(rect.y),
                        rect => mouse.column.saturating_sub(rect.x),
                    };
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
pub enum ListData {
    Static(Rc<[Blueprint]>),
    Lazy {
        count: Value<usize>,
        builder: Rc<dyn Fn(usize) -> Blueprint>,
    },
}

impl ListData {
    pub fn empty() -> Self {
        Self::Static(Rc::from([]))
    }

    pub fn count(&self) -> usize {
        match self {
            Self::Static(items) => items.len(),
            Self::Lazy { count, .. } => count.get(),
        }
    }

    pub fn get(&self, index: usize) -> Option<Blueprint> {
        match self {
            Self::Static(items) => items.get(index).cloned(),
            Self::Lazy { count, builder } => {
                if index < count.get() {
                    Some((builder)(index))
                } else {
                    None
                }
            }
        }
    }
}

impl Default for ListData {
    fn default() -> Self {
        Self::empty()
    }
}

impl PartialEq for ListData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Static(a), Self::Static(b)) => Rc::ptr_eq(a, b),
            (
                Self::Lazy {
                    count: c1,
                    builder: b1,
                },
                Self::Lazy {
                    count: c2,
                    builder: b2,
                },
            ) => c1 == c2 && Rc::ptr_eq(b1, b2),
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct ListProps {
    pub data: ListData,
    pub orientation: Value<Orientation>,
    pub gap: Value<u16>,
    pub scroll_margin: Value<u16>,
    pub fit: Value<ListFit>,
    pub overscan: Value<usize>,
    pub prefetch: Value<usize>,
    pub wrap: Value<bool>,
    pub selection: Option<ListSelection>,
    pub behavior: Rc<dyn Behavior<State = ListState, Intent = ListIntent>>,
    pub on_select: Option<Rc<dyn Fn(&mut Cx, usize)>>,
    pub on_activate: Option<Rc<dyn Fn(&mut Cx, usize)>>,
    pub on_visible_range: Option<Rc<dyn Fn(Range<usize>)>>,
    pub on_request_range: Option<Rc<dyn Fn(Range<usize>)>>,
}

impl Default for ListProps {
    fn default() -> Self {
        Self {
            data: ListData::empty(),
            orientation: Value::plain(Orientation::Vertical),
            gap: Value::plain(0),
            scroll_margin: Value::plain(0),
            fit: Value::plain(ListFit::Partial),
            overscan: Value::plain(2),
            prefetch: Value::plain(0),
            wrap: Value::plain(false),
            selection: None,
            behavior: Rc::new(ListBehavior::default()),
            on_select: None,
            on_activate: None,
            on_visible_range: None,
            on_request_range: None,
        }
    }
}

pub struct List {
    state: ListState,
    virt: Virtualizer,
    data: ListData,
    orientation: Orientation,
    gap: u16,
    scroll_margin: u16,
    fit: ListFit,
    overscan: usize,
    prefetch: usize,
    wrap: bool,
    selection: Option<Option<usize>>,
    visible_range: Range<usize>,
    rendered_range: Range<usize>,
    requested_range: Option<Range<usize>>,
    rev: Signal<u64>,
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
                data: ListData::Static(children.into_blueprint().into()),
                ..Default::default()
            },
        )
    }

    pub fn horizontal(children: impl IntoBlueprint) -> Blueprint {
        Self::with(ListProps {
            orientation: Value::plain(Orientation::Horizontal),
            data: ListData::Static(children.into_blueprint().into()),
            ..Default::default()
        })
    }

    pub fn lazy(
        count: impl kursor_core::state::IntoValue<usize>,
        item_builder: impl Fn(usize) -> Blueprint + 'static,
    ) -> Blueprint {
        ListBuilder::new().lazy(count, item_builder).build()
    }

    pub fn with(props: ListProps) -> Blueprint {
        Blueprint::new::<Self>(props)
    }

    fn rendered_bps(&self, range: &Range<usize>) -> Vec<Blueprint> {
        range
            .clone()
            .filter_map(|idx| {
                self.data.get(idx).map(|bp| {
                    if bp.key.is_none() {
                        bp.key(idx as u64)
                    } else {
                        bp
                    }
                })
            })
            .collect()
    }

    fn request_range(
        &mut self,
        visible: Range<usize>,
        callback: Option<&Rc<dyn Fn(Range<usize>)>>,
    ) {
        let Some(callback) = callback else {
            return;
        };
        if visible.is_empty() {
            return;
        }
        let prefetch = if self.prefetch > 0 {
            self.prefetch
        } else {
            self.overscan
        };
        let desired = visible.start.saturating_sub(prefetch)
            ..(visible.end + prefetch).min(self.state.count);
        let Some(previous) = self.requested_range.clone() else {
            callback(desired.clone());
            self.requested_range = Some(desired);
            return;
        };

        if desired.start > previous.end || desired.end < previous.start {
            callback(desired.clone());
            self.requested_range = Some(desired);
        } else {
            let mut start = previous.start;
            let mut end = previous.end;
            if desired.start < previous.start {
                start = desired.start.saturating_sub(prefetch);
                callback(start..previous.start);
            }
            if desired.end > previous.end {
                end = (desired.end + prefetch).min(self.state.count);
                callback(previous.end..end);
            }
            self.requested_range = Some(start..end);
        }
    }

    fn adjust_scroll(&mut self, index: usize) {
        if index >= self.state.count {
            return;
        }
        let item_start = self.virt.offset_of(index);
        let item_end = item_start.saturating_add(self.virt.extent_of(index));
        let margin = u32::from(self.scroll_margin).saturating_mul(self.virt.estimate());
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

    fn set_selected(&mut self, cx: &mut Cx, props: &ListProps, index: Option<usize>) -> bool {
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

    fn apply(&mut self, cx: &mut Cx, props: &ListProps, intent: ListIntent) -> bool {
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
                let first = self.virt.item_nearest(self.state.scroll_offset);
                let target = if delta > 0 {
                    (first + delta as usize).min(count.saturating_sub(1))
                } else {
                    first.saturating_sub((-delta) as usize)
                };
                let prev = self.state.scroll_offset;
                let max_scroll = self
                    .state
                    .content_size
                    .saturating_sub(self.state.viewport_size);
                self.state.scroll_offset = self.virt.offset_of(target).min(max_scroll);
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

    fn create(cx: &mut Cx, props: &Self::Props) -> Self {
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
            virt: Virtualizer::new(1),
            data: props.data.clone(),
            orientation: Orientation::Vertical,
            gap: 0,
            scroll_margin: 0,
            fit: ListFit::Partial,
            overscan: 2,
            prefetch: 0,
            wrap: false,
            selection: ext_sel,
            visible_range: 0..0,
            rendered_range: 0..0,
            requested_range: None,
            rev: cx.signal(0),
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
            || old.overscan != new.overscan
            || old.prefetch != new.prefetch
            || old.wrap != new.wrap
            || old.selection != new.selection
            || old.data != new.data
            || !Rc::ptr_eq(&old.behavior, &new.behavior)
            || old.on_select.as_ref().map(Rc::as_ptr) != new.on_select.as_ref().map(Rc::as_ptr)
            || old.on_activate.as_ref().map(Rc::as_ptr) != new.on_activate.as_ref().map(Rc::as_ptr)
            || old.on_visible_range.as_ref().map(Rc::as_ptr)
                != new.on_visible_range.as_ref().map(Rc::as_ptr)
            || old.on_request_range.as_ref().map(Rc::as_ptr)
                != new.on_request_range.as_ref().map(Rc::as_ptr)
    }

    fn mount(&mut self, _cx: &mut Cx, props: &Self::Props, children: &mut MountChildren) {
        self.orientation = props.orientation.get();
        self.gap = props.gap.get();
        self.scroll_margin = props.scroll_margin.get();
        self.fit = props.fit.get();
        self.overscan = props.overscan.get();
        self.prefetch = props.prefetch.get();
        self.wrap = props.wrap.get();
        self.data = props.data.clone();

        let count = self.data.count();
        self.state.count = count;
        self.virt.set_count(count);
        self.virt.set_gap(u32::from(self.gap));

        let (visible, rendered) = self.virt.range_at(0, 50, self.overscan);
        self.visible_range = visible.clone();
        self.rendered_range = rendered.clone();
        if let Some(on_visible) = &props.on_visible_range {
            (on_visible)(visible.clone());
        }
        self.request_range(visible, props.on_request_range.as_ref());

        children.replace(self.rendered_bps(&rendered));
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        self.rev.read();

        let orientation = props.orientation.get();
        let gap = props.gap.get();
        let scroll_margin = props.scroll_margin.get();
        let fit = props.fit.get();
        let overscan = props.overscan.get();
        let prefetch = props.prefetch.get();
        let wrap = props.wrap.get();

        self.data = props.data.clone();
        let count = self.data.count();
        let mut structure_changed = self.state.count != count;
        self.state.count = count;
        self.virt.set_count(count);

        if self.orientation != orientation || self.gap != gap {
            self.orientation = orientation;
            self.gap = gap;
            self.virt.set_gap(u32::from(gap));
            structure_changed = true;
        }

        self.scroll_margin = scroll_margin;
        self.fit = fit;
        self.overscan = overscan;
        self.prefetch = prefetch;
        self.wrap = wrap;

        if let Some(selection) = &props.selection {
            let ext_selected = match selection {
                ListSelection::Optional(sig) => sig.read(),
                ListSelection::Exact(sig) => Some(sig.read()),
            };
            if self.selection != Some(ext_selected) {
                self.state.selected = ext_selected;
                self.selection = Some(ext_selected);
                if let Some(idx) = ext_selected {
                    self.adjust_scroll(idx);
                }
                structure_changed = true;
            }
        }

        let viewport = self.state.viewport_size;
        let (visible_range, rendered_range) =
            self.virt.range_at(self.state.scroll_offset, viewport, self.overscan);

        let range_changed = self.rendered_range != rendered_range;
        self.visible_range = visible_range.clone();
        self.rendered_range = rendered_range.clone();
        if let Some(on_visible) = &props.on_visible_range {
            (on_visible)(visible_range.clone());
        }
        self.request_range(visible_range, props.on_request_range.as_ref());

        if structure_changed || range_changed {
            Update::children(self.rendered_bps(&rendered_range))
        } else {
            Update::LAYOUT
        }
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        available: Size,
        children: &mut MeasureCx,
    ) -> Size {
        self.virt.set_count(self.state.count);
        self.virt.set_gap(u32::from(self.gap));

        let mut max_cross: u16 = 0;
        for (local_idx, global_idx) in self.rendered_range.clone().enumerate() {
            if local_idx >= children.len() {
                break;
            }
            let child_available = match self.orientation {
                Orientation::Vertical => Size::new(available.width, u16::MAX),
                Orientation::Horizontal => Size::new(u16::MAX, available.height),
            };
            let child_size = children.measure(local_idx, child_available);
            let (main_size, cross_size) = match self.orientation {
                Orientation::Vertical => (child_size.height, child_size.width),
                Orientation::Horizontal => (child_size.width, child_size.height),
            };
            self.virt.observe(global_idx, u32::from(main_size));
            max_cross = max_cross.max(cross_size);
        }

        let total_main = self.virt.total_extent();
        self.state.content_size = total_main;

        match self.orientation {
            Orientation::Vertical => Size::new(
                max_cross.min(available.width),
                total_main.min(u32::from(available.height)) as u16,
            ),
            Orientation::Horizontal => Size::new(
                total_main.min(u32::from(available.width)) as u16,
                max_cross.min(available.height),
            ),
        }
    }

    fn layout(&mut self, _cx: &mut Cx, props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        let (viewport_size, cross_size) = match self.orientation {
            Orientation::Vertical => (u32::from(area.height), area.width),
            Orientation::Horizontal => (u32::from(area.width), area.height),
        };
        self.state.viewport_size = viewport_size;

        let count = self.state.count;
        let mut offsets = Vec::with_capacity(count + 1);
        let mut extents = Vec::with_capacity(count);
        for i in 0..count {
            offsets.push(self.virt.offset_of(i));
            extents.push(self.virt.extent_of(i));
        }
        let total = self.virt.total_extent();
        offsets.push(total);
        self.state.content_size = total;
        self.state.offsets = offsets.into();
        self.state.extents = extents.into();

        let max_scroll = total.saturating_sub(viewport_size);
        self.state.scroll_offset = self.state.scroll_offset.min(max_scroll);

        let scroll_offset = self.state.scroll_offset;
        let rendered_start = self.rendered_range.start;

        for (local_idx, global_idx) in self.rendered_range.clone().enumerate() {
            if local_idx >= children.len() {
                break;
            }
            let item_offset = self.state.offsets.get(global_idx).copied().unwrap_or(0);
            let item_size = self.state.extents.get(global_idx).copied().unwrap_or(0);

            match self.orientation {
                Orientation::Vertical => {
                    let visible_start = i64::from(item_offset) - i64::from(scroll_offset);
                    let visible_end = visible_start + i64::from(item_size);
                    let hidden = self.fit == ListFit::Whole
                        && (visible_start < 0 || visible_end > i64::from(viewport_size));
                    let offset = visible_start.min(0).max(i64::from(i32::MIN)) as i32;
                    let position = visible_start.max(0).min(i64::from(u16::MAX)) as u16;
                    children.set(
                        local_idx,
                        Rect::new(
                            area.x,
                            area.y.saturating_add(position),
                            cross_size,
                            if hidden { 0 } else { item_size.min(u32::from(u16::MAX)) as u16 },
                        ),
                    );
                    children.translate(local_idx, Offset::new(0, offset));
                }
                Orientation::Horizontal => {
                    let visible_start = i64::from(item_offset) - i64::from(scroll_offset);
                    let visible_end = visible_start + i64::from(item_size);
                    let hidden = self.fit == ListFit::Whole
                        && (visible_start < 0 || visible_end > i64::from(viewport_size));
                    let offset = visible_start.min(0).max(i64::from(i32::MIN)) as i32;
                    let position = visible_start.max(0).min(i64::from(u16::MAX)) as u16;
                    children.set(
                        local_idx,
                        Rect::new(
                            area.x.saturating_add(position),
                            area.y,
                            if hidden { 0 } else { item_size.min(u32::from(u16::MAX)) as u16 },
                            cross_size,
                        ),
                    );
                    children.translate(local_idx, Offset::new(offset, 0));
                }
            }
            let _ = rendered_start;
        }

        let (visible_range, _) = self.virt.range_at(scroll_offset, viewport_size, self.overscan);
        if self.visible_range != visible_range {
            self.visible_range = visible_range.clone();
            if let Some(on_visible) = &props.on_visible_range {
                (on_visible)(visible_range.clone());
            }
        }
        self.request_range(visible_range, props.on_request_range.as_ref());
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
                self.rev.set(self.rev.peek().wrapping_add(1));
                if matches!(event, Event::Mouse(_)) {
                    cx.focus(true);
                }
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
