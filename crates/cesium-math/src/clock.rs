use std::time::Instant;

use crate::{DeveloperError, Event, JulianDate};

/// Range behaviors for Clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClockRange {
    #[default]
    Unbounded = 0,
    Clamped = 1,
    LoopStop = 2,
}

impl ClockRange {
    pub const UNBOUNDED: Self = Self::Unbounded;
    pub const CLAMPED: Self = Self::Clamped;
    pub const LOOP_STOP: Self = Self::LoopStop;
    pub const CLAMP: Self = Self::Clamped;
}

/// Advance modes for Clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClockStep {
    TickDependent = 0,
    #[default]
    SystemClockMultiplier = 1,
    SystemClock = 2,
}

impl ClockStep {
    pub const TICK_DEPENDENT: Self = Self::TickDependent;
    pub const SYSTEM_CLOCK_MULTIPLIER: Self = Self::SystemClockMultiplier;
    pub const SYSTEM_CLOCK: Self = Self::SystemClock;
}

/// Construction options for Clock.
#[derive(Debug, Clone, Default)]
pub struct ClockOptions {
    pub start_time: Option<JulianDate>,
    pub stop_time: Option<JulianDate>,
    pub current_time: Option<JulianDate>,
    pub clock_step: Option<ClockStep>,
    pub multiplier: Option<f64>,
    pub clock_range: Option<ClockRange>,
    pub can_animate: Option<bool>,
    pub should_animate: Option<bool>,
}

/// A simulation clock that tracks JulianDate and supports configurable advance steps and bounds.
#[derive(Debug, Clone)]
pub struct Clock {
    start_time: JulianDate,
    stop_time: JulianDate,
    current_time: JulianDate,
    clock_step: ClockStep,
    multiplier: f64,
    clock_range: ClockRange,
    pub can_animate: bool,
    should_animate: bool,

    pub on_stop: Event<Self>,
    pub on_tick: Event<Self>,

    last_system_time: Option<Instant>,
}

impl Default for Clock {
    fn default() -> Self {
        Self::new(ClockOptions::default()).expect("default clock is valid")
    }
}

impl Clock {
    pub fn new(options: ClockOptions) -> Result<Self, DeveloperError> {
        let (start, stop, current) = match (options.start_time, options.stop_time, options.current_time) {
            (Some(start), Some(stop), Some(current)) => {
                if start.greater_than(&stop) {
                    return Err(DeveloperError::new("startTime must come before stopTime"));
                }
                (start, stop, current)
            }
            (Some(start), Some(stop), None) => {
                if start.greater_than(&stop) {
                    return Err(DeveloperError::new("startTime must come before stopTime"));
                }
                (start, stop, start)
            }
            (None, Some(stop), Some(current)) => (current, stop, current),
            (Some(start), None, Some(current)) => (start, start.add_days(1.0), current),
            (None, None, Some(current)) => (current, current.add_days(1.0), current),
            (None, Some(stop), None) => (stop.add_days(-1.0), stop, stop.add_days(-1.0)),
            (Some(start), None, None) => (start, start.add_days(1.0), start),
            (None, None, None) => {
                let now = JulianDate::now();
                (now, now.add_days(1.0), now)
            }
        };

        let clock_step = options
            .clock_step
            .unwrap_or(ClockStep::SystemClockMultiplier);

        let mut clock = Self {
            start_time: start,
            stop_time: stop,
            current_time: current,
            clock_step,
            multiplier: options.multiplier.unwrap_or(1.0),
            clock_range: options.clock_range.unwrap_or(ClockRange::Unbounded),
            can_animate: options.can_animate.unwrap_or(true),
            should_animate: options.should_animate.unwrap_or(false),
            on_stop: Event::new(),
            on_tick: Event::new(),
            last_system_time: None,
        };

        if clock_step == ClockStep::SystemClock {
            clock.current_time = JulianDate::now();
            clock.multiplier = 1.0;
            clock.should_animate = true;
        }

        Ok(clock)
    }

    pub fn start_time(&self) -> JulianDate {
        self.start_time
    }

    pub fn set_start_time(&mut self, time: JulianDate) {
        self.start_time = time;
    }

    pub fn stop_time(&self) -> JulianDate {
        self.stop_time
    }

    pub fn set_stop_time(&mut self, time: JulianDate) {
        self.stop_time = time;
    }

