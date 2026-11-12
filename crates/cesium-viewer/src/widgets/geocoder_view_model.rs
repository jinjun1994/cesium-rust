use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cesium_math::{Cartesian3, DeveloperError, Event};

/// A geocoded destination result.
#[derive(Clone, Debug, PartialEq)]
pub struct GeocoderResult {
    pub display_name: String,
    pub destination: Cartesian3,
}

impl GeocoderResult {
    pub fn new(display_name: impl Into<String>, destination: Cartesian3) -> Self {
        Self {
            display_name: display_name.into(),
            destination,
        }
    }
}

pub type GeocoderServiceFn = Rc<dyn Fn(&str) -> Vec<GeocoderResult>>;
pub type DestinationFoundFn = Rc<dyn Fn(&GeocoderViewModel, &GeocoderResult)>;

/// Options for `GeocoderViewModel`.
#[derive(Clone, Default)]
pub struct GeocoderViewModelOptions {
    pub scene_id: Option<String>,
    pub flight_duration: Option<f64>,
    pub geocoder_services: Vec<GeocoderServiceFn>,
    pub destination_found: Option<DestinationFoundFn>,
}

/// View model for `Geocoder`.
#[derive(Clone)]
pub struct GeocoderViewModel {
    scene_id: String,
    flight_duration: Cell<f64>,
    keep_expanded: Cell<bool>,
    search_text: RefCell<String>,
    is_searching: Cell<bool>,
    suggestions: RefCell<Vec<GeocoderResult>>,
    geocoder_services: RefCell<Vec<GeocoderServiceFn>>,
    destination_found: RefCell<Option<DestinationFoundFn>>,
    pub complete: Event<()>,
    is_destroyed: Cell<bool>,
}

impl GeocoderViewModel {
    /// Creates a new `GeocoderViewModel`.
    pub fn new(options: GeocoderViewModelOptions) -> Result<Self, DeveloperError> {
        let scene_id = options
            .scene_id
            .filter(|s| !s.is_empty())
            .ok_or_else(|| DeveloperError::new("scene is required."))?;

        let flight_duration = options.flight_duration.unwrap_or(1.5);
        if flight_duration < 0.0 {
            return Err(DeveloperError::new("flightDuration must be positive."));
        }

        Ok(Self {
            scene_id,
            flight_duration: Cell::new(flight_duration),
            keep_expanded: Cell::new(false),
            search_text: RefCell::new(String::new()),
            is_searching: Cell::new(false),
            suggestions: RefCell::new(Vec::new()),
            geocoder_services: RefCell::new(options.geocoder_services),
            destination_found: RefCell::new(options.destination_found),
            complete: Event::new(),
            is_destroyed: Cell::new(false),
        })
    }

    /// Gets scene ID.
    pub fn scene_id(&self) -> &str {
        &self.scene_id
    }

    /// Gets flight duration in seconds.
    pub fn flight_duration(&self) -> f64 {
        self.flight_duration.get()
    }

    /// Sets flight duration in seconds.
    pub fn set_flight_duration(&self, duration: f64) -> Result<(), DeveloperError> {
        if duration < 0.0 {
            return Err(DeveloperError::new("flightDuration must be positive."));
        }
        self.flight_duration.set(duration);
        Ok(())
    }

    /// Gets keep expanded status.
    pub fn keep_expanded(&self) -> bool {
        self.keep_expanded.get()
    }

    /// Sets keep expanded status.
    pub fn set_keep_expanded(&self, value: bool) {
        self.keep_expanded.set(value);
    }

    /// Gets search text.
    pub fn search_text(&self) -> String {
        self.search_text.borrow().clone()
    }

    /// Sets search text.
    pub fn set_search_text(&self, text: impl Into<String>) {
        *self.search_text.borrow_mut() = text.into();
    }

    /// Gets current suggestions.
    pub fn suggestions(&self) -> Vec<GeocoderResult> {
        self.suggestions.borrow().clone()
    }

    /// Updates search suggestions from registered geocoder services.
    pub fn update_search_suggestions(&self) {
        let text = self.search_text.borrow().clone();
        if text.trim().is_empty() {
            self.suggestions.borrow_mut().clear();
            return;
        }

        let mut results = Vec::new();
        for svc in self.geocoder_services.borrow().iter() {
            let res = svc(&text);
            if !res.is_empty() {
                results = res;
                break;
            }
        }
        *self.suggestions.borrow_mut() = results;
    }

    /// Activates a selected search suggestion, updates search text and invokes destination found.
    pub fn activate_suggestion(&self, suggestion: &GeocoderResult) {
        *self.search_text.borrow_mut() = suggestion.display_name.clone();
        if let Some(ref cb) = *self.destination_found.borrow() {
            cb(self, suggestion);
        }
        self.complete.raise_event(&());
    }

    /// Executes search across geocoder services.
    pub fn search(&self) {
        self.is_searching.set(true);
        let text = self.search_text.borrow().clone();
        if !text.trim().is_empty() {
            for svc in self.geocoder_services.borrow().iter() {
                let results = svc(&text);
                if let Some(first) = results.first() {
                    if let Some(ref cb) = *self.destination_found.borrow() {
                        cb(self, first);
                    }
                    break;
                }
            }
        }
        self.is_searching.set(false);
        self.complete.raise_event(&());
    }

    /// Returns whether the view model has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.get()
    }

    /// Destroys the view model.
    pub fn destroy(&self) {
        self.is_destroyed.set(true);
    }
}
