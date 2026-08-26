pub mod builder;
pub use builder::FlexBuilder;

use std::rc::Rc;

use kursor_core::{
    component::{
        Component, MountChildren, Update,
        blueprint::{Blueprint, IntoBlueprint},
        context::Cx,
    },
    layout::{
        Orientation,
        context::{LayoutCx, MeasureCx},
        rect::Rect,
        size::Size,
    },
    state::{IntoValue, Value},
};

use crate::layout::{ResolvedTrack, Track};

#[derive(Clone)]
pub struct FlexItem {
    size: Track,
    child: Blueprint,
}

impl FlexItem {
    pub fn content(child: impl IntoBlueprint) -> Self {
        Self::new(Track::content(), child)
    }

    pub fn fixed(size: impl IntoValue<u16>, child: impl IntoBlueprint) -> Self {
        Self::new(Track::fixed(size), child)
    }

    pub fn fill(weight: impl IntoValue<u16>, child: impl IntoBlueprint) -> Self {
        Self::new(Track::fill(weight), child)
    }

    fn new(size: Track, child: impl IntoBlueprint) -> Self {
        let mut children = child.into_blueprint();
        Self {
            size,
            child: children.pop().expect("need at least one child"),
        }
    }
}

pub trait IntoFlexItems {
    fn into_flex_items(self) -> Vec<FlexItem>;
}

impl IntoFlexItems for FlexItem {
    fn into_flex_items(self) -> Vec<FlexItem> {
        vec![self]
    }
}

impl IntoFlexItems for Vec<FlexItem> {
    fn into_flex_items(self) -> Vec<FlexItem> {
        self
    }
}

impl<const N: usize> IntoFlexItems for [FlexItem; N] {
    fn into_flex_items(self) -> Vec<FlexItem> {
        self.into()
    }
}

macro_rules! flex_items_tuple {
    ($($type:ident: $value:ident),+ $(,)?) => {
        impl<$($type: IntoFlexItems),+> IntoFlexItems for ($($type,)+) {
            fn into_flex_items(self) -> Vec<FlexItem> {
                let ($($value,)+) = self;
                let mut items = Vec::new();
                $(items.extend($value.into_flex_items());)+
                items
            }
        }
    };
}

flex_items_tuple!(A: a, B: b);
flex_items_tuple!(A: a, B: b, C: c);
flex_items_tuple!(A: a, B: b, C: c, D: d);
flex_items_tuple!(A: a, B: b, C: c, D: d, E: e);
flex_items_tuple!(A: a, B: b, C: c, D: d, E: e, F: f);
flex_items_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g);
flex_items_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h);
flex_items_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i);
flex_items_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i, J: j);

#[derive(Clone)]
pub struct FlexProps {
    pub direction: Value<Orientation>,
    pub gap: Value<u16>,
    items: Rc<[FlexItem]>,
}

pub struct Flex {
    direction: Orientation,
    gap: u16,
    items: Rc<[FlexItem]>,
    sizes: Vec<ResolvedTrack>,
}

impl Flex {
    pub fn builder(direction: impl IntoValue<Orientation>) -> FlexBuilder {
        FlexBuilder::new(direction)
    }

    pub fn row_builder() -> FlexBuilder {
        FlexBuilder::new(Orientation::Horizontal)
    }

    pub fn column_builder() -> FlexBuilder {
        FlexBuilder::new(Orientation::Vertical)
    }

    pub fn row(items: impl IntoFlexItems) -> Blueprint {
        Self::with(Orientation::Horizontal, 0, items)
    }

    pub fn column(items: impl IntoFlexItems) -> Blueprint {
        Self::with(Orientation::Vertical, 0, items)
    }

    pub fn row_spaced(gap: impl IntoValue<u16>, items: impl IntoFlexItems) -> Blueprint {
        Self::with(Orientation::Horizontal, gap, items)
    }

    pub fn column_spaced(gap: impl IntoValue<u16>, items: impl IntoFlexItems) -> Blueprint {
        Self::with(Orientation::Vertical, gap, items)
    }

    pub fn with(
        direction: impl IntoValue<Orientation>,
        gap: impl IntoValue<u16>,
        items: impl IntoFlexItems,
    ) -> Blueprint {
        Blueprint::new::<Self>(FlexProps {
            direction: direction.into_value(),
            gap: gap.into_value(),
            items: items.into_flex_items().into(),
        })
    }

    fn child_blueprints(&self) -> Vec<Blueprint> {
        self.items.iter().map(|item| item.child.clone()).collect()
    }

    fn main(size: Size, direction: Orientation) -> u16 {
        match direction {
            Orientation::Horizontal => size.width,
            Orientation::Vertical => size.height,
        }
    }

    fn cross(size: Size, direction: Orientation) -> u16 {
        match direction {
            Orientation::Horizontal => size.height,
            Orientation::Vertical => size.width,
        }
    }

