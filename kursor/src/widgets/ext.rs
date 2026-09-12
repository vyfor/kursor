use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    layout::{Alignment, Insets},
    state::IntoValue,
    theme::Theme,
};

use super::{
    Align, Block, Border, Bounds, Padding, Scroll, Show, Stack, Themed, Wrap,
};

pub trait IntoBlueprintExt: IntoBlueprint + Sized {
    fn align(self, alignment: impl IntoValue<Alignment>) -> Blueprint {
        Align::new(self).alignment(alignment).build()
    }

    fn top_left(self) -> Blueprint {
        Align::top_left(self)
    }

    fn top_center(self) -> Blueprint {
        Align::top_center(self)
    }

    fn top_right(self) -> Blueprint {
        Align::top_right(self)
    }

    fn center_left(self) -> Blueprint {
        Align::center_left(self)
    }

    fn center(self) -> Blueprint {
        Align::center(self)
    }

    fn center_right(self) -> Blueprint {
        Align::center_right(self)
    }

    fn bottom_left(self) -> Blueprint {
        Align::bottom_left(self)
    }

    fn bottom_center(self) -> Blueprint {
        Align::bottom_center(self)
    }

    fn bottom_right(self) -> Blueprint {
        Align::bottom_right(self)
    }

    fn bounds(self, width: u16, height: u16) -> Blueprint {
        Bounds::exact(width, height, self)
    }

    fn min_width(self, width: impl IntoValue<u16>) -> Blueprint {
        Bounds::new(self).min_width(width).build()
    }

    fn max_width(self, width: impl IntoValue<u16>) -> Blueprint {
        Bounds::new(self).max_width(width).build()
    }

    fn min_height(self, height: impl IntoValue<u16>) -> Blueprint {
        Bounds::new(self).min_height(height).build()
    }

    fn max_height(self, height: impl IntoValue<u16>) -> Blueprint {
        Bounds::new(self).max_height(height).build()
    }

    fn padding(self, value: u16) -> Blueprint {
        Padding::all(value, self)
    }

    fn padding_insets(self, insets: impl IntoValue<Insets>) -> Blueprint {
        Padding::new(self).insets(insets).build()
    }

    fn padding_horizontal(self, value: u16) -> Blueprint {
        Padding::horizontal(value, self)
    }

    fn padding_vertical(self, value: u16) -> Blueprint {
        Padding::vertical(value, self)
    }

    fn padding_top(self, value: u16) -> Blueprint {
        Padding::top(value, self)
    }

    fn padding_bottom(self, value: u16) -> Blueprint {
        Padding::bottom(value, self)
    }

    fn padding_left(self, value: u16) -> Blueprint {
        Padding::left(value, self)
    }

    fn padding_right(self, value: u16) -> Blueprint {
        Padding::right(value, self)
    }

    fn border(self, border: impl IntoValue<Border>) -> Blueprint {
        Block::new(self).border(border).build()
    }

    fn border_plain(self) -> Blueprint {
        Block::new(self).plain().build()
    }

    fn border_rounded(self) -> Blueprint {
        Block::new(self).rounded().build()
    }

    fn border_double(self) -> Blueprint {
        Block::new(self).double().build()
    }

    fn border_heavy(self) -> Blueprint {
        Block::new(self).heavy().build()
    }

    fn scroll_vertical(self) -> Blueprint {
        Scroll::new(self).build()
    }

    fn scroll_horizontal(self) -> Blueprint {
        Scroll::horizontal(self)
    }

    fn scroll_both(self) -> Blueprint {
        Scroll::both(self)
    }

    fn themed(self, theme: impl IntoValue<Theme>) -> Blueprint {
        Themed::new(self).theme(theme).build()
    }

    fn stacked(self) -> Blueprint {
        Stack::new(self)
    }

    fn show_when(self, condition: impl IntoValue<bool>) -> Blueprint {
        Show::new(condition).child(self).build()
    }

    fn show_when_else(
        self,
        condition: impl IntoValue<bool>,
        fallback: impl IntoBlueprint,
    ) -> Blueprint {
        Show::new(condition).child(self).fallback(fallback).build()
    }

    fn wrap(
        self,
        gap: impl IntoValue<u16>,
        line_gap: impl IntoValue<u16>,
    ) -> Blueprint {
        Wrap::spaced(gap, line_gap, self)
    }

    fn wrap_uniform(self, gap: impl IntoValue<u16>) -> Blueprint {
        Wrap::uniform(gap, self)
    }
}

impl<T> IntoBlueprintExt for T where T: IntoBlueprint {}
