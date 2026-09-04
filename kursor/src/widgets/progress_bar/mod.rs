pub mod builder;
pub use builder::ProgressBarBuilder;

use kursor_core::{
    component::{Component, Update, blueprint::Blueprint, context::Cx},
    into_value,
    layout::{Direction, context::MeasureCx, size::Size},
    render::{canvas::Canvas, cell::Cell, color::Color, style::Style, subcell::Subcell},
    state::{IntoValue, Transition, Value},
};

#[derive(Clone, PartialEq)]
pub struct ProgressSegment {
    pub start: Value<f32>,
    pub end: Value<f32>,
    pub style: Option<Value<Style>>,
    pub fill_char: Option<Value<char>>,
    pub head_char: Option<Value<char>>,
    pub subcell: Option<Value<Subcell>>,
    pub transition: Option<Transition>,
}

impl ProgressSegment {
    pub fn new(end: impl IntoValue<f32>) -> Self {
        Self {
            start: Value::plain(0.0),
            end: end.into_value(),
            style: None,
            fill_char: None,
            head_char: None,
            subcell: None,
            transition: None,
        }
    }

    pub fn range(start: impl IntoValue<f32>, end: impl IntoValue<f32>) -> Self {
        Self {
            start: start.into_value(),
            end: end.into_value(),
            style: None,
            fill_char: None,
            head_char: None,
            subcell: None,
            transition: None,
        }
    }

    pub fn style(mut self, style: impl IntoValue<Style>) -> Self {
        self.style = Some(style.into_value());
        self
    }

    pub fn fill_char(mut self, ch: impl IntoValue<char>) -> Self {
        self.fill_char = Some(ch.into_value());
        self
    }

    pub fn head_char(mut self, ch: impl IntoValue<char>) -> Self {
        self.head_char = Some(ch.into_value());
        self
    }

    pub fn subcell(mut self, subcell: impl IntoValue<Subcell>) -> Self {
        self.subcell = Some(subcell.into_value());
        self
    }

    pub fn transition(mut self, transition: impl Into<Transition>) -> Self {
        self.transition = Some(transition.into());
        self
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct ProgressBarStyles {
    pub fill: Option<Value<Style>>,
    pub track: Option<Value<Style>>,
}

into_value!(ProgressBarStyles);

impl ProgressBarStyles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fill(mut self, style: impl IntoValue<Style>) -> Self {
        self.fill = Some(style.into_value());
        self
    }

    pub fn track(mut self, style: impl IntoValue<Style>) -> Self {
        self.track = Some(style.into_value());
        self
    }
}

#[derive(Clone, PartialEq)]
pub struct ProgressBarProps {
    pub segments: Vec<ProgressSegment>,
    pub direction: Value<Direction>,
    pub subcell: Value<Subcell>,
    pub styles: Value<ProgressBarStyles>,
    pub fill_char: Option<Value<char>>,
    pub head_char: Option<Value<char>>,
    pub track_char: Value<char>,
    pub transition: Option<Transition>,
}

impl Default for ProgressBarProps {
    fn default() -> Self {
        Self {
            segments: Vec::new(),
            direction: Value::plain(Direction::Right),
            subcell: Value::plain(Subcell::Eighth),
            styles: Value::plain(ProgressBarStyles::default()),
            fill_char: None,
            head_char: None,
            track_char: Value::plain(' '),
            transition: None,
        }
    }
}

pub struct ProgressBar {
    ranges: Vec<(f32, f32)>,
    direction: Direction,
    subcell: Subcell,
    styles: ProgressBarStyles,
    fill_char: Option<char>,
    head_char: Option<char>,
    track_char: char,
    transition: Option<Transition>,
}

impl ProgressBar {
    pub fn builder(value: impl IntoValue<f32>) -> ProgressBarBuilder {
        ProgressBarBuilder::new(value)
    }

    pub fn segmented(segments: impl IntoIterator<Item = ProgressSegment>) -> ProgressBarBuilder {
        ProgressBarBuilder::segmented(segments)
    }

    pub fn new(value: impl IntoValue<f32>) -> Blueprint {
        Self::builder(value).build()
    }

    pub fn vertical(value: impl IntoValue<f32>) -> Blueprint {
        Self::builder(value)
            .direction(Direction::Up)
            .build()
    }

