pub mod builder;
pub use builder::GridBuilder;

use std::rc::Rc;

use kursor_core::{
    component::{
        Component, MountChildren, Update,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::{
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    state::{IntoValue, Value},
};

use crate::layout::{IntoTracks, ResolvedTrack, Track, allocate_tracks};

#[derive(Clone)]
pub struct GridItem {
    column: usize,
    row: usize,
    child: Blueprint,
}

impl GridItem {
    pub fn at(column: usize, row: usize, child: impl IntoBlueprint) -> Self {
        let mut children = child.into_blueprint();
        Self {
            column,
            row,
            child: children.pop().expect("need at least one child"),
        }
    }
}

pub trait IntoGridItems {
    fn into_grid_items(self) -> Vec<GridItem>;
}

impl IntoGridItems for GridItem {
    fn into_grid_items(self) -> Vec<GridItem> {
        vec![self]
    }
}

impl IntoGridItems for Vec<GridItem> {
    fn into_grid_items(self) -> Vec<GridItem> {
        self
    }
}

impl<const N: usize> IntoGridItems for [GridItem; N] {
    fn into_grid_items(self) -> Vec<GridItem> {
        self.into()
    }
}

macro_rules! grid_items_tuple {
    ($($type:ident: $value:ident),+ $(,)?) => {
        impl<$($type: IntoGridItems),+> IntoGridItems for ($($type,)+) {
            fn into_grid_items(self) -> Vec<GridItem> {
                let ($($value,)+) = self;
                let mut items = Vec::new();
                $(items.extend($value.into_grid_items());)+
                items
            }
        }
    };
}

grid_items_tuple!(A: a, B: b);
grid_items_tuple!(A: a, B: b, C: c);
grid_items_tuple!(A: a, B: b, C: c, D: d);
grid_items_tuple!(A: a, B: b, C: c, D: d, E: e);
grid_items_tuple!(A: a, B: b, C: c, D: d, E: e, F: f);
grid_items_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g);
grid_items_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h);
grid_items_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i);
grid_items_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i, J: j);

#[derive(Clone)]
pub struct GridProps {
    columns: Rc<[Track]>,
    rows: Rc<[Track]>,
    column_gap: Value<u16>,
    row_gap: Value<u16>,
    items: Rc<[GridItem]>,
}

/// grid implementation inspired by css.
///
/// arranges its children into rows and columns, with each row and column sized
/// independently through [`Track`]s.
pub struct Grid {
    columns: Rc<[Track]>,
    rows: Rc<[Track]>,
    items: Rc<[GridItem]>,
    column_tracks: Vec<ResolvedTrack>,
    row_tracks: Vec<ResolvedTrack>,
    column_gap: u16,
    row_gap: u16,
}

impl Grid {
    pub fn builder(
        columns: impl IntoTracks,
        rows: impl IntoTracks,
    ) -> GridBuilder {
        GridBuilder::new(columns, rows)
    }

    pub fn new(
        columns: impl IntoTracks,
        rows: impl IntoTracks,
        items: impl IntoGridItems,
    ) -> Blueprint {
        Self::with(columns, rows, 0, 0, items)
    }

    pub fn with(
        columns: impl IntoTracks,
        rows: impl IntoTracks,
        column_gap: impl IntoValue<u16>,
        row_gap: impl IntoValue<u16>,
        items: impl IntoGridItems,
    ) -> Blueprint {
        Blueprint::new::<Self>(GridProps {
            columns: columns.into_tracks().into(),
            rows: rows.into_tracks().into(),
            column_gap: column_gap.into_value(),
            row_gap: row_gap.into_value(),
            items: items.into_grid_items().into(),
        })
    }

    fn child_blueprints(&self) -> Vec<Blueprint> {
        self.items.iter().map(|item| item.child.clone()).collect()
    }

    fn resolve_tracks(tracks: &[Track]) -> Vec<ResolvedTrack> {
        tracks.iter().map(Track::resolve).collect()
    }

    fn content_sizes(
        &self,
        children: &mut MeasureCx,
        available: Size,
    ) -> (Vec<u16>, Vec<u16>) {
        let mut columns = vec![0; self.column_tracks.len()];
        let mut rows = vec![0; self.row_tracks.len()];
        for (index, item) in self.items.iter().enumerate() {
            if index >= children.len()
                || item.column >= self.column_tracks.len()
                || item.row >= self.row_tracks.len()
            {
                continue;
            }
            let child = children.measure(index, available);
            if self.column_tracks[item.column] == ResolvedTrack::Content {
                columns[item.column] = columns[item.column].max(child.width);
            }
            if self.row_tracks[item.row] == ResolvedTrack::Content {
                rows[item.row] = rows[item.row].max(child.height);
            }
        }
        (columns, rows)
    }

