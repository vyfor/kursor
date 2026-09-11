use kursor_core::{
    component::blueprint::Blueprint, layout::Alignment, state::IntoValue,
};

use crate::layout::Track;
use crate::widgets::table::row::IntoTCell;

#[derive(Clone)]
pub struct TColumn {
    pub track: Track,
    pub header: Option<Blueprint>,
    pub align: Alignment,
}

impl TColumn {
    pub fn new(track: impl Into<Track>) -> Self {
        Self {
            track: track.into(),
            header: None,
            align: Alignment::TOP_LEFT,
        }
    }

    pub fn fill(weight: impl IntoValue<u16>) -> Self {
        Self::new(Track::fill(weight))
    }

    pub fn fixed(width: impl IntoValue<u16>) -> Self {
        Self::new(Track::fixed(width))
    }

    pub fn content() -> Self {
        Self::new(Track::content())
    }

    pub fn header(mut self, header: impl IntoTCell) -> Self {
        self.header = Some(header.into_tcell());
        self
    }

    pub fn align(mut self, align: Alignment) -> Self {
        self.align = align;
        self
    }
}

pub trait IntoTColumn {
    fn into_tcolumn(self) -> TColumn;
}

impl IntoTColumn for TColumn {
    fn into_tcolumn(self) -> TColumn {
        self
    }
}

impl IntoTColumn for Track {
    fn into_tcolumn(self) -> TColumn {
        TColumn::new(self)
    }
}

impl IntoTColumn for &'static str {
    fn into_tcolumn(self) -> TColumn {
        TColumn::content().header(self)
    }
}

impl IntoTColumn for String {
    fn into_tcolumn(self) -> TColumn {
        TColumn::content().header(self)
    }
}

impl<C: IntoTCell> IntoTColumn for (Track, C) {
    fn into_tcolumn(self) -> TColumn {
        TColumn::new(self.0).header(self.1)
    }
}

pub trait IntoTColumns {
    fn into_tcolumns(self) -> Vec<TColumn>;
}

impl IntoTColumns for TColumn {
    fn into_tcolumns(self) -> Vec<TColumn> {
        vec![self]
    }
}

impl IntoTColumns for Vec<TColumn> {
    fn into_tcolumns(self) -> Vec<TColumn> {
        self
    }
}

impl<const N: usize> IntoTColumns for [TColumn; N] {
    fn into_tcolumns(self) -> Vec<TColumn> {
        self.into()
    }
}

impl<const N: usize> IntoTColumns for [Track; N] {
    fn into_tcolumns(self) -> Vec<TColumn> {
        self.into_iter().map(TColumn::new).collect()
    }
}

impl IntoTColumns for Vec<Track> {
    fn into_tcolumns(self) -> Vec<TColumn> {
        self.into_iter().map(TColumn::new).collect()
    }
}

impl<const N: usize> IntoTColumns for [&'static str; N] {
    fn into_tcolumns(self) -> Vec<TColumn> {
        self.into_iter()
            .map(|s| TColumn::content().header(s))
            .collect()
    }
}

macro_rules! into_tcolumns {
    ($($type:ident: $value:ident),+ $(,)?) => {
        impl<$($type: IntoTColumn),+> IntoTColumns for ($($type,)+) {
            fn into_tcolumns(self) -> Vec<TColumn> {
                let ($($value,)+) = self;
                vec![$($value.into_tcolumn()),+]
            }
        }
    };
}

into_tcolumns!(A: a);
into_tcolumns!(A: a, B: b);
into_tcolumns!(A: a, B: b, C: c);
into_tcolumns!(A: a, B: b, C: c, D: d);
into_tcolumns!(A: a, B: b, C: c, D: d, E: e);
into_tcolumns!(A: a, B: b, C: c, D: d, E: e, F: f);
into_tcolumns!(A: a, B: b, C: c, D: d, E: e, F: f, G: g);
into_tcolumns!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h);
into_tcolumns!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i);
into_tcolumns!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i, J: j);
