pub mod builder;
pub use builder::LazyListBuilder;

use std::{ops::Range, rc::Rc};

use kursor_core::{
    component::{
        Component, Focus, Update,
        behavior::{Behavior, BehaviorCx},
        blueprint::Blueprint,
        context::Cx,
    },
    event::{
        Event, EventResult, Phase,
        key::KeyCode,
        mouse::{MouseButton, MouseKind},
    },
    layout::{
        Orientation,
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    state::{IntoValue, Value},
};

use super::list::{ListIntent, ListSelection};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LazyListState {
    pub selected: Option<usize>,
    pub count: usize,
    pub scroll_offset: u32,
    pub viewport_size: u32,
    pub content_size: u32,
    pub visible_range: Range<usize>,
    pub rendered_range: Range<usize>,
    pub item_extent: u16,
    pub gap: u16,
}

impl LazyListState {
    pub fn item_at(&self, position: u32) -> Option<usize> {
        let stride = u32::from(self.item_extent.saturating_add(self.gap)).max(1);
        if stride == 0 || self.count == 0 {
            return None;
        }
        let index = (position / stride) as usize;
        if index < self.count {
            let offset_in_item = position % stride;
            if offset_in_item < u32::from(self.item_extent) {
                Some(index)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct LazyListBehavior {
    pub wrap: bool,
    pub page_step: usize,
    pub wheel_step: u16,
}

impl Default for LazyListBehavior {
    fn default() -> Self {
        Self {
            wrap: false,
            page_step: 10,
            wheel_step: 3,
        }
    }
}

impl Behavior for LazyListBehavior {
    type State = LazyListState;
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
                MouseKind::Down(MouseButton::Left) | MouseKind::Click(MouseButton::Left) => {
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

#[derive(Clone)]
pub struct LazyListProps {
    pub count: Value<usize>,
    pub item_extent: Value<u16>,
    pub orientation: Value<Orientation>,
    pub gap: Value<u16>,
    pub scroll_margin: Value<u16>,
    pub overscan: Value<usize>,
    pub prefetch: Value<usize>,
    pub wrap: Value<bool>,
    pub selection: Option<ListSelection>,
    pub behavior: Rc<dyn Behavior<State = LazyListState, Intent = ListIntent>>,
    pub item_builder: Rc<dyn Fn(usize) -> Blueprint>,
    pub on_select: Option<Rc<dyn Fn(&mut Cx, usize)>>,
    pub on_activate: Option<Rc<dyn Fn(&mut Cx, usize)>>,
    pub on_visible_range: Option<Rc<dyn Fn(Range<usize>)>>,
    pub on_request_range: Option<Rc<dyn Fn(Range<usize>)>>,
}

pub struct LazyList {
    state: LazyListState,
    count: usize,
    item_extent: u16,
    orientation: Orientation,
    gap: u16,
    scroll_margin: u16,
    overscan: usize,
    prefetch: usize,
    wrap: bool,
    item_builder: Rc<dyn Fn(usize) -> Blueprint>,
    requested_range: Option<Range<usize>>,
    selection: Option<Option<usize>>,
}

impl LazyList {
    pub fn builder(
        count: impl IntoValue<usize>,
        item_builder: impl Fn(usize) -> Blueprint + 'static,
    ) -> LazyListBuilder {
        LazyListBuilder::new(count, item_builder)
    }

    pub fn new(
        count: impl IntoValue<usize>,
        item_builder: impl Fn(usize) -> Blueprint + 'static,
    ) -> Blueprint {
        Self::builder(count, item_builder).build()
    }

    pub fn with(props: LazyListProps) -> Blueprint {
        Blueprint::new::<Self>(props)
    }

    fn ranges(&self, viewport_size: u32) -> (Range<usize>, Range<usize>) {
        if self.count == 0 || self.item_extent == 0 {
            return (0..0, 0..0);
        }

        let stride = self.item_extent.saturating_add(self.gap).max(1) as usize;
        let offset = self.state.scroll_offset as usize;
        let viewport = viewport_size as usize;

        let visible_start = (offset / stride).min(self.count);
        let visible_count = if stride > 0 {
            (viewport + stride.saturating_sub(1)) / stride + 1
        } else {
            0
        };
        let visible_end = (visible_start + visible_count).min(self.count);

        let rendered_start = visible_start.saturating_sub(self.overscan);
        let rendered_end = (visible_end + self.overscan).min(self.count);

        (visible_start..visible_end, rendered_start..rendered_end)
    }

    fn rendered_bps(&self, range: &Range<usize>) -> Vec<Blueprint> {
        range
            .clone()
            .map(|idx| (self.item_builder)(idx).key(idx as u64))
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

        let desired = visible.start.saturating_sub(self.prefetch)
            ..(visible.end + self.prefetch).min(self.count);
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
                start = desired.start.saturating_sub(self.prefetch);
                callback(start..previous.start);
            }
            if desired.end > previous.end {
                end = (desired.end + self.prefetch).min(self.count);
                callback(previous.end..end);
            }

            self.requested_range = Some(start..end);
        }
    }

    fn adjust_scroll(&mut self, index: usize) {
        if index >= self.count || self.item_extent == 0 {
            return;
        }

        let stride = self.item_extent.saturating_add(self.gap);
        let item_start = (index as u32).saturating_mul(u32::from(stride));
        let item_end = item_start.saturating_add(u32::from(self.item_extent));
        let margin = u32::from(self.scroll_margin).saturating_mul(u32::from(stride));
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

    fn set_selected(&mut self, cx: &mut Cx, props: &LazyListProps, index: Option<usize>) -> bool {
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

    fn apply(&mut self, cx: &mut Cx, props: &LazyListProps, intent: ListIntent) -> bool {
        let count = self.count;
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
                let max_scroll = self
                    .state
                    .content_size
                    .saturating_sub(self.state.viewport_size);
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

impl Component for LazyList {
    type Props = LazyListProps;

    fn create(_cx: &mut Cx, props: &Self::Props) -> Self {
        let init_sel = props.selection.as_ref().and_then(|sel| match sel {
            ListSelection::Optional(sig) => sig.peek(),
            ListSelection::Exact(sig) => Some(sig.peek()),
        });
        let ext_sel = props.selection.as_ref().map(|sel| match sel {
            ListSelection::Optional(signal) => signal.peek(),
            ListSelection::Exact(signal) => Some(signal.peek()),
        });

        Self {
            state: LazyListState {
                selected: init_sel,
                ..Default::default()
            },
            count: 0,
            item_extent: 1,
            orientation: Orientation::Vertical,
            gap: 0,
            scroll_margin: 0,
            overscan: 3,
            prefetch: 10,
            wrap: false,
            item_builder: props.item_builder.clone(),
            requested_range: None,
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
        old.count != new.count
            || old.item_extent != new.item_extent
            || old.orientation != new.orientation
            || old.gap != new.gap
            || old.scroll_margin != new.scroll_margin
            || old.overscan != new.overscan
            || old.prefetch != new.prefetch
            || old.wrap != new.wrap
            || old.selection != new.selection
            || !Rc::ptr_eq(&old.behavior, &new.behavior)
            || !Rc::ptr_eq(&old.item_builder, &new.item_builder)
            || old.on_select.as_ref().map(Rc::as_ptr) != new.on_select.as_ref().map(Rc::as_ptr)
            || old.on_activate.as_ref().map(Rc::as_ptr) != new.on_activate.as_ref().map(Rc::as_ptr)
            || old.on_visible_range.as_ref().map(Rc::as_ptr)
                != new.on_visible_range.as_ref().map(Rc::as_ptr)
            || old.on_request_range.as_ref().map(Rc::as_ptr)
                != new.on_request_range.as_ref().map(Rc::as_ptr)
    }

    fn mount(
        &mut self,
        _cx: &mut Cx,
        props: &Self::Props,
        children: &mut kursor_core::component::MountChildren,
    ) {
        self.count = props.count.get();
        self.item_extent = props.item_extent.get().max(1);
        self.orientation = props.orientation.get();
        self.gap = props.gap.get();
        self.scroll_margin = props.scroll_margin.get();
        self.overscan = props.overscan.get();
        self.prefetch = props.prefetch.get();
        self.wrap = props.wrap.get();
        self.item_builder = props.item_builder.clone();

        let (visible_range, rendered_range) = self.ranges(50);
        self.state.visible_range = visible_range.clone();
        self.state.rendered_range = rendered_range.clone();

        if let Some(on_visible) = &props.on_visible_range {
            (on_visible)(visible_range.clone());
        }
        self.request_range(visible_range, props.on_request_range.as_ref());

        children.replace(self.rendered_bps(&rendered_range));
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let count = props.count.get();
        let item_extent = props.item_extent.get().max(1);
        let orientation = props.orientation.get();
        let gap = props.gap.get();
        let scroll_margin = props.scroll_margin.get();
        let overscan = props.overscan.get();
        let prefetch = props.prefetch.get();
        let wrap = props.wrap.get();

        self.item_builder = props.item_builder.clone();

        let mut structure_changed = false;
        let mut measure_needed = false;
        let previous_scroll = self.state.scroll_offset;
        let previous_margin = self.scroll_margin;
        let previous_wrap = self.wrap;

        if self.count != count
            || self.item_extent != item_extent
            || self.orientation != orientation
            || self.gap != gap
        {
            self.count = count;
            self.requested_range = None;
            self.item_extent = item_extent;
            self.orientation = orientation;
            self.gap = gap;
            measure_needed = true;
            structure_changed = true;
        }

        self.scroll_margin = scroll_margin;
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
        if viewport == 0 {
            return Update::NONE;
        }
        let (visible_range, rendered_range) = self.ranges(viewport);

        let scroll_changed = self.state.scroll_offset != previous_scroll
            || self.scroll_margin != previous_margin
            || self.wrap != previous_wrap;
        let range_changed = self.state.rendered_range != rendered_range;

        if range_changed || structure_changed {
            self.state.visible_range = visible_range.clone();
            self.state.rendered_range = rendered_range.clone();

            if let Some(on_visible) = &props.on_visible_range {
                (on_visible)(visible_range.clone());
            }
            self.request_range(visible_range.clone(), props.on_request_range.as_ref());

            Update::children(self.rendered_bps(&rendered_range))
        } else if measure_needed {
            Update::MEASURE
        } else if scroll_changed {
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
        self.state.count = self.count;
        self.state.item_extent = self.item_extent;
        self.state.gap = self.gap;

        let stride = usize::from(self.item_extent).saturating_add(usize::from(self.gap));
        let total_main = if self.count > 0 {
            (self.count as u32)
                .saturating_mul(stride as u32)
                .saturating_sub(u32::from(self.gap))
        } else {
            0
        };
        self.state.content_size = total_main;

        for index in 0..children.len() {
            let child_available = match self.orientation {
                Orientation::Vertical => Size::new(available.width, self.item_extent),
                Orientation::Horizontal => Size::new(self.item_extent, available.height),
            };
            children.measure(index, child_available);
        }

        match self.orientation {
            Orientation::Vertical => Size::new(
                available.width,
                total_main.min(u32::from(available.height)) as u16,
            ),
            Orientation::Horizontal => Size::new(
                total_main.min(u32::from(available.width)) as u16,
                available.height,
            ),
        }
    }

    fn layout(&mut self, _cx: &mut Cx, props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        let (viewport_size, cross_size) = match self.orientation {
            Orientation::Vertical => (u32::from(area.height), area.width),
            Orientation::Horizontal => (u32::from(area.width), area.height),
        };
        self.state.viewport_size = viewport_size;

        let stride = self.item_extent.saturating_add(self.gap);
        let rendered_start = self.state.rendered_range.start;
        let scroll_offset = self.state.scroll_offset;

        for (local_idx, _) in (0..children.len()).enumerate() {
            let global_idx = rendered_start + local_idx;
            let item_pos = (global_idx as u32).saturating_mul(u32::from(stride));
            let visible_pos = i64::from(item_pos) - i64::from(scroll_offset);
            let offset = visible_pos.min(0).max(i64::from(i32::MIN)) as i32;
            let position = visible_pos.max(0).min(i64::from(u16::MAX)) as u16;

            match self.orientation {
                Orientation::Vertical => {
                    let y = area.y.saturating_add(position);
                    children.set(
                        local_idx,
                        Rect::new(area.x, y, cross_size, self.item_extent),
                    );
                    children.translate(local_idx, kursor_core::layout::Offset::new(0, offset));
                }
                Orientation::Horizontal => {
                    let x = area.x.saturating_add(position);
                    children.set(
                        local_idx,
                        Rect::new(x, area.y, self.item_extent, cross_size),
                    );
                    children.translate(local_idx, kursor_core::layout::Offset::new(offset, 0));
                }
            }
        }

        let (visible_range, _) = self.ranges(self.state.viewport_size);
        if self.state.visible_range != visible_range {
            self.state.visible_range = visible_range.clone();
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
