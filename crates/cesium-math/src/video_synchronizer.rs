use crate::JulianDate;

/// Abstract target representing a video player or video element.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoElementMock {
    pub current_time: f64,
    pub duration: f64,
    pub loop_playback: bool,
    pub paused: bool,
    pub ready_state: u8,
    pub play_calls: usize,
    pub pause_calls: usize,
}

impl Default for VideoElementMock {
    fn default() -> Self {
        Self {
            current_time: 0.0,
            duration: 10.0,
            loop_playback: false,
            paused: true,
            ready_state: 4,
            play_calls: 0,
            pause_calls: 0,
        }
    }
}

impl VideoElementMock {
    pub fn play(&mut self) {
        self.paused = false;
        self.play_calls += 1;
    }

    pub fn pause(&mut self) {
        self.paused = true;
        self.pause_calls += 1;
    }
}

/// Options for `VideoSynchronizer`.
#[derive(Debug, Clone, Default)]
pub struct VideoSynchronizerOptions {
    pub epoch: Option<JulianDate>,
    pub tolerance: Option<f64>,
}

/// Synchronizes a video element with a simulation clock.
#[derive(Debug, Clone)]
pub struct VideoSynchronizer {
    pub epoch: JulianDate,
    pub tolerance: f64,
    is_destroyed: bool,
}

impl Default for VideoSynchronizer {
    fn default() -> Self {
        Self::new(None)
    }
}

impl VideoSynchronizer {
    pub fn new(options: Option<VideoSynchronizerOptions>) -> Self {
        let opts = options.unwrap_or_default();
        Self {
            epoch: opts.epoch.unwrap_or(JulianDate::MINIMUM_VALUE),
            tolerance: opts.tolerance.unwrap_or(1.0),
            is_destroyed: false,
        }
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }

    /// Computes the desired video playback time from the simulation clock's current time.
    pub fn compute_desired_time(
        current_time: &JulianDate,
        epoch: &JulianDate,
        duration: f64,
        is_looping: bool,
    ) -> f64 {
        if duration <= 0.0 {
            return 0.0;
        }

        let mut video_time = current_time.seconds_difference(epoch);

        if is_looping {
            video_time %= duration;
            if video_time < 0.0 {
                video_time += duration;
            }
            video_time
        } else if video_time > duration {
            duration
        } else if video_time < 0.0 {
            0.0
        } else {
            video_time
        }
    }

    /// Advances synchronization with a target video element.
    pub fn sync_tick(
        &self,
        current_clock_time: &JulianDate,
        should_animate: bool,
        element: &mut VideoElementMock,
    ) {
        if self.is_destroyed || element.ready_state < 2 {
            return;
        }

        // Play/Pause sync
        if should_animate && element.paused {
            element.play();
        } else if !should_animate && !element.paused {
            element.pause();
        }

        let desired_time = Self::compute_desired_time(
            current_clock_time,
            &self.epoch,
            element.duration,
            element.loop_playback,
        );

        let tol = if should_animate { self.tolerance } else { 0.001 };
        if (desired_time - element.current_time).abs() > tol {
            element.current_time = desired_time;
        }
    }
}
