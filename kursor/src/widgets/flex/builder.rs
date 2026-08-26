use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    layout::Orientation,
    state::{IntoValue, Value},
};

use super::{Flex, FlexItem, FlexProps, IntoFlexItems};

pub struct FlexBuilder {
    direction: Value<Orientation>,
    gap: Value<u16>,
    items: Vec<FlexItem>,
}

impl FlexBuilder {
    pub(crate) fn new(direction: impl IntoValue<Orientation>) -> Self {
        Self {
            direction: direction.into_value(),
            gap: Value::plain(0),
            items: Vec::new(),
        }
    }

    pub fn direction(mut self, direction: impl IntoValue<Orientation>) -> Self {
        self.direction = direction.into_value();
        self
    }

    pub fn gap(mut self, gap: impl IntoValue<u16>) -> Self {
        self.gap = gap.into_value();
        self
    }

    pub fn item(mut self, item: FlexItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoFlexItems) -> Self {
        self.items.extend(items.into_flex_items());
        self
    }
}

impl IntoBlueprint for FlexBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![Blueprint::new::<Flex>(FlexProps {
            direction: self.direction,
            gap: self.gap,
            items: self.items.into(),
        })]
    }
}
