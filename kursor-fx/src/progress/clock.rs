use std::time::Duration;

use animate::Time;

use super::Advance;

/// clock-driven progress.
#[derive(Clone)]
pub struct ClockState {
    pub(crate) start: Option<Time>,
    pub(crate) duration: Duration,
}

impl ClockState {
    pub fn advance(&mut self, time: Time) -> Advance {
        let begin = *self.start.get_or_insert(time);
        let span = self.duration.max(Duration::from_millis(1));
        let elapsed = time.elapsed.saturating_sub(begin.elapsed);
        let value = (elapsed.as_secs_f32() / span.as_secs_f32()).min(1.0);

        Advance {
            value,
            settled: value >= 1.0,
        }
    }

    pub fn reset(&mut self) {
        self.start = None;
    }
}
