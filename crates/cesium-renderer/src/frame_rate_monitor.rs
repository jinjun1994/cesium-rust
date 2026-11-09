//! FrameRateMonitor matching CesiumJS `Scene/FrameRateMonitor.js` (Apache-2.0).

use cesium_math::{DeveloperError, Event};
use std::collections::VecDeque;

/// Default settings for `FrameRateMonitor`.
pub const DEFAULT_SAMPLING_WINDOW: f64 = 5.0;
pub const DEFAULT_QUIET_PERIOD: f64 = 2.0;
pub const DEFAULT_WARMUP_PERIOD: f64 = 5.0;
pub const DEFAULT_MIN_FPS_WARMUP: f64 = 4.0;
pub const DEFAULT_MIN_FPS_AFTER_WARMUP: f64 = 8.0;

/// Initialization options for `FrameRateMonitor`.
#[derive(Debug, Clone, Default)]
pub struct FrameRateMonitorOptions {
    /// The length of sliding window over which to compute average FPS, in seconds. Defaults to 5.0.
    pub sampling_window: Option<f64>,
    /// The length of time to wait before starting to measure performance, in seconds. Defaults to 2.0.
    pub quiet_period: Option<f64>,
    /// The length of warmup period, in seconds. Defaults to 5.0.
    pub warmup_period: Option<f64>,
    /// The minimum FPS required during warmup period. Defaults to 4.
    pub minimum_frame_rate_during_warmup: Option<f64>,
    /// The minimum FPS required after warmup period. Defaults to 8.
    pub minimum_frame_rate_after_warmup: Option<f64>,
}

/// Monitors the frame rate (frames per second) in a scene and raises an event if the
/// frame rate is lower than a threshold.
#[derive(Debug, Clone)]
pub struct FrameRateMonitor {
    /// Length of the sliding window in seconds.
    pub sampling_window: f64,
    /// Length of quiet period in seconds.
    pub quiet_period: f64,
    /// Length of warmup period in seconds.
    pub warmup_period: f64,
    /// Minimum FPS required during warmup.
    pub minimum_frame_rate_during_warmup: f64,
    /// Minimum FPS required after warmup.
    pub minimum_frame_rate_after_warmup: f64,

    low_frame_rate: Event<f64>,
    nominal_frame_rate: Event<f64>,

    frame_times: VecDeque<f64>,
    needs_quiet_period: bool,
    quiet_period_end_time: f64,
    warmup_period_end_time: f64,
    frame_rate_is_low: bool,
    last_frames_per_second: Option<f64>,
    pause_count: usize,
    is_destroyed: bool,
}

impl Default for FrameRateMonitor {
    fn default() -> Self {
        Self::new(None)
    }
}

impl FrameRateMonitor {
    /// Creates a new `FrameRateMonitor`.
    pub fn new(options: Option<FrameRateMonitorOptions>) -> Self {
        let opts = options.unwrap_or_default();
        Self {
            sampling_window: opts.sampling_window.unwrap_or(DEFAULT_SAMPLING_WINDOW),
            quiet_period: opts.quiet_period.unwrap_or(DEFAULT_QUIET_PERIOD),
            warmup_period: opts.warmup_period.unwrap_or(DEFAULT_WARMUP_PERIOD),
            minimum_frame_rate_during_warmup: opts
                .minimum_frame_rate_during_warmup
                .unwrap_or(DEFAULT_MIN_FPS_WARMUP),
            minimum_frame_rate_after_warmup: opts
                .minimum_frame_rate_after_warmup
                .unwrap_or(DEFAULT_MIN_FPS_AFTER_WARMUP),
            low_frame_rate: Event::new(),
            nominal_frame_rate: Event::new(),
            frame_times: VecDeque::new(),
            needs_quiet_period: true,
            quiet_period_end_time: 0.0,
            warmup_period_end_time: 0.0,
            frame_rate_is_low: false,
            last_frames_per_second: None,
            pause_count: 0,
            is_destroyed: false,
        }
    }

    /// Event raised when a low frame rate is detected.
    pub fn low_frame_rate(&self) -> &Event<f64> {
        &self.low_frame_rate
    }

    /// Event raised when frame rate returns to normal level.
    pub fn nominal_frame_rate(&self) -> &Event<f64> {
        &self.nominal_frame_rate
    }

    /// Most recently computed average frames-per-second.
    pub fn last_frames_per_second(&self) -> Option<f64> {
        self.last_frames_per_second
    }

    /// Returns the current pause count.
    pub fn pause_count(&self) -> usize {
        self.pause_count
    }

    /// Pauses monitoring of the frame rate.
    pub fn pause(&mut self) {
        self.pause_count += 1;
        if self.pause_count == 1 {
            self.frame_times.clear();
            self.last_frames_per_second = None;
        }
    }

    /// Resumes monitoring of the frame rate.
    pub fn unpause(&mut self) {
        if self.pause_count > 0 {
            self.pause_count -= 1;
            if self.pause_count == 0 {
                self.needs_quiet_period = true;
            }
        }
    }

    /// Updates frame rate statistics for a single rendered frame at `timestamp_ms` (in milliseconds).
    pub fn update(&mut self, timestamp_ms: f64) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        if self.pause_count > 0 {
            return Ok(());
        }

        let time_stamp = timestamp_ms;

        if self.needs_quiet_period {
            self.needs_quiet_period = false;
            self.frame_times.clear();
            self.quiet_period_end_time = time_stamp + self.quiet_period * 1000.0;
            self.warmup_period_end_time =
                self.quiet_period_end_time + (self.warmup_period + self.sampling_window) * 1000.0;
        } else if time_stamp >= self.quiet_period_end_time {
            self.frame_times.push_back(time_stamp);

            let beginning_of_window = time_stamp - self.sampling_window * 1000.0;

            if self.frame_times.len() >= 2 && self.frame_times[0] <= beginning_of_window {
                while self.frame_times.len() >= 2 && self.frame_times[1] < beginning_of_window {
                    self.frame_times.pop_front();
                }

                let average_time_between_frames =
                    (time_stamp - self.frame_times[0]) / (self.frame_times.len() - 1) as f64;

                let fps = 1000.0 / average_time_between_frames;
                self.last_frames_per_second = Some(fps);

                let min_fps = if time_stamp > self.warmup_period_end_time {
                    self.minimum_frame_rate_after_warmup
                } else {
                    self.minimum_frame_rate_during_warmup
                };
                let maximum_frame_time = 1000.0 / min_fps;

                if average_time_between_frames > maximum_frame_time {
                    if !self.frame_rate_is_low {
                        self.frame_rate_is_low = true;
                        self.needs_quiet_period = true;
                        self.low_frame_rate.raise_event(&fps);
                    }
                } else if self.frame_rate_is_low {
                    self.frame_rate_is_low = false;
                    self.needs_quiet_period = true;
                    self.nominal_frame_rate.raise_event(&fps);
                }
            }
        }

        Ok(())
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys resources.
    pub fn destroy(&mut self) {
        self.frame_times.clear();
        self.is_destroyed = true;
    }
}
