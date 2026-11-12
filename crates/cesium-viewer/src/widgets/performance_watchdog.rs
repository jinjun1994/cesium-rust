//! PerformanceWatchdog widget matching CesiumJS `Widgets/PerformanceWatchdog/PerformanceWatchdog.js` (Apache-2.0).

use crate::widgets::performance_watchdog_view_model::{
    PerformanceWatchdogViewModel, PerformanceWatchdogViewModelOptions, WatchdogScene,
};
use cesium_math::DeveloperError;
use std::cell::RefCell;
use std::rc::Rc;

/// Simulated DOM element inside PerformanceWatchdog.
#[derive(Clone, Debug, Default)]
pub struct PerformanceWatchdogElement {
    pub tag: String,
    pub class_name: String,
    pub children: Vec<Rc<RefCell<PerformanceWatchdogElement>>>,
}

impl PerformanceWatchdogElement {
    /// Creates a new element.
    pub fn new(tag: impl Into<String>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            tag: tag.into(),
            class_name: String::new(),
            children: Vec::new(),
        }))
    }
}

/// Simulated container DOM element for PerformanceWatchdog.
#[derive(Clone, Debug, Default)]
pub struct PerformanceWatchdogContainer {
    pub id: String,
    pub children: Vec<Rc<RefCell<PerformanceWatchdogElement>>>,
}

impl PerformanceWatchdogContainer {
    /// Creates a new container with an ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
        }
    }
}

/// Initialization options for [`PerformanceWatchdog`].
#[derive(Clone, Debug, Default)]
pub struct PerformanceWatchdogOptions {
    /// The DOM element or ID that will contain the widget.
    pub container: Option<Rc<RefCell<PerformanceWatchdogContainer>>>,
    /// The Scene for which to monitor performance.
    pub scene: Option<Rc<RefCell<WatchdogScene>>>,
    /// The message to display when a low frame rate is detected.
    pub low_frame_rate_message: Option<String>,
}

/// Monitors performance of the application and displays a message if poor performance is detected.
pub struct PerformanceWatchdog {
    container: Option<Rc<RefCell<PerformanceWatchdogContainer>>>,
    element: Rc<RefCell<PerformanceWatchdogElement>>,
    view_model: PerformanceWatchdogViewModel,
    is_destroyed: bool,
}

impl PerformanceWatchdog {
    /// Creates a new `PerformanceWatchdog`.
    pub fn new(options: Option<PerformanceWatchdogOptions>) -> Result<Self, DeveloperError> {
        let opts = options.ok_or_else(|| DeveloperError::new("options.container is required."))?;
        let container = opts
            .container
            .ok_or_else(|| DeveloperError::new("options.container is required."))?;
        let scene = opts
            .scene
            .ok_or_else(|| DeveloperError::new("options.scene is required."))?;

        let view_model =
            PerformanceWatchdogViewModel::new(Some(PerformanceWatchdogViewModelOptions {
                scene: Some(scene),
                low_frame_rate_message: opts.low_frame_rate_message,
            }))?;

        let element = PerformanceWatchdogElement::new("div");
        element.borrow_mut().class_name = "cesium-performance-watchdog-message-area".to_string();

        let dismiss_button = PerformanceWatchdogElement::new("button");
        dismiss_button.borrow_mut().class_name =
            "cesium-performance-watchdog-message-dismiss".to_string();
        element.borrow_mut().children.push(dismiss_button);

        let message = PerformanceWatchdogElement::new("div");
        message.borrow_mut().class_name = "cesium-performance-watchdog-message".to_string();
        element.borrow_mut().children.push(message);

        container.borrow_mut().children.push(Rc::clone(&element));

        Ok(Self {
            container: Some(container),
            element,
            view_model,
            is_destroyed: false,
        })
    }

    /// Creates a `PerformanceWatchdog` from container ID.
    pub fn from_id(
        container_id: &str,
        options: Option<PerformanceWatchdogOptions>,
        lookup: impl Fn(&str) -> Option<Rc<RefCell<PerformanceWatchdogContainer>>>,
    ) -> Result<Self, DeveloperError> {
        if container_id.is_empty() {
            return Err(DeveloperError::new("options.container is required."));
        }
        let container = lookup(container_id).ok_or_else(|| {
            DeveloperError::new(format!(
                "Element with id \"{container_id}\" does not exist in the document."
            ))
        })?;
        let mut opts = options.unwrap_or_default();
        opts.container = Some(container);
        Self::new(Some(opts))
    }

    /// Gets the container element.
    pub fn container(&self) -> Option<Rc<RefCell<PerformanceWatchdogContainer>>> {
        self.container.clone()
    }

    /// Gets the root DOM element.
    pub fn element(&self) -> Rc<RefCell<PerformanceWatchdogElement>> {
        Rc::clone(&self.element)
    }

    /// Gets the view model.
    pub fn view_model(&self) -> &PerformanceWatchdogViewModel {
        &self.view_model
    }

    /// Gets the mutable view model.
    pub fn view_model_mut(&mut self) -> &mut PerformanceWatchdogViewModel {
        &mut self.view_model
    }

    /// Returns true if this widget was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys this widget and cleans up container children.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        if let Some(container) = self.container.take() {
            container.borrow_mut().children.clear();
        }
        self.view_model.destroy();
    }
}
