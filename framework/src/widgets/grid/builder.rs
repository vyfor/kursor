use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    state::{IntoValue, Value},
};

use crate::layout::{IntoTracks, Track};

use super::{Grid, GridItem, GridProps, IntoGridItems};

pub struct GridBuilder {
    columns: Vec<Track>,
    rows: Vec<Track>,
    column_gap: Value<u16>,
    row_gap: Value<u16>,
    items: Vec<GridItem>,
}

impl GridBuilder {
    pub(crate) fn new(columns: impl IntoTracks, rows: impl IntoTracks) -> Self {
        Self {
            columns: columns.into_tracks(),
            rows: rows.into_tracks(),
            column_gap: Value::plain(0),
            row_gap: Value::plain(0),
            items: Vec::new(),
        }
    }

    pub fn columns(mut self, columns: impl IntoTracks) -> Self {
        self.columns = columns.into_tracks();
        self
    }

    pub fn rows(mut self, rows: impl IntoTracks) -> Self {
        self.rows = rows.into_tracks();
        self
    }

    pub fn gap(mut self, gap: impl IntoValue<u16>) -> Self {
        let gap = gap.into_value();
        self.column_gap = gap.clone();
        self.row_gap = gap;
        self
    }

    pub fn column_gap(mut self, gap: impl IntoValue<u16>) -> Self {
        self.column_gap = gap.into_value();
        self
    }

    pub fn row_gap(mut self, gap: impl IntoValue<u16>) -> Self {
        self.row_gap = gap.into_value();
        self
    }

    pub fn item(mut self, item: GridItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoGridItems) -> Self {
        self.items.extend(items.into_grid_items());
        self
    }
}

impl IntoBlueprint for GridBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Grid>(GridProps {
            columns: self.columns.into(),
            rows: self.rows.into(),
            column_gap: self.column_gap,
            row_gap: self.row_gap,
            items: self.items.into(),
        })]
    }
}
