use std::ops::Range;
use std::rc::Rc;

use kursor_core::{
    component::behavior::{Behavior, BehaviorCx},
    event::{
        Event, Phase,
        key::KeyCode,
        mouse::{MouseButton, MouseKind},
    },
    state::Signal,
};

use crate::widgets::list::ListSelection;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TMode {
    #[default]
    Row,
    Cell,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableTarget {
    Row(usize),
    Cell(usize, usize),
    Column(usize),
}

#[derive(Clone, Debug, Default)]
pub struct TableState {
    pub mode: TMode,
    pub selected_row: Option<usize>,
    pub selected_col: Option<usize>,
    pub row_count: usize,
    pub column_count: usize,
    pub scroll_y: u32,
    pub scroll_x: u32,
    pub viewport_height: u32,
    pub viewport_width: u32,
    pub content_height: u32,
    pub content_width: u32,
    pub top_offset: u16,
    pub visible_rows: Range<usize>,
    pub rendered_rows: Range<usize>,
    pub row_offsets: Rc<[u32]>,
    pub row_extents: Rc<[u32]>,
    pub column_positions: Vec<i32>,
    pub column_widths: Vec<u16>,
}

impl PartialEq for TableState {
    fn eq(&self, other: &Self) -> bool {
        self.mode == other.mode
            && self.selected_row == other.selected_row
            && self.selected_col == other.selected_col
            && self.row_count == other.row_count
            && self.column_count == other.column_count
            && self.scroll_y == other.scroll_y
            && self.scroll_x == other.scroll_x
            && self.viewport_height == other.viewport_height
            && self.viewport_width == other.viewport_width
            && self.content_height == other.content_height
            && self.content_width == other.content_width
            && self.top_offset == other.top_offset
            && self.visible_rows == other.visible_rows
            && self.rendered_rows == other.rendered_rows
            && self.column_positions == other.column_positions
            && self.column_widths == other.column_widths
            && Rc::ptr_eq(&self.row_offsets, &other.row_offsets)
            && Rc::ptr_eq(&self.row_extents, &other.row_extents)
    }
}

impl TableState {
    pub fn row_at(&self, rel_y: u32) -> Option<usize> {
        let top = u32::from(self.top_offset);
        if rel_y < top || self.row_count == 0 || self.row_offsets.len() < 2 {
            return None;
        }
        let abs_y = (rel_y - top) + self.scroll_y;
        let idx = match self.row_offsets[1..].binary_search(&abs_y) {
            Ok(i) => (i + 1).min(self.row_count - 1),
            Err(i) => i.min(self.row_count - 1),
        };
        let start = self.row_offsets[idx];
        let end = start + self.row_extents.get(idx).copied().unwrap_or(0);
        if abs_y >= start && abs_y < end {
            Some(idx)
        } else {
            None
        }
    }

    pub fn column_at(&self, rel_x: u32) -> Option<usize> {
        let x = rel_x as i64;
        for (i, (&pos, &width)) in self
            .column_positions
            .iter()
            .zip(&self.column_widths)
            .enumerate()
        {
            let start = i64::from(pos);
            let end = start + i64::from(width) + 1;
            if x >= start && x < end {
                return Some(i);
            }
        }
        None
    }

    pub fn is_row_selected(&self, row: usize) -> bool {
        match self.mode {
            TMode::Row | TMode::Cell => self.selected_row == Some(row),
            TMode::Column => false,
        }
    }

    pub fn is_cell_selected(&self, row: usize, col: usize) -> bool {
        match self.mode {
            TMode::Cell => self.selected_row == Some(row) && self.selected_col == Some(col),
            TMode::Row => self.selected_row == Some(row),
            TMode::Column => self.selected_col == Some(col),
        }
    }

    pub fn is_column_selected(&self, col: usize) -> bool {
        match self.mode {
            TMode::Column => self.selected_col == Some(col),
            TMode::Cell => self.selected_col == Some(col),
            TMode::Row => false,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum TSelection {
    Row(Signal<Option<usize>>),
    ExactRow(Signal<usize>),
    Cell(Signal<Option<(usize, usize)>>),
    Column(Signal<Option<usize>>),
}

pub trait IntoTSelection {
    fn into_tselection(self) -> TSelection;
}

impl IntoTSelection for Signal<Option<usize>> {
    fn into_tselection(self) -> TSelection {
        TSelection::Row(self)
    }
}

impl IntoTSelection for Signal<usize> {
    fn into_tselection(self) -> TSelection {
        TSelection::ExactRow(self)
    }
}

impl IntoTSelection for Signal<Option<(usize, usize)>> {
    fn into_tselection(self) -> TSelection {
        TSelection::Cell(self)
    }
}

impl IntoTSelection for TSelection {
    fn into_tselection(self) -> TSelection {
        self
    }
}

impl IntoTSelection for ListSelection {
    fn into_tselection(self) -> TSelection {
        match self {
            ListSelection::Optional(sig) => TSelection::Row(sig),
            ListSelection::Exact(sig) => TSelection::ExactRow(sig),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableIntent {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp(usize),
    PageDown(usize),
    SelectRow(usize),
    SelectCell(usize, usize),
    SelectColumn(usize),
    HeaderClick(usize),
    ScrollY(i32),
    ScrollX(i32),
    ToggleMode,
    Activate,
}

#[derive(Clone, Debug)]
pub struct TableBehavior {
    pub page_step: usize,
    pub wheel_step: u16,
    pub horizontal_step: u16,
}

impl Default for TableBehavior {
    fn default() -> Self {
        Self {
            page_step: 10,
            wheel_step: 3,
            horizontal_step: 4,
        }
    }
}

impl Behavior for TableBehavior {
    type State = TableState;
    type Intent = TableIntent;

    fn event(&self, cx: &BehaviorCx, event: &Event, state: &Self::State) -> Option<Self::Intent> {
        if cx.phase != Phase::Bubble {
            return None;
        }

        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => Some(TableIntent::Up),
                KeyCode::Down => Some(TableIntent::Down),
                KeyCode::Left => Some(TableIntent::Left),
                KeyCode::Right => Some(TableIntent::Right),
                KeyCode::Home => Some(TableIntent::Home),
                KeyCode::End => Some(TableIntent::End),
                KeyCode::PageUp => Some(TableIntent::PageUp(self.page_step)),
                KeyCode::PageDown => Some(TableIntent::PageDown(self.page_step)),
                KeyCode::Enter => Some(TableIntent::Activate),
                KeyCode::Tab => Some(TableIntent::ToggleMode),
                _ => None,
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseKind::ScrollUp => Some(TableIntent::ScrollY(-i32::from(self.wheel_step))),
                MouseKind::ScrollDown => Some(TableIntent::ScrollY(i32::from(self.wheel_step))),
                MouseKind::Down(MouseButton::Left) => {
                    let rel_y = mouse.row.saturating_sub(cx.rect.y);
                    let rel_x = mouse.column.saturating_sub(cx.rect.x);

                    if u32::from(rel_y) < u32::from(state.top_offset) {
                        state
                            .column_at(u32::from(rel_x))
                            .map(TableIntent::HeaderClick)
                    } else if let Some(row) = state.row_at(u32::from(rel_y)) {
                        match state.mode {
                            TMode::Cell => {
                                let col = state.column_at(u32::from(rel_x)).unwrap_or(0);
                                Some(TableIntent::SelectCell(row, col))
                            }
                            TMode::Column => {
                                let col = state.column_at(u32::from(rel_x)).unwrap_or(0);
                                Some(TableIntent::SelectColumn(col))
                            }
                            TMode::Row => Some(TableIntent::SelectRow(row)),
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }
}
