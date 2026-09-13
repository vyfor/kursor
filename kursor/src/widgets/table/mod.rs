pub mod behavior;
pub mod builder;
pub mod column;
pub mod row;

pub use behavior::*;
pub use builder::*;
pub use column::*;
pub use row::*;

use std::{ops::Range, rc::Rc};

use kursor_core::{
    component::{
        Component, Focus, MountChildren, Update,
        behavior::{Behavior, BehaviorCx},
        blueprint::Blueprint,
        context::Cx,
        key::Key,
    },
    event::{Event, EventResult, Phase},
    layout::{
        Alignment, Offset,
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    render::{canvas::Canvas, color::Color, style::Style},
    state::{Signal, Value},
    theme::Theme,
};

use crate::layout::{ResolvedTrack, Virtualizer, allocate_tracks};
use crate::widgets::{
    align::Align, block::Border, spacer::Spacer, styled::Styled,
};

#[derive(Clone)]
pub struct TableProps {
    pub mode: Value<TMode>,
    pub columns: Rc<[TColumn]>,
    pub data: TData,
    pub border: Value<Border>,
    pub header_border: Option<Value<Border>>,
    pub row_height: Value<u16>,
    pub header_height: Value<u16>,
    pub column_gap: Value<u16>,
    pub row_gap: Value<u16>,
    pub header_divider: Value<bool>,
    pub column_divider: Value<bool>,
    pub row_divider: Value<bool>,
    pub scroll_margin: Value<u16>,
    pub overscan: Value<usize>,
    pub wrap: Value<bool>,
    pub selection: Option<TSelection>,
    pub header_style: Option<Value<Style>>,
    pub row_style: Option<Value<Style>>,
    pub alternate_row_style: Option<Value<Style>>,
    pub selected_style: Option<Value<Style>>,
    pub divider_style: Option<Value<Style>>,
    pub behavior: Rc<dyn Behavior<State = TableState, Intent = TableIntent>>,
    pub on_activate: Option<Rc<dyn Fn(&mut Cx, TableTarget)>>,
    pub on_select: Option<Rc<dyn Fn(&mut Cx, TableTarget)>>,
    pub on_header_click: Option<Rc<dyn Fn(&mut Cx, usize)>>,
}

impl Default for TableProps {
    fn default() -> Self {
        Self {
            mode: Value::plain(TMode::Row),
            columns: Rc::from([]),
            data: TData::empty(),
            border: Value::plain(Border::Plain),
            header_border: None,
            row_height: Value::plain(1),
            header_height: Value::plain(1),
            column_gap: Value::plain(1),
            row_gap: Value::plain(0),
            header_divider: Value::plain(true),
            column_divider: Value::plain(false),
            row_divider: Value::plain(false),
            scroll_margin: Value::plain(0),
            overscan: Value::plain(2),
            wrap: Value::plain(false),
            selection: None,
            header_style: None,
            row_style: None,
            alternate_row_style: None,
            selected_style: None,
            divider_style: None,
            behavior: Rc::new(TableBehavior::default()),
            on_activate: None,
            on_select: None,
            on_header_click: None,
        }
    }
}

pub struct Table {
    state: TableState,
    virt: Virtualizer,
    mode: TMode,
    declared_mode: TMode,
    columns: Rc<[TColumn]>,
    column_tracks: Vec<ResolvedTrack>,
    data: TData,
    border: Border,
    header_border: Option<Border>,
    row_height: u16,
    header_height: u16,
    column_gap: u16,
    row_gap: u16,
    header_divider: bool,
    column_divider: bool,
    row_divider: bool,
    scroll_margin: u16,
    overscan: usize,
    wrap: bool,
    selection_cache: Option<TableTarget>,
    refresh_children: bool,
    rev: Signal<u64>,
}

impl Table {
    pub fn new(
        columns: impl IntoTColumns,
        rows: impl IntoIterator<Item = impl IntoTRow>,
    ) -> TableBuilder {
        TableBuilder::new().columns(columns).rows(rows)
    }

    pub fn with(props: TableProps) -> Blueprint {
        Blueprint::new::<Self>(props)
    }

    pub fn has_header(&self) -> bool {
        self.columns.iter().any(|c| c.header.is_some())
    }

    fn inset(&self) -> u16 {
        match self.border {
            Border::None => 0,
            _ => 1,
        }
    }

    fn top_height(&self) -> u16 {
        if self.has_header() {
            self.header_height + if self.header_divider { 1 } else { 0 }
        } else {
            0
        }
    }

    fn resolve(columns: &[TColumn]) -> Vec<ResolvedTrack> {
        columns.iter().map(|col| col.track.resolve()).collect()
    }

    fn ranges(&mut self, viewport_height: u32) -> (Range<usize>, Range<usize>) {
        self.virt
            .range_at(self.state.scroll_y, viewport_height, self.overscan)
    }

    fn children(
        &mut self,
        rendered_range: &Range<usize>,
        props: &TableProps,
        theme: &Theme,
    ) -> Vec<Blueprint> {
        let num_cols = self.columns.len();
        if num_cols == 0 {
            return Vec::new();
        }

        let has_header = self.has_header();
        let header_count = if has_header { num_cols } else { 0 };
        let rendered_rows_count = rendered_range.end - rendered_range.start;
        let mut blueprints =
            Vec::with_capacity(header_count + rendered_rows_count * num_cols);

        if has_header {
            for (c_idx, col) in self.columns.iter().enumerate() {
                let bp = if let Some(header_bp) = &col.header {
                    let mut cell_bp = header_bp.clone().key(c_idx as u64);
                    if col.align != Alignment::TOP_LEFT {
                        cell_bp =
                            Align::new(cell_bp).alignment(col.align).build();
                    }
                    cell_bp
                } else {
                    Spacer::new(0).build().key(c_idx as u64)
                };
                blueprints.push(bp);
            }
        }

        for row_idx in rendered_range.clone() {
            let row_opt = self.data.get_row(row_idx);
            self.virt.observe(
                row_idx,
                u32::from(
                    row_opt
                        .as_ref()
                        .and_then(|row| row.height)
                        .unwrap_or(self.row_height),
                ),
            );
            for c_idx in 0..num_cols {
                let key = if let Some(row) = &row_opt
                    && let Some(row_key) = &row.key
                {
                    match row_key {
                        Key::Integer(k) => Key::Integer(
                            ((k.wrapping_add(1)) << 16) | (c_idx as u64),
                        ),
                        Key::String(s) => {
                            let mut h = 0xcbf29ce484222325_u64;
                            for &b in s.as_bytes() {
                                h = (h ^ (b as u64))
                                    .wrapping_mul(0x100000001b3);
                            }
                            Key::Integer(
                                ((h.wrapping_add(1)) << 16) | (c_idx as u64),
                            )
                        }
                    }
                } else {
                    Key::Integer(
                        (((row_idx as u64).wrapping_add(1)) << 16)
                            | (c_idx as u64),
                    )
                };

                let mut bp = if let Some(row) = &row_opt
                    && let Some(cell) = row.cells.get(c_idx)
                {
                    let mut cell_bp = cell.clone().key(key.clone());
                    let align = self.columns[c_idx].align;
                    if align != Alignment::TOP_LEFT {
                        cell_bp = Align::new(cell_bp).alignment(align).build();
                    }
                    cell_bp
                } else {
                    Spacer::new(0).build().key(key.clone())
                };

                let is_selected = self.state.is_cell_selected(row_idx, c_idx);
                if is_selected {
                    let fg = props
                        .selected_style
                        .as_ref()
                        .map(|s| s.get().fg)
                        .filter(|&fg| fg != Color::Unset)
                        .unwrap_or(theme.active.fg);
                    let sel_style = Style {
                        fg,
                        bg: Color::Unset,
                        ..Style::DEFAULT
                    };
                    bp = Styled::with(sel_style, bp).key(key);
                }
                blueprints.push(bp);
            }
        }

        blueprints
    }

    fn adjust_scroll_y(&mut self, row_idx: usize) {
        if self.state.row_count == 0 || row_idx >= self.state.row_count {
            return;
        }
        let item_start = self.virt.offset_of(row_idx);
        let item_end = item_start + self.virt.extent_of(row_idx);
        let margin = u32::from(self.scroll_margin) * self.virt.estimate();
        let viewport = self.state.viewport_height;
        let max_scroll = self.state.content_height.saturating_sub(viewport);

        let target_min = item_start.saturating_sub(margin);
        let target_max = item_end + margin;

        if target_min < self.state.scroll_y {
            self.state.scroll_y = target_min;
        } else if target_max > self.state.scroll_y + viewport {
            self.state.scroll_y = target_max - viewport;
        }

        let snapped = self.virt.item_nearest(self.state.scroll_y);
        self.state.scroll_y = self.virt.offset_of(snapped).min(max_scroll);
    }

    fn adjust_scroll_x(&mut self, col_idx: usize) {
        if col_idx >= self.state.column_positions.len()
            || col_idx >= self.state.column_widths.len()
        {
            return;
        }

        let start = (self.state.column_positions[col_idx]
            + self.state.scroll_x as i32)
            .max(0) as u32;
        let end = start + u32::from(self.state.column_widths[col_idx]);
        let viewport = self.state.viewport_width;
        let max_scroll = self.state.content_width.saturating_sub(viewport);

        if start < self.state.scroll_x {
            self.state.scroll_x = start;
        } else if end > self.state.scroll_x + viewport {
            self.state.scroll_x = end - viewport;
        }

        self.state.scroll_x = self.state.scroll_x.min(max_scroll);
    }

    fn set_target(
        &mut self,
        cx: &mut Cx,
        props: &TableProps,
        target: Option<TableTarget>,
    ) -> bool {
        let prev = self.selection_cache;
        self.selection_cache = target;

        match target {
            Some(TableTarget::Row(r)) => {
                self.state.selected_row = Some(r);
                if let Some(TSelection::Row(sig)) = &props.selection {
                    sig.set(Some(r));
                } else if let Some(TSelection::ExactRow(sig)) = &props.selection
                {
                    sig.set(r);
                }
                self.adjust_scroll_y(r);
                if let Some(on_select) = &props.on_select {
                    (on_select)(cx, TableTarget::Row(r));
                }
            }
            Some(TableTarget::Cell(r, c)) => {
                self.state.selected_row = Some(r);
                self.state.selected_col = Some(c);
                if let Some(TSelection::Cell(sig)) = &props.selection {
                    sig.set(Some((r, c)));
                } else if let Some(TSelection::Row(sig)) = &props.selection {
                    sig.set(Some(r));
                } else if let Some(TSelection::ExactRow(sig)) = &props.selection
                {
                    sig.set(r);
                }
                self.adjust_scroll_y(r);
                self.adjust_scroll_x(c);
                if let Some(on_select) = &props.on_select {
                    (on_select)(cx, TableTarget::Cell(r, c));
                }
            }
            Some(TableTarget::Column(c)) => {
                self.state.selected_col = Some(c);
                if let Some(TSelection::Column(sig)) = &props.selection {
                    sig.set(Some(c));
                }
                self.adjust_scroll_x(c);
                if let Some(on_select) = &props.on_select {
                    (on_select)(cx, TableTarget::Column(c));
                }
            }
            None => {
                self.state.selected_row = None;
                self.state.selected_col = None;
                if let Some(TSelection::Row(sig)) = &props.selection {
                    sig.set(None);
                } else if let Some(TSelection::Cell(sig)) = &props.selection {
                    sig.set(None);
                } else if let Some(TSelection::Column(sig)) = &props.selection {
                    sig.set(None);
                }
            }
        }

        prev != target
    }

    fn apply(
        &mut self,
        cx: &mut Cx,
        props: &TableProps,
        intent: TableIntent,
    ) -> bool {
        let row_count = self.state.row_count;
        let col_count = self.columns.len();
        if row_count == 0 && col_count == 0 {
            return false;
        }

        match intent {
            TableIntent::Up => match self.mode {
                TMode::Row | TMode::Cell => {
                    if row_count == 0 {
                        return false;
                    }
                    let curr_r = self.state.selected_row.unwrap_or(0);
                    let next_r = if curr_r == 0 {
                        if self.wrap { row_count - 1 } else { 0 }
                    } else {
                        curr_r - 1
                    };
                    if self.mode == TMode::Cell {
                        let c = self.state.selected_col.unwrap_or(0);
                        self.set_target(
                            cx,
                            props,
                            Some(TableTarget::Cell(next_r, c)),
                        )
                    } else {
                        self.set_target(
                            cx,
                            props,
                            Some(TableTarget::Row(next_r)),
                        )
                    }
                }
                TMode::Column => {
                    self.apply(cx, props, TableIntent::ScrollY(-1))
                }
            },
            TableIntent::Down => match self.mode {
                TMode::Row | TMode::Cell => {
                    if row_count == 0 {
                        return false;
                    }
                    let curr_r = self.state.selected_row.unwrap_or(0);
                    let next_r = if curr_r + 1 >= row_count {
                        if self.wrap { 0 } else { row_count - 1 }
                    } else {
                        curr_r + 1
                    };
                    if self.mode == TMode::Cell {
                        let c = self.state.selected_col.unwrap_or(0);
                        self.set_target(
                            cx,
                            props,
                            Some(TableTarget::Cell(next_r, c)),
                        )
                    } else {
                        self.set_target(
                            cx,
                            props,
                            Some(TableTarget::Row(next_r)),
                        )
                    }
                }
                TMode::Column => self.apply(cx, props, TableIntent::ScrollY(1)),
            },
            TableIntent::Left => match self.mode {
                TMode::Cell => {
                    if col_count == 0 {
                        return false;
                    }
                    let r = self.state.selected_row.unwrap_or(0);
                    let curr_c = self.state.selected_col.unwrap_or(0);
                    let next_c = if curr_c == 0 {
                        if self.wrap { col_count - 1 } else { 0 }
                    } else {
                        curr_c - 1
                    };
                    self.set_target(
                        cx,
                        props,
                        Some(TableTarget::Cell(r, next_c)),
                    )
                }
                TMode::Column => {
                    if col_count == 0 {
                        return false;
                    }
                    let curr_c = self.state.selected_col.unwrap_or(0);
                    let next_c = if curr_c == 0 {
                        if self.wrap { col_count - 1 } else { 0 }
                    } else {
                        curr_c - 1
                    };
                    self.set_target(
                        cx,
                        props,
                        Some(TableTarget::Column(next_c)),
                    )
                }
                TMode::Row => self.apply(cx, props, TableIntent::ScrollX(-8)),
            },
            TableIntent::Right => match self.mode {
                TMode::Cell => {
                    if col_count == 0 {
                        return false;
                    }
                    let r = self.state.selected_row.unwrap_or(0);
                    let curr_c = self.state.selected_col.unwrap_or(0);
                    let next_c = if curr_c + 1 >= col_count {
                        if self.wrap { 0 } else { col_count - 1 }
                    } else {
                        curr_c + 1
                    };
                    self.set_target(
                        cx,
                        props,
                        Some(TableTarget::Cell(r, next_c)),
                    )
                }
                TMode::Column => {
                    if col_count == 0 {
                        return false;
                    }
                    let curr_c = self.state.selected_col.unwrap_or(0);
                    let next_c = if curr_c + 1 >= col_count {
                        if self.wrap { 0 } else { col_count - 1 }
                    } else {
                        curr_c + 1
                    };
                    self.set_target(
                        cx,
                        props,
                        Some(TableTarget::Column(next_c)),
                    )
                }
                TMode::Row => self.apply(cx, props, TableIntent::ScrollX(8)),
            },
            TableIntent::Home => {
                if row_count == 0 {
                    return false;
                }
                if self.mode == TMode::Cell {
                    self.set_target(cx, props, Some(TableTarget::Cell(0, 0)))
                } else if self.mode == TMode::Column {
                    self.set_target(cx, props, Some(TableTarget::Column(0)))
                } else {
                    self.set_target(cx, props, Some(TableTarget::Row(0)))
                }
            }
            TableIntent::End => {
                if row_count == 0 {
                    return false;
                }
                let last_r = row_count - 1;
                let last_c = col_count - 1;
                if self.mode == TMode::Cell {
                    self.set_target(
                        cx,
                        props,
                        Some(TableTarget::Cell(last_r, last_c)),
                    )
                } else if self.mode == TMode::Column {
                    self.set_target(
                        cx,
                        props,
                        Some(TableTarget::Column(last_c)),
                    )
                } else {
                    self.set_target(cx, props, Some(TableTarget::Row(last_r)))
                }
            }
            TableIntent::PageUp(step) => {
                if self.mode == TMode::Column {
                    return self.apply(
                        cx,
                        props,
                        TableIntent::ScrollY(-(step as i32)),
                    );
                }
                if row_count == 0 {
                    return false;
                }
                let curr_r = self.state.selected_row.unwrap_or(0);
                let next_r = curr_r.saturating_sub(step);
                if self.mode == TMode::Cell {
                    let c = self.state.selected_col.unwrap_or(0);
                    self.set_target(
                        cx,
                        props,
                        Some(TableTarget::Cell(next_r, c)),
                    )
                } else {
                    self.set_target(cx, props, Some(TableTarget::Row(next_r)))
                }
            }
            TableIntent::PageDown(step) => {
                if self.mode == TMode::Column {
                    return self.apply(
                        cx,
                        props,
                        TableIntent::ScrollY(step as i32),
                    );
                }
                if row_count == 0 {
                    return false;
                }
                let curr_r = self.state.selected_row.unwrap_or(0);
                let next_r = (curr_r + step).min(row_count - 1);
                if self.mode == TMode::Cell {
                    let c = self.state.selected_col.unwrap_or(0);
                    self.set_target(
                        cx,
                        props,
                        Some(TableTarget::Cell(next_r, c)),
                    )
                } else {
                    self.set_target(cx, props, Some(TableTarget::Row(next_r)))
                }
            }
            TableIntent::SelectRow(r) => {
                let clamped = r.min(row_count - 1);
                self.set_target(cx, props, Some(TableTarget::Row(clamped)))
            }
            TableIntent::SelectCell(r, c) => {
                let clamped_r = r.min(row_count - 1);
                let clamped_c = c.min(col_count - 1);
                self.set_target(
                    cx,
                    props,
                    Some(TableTarget::Cell(clamped_r, clamped_c)),
                )
            }
            TableIntent::SelectColumn(c) => {
                let clamped = c.min(col_count - 1);
                self.set_target(cx, props, Some(TableTarget::Column(clamped)))
            }
            TableIntent::HeaderClick(col_idx) => {
                self.refresh_children = true;
                if self.mode == TMode::Column {
                    let clamped = col_idx.min(col_count - 1);
                    self.set_target(
                        cx,
                        props,
                        Some(TableTarget::Column(clamped)),
                    );
                }
                if let Some(on_header) = &props.on_header_click {
                    (on_header)(cx, col_idx);
                    true
                } else {
                    self.mode == TMode::Column
                }
            }
            TableIntent::ScrollY(delta) => {
                let count = self.state.row_count;
                if count == 0 {
                    return false;
                }
                let first = self.virt.item_nearest(self.state.scroll_y);
                let target = if delta > 0 {
                    (first + delta as usize).min(count - 1)
                } else {
                    first.saturating_sub((-delta) as usize)
                };
                let prev = self.state.scroll_y;
                let max_scroll = self
                    .state
                    .content_height
                    .saturating_sub(self.state.viewport_height);
                self.state.scroll_y =
                    self.virt.offset_of(target).min(max_scroll);
                self.state.scroll_y != prev
            }
            TableIntent::ScrollX(delta) => {
                let max_scroll = self
                    .state
                    .content_width
                    .saturating_sub(self.state.viewport_width);
                let prev = self.state.scroll_x;
                if delta > 0 {
                    self.state.scroll_x = self
                        .state
                        .scroll_x
                        .saturating_add(delta as u32)
                        .min(max_scroll);
                } else {
                    self.state.scroll_x =
                        self.state.scroll_x.saturating_sub((-delta) as u32);
                }
                self.state.scroll_x != prev
            }
            TableIntent::ToggleMode => {
                let next = match self.mode {
                    TMode::Row => TMode::Cell,
                    TMode::Cell => TMode::Column,
                    TMode::Column => TMode::Row,
                };
                if next == self.mode {
                    return false;
                }
                self.mode = next;
                self.state.mode = next;

                let target = match (
                    next,
                    self.state.selected_row,
                    self.state.selected_col,
                ) {
                    (TMode::Row, Some(r), _) => Some(TableTarget::Row(r)),
                    (TMode::Cell, Some(r), c) => {
                        Some(TableTarget::Cell(r, c.unwrap_or(0)))
                    }
                    (TMode::Column, _, c) => {
                        Some(TableTarget::Column(c.unwrap_or(0)))
                    }
                    _ => None,
                };
                self.selection_cache = target;
                if let Some(TableTarget::Cell(_, c))
                | Some(TableTarget::Column(c)) = target
                {
                    self.state.selected_col = Some(c);
                    self.adjust_scroll_x(c);
                }
                true
            }
            TableIntent::Activate => {
                if let Some(target) = self.selection_cache
                    && let Some(on_activate) = &props.on_activate
                {
                    (on_activate)(cx, target);
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl Component for Table {
    type Props = TableProps;

    fn create(_cx: &mut Cx, props: &Self::Props) -> Self {
        let (init_row, init_col, init_target) = match &props.selection {
            Some(TSelection::Row(sig)) => {
                (sig.peek(), None, sig.peek().map(TableTarget::Row))
            }
            Some(TSelection::ExactRow(sig)) => {
                let r = sig.peek();
                (Some(r), None, Some(TableTarget::Row(r)))
            }
            Some(TSelection::Cell(sig)) => match sig.peek() {
                Some((r, c)) => {
                    (Some(r), Some(c), Some(TableTarget::Cell(r, c)))
                }
                None => (None, None, None),
            },
            Some(TSelection::Column(sig)) => {
                (None, sig.peek(), sig.peek().map(TableTarget::Column))
            }
            None => (None, None, None),
        };

        let column_tracks = Self::resolve(&props.columns);

        Self {
            state: TableState {
                mode: TMode::Row,
                selected_row: init_row,
                selected_col: init_col,
                row_count: 0,
                column_count: props.columns.len(),
                ..Default::default()
            },
            virt: Virtualizer::new(1),
            mode: TMode::Row,
            declared_mode: TMode::Row,
            columns: props.columns.clone(),
            column_tracks,
            data: props.data.clone(),
            border: Border::Plain,
            header_border: None,
            row_height: 1,
            header_height: 1,
            column_gap: 1,
            row_gap: 0,
            header_divider: true,
            column_divider: false,
            row_divider: false,
            scroll_margin: 0,
            overscan: 2,
            wrap: false,
            selection_cache: init_target,
            refresh_children: false,
            rev: _cx.signal(0),
        }
    }

    fn focus(&self, _props: &Self::Props) -> Focus {
        Focus {
            focusable: true,
            trap: false,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old.mode != new.mode
            || old.border != new.border
            || old.header_border != new.header_border
            || old.row_height != new.row_height
            || old.header_height != new.header_height
            || old.column_gap != new.column_gap
            || old.row_gap != new.row_gap
            || old.header_divider != new.header_divider
            || old.column_divider != new.column_divider
            || old.row_divider != new.row_divider
            || old.scroll_margin != new.scroll_margin
            || old.overscan != new.overscan
            || old.wrap != new.wrap
            || old.selection != new.selection
            || old.data != new.data
            || old.header_style != new.header_style
            || old.row_style != new.row_style
            || old.alternate_row_style != new.alternate_row_style
            || old.selected_style != new.selected_style
            || old.divider_style != new.divider_style
            || !Rc::ptr_eq(&old.columns, &new.columns)
            || !Rc::ptr_eq(&old.behavior, &new.behavior)
            || old.on_activate.as_ref().map(Rc::as_ptr)
                != new.on_activate.as_ref().map(Rc::as_ptr)
            || old.on_select.as_ref().map(Rc::as_ptr)
                != new.on_select.as_ref().map(Rc::as_ptr)
            || old.on_header_click.as_ref().map(Rc::as_ptr)
                != new.on_header_click.as_ref().map(Rc::as_ptr)
    }

    fn mount(
        &mut self,
        cx: &mut Cx,
        props: &Self::Props,
        children: &mut MountChildren,
    ) {
        self.mode = props.mode.get();
        self.declared_mode = self.mode;
        self.border = props.border.get();
        self.header_border = props.header_border.as_ref().map(|b| b.get());
        self.row_height = props.row_height.get().max(1);
        self.header_height = props.header_height.get().max(1);
        self.column_gap = props.column_gap.get();
        self.row_gap = props.row_gap.get();
        self.header_divider = props.header_divider.get();
        self.column_divider = props.column_divider.get();
        self.row_divider = props.row_divider.get();
        self.scroll_margin = props.scroll_margin.get();
        self.overscan = props.overscan.get();
        self.wrap = props.wrap.get();
        self.columns = props.columns.clone();
        self.column_tracks = Self::resolve(&self.columns);
        self.data = props.data.clone();

        let row_count = self.data.count();
        self.state.mode = self.mode;
        self.state.row_count = row_count;
        self.state.column_count = self.columns.len();
        self.state.top_offset = self.top_height();
        self.virt = Virtualizer::new(u32::from(self.row_height));
        self.virt.set_count(row_count);
        self.virt.set_gap(if self.row_divider {
            1
        } else {
            u32::from(self.row_gap)
        });

        let (visible_range, rendered_range) = self.ranges(50);
        self.state.visible_rows = visible_range;
        self.state.rendered_rows = rendered_range.clone();

        children.replace(self.children(&rendered_range, props, cx.theme()));
    }

    fn update(&mut self, cx: &mut Cx, props: &Self::Props) -> Update {
        self.rev.read();
        let declared_mode = props.mode.get();
        let border = props.border.get();
        let header_border = props.header_border.as_ref().map(|b| b.get());
        let row_height = props.row_height.get().max(1);
        let header_height = props.header_height.get().max(1);
        let column_gap = props.column_gap.get();
        let row_gap = props.row_gap.get();
        let header_divider = props.header_divider.get();
        let column_divider = props.column_divider.get();
        let row_divider = props.row_divider.get();
        let scroll_margin = props.scroll_margin.get();
        let overscan = props.overscan.get();
        let wrap = props.wrap.get();

        let columns_changed = !Rc::ptr_eq(&self.columns, &props.columns);
        self.columns = props.columns.clone();
        self.column_tracks = Self::resolve(&self.columns);
        let data_changed = self.data != props.data;
        self.data = props.data.clone();

        let row_count = self.data.count();
        let mut structure_changed = columns_changed
            || data_changed
            || self.state.row_count != row_count;
        self.state.row_count = row_count;
        self.state.column_count = self.columns.len();
        self.virt.set_count(row_count);

        if declared_mode != self.declared_mode {
            self.declared_mode = declared_mode;
            self.mode = declared_mode;
            structure_changed = true;
        }
        self.state.mode = self.mode;

        let mut measure_needed = false;
        let mut paint_needed = false;
        if self.border != border || self.header_border != header_border {
            self.border = border;
            self.header_border = header_border;
            paint_needed = true;
        }

        if self.row_height != row_height
            || self.header_height != header_height
            || self.column_gap != column_gap
            || self.row_gap != row_gap
            || self.header_divider != header_divider
            || self.column_divider != column_divider
            || self.row_divider != row_divider
            || columns_changed
        {
            self.row_height = row_height;
            self.header_height = header_height;
            self.column_gap = column_gap;
            self.row_gap = row_gap;
            self.header_divider = header_divider;
            self.column_divider = column_divider;
            self.row_divider = row_divider;
            measure_needed = true;
            structure_changed = true;
        }

        self.scroll_margin = scroll_margin;
        self.overscan = overscan;
        self.wrap = wrap;
        self.virt.set_gap(if self.row_divider {
            1
        } else {
            u32::from(self.row_gap)
        });
        self.state.top_offset = self.top_height();

        if let Some(selection) = &props.selection {
            let ext_target = match selection {
                TSelection::Row(sig) => sig.read().map(TableTarget::Row),
                TSelection::ExactRow(sig) => Some(TableTarget::Row(sig.read())),
                TSelection::Cell(sig) => {
                    sig.read().map(|(r, c)| TableTarget::Cell(r, c))
                }
                TSelection::Column(sig) => sig.read().map(TableTarget::Column),
            };
            if self.selection_cache != ext_target {
                self.selection_cache = ext_target;
                match ext_target {
                    Some(TableTarget::Row(r)) => {
                        self.state.selected_row = Some(r);
                        self.adjust_scroll_y(r);
                    }
                    Some(TableTarget::Cell(r, c)) => {
                        self.state.selected_row = Some(r);
                        self.state.selected_col = Some(c);
                        self.adjust_scroll_y(r);
                        self.adjust_scroll_x(c);
                    }
                    Some(TableTarget::Column(c)) => {
                        self.state.selected_col = Some(c);
                        self.adjust_scroll_x(c);
                    }
                    None => {
                        self.state.selected_row = None;
                        self.state.selected_col = None;
                    }
                }
                structure_changed = true;
            }
        }

        let viewport = self.state.viewport_height;
        let (visible_range, rendered_range) =
            self.ranges(if viewport > 0 { viewport } else { 50 });

        let range_changed = self.state.rendered_rows != rendered_range;
        let needs_children =
            structure_changed || range_changed || self.refresh_children;
        self.refresh_children = false;

        self.state.visible_rows = visible_range;
        self.state.rendered_rows = rendered_range.clone();

        if needs_children {
            Update::children(self.children(&rendered_range, props, cx.theme()))
        } else if measure_needed {
            Update::MEASURE
        } else if paint_needed {
            Update::PAINT
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
        let num_cols = self.columns.len();
        if num_cols == 0 {
            return Size::default();
        }

        let inset = self.inset();
        let avail_w = available.width.saturating_sub(inset * 2);

        let has_content_track = self
            .column_tracks
            .iter()
            .any(|t| *t == ResolvedTrack::Content);
        let mut content_widths = vec![0; num_cols];

        if has_content_track && !children.is_empty() {
            let has_header = self.has_header();
            let header_count = if has_header { num_cols } else { 0 };

            if has_header {
                for c in 0..num_cols {
                    if self.column_tracks[c] == ResolvedTrack::Content
                        && c < children.len()
                    {
                        let size = children
                            .measure(c, Size::new(avail_w, self.header_height));
                        content_widths[c] = content_widths[c].max(size.width);
                    }
                }
            }

            for (local_row, _) in self.state.rendered_rows.clone().enumerate() {
                for c in 0..num_cols {
                    let child_idx = header_count + local_row * num_cols + c;
                    if self.column_tracks[c] == ResolvedTrack::Content
                        && child_idx < children.len()
                    {
                        let size = children.measure(
                            child_idx,
                            Size::new(avail_w, self.row_height),
                        );
                        content_widths[c] = content_widths[c].max(size.width);
                    }
                }
            }
        }

        let col_gap = if self.column_divider {
            1
        } else {
            self.column_gap
        };
        let allocated_widths = allocate_tracks(
            &self.column_tracks,
            &content_widths,
            avail_w,
            col_gap,
        );

        let total_width = allocated_widths.iter().copied().sum::<u16>()
            + col_gap * (allocated_widths.len() - 1) as u16
            + inset * 2;

        let top_height = self.top_height();
        let total_height = u32::from(top_height)
            + self.virt.total_extent()
            + u32::from(inset * 2);

        Size::new(
            total_width.min(available.width),
            total_height.min(u32::from(available.height)) as u16,
        )
    }

    fn layout(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        area: Rect,
        children: &mut LayoutCx,
    ) {
        let num_cols = self.columns.len();
        if num_cols == 0 {
            return;
        }

        let inset = self.inset();
        let inner_x = area.x.saturating_add(inset);
        let inner_y = area.y.saturating_add(inset);
        let inner_w = area.width.saturating_sub(inset * 2);
        let inner_h = area.height.saturating_sub(inset * 2);

        let top_height = self.top_height();
        let body_height = inner_h.saturating_sub(top_height);
        self.state.viewport_height = u32::from(body_height);
        self.state.viewport_width = u32::from(inner_w);
        self.state.top_offset = top_height + inset;

        let total_body_height = self.virt.total_extent();
        self.state.content_height = total_body_height;

        let max_scroll_y =
            total_body_height.saturating_sub(self.state.viewport_height);
        self.state.scroll_y = self.state.scroll_y.min(max_scroll_y);

        let (visible_range, _) = self.ranges(u32::from(body_height));
        self.state.visible_rows = visible_range;

        let row_count = self.state.row_count;
        let mut row_offsets = Vec::with_capacity(row_count + 1);
        let mut row_extents = Vec::with_capacity(row_count);
        for i in 0..row_count {
            row_offsets.push(self.virt.offset_of(i));
            row_extents.push(self.virt.extent_of(i));
        }
        row_offsets.push(total_body_height);
        self.state.row_offsets = row_offsets.into();
        self.state.row_extents = row_extents.into();

        let mut content_widths = vec![0; num_cols];
        for c in 0..num_cols {
            if self.column_tracks[c] == ResolvedTrack::Content {
                let has_header = self.has_header();
                let header_count = if has_header { num_cols } else { 0 };
                for local_row in 0..self.state.rendered_rows.len() {
                    let child_idx = header_count + local_row * num_cols + c;
                    if child_idx < children.len() {
                        content_widths[c] = content_widths[c]
                            .max(children.size(child_idx).width);
                    }
                }
            }
        }

        let col_gap = if self.column_divider {
            1
        } else {
            self.column_gap
        };
        let allocated_widths = allocate_tracks(
            &self.column_tracks,
            &content_widths,
            inner_w,
            col_gap,
        );

        let total_width = allocated_widths.iter().copied().sum::<u16>()
            + col_gap * (allocated_widths.len() - 1) as u16;
        self.state.content_width = u32::from(total_width);

        let max_scroll_x = self
            .state
            .content_width
            .saturating_sub(self.state.viewport_width);
        self.state.scroll_x = self.state.scroll_x.min(max_scroll_x);

        let mut positions = Vec::with_capacity(num_cols);
        let mut cursor = i32::from(inset) - (self.state.scroll_x as i32);
        for &w in &allocated_widths {
            positions.push(cursor);
            cursor += i32::from(w) + i32::from(col_gap);
        }
        self.state.column_widths = allocated_widths;
        self.state.column_positions = positions;

        let has_header = self.has_header();
        let header_count = if has_header { num_cols } else { 0 };

        if has_header {
            for c in 0..num_cols {
                if c < children.len() {
                    let rel_x = self.state.column_positions[c];
                    let offset_x = (rel_x - i32::from(inset)).min(0);
                    let pos_x = (rel_x - i32::from(inset)).max(0) as u16;
                    let width = self.state.column_widths[c];
                    children.set(
                        c,
                        Rect::new(
                            inner_x + pos_x,
                            inner_y,
                            width,
                            self.header_height,
                        ),
                    );
                    if offset_x != 0 {
                        children.translate(c, Offset::new(offset_x, 0));
                    }
                }
            }
        }

        for (local_row, row_idx) in self.state.rendered_rows.clone().enumerate()
        {
            let item_pos =
                self.state.row_offsets.get(row_idx).copied().unwrap_or(0);
            let extent =
                self.state.row_extents.get(row_idx).copied().unwrap_or(1);
            let visible_pos =
                i64::from(item_pos) - i64::from(self.state.scroll_y);

            let row_hidden = visible_pos + i64::from(extent) <= 0;
            let pos_y = visible_pos.max(0).min(i64::from(u16::MAX)) as u16;
            let cell_y = inner_y + top_height + pos_y;
            let inner_bottom = inner_y + inner_h;
            let visible_h = if row_hidden || cell_y >= inner_bottom {
                0
            } else {
                (extent as u16).min(inner_bottom.saturating_sub(cell_y))
            };

            for c in 0..num_cols {
                let child_idx = header_count + local_row * num_cols + c;
                if child_idx < children.len() {
                    let rel_x = self.state.column_positions[c];
                    let offset_x = (rel_x - i32::from(inset)).min(0);
                    let pos_x = (rel_x - i32::from(inset)).max(0) as u16;
                    let width = self.state.column_widths[c];

                    children.set(
                        child_idx,
                        Rect::new(inner_x + pos_x, cell_y, width, visible_h),
                    );
                    if offset_x != 0 {
                        children.translate(child_idx, Offset::new(offset_x, 0));
                    }
                }
            }
        }
    }

    fn paint(&self, cx: &mut Cx, props: &Self::Props, canvas: &mut Canvas) {
        let area = cx.rect;
        let num_cols = self.columns.len();
        if num_cols == 0 {
            return;
        }

        let inset = self.inset();
        let inner_x = area.x.saturating_add(inset);
        let inner_y = area.y.saturating_add(inset);
        let inner_w = area.width.saturating_sub(inset * 2);
        let inner_h = area.height.saturating_sub(inset * 2);

        let theme_active = cx.theme().active;
        let theme_muted = cx.theme().palette.muted;
        let header_style = cx.resolve_or(
            "header_style",
            &props.header_style,
            Style::new().bold(),
            None,
        );
        let selected_style = cx.resolve_or(
            "selected_style",
            &props.selected_style,
            theme_active,
            None,
        );
        let divider_style = cx.resolve_or(
            "divider_style",
            &props.divider_style,
            Style::new().fg(theme_muted),
            None,
        );
        let alternate_style = props
            .alternate_row_style
            .as_ref()
            .map(|s| cx.resolve("alternate_style", s));

        let default_surface = cx.theme().surface;
        let default_row_style =
            cx.resolve_or("row_style", &props.row_style, default_surface, None);

        let top_height = self.top_height();
        let body_y = inner_y + top_height;
        let body_height = inner_h.saturating_sub(top_height);

        if self.border != Border::None && area.width > 0 && area.height > 0 {
            let chars = self.border.chars();
            let right = area.right() - 1;
            let bottom = area.bottom() - 1;

            if right > area.x {
                for x in (area.x + 1)..right {
                    canvas.set(x, area.y, chars.horizontal, divider_style);
                    canvas.set(x, bottom, chars.horizontal, divider_style);
                }
            }
            if bottom > area.y {
                for y in (area.y + 1)..bottom {
                    canvas.set(area.x, y, chars.vertical, divider_style);
                    canvas.set(right, y, chars.vertical, divider_style);
                }
            }
            canvas.set(area.x, area.y, chars.top_left, divider_style);
            canvas.set(right, area.y, chars.top_right, divider_style);
            canvas.set(area.x, bottom, chars.bottom_left, divider_style);
            canvas.set(right, bottom, chars.bottom_right, divider_style);
        }

        let mut base_header_style = header_style;
        if base_header_style.bg == Color::Unset {
            base_header_style.bg = default_surface.bg;
        }

        let has_header = self.has_header();
        if has_header && inner_w > 0 {
            canvas.fill(
                Rect::new(inner_x, inner_y, inner_w, self.header_height),
                ' ',
                base_header_style,
            );

            if self.mode == TMode::Column
                && let Some(col_idx) = self.state.selected_col
            {
                if col_idx < self.state.column_positions.len()
                    && col_idx < self.state.column_widths.len()
                {
                    let rel_x = self.state.column_positions[col_idx];
                    let cell_w = self.state.column_widths[col_idx];
                    let cell_x = i32::from(area.x) + rel_x;
                    let clipped_x = cell_x.max(i32::from(inner_x));
                    let clipped_end = (cell_x + i32::from(cell_w))
                        .min(i32::from(inner_x + inner_w));
                    if clipped_end > clipped_x {
                        let rect = Rect::new(
                            clipped_x as u16,
                            inner_y,
                            (clipped_end - clipped_x) as u16,
                            self.header_height,
                        );
                        canvas.fill(rect, ' ', selected_style);
                    }
                }
            }
        }

        let mut last_painted_y = body_y;

        for row_idx in self.state.visible_rows.clone() {
            let item_pos =
                self.state.row_offsets.get(row_idx).copied().unwrap_or(0);
            let extent =
                self.state.row_extents.get(row_idx).copied().unwrap_or(1);
            let visible_pos =
                i64::from(item_pos) - i64::from(self.state.scroll_y);
            if visible_pos < 0 || visible_pos >= i64::from(body_height) {
                continue;
            }
            let row_y = body_y.saturating_add(visible_pos as u16);
            let row_h =
                (extent as u16).min((inner_y + inner_h).saturating_sub(row_y));
            if row_h == 0 {
                continue;
            }
            let row_rect = Rect::new(inner_x, row_y, inner_w, row_h);

            let is_row_selected = self.state.is_row_selected(row_idx);
            let row_bg = if is_row_selected && self.mode == TMode::Row {
                selected_style
            } else if row_idx % 2 == 1 && alternate_style.is_some() {
                alternate_style.unwrap()
            } else {
                default_row_style
            };

            canvas.fill(row_rect, ' ', row_bg);

            let is_col_highlighted = (self.mode == TMode::Cell
                && self.state.selected_row == Some(row_idx))
                || self.mode == TMode::Column;

            if is_col_highlighted && let Some(col_idx) = self.state.selected_col
            {
                if col_idx < self.state.column_positions.len()
                    && col_idx < self.state.column_widths.len()
                {
                    let rel_x = self.state.column_positions[col_idx];
                    let cell_w = self.state.column_widths[col_idx];
                    let cell_x = i32::from(area.x) + rel_x;
                    let clipped_x = cell_x.max(i32::from(inner_x));
                    let clipped_end = (cell_x + i32::from(cell_w))
                        .min(i32::from(inner_x + inner_w));
                    if clipped_end > clipped_x {
                        let rect = Rect::new(
                            clipped_x as u16,
                            row_y,
                            (clipped_end - clipped_x) as u16,
                            row_h,
                        );
                        canvas.fill(rect, ' ', selected_style);
                    }
                }
            }

            last_painted_y = row_y.saturating_add(row_h);
        }

        let inner_bottom = inner_y + inner_h;
        if last_painted_y < inner_bottom && inner_w > 0 {
            canvas.fill(
                Rect::new(
                    inner_x,
                    last_painted_y,
                    inner_w,
                    inner_bottom - last_painted_y,
                ),
                ' ',
                default_row_style,
            );
        }

        let border_chars = self.border.chars();
        let header_chars = self
            .header_border
            .map(|b| b.chars())
            .unwrap_or(border_chars);

        if has_header && self.header_divider && inner_h > 0 {
            let div_y = inner_y + self.header_height;
            let right = area.right() - 1;
            if div_y < area.bottom() {
                for x in inner_x..(inner_x + inner_w) {
                    canvas.set(
                        x,
                        div_y,
                        header_chars.horizontal,
                        divider_style,
                    );
                }
                if self.border != Border::None {
                    let left_junc = if header_chars.horizontal == '═' {
                        '╞'
                    } else {
                        '├'
                    };
                    let right_junc = if header_chars.horizontal == '═' {
                        '╡'
                    } else {
                        '┤'
                    };
                    canvas.set(area.left(), div_y, left_junc, divider_style);
                    canvas.set(right, div_y, right_junc, divider_style);
                }
            }
        }

        if self.column_divider
            && !self.state.column_positions.is_empty()
            && inner_h > 0
        {
            let right = area.right() - 1;
            let bottom = area.bottom() - 1;

            for c in 0..num_cols - 1 {
                let rel_end = self.state.column_positions[c]
                    + i32::from(self.state.column_widths[c]);
                let div_x = i32::from(area.x) + rel_end;
                if div_x > i32::from(area.x) && div_x < i32::from(right) {
                    let x = div_x as u16;
                    for y in inner_y..inner_bottom {
                        canvas.set(x, y, border_chars.vertical, divider_style);
                    }
                    if self.border != Border::None {
                        canvas.set(x, area.top(), '╷', divider_style);
                        canvas.set(x, bottom, '╵', divider_style);
                    }
                }
            }
        }

        if self.row_divider && inner_w > 0 {
            let right = area.right() - 1;
            for row_idx in self.state.visible_rows.clone() {
                let item_pos =
                    self.state.row_offsets.get(row_idx).copied().unwrap_or(0);
                let extent =
                    self.state.row_extents.get(row_idx).copied().unwrap_or(1);
                let visible_pos =
                    i64::from(item_pos) - i64::from(self.state.scroll_y);
                let div_y = body_y + visible_pos.max(0) as u16 + extent as u16;
                if visible_pos >= 0 && div_y >= body_y && div_y < inner_bottom {
                    for x in inner_x..(inner_x + inner_w) {
                        canvas.set(
                            x,
                            div_y,
                            border_chars.horizontal,
                            divider_style,
                        );
                    }
                    if self.border != Border::None {
                        canvas.set(area.left(), div_y, '├', divider_style);
                        canvas.set(right, div_y, '┤', divider_style);
                    }
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
