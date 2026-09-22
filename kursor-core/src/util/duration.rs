use std::time::Duration;

pub trait IntoDuration {
    fn into_duration(self) -> Option<Duration>;
}

impl IntoDuration for Duration {
    fn into_duration(self) -> Option<Duration> {
        Some(self)
    }
}

impl IntoDuration for Option<Duration> {
    fn into_duration(self) -> Option<Duration> {
        self
    }
}
