//! UTC Unix-second animation clock. This is not Cesium's leap-second-aware JulianDate.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Range {
    Unbounded,
    Clamped,
    #[default]
    LoopStop,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct Clock {
    pub start_time: f64,
    pub stop_time: f64,
    pub current_time: f64,
    pub multiplier: f64,
    pub should_animate: bool,
    pub range: Range,
}
impl Default for Clock {
    fn default() -> Self {
        Self {
            start_time: 0.,
            stop_time: 60.,
            current_time: 0.,
            multiplier: 1.,
            should_animate: false,
            range: Range::LoopStop,
        }
    }
}
impl Clock {
    pub fn validate(&self) -> Result<(), String> {
        if ![
            self.start_time,
            self.stop_time,
            self.current_time,
            self.multiplier,
        ]
        .iter()
        .all(|v| v.is_finite())
            || self.start_time >= self.stop_time
            || (self.stop_time - self.start_time) > 1e12
            || self.multiplier.abs() > 1e6
            || self.current_time.abs() > 1e12
            || self.start_time.abs() > 1e12
            || self.stop_time.abs() > 1e12
        {
            return Err(
                "clock finite times, increasing interval and bounded multiplier required".into(),
            );
        }
        Ok(())
    }
    pub fn tick(&mut self, elapsed: f64) {
        if !self.should_animate || !elapsed.is_finite() || !(0.0..=60.0).contains(&elapsed) {
            return;
        }
        let next = self.current_time + elapsed * self.multiplier;
        self.current_time = match self.range {
            Range::Unbounded => next,
            Range::Clamped => next.clamp(self.start_time, self.stop_time),
            // Cesium LOOP_STOP clamps reverse travel at start and wraps forward travel after stop.
            Range::LoopStop => {
                if next < self.start_time {
                    self.start_time
                } else if next > self.stop_time {
                    {
                        let remainder =
                            (next - self.stop_time) % (self.stop_time - self.start_time);
                        if remainder == 0. {
                            self.stop_time
                        } else {
                            self.start_time + remainder
                        }
                    }
                } else {
                    next
                }
            }
        };
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loop_and_reverse() {
        let mut c = Clock {
            should_animate: true,
            multiplier: 3.,
            ..Default::default()
        };
        c.tick(60.);
        assert_eq!(c.current_time, 60.);
        c.multiplier = -100.;
        c.tick(1.);
        assert_eq!(c.current_time, 0.);
        c.range = Range::Unbounded;
        c.multiplier = -1.;
        c.tick(2.);
        assert_eq!(c.current_time, -2.);
    }
    #[test]
    fn clamp_pause_invalid() {
        let mut c = Clock {
            range: Range::Clamped,
            should_animate: true,
            current_time: 59.,
            ..Default::default()
        };
        c.tick(5.);
        assert_eq!(c.current_time, 60.);
        c.should_animate = false;
        c.tick(3.);
        assert_eq!(c.current_time, 60.);
        c.stop_time = 0.;
        assert!(c.validate().is_err());
    }
}