    fn content_sizes_layouted(
        &self,
        children: &LayoutCx,
    ) -> (Vec<u16>, Vec<u16>) {
        let mut columns = vec![0; self.column_tracks.len()];
        let mut rows = vec![0; self.row_tracks.len()];
        for (index, item) in self.items.iter().enumerate() {
            if index >= children.len()
                || item.column >= self.column_tracks.len()
                || item.row >= self.row_tracks.len()
            {
                continue;
            }
            let child = children.size(index);
            if self.column_tracks[item.column] == ResolvedTrack::Content {
                columns[item.column] = columns[item.column].max(child.width);
            }
            if self.row_tracks[item.row] == ResolvedTrack::Content {
                rows[item.row] = rows[item.row].max(child.height);
            }
        }
        (columns, rows)
    }
}

impl Component for Grid {
    type Props = GridProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            columns: Rc::from([]),
            rows: Rc::from([]),
            items: Rc::from([]),
            column_tracks: Vec::new(),
            row_tracks: Vec::new(),
            column_gap: 0,
            row_gap: 0,
        }
    }

    fn changed(&self, _old: &Self::Props, _new: &Self::Props) -> bool {
        true
    }

    fn mount(
        &mut self,
        _cx: &mut Cx,
        props: &Self::Props,
        children: &mut MountChildren,
    ) {
        self.columns = props.columns.clone();
        self.rows = props.rows.clone();
        self.items = props.items.clone();
        children.replace(self.child_blueprints());
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let column_tracks = Self::resolve_tracks(&props.columns);
        let row_tracks = Self::resolve_tracks(&props.rows);
        let column_gap = props.column_gap.get();
        let row_gap = props.row_gap.get();
        let items_changed = !Rc::ptr_eq(&self.items, &props.items);
        let layout_changed = self.column_tracks != column_tracks
            || self.row_tracks != row_tracks
            || self.column_gap != column_gap
            || self.row_gap != row_gap;
        self.columns = props.columns.clone();
        self.rows = props.rows.clone();
        self.items = props.items.clone();
        self.column_tracks = column_tracks;
        self.row_tracks = row_tracks;
        self.column_gap = column_gap;
        self.row_gap = row_gap;
        if items_changed {
            Update::children(self.child_blueprints())
        } else if layout_changed {
            Update::MEASURE
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
        let (column_content, row_content) =
            self.content_sizes(children, available);
        let columns = allocate_tracks(
            &self.column_tracks,
            &column_content,
            available.width,
            self.column_gap,
        );
        let rows = allocate_tracks(
            &self.row_tracks,
            &row_content,
            available.height,
            self.row_gap,
        );
        let width = columns.iter().copied().sum::<u16>().saturating_add(
            self.column_gap
                .saturating_mul(columns.len().saturating_sub(1) as u16),
        );
        let height = rows.iter().copied().sum::<u16>().saturating_add(
            self.row_gap
                .saturating_mul(rows.len().saturating_sub(1) as u16),
        );

        Size::new(width.min(available.width), height.min(available.height))
    }

    fn layout(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        area: Rect,
        children: &mut LayoutCx,
    ) {
        let (column_content, row_content) =
            self.content_sizes_layouted(children);
        let columns = allocate_tracks(
            &self.column_tracks,
            &column_content,
            area.width,
            self.column_gap,
        );
        let rows = allocate_tracks(
            &self.row_tracks,
            &row_content,
            area.height,
            self.row_gap,
        );

        let mut x = Vec::with_capacity(columns.len());
        let mut cursor = area.x;
        for width in &columns {
            x.push(cursor);
            cursor = cursor
                .saturating_add(*width)
                .saturating_add(self.column_gap);
        }
        let mut y = Vec::with_capacity(rows.len());
        let mut cursor = area.y;
        for height in &rows {
            y.push(cursor);
            cursor =
                cursor.saturating_add(*height).saturating_add(self.row_gap);
        }

        for (index, item) in self.items.iter().enumerate() {
            if index >= children.len()
                || item.column >= columns.len()
                || item.row >= rows.len()
            {
                continue;
            }
            let width = columns[item.column]
                .min(area.right().saturating_sub(x[item.column]));
            let height =
                rows[item.row].min(area.bottom().saturating_sub(y[item.row]));
            children.set(
                index,
                Rect::new(x[item.column], y[item.row], width, height),
            );
        }
    }
}
