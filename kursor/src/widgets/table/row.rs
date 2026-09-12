use std::rc::Rc;

use kursor_core::{
    component::{blueprint::Blueprint, key::Key},
    state::{IntoValue, Value},
};

use crate::widgets::{
    spacer::Spacer,
    text::{Line, Span, Text},
};

pub trait IntoTCell {
    fn into_tcell(self) -> Blueprint;
}

impl IntoTCell for Blueprint {
    fn into_tcell(self) -> Blueprint {
        self
    }
}

impl IntoTCell for &str {
    fn into_tcell(self) -> Blueprint {
        Text::new(self).build()
    }
}

impl IntoTCell for String {
    fn into_tcell(self) -> Blueprint {
        Text::new(self).build()
    }
}

impl IntoTCell for &String {
    fn into_tcell(self) -> Blueprint {
        Text::new(self.as_str()).build()
    }
}

impl IntoTCell for Span {
    fn into_tcell(self) -> Blueprint {
        Text::new(self).build()
    }
}

impl IntoTCell for Line {
    fn into_tcell(self) -> Blueprint {
        Text::new(self).build()
    }
}

impl IntoTCell for Vec<Line> {
    fn into_tcell(self) -> Blueprint {
        Text::new(self).build()
    }
}

impl<const N: usize> IntoTCell for [Line; N] {
    fn into_tcell(self) -> Blueprint {
        Text::new(self).build()
    }
}

impl IntoTCell for () {
    fn into_tcell(self) -> Blueprint {
        Spacer::new(0).build()
    }
}

macro_rules! into_tcell_text {
    ($($ty:ident),+ $(,)?) => {
        $(
            impl IntoTCell for $ty {
                fn into_tcell(self) -> Blueprint {
                    Text::new(self.to_string()).build()
                }
            }
        )+
    };
}

into_tcell_text!(
    usize, u64, u32, u16, u8, isize, i64, i32, i16, i8, f64, f32, bool
);

#[derive(Clone, Default)]
pub struct TRow {
    pub cells: Vec<Blueprint>,
    pub height: Option<u16>,
    pub key: Option<Key>,
}

impl TRow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cells(cells: impl IntoIterator<Item = impl IntoTCell>) -> Self {
        Self {
            cells: cells.into_iter().map(IntoTCell::into_tcell).collect(),
            height: None,
            key: None,
        }
    }

    pub fn cell(mut self, cell: impl IntoTCell) -> Self {
        self.cells.push(cell.into_tcell());
        self
    }

    pub fn cells(
        mut self,
        cells: impl IntoIterator<Item = impl IntoTCell>,
    ) -> Self {
        self.cells
            .extend(cells.into_iter().map(IntoTCell::into_tcell));
        self
    }

    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }
}

pub trait IntoTRow {
    fn into_trow(self) -> TRow;
}

impl IntoTRow for TRow {
    fn into_trow(self) -> TRow {
        self
    }
}

impl<C: IntoTCell> IntoTRow for Vec<C> {
    fn into_trow(self) -> TRow {
        TRow {
            cells: self.into_iter().map(IntoTCell::into_tcell).collect(),
            height: None,
            key: None,
        }
    }
}

impl<C: IntoTCell, const N: usize> IntoTRow for [C; N] {
    fn into_trow(self) -> TRow {
        TRow {
            cells: self.into_iter().map(IntoTCell::into_tcell).collect(),
            height: None,
            key: None,
        }
    }
}

macro_rules! into_trow {
    ($($type:ident: $value:ident),+ $(,)?) => {
        impl<$($type: IntoTCell),+> IntoTRow for ($($type,)+) {
            fn into_trow(self) -> TRow {
                let ($($value,)+) = self;
                let mut cells = Vec::new();
                $(cells.push($value.into_tcell());)+
                TRow {
                    cells,
                    height: None,
                    key: None,
                }
            }
        }
    };
}

into_trow!(A: a, B: b);
into_trow!(A: a, B: b, C: c);
into_trow!(A: a, B: b, C: c, D: d);
into_trow!(A: a, B: b, C: c, D: d, E: e);
into_trow!(A: a, B: b, C: c, D: d, E: e, F: f);
into_trow!(A: a, B: b, C: c, D: d, E: e, F: f, G: g);
into_trow!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h);
into_trow!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i);
into_trow!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i, J: j);

#[derive(Clone)]
pub enum TData {
    Static(Rc<[TRow]>),
    Lazy {
        count: Value<usize>,
        builder: Rc<dyn Fn(usize) -> TRow>,
    },
}

impl TData {
    pub fn empty() -> Self {
        Self::Static(Rc::from([]))
    }

    pub fn from_rows(rows: impl IntoIterator<Item = impl IntoTRow>) -> Self {
        let rows_vec: Vec<TRow> =
            rows.into_iter().map(IntoTRow::into_trow).collect();
        Self::Static(rows_vec.into())
    }

    pub fn lazy(
        count: impl IntoValue<usize>,
        builder: impl Fn(usize) -> TRow + 'static,
    ) -> Self {
        Self::Lazy {
            count: count.into_value(),
            builder: Rc::new(builder),
        }
    }

    pub fn count(&self) -> usize {
        match self {
            Self::Static(rows) => rows.len(),
            Self::Lazy { count, .. } => count.get(),
        }
    }

    pub fn get_row(&self, index: usize) -> Option<TRow> {
        match self {
            Self::Static(rows) => rows.get(index).cloned(),
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

impl PartialEq for TData {
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

impl Default for TData {
    fn default() -> Self {
        Self::empty()
    }
}
