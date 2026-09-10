use std::rc::Rc;

use kursor_core::{
    component::{
        behavior::Behavior,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    render::style::Style,
    state::{IntoValue, Value},
};

use crate::layout::Track;
use crate::widgets::block::Border;
use crate::widgets::table::{
    Table, TableBehavior, TableIntent, TableProps, TableState, TableTarget,
    behavior::{IntoTSelection, TMode, TSelection},
    column::{IntoTColumn, IntoTColumns, TColumn},
    row::{IntoTRow, TData, TRow},
};

pub struct TableBuilder {
    mode: Value<TMode>,
    columns: Vec<TColumn>,
    header_row: Option<TRow>,
    static_rows: Vec<TRow>,
    data: Option<TData>,
    border: Value<Border>,
    header_border: Option<Value<Border>>,
    row_height: Value<u16>,
    header_height: Value<u16>,
    column_gap: Value<u16>,
    row_gap: Value<u16>,
    header_divider: Value<bool>,
    column_divider: Value<bool>,
    row_divider: Value<bool>,
    scroll_margin: Value<u16>,
    overscan: Value<usize>,
    wrap: Value<bool>,
    selection: Option<TSelection>,
    header_style: Option<Value<Style>>,
    row_style: Option<Value<Style>>,
    alternate_row_style: Option<Value<Style>>,
    selected_style: Option<Value<Style>>,
    divider_style: Option<Value<Style>>,
    behavior: Option<Rc<dyn Behavior<State = TableState, Intent = TableIntent>>>,
    on_activate: Option<Rc<dyn Fn(&mut Cx, TableTarget)>>,
    on_select: Option<Rc<dyn Fn(&mut Cx, TableTarget)>>,
    on_header_click: Option<Rc<dyn Fn(&mut Cx, usize)>>,
}

impl Default for TableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TableBuilder {
    pub fn new() -> Self {
        Self {
            mode: Value::plain(TMode::Row),
            columns: Vec::new(),
            header_row: None,
            static_rows: Vec::new(),
            data: None,
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
            behavior: None,
            on_activate: None,
            on_select: None,
            on_header_click: None,
        }
    }

    pub fn mode(mut self, mode: impl IntoValue<TMode>) -> Self {
        self.mode = mode.into_value();
        self
    }

    pub fn column(mut self, column: impl IntoTColumn) -> Self {
        self.columns.push(column.into_tcolumn());
        self
    }

    pub fn columns(mut self, columns: impl IntoTColumns) -> Self {
        self.columns.extend(columns.into_tcolumns());
        self
    }

    pub fn header(mut self, row: impl IntoTRow) -> Self {
        self.header_row = Some(row.into_trow());
        self
    }

    pub fn header_height(mut self, height: impl IntoValue<u16>) -> Self {
        self.header_height = height.into_value();
        self
    }

    pub fn header_divider(mut self, divider: impl IntoValue<bool>) -> Self {
        self.header_divider = divider.into_value();
        self
    }

    pub fn row(mut self, row: impl IntoTRow) -> Self {
        self.static_rows.push(row.into_trow());
        self
    }

    pub fn rows(mut self, rows: impl IntoIterator<Item = impl IntoTRow>) -> Self {
        self.static_rows
            .extend(rows.into_iter().map(IntoTRow::into_trow));
        self
    }

    pub fn data(mut self, data: TData) -> Self {
        self.data = Some(data);
        self
    }

    pub fn lazy(
        mut self,
        count: impl IntoValue<usize>,
        builder: impl Fn(usize) -> TRow + 'static,
    ) -> Self {
        self.data = Some(TData::lazy(count, builder));
        self
    }

    pub fn row_height(mut self, height: impl IntoValue<u16>) -> Self {
        self.row_height = height.into_value();
        self
    }

    pub fn row_gap(mut self, gap: impl IntoValue<u16>) -> Self {
        self.row_gap = gap.into_value();
        self
    }

    pub fn row_divider(mut self, divider: impl IntoValue<bool>) -> Self {
        self.row_divider = divider.into_value();
        self
    }

    pub fn column_gap(mut self, gap: impl IntoValue<u16>) -> Self {
        self.column_gap = gap.into_value();
        self
    }

    pub fn column_divider(mut self, divider: impl IntoValue<bool>) -> Self {
        self.column_divider = divider.into_value();
        self
    }

    pub fn border(mut self, border: impl IntoValue<Border>) -> Self {
        self.border = border.into_value();
        self
    }

    pub fn header_border(mut self, border: impl IntoValue<Border>) -> Self {
        self.header_border = Some(border.into_value());
        self
    }

    pub fn scroll_margin(mut self, margin: impl IntoValue<u16>) -> Self {
        self.scroll_margin = margin.into_value();
        self
    }

    pub fn overscan(mut self, overscan: impl IntoValue<usize>) -> Self {
        self.overscan = overscan.into_value();
        self
    }

    pub fn wrap(mut self, wrap: impl IntoValue<bool>) -> Self {
        self.wrap = wrap.into_value();
        self
    }

    pub fn selection(mut self, selection: impl IntoTSelection) -> Self {
        self.selection = Some(selection.into_tselection());
        self
    }

    pub fn on_activate(mut self, callback: impl Fn(&mut Cx, TableTarget) + 'static) -> Self {
        self.on_activate = Some(Rc::new(callback));
        self
    }

    pub fn on_select(mut self, callback: impl Fn(&mut Cx, TableTarget) + 'static) -> Self {
        self.on_select = Some(Rc::new(callback));
        self
    }

    pub fn on_header_click(mut self, callback: impl Fn(&mut Cx, usize) + 'static) -> Self {
        self.on_header_click = Some(Rc::new(callback));
        self
    }

    pub fn behavior(
        mut self,
        behavior: impl Behavior<State = TableState, Intent = TableIntent> + 'static,
    ) -> Self {
        self.behavior = Some(Rc::new(behavior));
        self
    }

    pub fn header_style(mut self, style: impl IntoValue<Style>) -> Self {
        self.header_style = Some(style.into_value());
        self
    }

    pub fn row_style(mut self, style: impl IntoValue<Style>) -> Self {
        self.row_style = Some(style.into_value());
        self
    }

    pub fn alternate_row_style(mut self, style: impl IntoValue<Style>) -> Self {
        self.alternate_row_style = Some(style.into_value());
        self
    }

    pub fn selected_style(mut self, style: impl IntoValue<Style>) -> Self {
        self.selected_style = Some(style.into_value());
        self
    }

    pub fn divider_style(mut self, style: impl IntoValue<Style>) -> Self {
        self.divider_style = Some(style.into_value());
        self
    }

    pub fn build(mut self) -> Blueprint {
        if let Some(header_row) = self.header_row {
            for (idx, cell) in header_row.cells.into_iter().enumerate() {
                if idx < self.columns.len() {
                    if self.columns[idx].header.is_none() {
                        self.columns[idx].header = Some(cell);
                    }
                } else {
                    self.columns.push(TColumn::new(Track::Content).header(cell));
                }
            }
        }

        if self.columns.is_empty() {
            let num = self.static_rows.first().map(|r| r.cells.len()).unwrap_or(0);
            if num > 0 {
                self.columns = (0..num).map(|_| TColumn::fill(1)).collect();
            }
        }

        let data = if let Some(d) = self.data {
            d
        } else {
            TData::from_rows(self.static_rows)
        };

        let behavior = self
            .behavior
            .unwrap_or_else(|| Rc::new(TableBehavior::default()));

        Table::with(TableProps {
            mode: self.mode,
            columns: self.columns.into(),
            data,
            border: self.border,
            header_border: self.header_border,
            row_height: self.row_height,
            header_height: self.header_height,
            column_gap: self.column_gap,
            row_gap: self.row_gap,
            header_divider: self.header_divider,
            column_divider: self.column_divider,
            row_divider: self.row_divider,
            scroll_margin: self.scroll_margin,
            overscan: self.overscan,
            wrap: self.wrap,
            selection: self.selection,
            header_style: self.header_style,
            row_style: self.row_style,
            alternate_row_style: self.alternate_row_style,
            selected_style: self.selected_style,
            divider_style: self.divider_style,
            behavior,
            on_activate: self.on_activate,
            on_select: self.on_select,
            on_header_click: self.on_header_click,
        })
    }
}

impl IntoBlueprint for TableBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}
