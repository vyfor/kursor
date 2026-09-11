use kursor_core::state::{IntoValue, Value};

/// sizing rules for a column or row.
#[derive(Clone)]
pub enum Track {
    Content,
    Fixed(Value<u16>),
    Fill(Value<u16>),
}

impl Track {
    pub const fn content() -> Self {
        Self::Content
    }

    pub fn fixed(size: impl IntoValue<u16>) -> Self {
        Self::Fixed(size.into_value())
    }

    pub fn fill(weight: impl IntoValue<u16>) -> Self {
        Self::Fill(weight.into_value())
    }

    pub(crate) fn resolve(&self) -> ResolvedTrack {
        match self {
            Self::Content => ResolvedTrack::Content,
            Self::Fixed(size) => ResolvedTrack::Fixed(size.get()),
            Self::Fill(weight) => ResolvedTrack::Fill(weight.get()),
        }
    }
}

pub trait IntoTracks {
    fn into_tracks(self) -> Vec<Track>;
}

impl IntoTracks for Track {
    fn into_tracks(self) -> Vec<Track> {
        vec![self]
    }
}

impl IntoTracks for Vec<Track> {
    fn into_tracks(self) -> Vec<Track> {
        self
    }
}

impl<const N: usize> IntoTracks for [Track; N] {
    fn into_tracks(self) -> Vec<Track> {
        self.into()
    }
}

macro_rules! tracks_tuple {
    ($($type:ident: $value:ident),+ $(,)?) => {
        impl<$($type: IntoTracks),+> IntoTracks for ($($type,)+) {
            fn into_tracks(self) -> Vec<Track> {
                let ($($value,)+) = self;
                let mut tracks = Vec::new();
                $(tracks.extend($value.into_tracks());)+
                tracks
            }
        }
    };
}

tracks_tuple!(A: a, B: b);
tracks_tuple!(A: a, B: b, C: c);
tracks_tuple!(A: a, B: b, C: c, D: d);
tracks_tuple!(A: a, B: b, C: c, D: d, E: e);
tracks_tuple!(A: a, B: b, C: c, D: d, E: e, F: f);
tracks_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g);
tracks_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h);
tracks_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i);
tracks_tuple!(A: a, B: b, C: c, D: d, E: e, F: f, G: g, H: h, I: i, J: j);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedTrack {
    Content,
    Fixed(u16),
    Fill(u16),
}

pub(crate) fn allocate_tracks(
    tracks: &[ResolvedTrack],
    content: &[u16],
    available: u16,
    gap: u16,
) -> Vec<u16> {
    let gaps = gap.saturating_mul((tracks.len().saturating_sub(1)) as u16);
    let mut allocated = gaps;
    let mut total_weight = 0_u32;
    let mut fill_count = 0_usize;

    for (index, track) in tracks.iter().enumerate() {
        match track {
            ResolvedTrack::Content => {
                allocated = allocated
                    .saturating_add(content.get(index).copied().unwrap_or(0));
            }
            ResolvedTrack::Fixed(size) => {
                allocated = allocated.saturating_add(*size)
            }
            ResolvedTrack::Fill(weight) => {
                total_weight = total_weight.saturating_add(u32::from(*weight));
                fill_count += 1;
            }
        }
    }

    let remaining = available.saturating_sub(allocated);
    let mut distributed = 0_u16;
    let mut fills_seen = 0_usize;
    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| match track {
            ResolvedTrack::Content => content.get(index).copied().unwrap_or(0),
            ResolvedTrack::Fixed(size) => *size,
            ResolvedTrack::Fill(weight) if total_weight > 0 => {
                fills_seen += 1;
                if fills_seen == fill_count {
                    remaining.saturating_sub(distributed)
                } else {
                    let share = (u32::from(remaining) * u32::from(*weight)
                        / total_weight) as u16;
                    distributed = distributed.saturating_add(share);
                    share
                }
            }
            ResolvedTrack::Fill(_) => 0,
        })
        .collect()
}
