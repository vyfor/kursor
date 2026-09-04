use kursor_core::{
    component::blueprint::{Blueprint, IntoBlueprint},
    layout::Direction,
    render::{style::Style, subcell::Subcell},
    state::{IntoValue, Transition, Value},
};

use super::{ProgressBar, ProgressBarProps, ProgressBarStyles, ProgressSegment};

pub struct ProgressBarBuilder {
    props: ProgressBarProps,
}

impl ProgressBarBuilder {
    pub(crate) fn new(value: impl IntoValue<f32>) -> Self {
        Self {
            props: ProgressBarProps {
                segments: vec![ProgressSegment::new(value)],
                ..Default::default()
            },
        }
    }

    pub(crate) fn segmented(segments: impl IntoIterator<Item = ProgressSegment>) -> Self {
        Self {
            props: ProgressBarProps {
                segments: segments.into_iter().collect(),
                ..Default::default()
            },
        }
    }

    pub fn value(mut self, value: impl IntoValue<f32>) -> Self {
        if let Some(first) = self.props.segments.first_mut() {
            first.end = value.into_value();
        } else {
            self.props.segments.push(ProgressSegment::new(value));
        }
        self
    }

    pub fn range(mut self, start: impl IntoValue<f32>, end: impl IntoValue<f32>) -> Self {
        if let Some(first) = self.props.segments.first_mut() {
            first.start = start.into_value();
            first.end = end.into_value();
        } else {
            self.props.segments.push(ProgressSegment::range(start, end));
        }
        self
    }

    pub fn segment(mut self, segment: ProgressSegment) -> Self {
        self.props.segments.push(segment);
        self
    }

    pub fn segments(mut self, segments: impl IntoIterator<Item = ProgressSegment>) -> Self {
        self.props.segments.extend(segments);
        self
    }

    pub fn direction(mut self, direction: impl IntoValue<Direction>) -> Self {
        self.props.direction = direction.into_value();
        self
    }

    pub fn subcell(mut self, subcell: impl IntoValue<Subcell>) -> Self {
        self.props.subcell = subcell.into_value();
        self
    }

    pub fn styles(mut self, styles: impl IntoValue<ProgressBarStyles>) -> Self {
        self.props.styles = styles.into_value();
        self
    }

    pub fn fill_style(mut self, style: impl IntoValue<Style>) -> Self {
        let style = style.into_value();
        if let Some(first) = self.props.segments.first_mut() {
            first.style = Some(style.clone());
        }
        let mut styles = self.props.styles.get();
        styles.fill = Some(style);
        self.props.styles = Value::plain(styles);
        self
    }

    pub fn track_style(mut self, style: impl IntoValue<Style>) -> Self {
        let mut styles = self.props.styles.get();
        styles.track = Some(style.into_value());
        self.props.styles = Value::plain(styles);
        self
    }

    pub fn fill_char(mut self, ch: impl IntoValue<char>) -> Self {
        let ch = ch.into_value();
        if let Some(first) = self.props.segments.first_mut() {
            first.fill_char = Some(ch.clone());
        }
        self.props.fill_char = Some(ch);
        self
    }

    pub fn head_char(mut self, ch: impl IntoValue<char>) -> Self {
        let ch = ch.into_value();
        if let Some(first) = self.props.segments.first_mut() {
            first.head_char = Some(ch.clone());
        }
        self.props.head_char = Some(ch);
        self
    }

    pub fn track_char(mut self, ch: impl IntoValue<char>) -> Self {
        self.props.track_char = ch.into_value();
        self
    }

    pub fn transition(mut self, transition: impl Into<Transition>) -> Self {
        self.props.transition = Some(transition.into());
        self
    }

    pub fn build(self) -> Blueprint {
        ProgressBar::with(self.props)
    }
}

impl IntoBlueprint for ProgressBarBuilder {
    fn into_blueprint(self) -> Vec<Blueprint> {
        vec![self.build()]
    }
}

impl From<ProgressBarBuilder> for Blueprint {
    fn from(builder: ProgressBarBuilder) -> Self {
        builder.build()
    }
}
