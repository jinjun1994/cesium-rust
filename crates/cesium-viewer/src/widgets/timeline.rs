//! Timeline widget matching CesiumJS `Widgets/Timeline/Timeline.js` (Apache-2.0).

use cesium_math::{Clock, DeveloperError, JulianDate, ListenerId};
use std::cell::RefCell;
use std::rc::Rc;

/// Highlight range on the timeline.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineHighlightRange {
    pub color: String,
    pub height_in_px: f64,
    pub base: f64,
}

impl TimelineHighlightRange {
    pub fn new(color: impl Into<String>, height_in_px: f64, base: f64) -> Self {
        Self {
            color: color.into(),
            height_in_px,
            base,
        }
    }
}

/// A track interval on the timeline.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineTrack {
    pub interval_start: JulianDate,
    pub interval_stop: JulianDate,
    pub height: f64,
    pub color: String,
    pub background_color: String,
}

impl TimelineTrack {
    pub fn new(
        interval_start: JulianDate,
        interval_stop: JulianDate,
        height: f64,
        color: impl Into<String>,
        background_color: impl Into<String>,
    ) -> Self {
        Self {
            interval_start,
            interval_stop,
            height,
            color: color.into(),
            background_color: background_color.into(),
        }
    }
}

/// Simulated container element for Timeline widget.
#[derive(Clone, Debug)]
pub struct TimelineContainer {
    pub id: String,
    pub client_width: f64,
    pub client_height: f64,
    pub children: Vec<String>,
}

impl TimelineContainer {
    /// Creates a new container with dimension 1x1 default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            client_width: 1.0,
            client_height: 1.0,
            children: Vec::new(),
        }
    }

    /// Sets container size in px.
    pub fn set_size(&mut self, width: f64, height: f64) {
        self.client_width = width;
        self.client_height = height;
    }
}

/// The Timeline is a widget for displaying and controlling the current scene time.
pub struct Timeline {
    container: Option<Rc<RefCell<TimelineContainer>>>,
    clock: Rc<RefCell<Clock>>,
    start_julian: JulianDate,
    end_julian: JulianDate,
    time_bar_seconds_span: f64,
    needle_pos_px: f64,
    highlight_ranges: Vec<TimelineHighlightRange>,
    track_list: Vec<TimelineTrack>,
    last_width: Option<f64>,
    last_height: Option<f64>,
    listener_id: Option<ListenerId>,
    is_destroyed: bool,
}

impl Timeline {
    /// Creates a new `Timeline`.
    pub fn new(
        container: Option<Rc<RefCell<TimelineContainer>>>,
        clock: Option<Rc<RefCell<Clock>>>,
    ) -> Result<Self, DeveloperError> {
        let container = container.ok_or_else(|| DeveloperError::new("container is required."))?;
        let clock = clock.ok_or_else(|| DeveloperError::new("clock is required."))?;

        let (clock_start, clock_stop) = {
            let clk = clock.borrow();
            (clk.start_time(), clk.stop_time())
        };

        if clock_stop <= clock_start {
            return Err(DeveloperError::new(
                "Start time must come before end time.",
            ));
        }

        container
            .borrow_mut()
            .children
            .push("cesium-timeline-main".to_string());

        let time_bar_seconds_span = clock_stop.seconds_difference(&clock_start);

        let mut timeline = Self {
            container: Some(container),
            clock: Rc::clone(&clock),
            start_julian: clock_start,
            end_julian: clock_stop,
            time_bar_seconds_span,
            needle_pos_px: 0.0,
            highlight_ranges: Vec::new(),
            track_list: Vec::new(),
            last_width: None,
            last_height: None,
            listener_id: None,
            is_destroyed: false,
        };

        timeline.update_from_clock();
        timeline.resize();

        Ok(timeline)
    }

    /// Creates a `Timeline` from a container ID.
    pub fn from_id(
        container_id: &str,
        clock: Option<Rc<RefCell<Clock>>>,
        lookup: impl Fn(&str) -> Option<Rc<RefCell<TimelineContainer>>>,
    ) -> Result<Self, DeveloperError> {
        if container_id.is_empty() {
            return Err(DeveloperError::new("container is required."));
        }
        let container = lookup(container_id).ok_or_else(|| {
            DeveloperError::new(format!(
                "Element with id \"{container_id}\" does not exist in the document."
            ))
        })?;
        Self::new(Some(container), clock)
    }

    /// Gets the parent container.
    pub fn container(&self) -> Option<Rc<RefCell<TimelineContainer>>> {
        self.container.clone()
    }

    /// Gets the clock.
    pub fn clock(&self) -> Rc<RefCell<Clock>> {
        Rc::clone(&self.clock)
    }

    /// Gets needle position in container X coordinate.
    pub fn needle_pos_px(&self) -> f64 {
        self.needle_pos_px
    }

    /// Gets start JulianDate.
    pub fn start_julian(&self) -> JulianDate {
        self.start_julian
    }

    /// Gets end JulianDate.
    pub fn end_julian(&self) -> JulianDate {
        self.end_julian
    }

    /// Sets the view to the provided times.
    pub fn zoom_to(
        &mut self,
        start_time: JulianDate,
        stop_time: JulianDate,
    ) -> Result<(), DeveloperError> {
        if stop_time <= start_time {
            return Err(DeveloperError::new(
                "Start time must come before end time.",
            ));
        }

        self.start_julian = start_time;
        self.end_julian = stop_time;
        self.time_bar_seconds_span = stop_time.seconds_difference(&start_time);
        self.update_from_clock();
        self.resize();
        Ok(())
    }

    /// Resizes the widget to fit container dimensions.
    pub fn resize(&mut self) {
        if let Some(container) = &self.container {
            let (w, h) = {
                let c = container.borrow();
                (c.client_width, c.client_height)
            };

            if self.last_width == Some(w) && self.last_height == Some(h) {
                return;
            }

            self.last_width = Some(w);
            self.last_height = Some(h);
            self.update_from_clock();
        }
    }

    /// Updates needle position from current clock time.
    pub fn update_from_clock(&mut self) {
        let current_time = self.clock.borrow().current_time();
        let diff = current_time.seconds_difference(&self.start_julian);
        let ratio = if self.time_bar_seconds_span > 0.0 {
            (diff / self.time_bar_seconds_span).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let width = self
            .container
            .as_ref()
            .map(|c| c.borrow().client_width)
            .unwrap_or(1.0);
        self.needle_pos_px = ratio * width;
    }

    /// Adds a highlight range.
    pub fn add_highlight_range(
        &mut self,
        color: impl Into<String>,
        height_in_px: f64,
        base: f64,
    ) -> TimelineHighlightRange {
        let range = TimelineHighlightRange::new(color, height_in_px, base);
        self.highlight_ranges.push(range.clone());
        self.resize();
        range
    }

    /// Adds a track.
    pub fn add_track(
        &mut self,
        interval_start: JulianDate,
        interval_stop: JulianDate,
        height_in_px: f64,
        color: impl Into<String>,
        background_color: impl Into<String>,
    ) -> TimelineTrack {
        let track = TimelineTrack::new(
            interval_start,
            interval_stop,
            height_in_px,
            color,
            background_color,
        );
        self.track_list.push(track.clone());
        self.last_height = None;
        self.resize();
        track
    }

    /// Returns true if this widget was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys this widget and cleans up container elements.
    pub fn destroy(&mut self) {
        if let Some(id) = self.listener_id.take() {
            self.clock.borrow().on_tick.remove_listener(id);
        }
        if let Some(container) = self.container.take() {
            container.borrow_mut().children.clear();
        }
        self.is_destroyed = true;
    }
}
