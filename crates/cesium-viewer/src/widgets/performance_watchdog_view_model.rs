//! PerformanceWatchdogViewModel matching CesiumJS `Widgets/PerformanceWatchdog/PerformanceWatchdogViewModel.js` (Apache-2.0).

use crate::widgets::create_command::{create_command, Command};
use cesium_math::{DeveloperError, ListenerId};
use cesium_renderer::frame_rate_monitor::FrameRateMonitor;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub const DEFAULT_LOW_FRAME_RATE_MESSAGE: &str =
    "This application appears to be performing poorly on your system.  Please try using a different web browser or updating your video drivers.";

/// Simulated or actual Scene state for performance monitoring.
#[derive(Clone, Debug)]
pub struct WatchdogScene {
    pub monitor: Rc<RefCell<FrameRateMonitor>>,
    pub is_destroyed: bool,
}

impl Default for WatchdogScene {
    fn default() -> Self {
        Self::new()
    }
}

impl WatchdogScene {
    /// Creates a new `WatchdogScene`.
    pub fn new() -> Self {
        Self {
            monitor: Rc::new(RefCell::new(FrameRateMonitor::default())),
            is_destroyed: false,
        }
    }

    /// Gets the underlying [`FrameRateMonitor`].
    pub fn monitor(&self) -> Rc<RefCell<FrameRateMonitor>> {
        Rc::clone(&self.monitor)
    }

    /// Simulates rendering a frame at the given timestamp in milliseconds.
    pub fn render_at(&self, timestamp_ms: f64) {
        let _ = self.monitor.borrow_mut().update(timestamp_ms);
    }

    /// Simulates rendering a frame using the current system timestamp.
    pub fn render(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0);
        #[cfg(target_arch = "wasm32")]
        let ms = js_sys::Date::now();
        self.render_at(ms);
    }

    /// Destroys the scene and monitor.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.monitor.borrow_mut().destroy();
    }
}

/// Initialization options for [`PerformanceWatchdogViewModel`].
#[derive(Clone, Debug, Default)]
pub struct PerformanceWatchdogViewModelOptions {
    /// The Scene instance for which to monitor performance.
    pub scene: Option<Rc<RefCell<WatchdogScene>>>,
    /// The message to display when a low frame rate is detected.
    pub low_frame_rate_message: Option<String>,
}

/// View model for [`PerformanceWatchdog`].
pub struct PerformanceWatchdogViewModel {
    scene: Rc<RefCell<WatchdogScene>>,
    pub low_frame_rate_message: String,
    low_frame_rate_message_dismissed: Rc<Cell<bool>>,
    showing_low_frame_rate_message: Rc<Cell<bool>>,
    dismiss_message: Command,
    unsubscribe_low: Option<ListenerId>,
    unsubscribe_nominal: Option<ListenerId>,
    is_destroyed: bool,
}

impl PerformanceWatchdogViewModel {
    /// Creates a new `PerformanceWatchdogViewModel`.
    pub fn new(
        options: Option<PerformanceWatchdogViewModelOptions>,
    ) -> Result<Self, DeveloperError> {
        let opts = options.ok_or_else(|| DeveloperError::new("options.scene is required."))?;
        let scene = opts
            .scene
            .ok_or_else(|| DeveloperError::new("options.scene is required."))?;

        let low_frame_rate_message = opts
            .low_frame_rate_message
            .unwrap_or_else(|| DEFAULT_LOW_FRAME_RATE_MESSAGE.to_string());

        let low_frame_rate_message_dismissed = Rc::new(Cell::new(false));
        let showing_low_frame_rate_message = Rc::new(Cell::new(false));

        let dismissed_clone = Rc::clone(&low_frame_rate_message_dismissed);
        let showing_clone = Rc::clone(&showing_low_frame_rate_message);
        let dismiss_message = create_command(move |_| {
            showing_clone.set(false);
            dismissed_clone.set(true);
        });

        let monitor = scene.borrow().monitor();

        let showing_for_low = Rc::clone(&showing_low_frame_rate_message);
        let dismissed_for_low = Rc::clone(&low_frame_rate_message_dismissed);
        let unsubscribe_low = monitor.borrow().low_frame_rate().add_listener(move |_| {
            if !dismissed_for_low.get() {
                showing_for_low.set(true);
            }
        });

        let showing_for_nom = Rc::clone(&showing_low_frame_rate_message);
        let unsubscribe_nominal = monitor
            .borrow()
            .nominal_frame_rate()
            .add_listener(move |_| {
                showing_for_nom.set(false);
            });

        Ok(Self {
            scene,
            low_frame_rate_message,
            low_frame_rate_message_dismissed,
            showing_low_frame_rate_message,
            dismiss_message,
            unsubscribe_low: Some(unsubscribe_low),
            unsubscribe_nominal: Some(unsubscribe_nominal),
            is_destroyed: false,
        })
    }

    /// Gets the scene associated with this view model.
    pub fn scene(&self) -> Rc<RefCell<WatchdogScene>> {
        Rc::clone(&self.scene)
    }

    /// Gets whether the low frame rate message was dismissed by the user.
    pub fn low_frame_rate_message_dismissed(&self) -> bool {
        self.low_frame_rate_message_dismissed.get()
    }

    /// Gets whether the low frame rate message is currently shown.
    pub fn showing_low_frame_rate_message(&self) -> bool {
        self.showing_low_frame_rate_message.get()
    }

    /// Gets the command that dismisses the message.
    pub fn dismiss_message(&self) -> Command {
        self.dismiss_message.clone()
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys the view model and unregisters listeners.
    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.is_destroyed = true;
            let monitor = self.scene.borrow().monitor();
            if let Some(id) = self.unsubscribe_low.take() {
                monitor.borrow().low_frame_rate().remove_listener(id);
            }
            if let Some(id) = self.unsubscribe_nominal.take() {
                monitor.borrow().nominal_frame_rate().remove_listener(id);
            }
        }
    }
}

impl Drop for PerformanceWatchdogViewModel {
    fn drop(&mut self) {
        self.destroy();
    }
}