    fn with_main(size: Size, direction: Orientation, main: u16) -> Size {
        match direction {
            Orientation::Horizontal => Size::new(main, size.height),
            Orientation::Vertical => Size::new(size.width, main),
        }
    }
}

impl Component for Flex {
    type Props = FlexProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            direction: Orientation::Horizontal,
            gap: 0,
            items: Rc::from([]),
            sizes: Vec::new(),
        }
    }

    fn changed(&self, _old: &Self::Props, _new: &Self::Props) -> bool {
        true
    }

    fn mount(&mut self, _cx: &mut Cx, props: &Self::Props, children: &mut MountChildren) {
        self.items = props.items.clone();
        children.replace(self.child_blueprints());
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let direction = props.direction.get();
        let gap = props.gap.get();
        let sizes: Vec<_> = props.items.iter().map(|item| item.size.resolve()).collect();
        let structure_changed = !Rc::ptr_eq(&self.items, &props.items);
        let layout_changed = self.direction != direction || self.gap != gap || self.sizes != sizes;
        self.direction = direction;
        self.gap = gap;
        self.items = props.items.clone();
        self.sizes = sizes;

        if structure_changed {
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
        let count = children.len();
        if count == 0 {
            return Size::default();
        }

        let available_main = Self::main(available, self.direction);
        let mut main = 0_u16;
        let mut cross = 0_u16;
        let mut has_fill = false;

        for index in 0..count {
            let size = self
                .sizes
                .get(index)
                .copied()
                .unwrap_or(ResolvedTrack::Content);
            let child_available = match size {
                ResolvedTrack::Fixed(main) => Self::with_main(available, self.direction, main),
                _ => available,
            };
            let child = children.measure(index, child_available);
            cross = cross.max(Self::cross(child, self.direction));
            match size {
                ResolvedTrack::Content => {
                    main = main.saturating_add(Self::main(child, self.direction));
                }
                ResolvedTrack::Fixed(fixed) => {
                    main = main.saturating_add(fixed);
                }
                ResolvedTrack::Fill(_) => {
                    has_fill = true;
                }
            }
        }

        main = main.saturating_add(self.gap.saturating_mul((count.saturating_sub(1)) as u16));
        if has_fill {
            main = available_main;
        }

        match self.direction {
            Orientation::Horizontal => {
                Size::new(main.min(available.width), cross.min(available.height))
            }
            Orientation::Vertical => {
                Size::new(cross.min(available.width), main.min(available.height))
            }
        }
    }

    fn layout(&mut self, _cx: &mut Cx, _props: &Self::Props, area: Rect, children: &mut LayoutCx) {
        let count = children.len();
        if count == 0 {
            return;
        }

        let gaps = self.gap.saturating_mul((count.saturating_sub(1)) as u16);
        let available_main = Self::main(Size::new(area.width, area.height), self.direction);
        let mut allocated = gaps;
        let mut total_weight = 0_u32;

        for index in 0..count {
            match self
                .sizes
                .get(index)
                .copied()
                .unwrap_or(ResolvedTrack::Content)
            {
                ResolvedTrack::Content => {
                    allocated =
                        allocated.saturating_add(Self::main(children.size(index), self.direction));
                }
                ResolvedTrack::Fixed(size) => allocated = allocated.saturating_add(size),
                ResolvedTrack::Fill(weight) => {
                    total_weight = total_weight.saturating_add(u32::from(weight))
                }
            }
        }

        let remaining = available_main.saturating_sub(allocated);
        let mut cursor = match self.direction {
            Orientation::Horizontal => area.x,
            Orientation::Vertical => area.y,
        };
        let mut distributed = 0_u16;

        for index in 0..count {
            let size = self
                .sizes
                .get(index)
                .copied()
                .unwrap_or(ResolvedTrack::Content);
            let main = match size {
                ResolvedTrack::Content => Self::main(children.size(index), self.direction),
                ResolvedTrack::Fixed(size) => size,
                ResolvedTrack::Fill(weight) if total_weight > 0 => {
                    let share = (u32::from(remaining) * u32::from(weight) / total_weight) as u16;
                    distributed = distributed.saturating_add(share);
                    if index + 1 == count {
                        remaining.saturating_sub(distributed.saturating_sub(share))
                    } else {
                        share
                    }
                }
                ResolvedTrack::Fill(_) => 0,
            };
            let max_main = match self.direction {
                Orientation::Horizontal => area.right().saturating_sub(cursor),
                Orientation::Vertical => area.bottom().saturating_sub(cursor),
            };
            let main = main.min(max_main);
            let rect = match self.direction {
                Orientation::Horizontal => Rect::new(cursor, area.y, main, area.height),
                Orientation::Vertical => Rect::new(area.x, cursor, area.width, main),
            };
            children.set(index, rect);
            cursor = cursor.saturating_add(main);
            if index + 1 < count {
                cursor = cursor.saturating_add(self.gap);
            }
        }
    }
}
