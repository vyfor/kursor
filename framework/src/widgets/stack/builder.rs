use std::marker::PhantomData;

use kursor_core::component::blueprint::{Blueprint, IntoBlueprint};

use super::{Column, Row, StackProps};

pub struct StackBuilder<T> {
    props: StackProps,
    children: Vec<Blueprint>,
    marker: PhantomData<T>,
}

pub type RowBuilder = StackBuilder<Row>;
pub type ColumnBuilder = StackBuilder<Column>;

impl<T> StackBuilder<T> {
    fn new(children: impl IntoBlueprint) -> Self {
        Self {
            props: StackProps { gap: 0 },
            children: children.into_blueprint(),
            marker: PhantomData,
        }
    }

    pub fn gap(mut self, gap: u16) -> Self {
        self.props.gap = gap;
        self
    }
}

impl StackBuilder<Row> {
    pub(crate) fn row(children: impl IntoBlueprint) -> Self {
        Self::new(children)
    }
}

impl StackBuilder<Column> {
    pub(crate) fn column(children: impl IntoBlueprint) -> Self {
        Self::new(children)
    }
}

impl IntoBlueprint for StackBuilder<Row> {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Row>(self.props).children(self.children)]
    }
}

impl IntoBlueprint for StackBuilder<Column> {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Column>(self.props).children(self.children)]
    }
}