    pub fn with(props: ProgressBarProps) -> Blueprint {
        Blueprint::new::<Self>(props)
    }
}

impl Component for ProgressBar {
    type Props = ProgressBarProps;

    fn create(_cx: &mut Cx, _props: &Self::Props) -> Self {
        Self {
            ranges: Vec::new(),
            direction: Direction::Right,
            subcell: Subcell::Eighth,
            styles: ProgressBarStyles::default(),
            fill_char: None,
            head_char: None,
            track_char: ' ',
            transition: None,
        }
    }

    fn changed(&self, old: &Self::Props, new: &Self::Props) -> bool {
        old != new
    }

    fn update(&mut self, _cx: &mut Cx, props: &Self::Props) -> Update {
        let ranges: Vec<(f32, f32)> = props
            .segments
            .iter()
            .map(|seg| (seg.start.get(), seg.end.get()))
            .collect();
        let direction = props.direction.get();
        let subcell = props.subcell.get();
        let styles = props.styles.get();
        let fill_char = props.fill_char.as_ref().map(Value::get);
        let head_char = props.head_char.as_ref().map(Value::get);
        let track_char = props.track_char.get();
        let transition = props.transition.clone();

        let direction_changed = self.direction != direction;
        let visual_changed = self.ranges != ranges
            || self.subcell != subcell
            || self.styles != styles
            || self.fill_char != fill_char
            || self.head_char != head_char
            || self.track_char != track_char
            || self.transition != transition;

        self.ranges = ranges;
        self.direction = direction;
        self.subcell = subcell;
        self.styles = styles;
        self.fill_char = fill_char;
        self.head_char = head_char;
        self.track_char = track_char;
        self.transition = transition;

        if direction_changed {
            Update::MEASURE
        } else if visual_changed {
            Update::PAINT
        } else {
            Update::NONE
        }
    }

    fn measure(
        &mut self,
        _cx: &mut Cx,
        _props: &Self::Props,
        available: Size,
        _children: &mut MeasureCx,
    ) -> Size {
        match self.direction {
            Direction::Right | Direction::Left => {
                Size::new(available.width, available.height.min(1))
            }
            Direction::Up | Direction::Down => {
                Size::new(available.width.min(1), available.height)
            }
        }
    }