    pub fn current_time(&self) -> JulianDate {
        self.current_time
    }

    pub fn set_current_time(&mut self, time: JulianDate) {
        if self.clock_step == ClockStep::SystemClock && time != self.current_time {
            self.clock_step = ClockStep::SystemClockMultiplier;
        }
        self.current_time = time;
    }

    pub fn clock_step(&self) -> ClockStep {
        self.clock_step
    }

    pub fn set_clock_step(&mut self, step: ClockStep) {
        if step == ClockStep::SystemClock {
            self.current_time = JulianDate::now();
            self.multiplier = 1.0;
            self.should_animate = true;
        }
        self.clock_step = step;
        self.last_system_time = None;
    }

    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }

    pub fn set_multiplier(&mut self, multiplier: f64) {
        if self.clock_step == ClockStep::SystemClock && (multiplier - self.multiplier).abs() > 1e-12 {
            self.clock_step = ClockStep::SystemClockMultiplier;
        }
        self.multiplier = multiplier;
    }

    pub fn clock_range(&self) -> ClockRange {
        self.clock_range
    }

    pub fn set_clock_range(&mut self, range: ClockRange) {
        self.clock_range = range;
    }

    pub fn should_animate(&self) -> bool {
        self.should_animate
    }

    pub fn set_should_animate(&mut self, animate: bool) {
        if self.clock_step == ClockStep::SystemClock && !animate {
            self.clock_step = ClockStep::SystemClockMultiplier;
        }
        self.should_animate = animate;
        if !animate {
            self.last_system_time = None;
        }
    }

    /// Advances the clock and returns the updated `currentTime`.
    pub fn tick(&mut self) -> JulianDate {
        self.tick_with_delta(None)
    }

    /// Advances the clock with an explicit elapsed seconds delta (for deterministic simulation and testing).
    pub fn tick_with_delta(&mut self, simulated_delta_seconds: Option<f64>) -> JulianDate {
        let current = self.current_time;

        if !self.can_animate || !self.should_animate {
            self.last_system_time = None;
            let clone = self.clone();
            self.on_tick.raise_event(&clone);
            return current;
        }

        let next = match self.clock_step {
            ClockStep::SystemClock => match simulated_delta_seconds {
                Some(delta) => current.add_seconds(delta),
                None => JulianDate::now(),
            },
            ClockStep::SystemClockMultiplier => {
                let delta_seconds = match simulated_delta_seconds {
                    Some(s) => s,
                    None => {
                        let now = Instant::now();
                        let delta = if let Some(last) = self.last_system_time {
                            now.duration_since(last).as_secs_f64()
                        } else {
                            0.0
                        };
                        self.last_system_time = Some(now);
                        delta
                    }
                };
                current.add_seconds(delta_seconds * self.multiplier)
            }
            ClockStep::TickDependent => {
                let advance = simulated_delta_seconds.unwrap_or(self.multiplier);
                current.add_seconds(advance)
            }
        };

        let mut fire_stop = false;

        match self.clock_range {
            ClockRange::Unbounded => {
                self.current_time = next;
            }
            ClockRange::Clamped => {
                if self.multiplier > 0.0 && next.greater_than_or_equals(&self.stop_time) {
                    if current.less_than(&self.stop_time) || next.greater_than(&self.stop_time) {
                        fire_stop = true;
                    }
                    self.current_time = self.stop_time;
                } else if self.multiplier < 0.0 && next.less_than_or_equals(&self.start_time) {
                    self.current_time = self.start_time;
                } else {
                    self.current_time = next;
                }
            }
            ClockRange::LoopStop => {
                if self.multiplier > 0.0 && next.greater_than_or_equals(&self.stop_time) {
                    fire_stop = true;
                    let total_range = self.stop_time.seconds_difference(&self.start_time);
                    if total_range > 0.0 {
                        let overflow = next.seconds_difference(&self.stop_time);
                        let remainder = overflow % total_range;
                        self.current_time = self.start_time.add_seconds(remainder);
                    } else {
                        self.current_time = self.start_time;
                    }
                } else if self.multiplier < 0.0 && next.less_than(&self.start_time) {
                    self.current_time = self.start_time;
                } else {
                    self.current_time = next;
                }
            }
        }

        let clone = self.clone();
        if fire_stop {
            self.on_stop.raise_event(&clone);
        }
        self.on_tick.raise_event(&clone);

        self.current_time
    }
}