    fn paint(&self, cx: &mut Cx, props: &Self::Props, canvas: &mut Canvas) {
        let rect = cx.rect;
        if rect.width == 0 || rect.height == 0 {
            return;
        }

        let theme = *cx.theme();
        let def_track = cx.resolve_or(
            "track_style",
            &props.styles.get().track,
            theme.surface,
            self.transition.clone(),
        );
        let def_fill = cx.resolve_or(
            "fill_style",
            &props.styles.get().fill,
            theme.primary,
            self.transition.clone(),
        );

        let (length, breadth, is_horizontal, is_reversed) = match self.direction {
            Direction::Right => (rect.width, rect.height, true, false),
            Direction::Left => (rect.width, rect.height, true, true),
            Direction::Down => (rect.height, rect.width, false, false),
            Direction::Up => (rect.height, rect.width, false, true),
        };
        for cross_idx in 0..breadth {
            for main_idx in 0..length {
                let (x, y) = if is_horizontal {
                    (rect.x.saturating_add(main_idx), rect.y.saturating_add(cross_idx))
                } else {
                    (rect.x.saturating_add(cross_idx), rect.y.saturating_add(main_idx))
                };
                canvas.set_cell(x, y, Cell::new(self.track_char, def_track));
            }
        }

        for (seg_idx, seg) in props.segments.iter().enumerate() {
            let (target_start, target_end) = self.ranges.get(seg_idx).copied().map_or_else(
                || {
                    let s = seg.start.get();
                    let e = seg.end.get();
                    (s.min(e).clamp(0.0, 1.0), e.max(s).clamp(0.0, 1.0))
                },
                |(s, e)| (s.min(e).clamp(0.0, 1.0), e.max(s).clamp(0.0, 1.0)),
            );

            let transition = seg.transition.clone().or_else(|| self.transition.clone());
            let cur_start: f32 = cx
                .transition(
                    seg_idx as u64 * 2 + 1,
                    target_start,
                    transition.clone(),
                )
                .clamp(0.0, 1.0);
            let cur_end: f32 = cx
                .transition(
                    seg_idx as u64 * 2,
                    target_end,
                    transition,
                )
                .clamp(0.0, 1.0);

            if cur_start >= cur_end {
                continue;
            }

            let seg_style = cx.resolve_or(
                seg_idx as u64 + 10_000,
                &seg.style,
                def_fill,
                None,
            );

            let seg_fill_fg = if seg_style.fg != Color::Unset && seg_style.fg != Color::Reset {
                seg_style.fg
            } else if seg_style.bg != Color::Unset && seg_style.bg != Color::Reset {
                seg_style.bg
            } else {
                theme.palette.primary
            };

            let seg_subcell = seg
                .subcell
                .as_ref()
                .map(Value::get)
                .unwrap_or(self.subcell);
            let seg_fill_char = seg
                .fill_char
                .as_ref()
                .map(Value::get)
                .or(self.fill_char)
                .unwrap_or_else(|| seg_subcell.fill_left(1.0));
            let seg_head_char = seg
                .head_char
                .as_ref()
                .map(Value::get)
                .or(self.head_char);

            let start_cells = cur_start * length as f32;
            let end_cells = cur_end * length as f32;

            let start_whole = (start_cells.floor() as usize).min(length as usize);
            let start_frac = (start_cells - start_whole as f32).clamp(0.0, 1.0);

            let end_whole = (end_cells.floor() as usize).min(length as usize);
            let end_frac = (end_cells - end_whole as f32).clamp(0.0, 1.0);

            for main_idx in 0..length {
                let fill_index = if is_reversed {
                    length.saturating_sub(1).saturating_sub(main_idx) as usize
                } else {
                    main_idx as usize
                };

                let (x_coord, y_coord) = if is_horizontal {
                    (rect.x.saturating_add(main_idx), rect.y)
                } else {
                    (rect.x, rect.y.saturating_add(main_idx))
                };

                let existing = canvas.cell(x_coord, y_coord);
                let underlay = existing.map_or(def_track.bg, |c| {
                    if c.ch == '\u{2588}' {
                        c.style.fg
                    } else if c.style.bg != Color::Unset && c.style.bg != Color::Reset {
                        c.style.bg
                    } else if c.style.fg != Color::Unset && c.style.fg != Color::Reset {
                        c.style.fg
                    } else {
                        def_track.bg
                    }
                });

                let cell: Option<(char, Style)> = if fill_index >= start_whole && fill_index < end_whole {
                    if fill_index == start_whole && start_frac > 0.0 && seg_subcell != Subcell::None {
                        let fill_amount = 1.0 - start_frac;
                        let ch = match self.direction {
                            Direction::Right => seg_subcell.fill_right(fill_amount),
                            Direction::Left => seg_subcell.fill_left(fill_amount),
                            Direction::Down => seg_subcell.fill_bottom(fill_amount),
                            Direction::Up => seg_subcell.fill_top(fill_amount),
                        };
                        Some((ch, Style::new().fg(seg_fill_fg).bg(underlay)))
                    } else {
                        Some((seg_fill_char, Style::new().fg(seg_fill_fg).bg(underlay)))
                    }
                } else if fill_index == end_whole {
                    if let Some(head) = seg_head_char {
                        Some((head, Style::new().fg(seg_fill_fg).bg(underlay)))
                    } else if end_frac > 0.0 && seg_subcell != Subcell::None {
                        let ch = match self.direction {
                            Direction::Right => seg_subcell.fill_left(end_frac),
                            Direction::Left => seg_subcell.fill_right(end_frac),
                            Direction::Up => seg_subcell.fill_bottom(end_frac),
                            Direction::Down => seg_subcell.fill_top(end_frac),
                        };
                        if ch == ' ' {
                            None
                        } else if ch == seg_fill_char {
                            Some((seg_fill_char, Style::new().fg(seg_fill_fg).bg(underlay)))
                        } else {
                            Some((ch, Style::new().fg(seg_fill_fg).bg(underlay)))
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some((ch, style)) = cell {
                    for cross_idx in 0..breadth {
                        let (x, y) = if is_horizontal {
                            (rect.x.saturating_add(main_idx), rect.y.saturating_add(cross_idx))
                        } else {
                            (rect.x.saturating_add(cross_idx), rect.y.saturating_add(main_idx))
                        };
                        canvas.set_cell(x, y, Cell::new(ch, style));
                    }
                }
            }
        }
    }
}

pub type Progress = ProgressBar;
