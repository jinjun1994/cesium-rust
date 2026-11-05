//! A 3D Tiles tileset represented by an Entity.
//! Direct port of CesiumJS `DataSources/Cesium3DTilesetGraphics.js`.

use std::rc::Rc;

use crate::{ConstantProperty, DeveloperError, DynProperty, Event, JulianDate, Mergeable};

/// Initialization options for Cesium3DTilesetGraphics.
#[derive(Default, Clone)]
pub struct Cesium3DTilesetGraphicsOptions {
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub uri: Option<Rc<dyn DynProperty<String>>>,
    pub maximum_screen_space_error: Option<Rc<dyn DynProperty<f64>>>,
}

/// Resolved values for Cesium3DTilesetGraphics at a specific time.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Cesium3DTilesetGraphicsValues {
    pub show: Option<bool>,
    pub uri: Option<String>,
    pub maximum_screen_space_error: Option<f64>,
}

/// A 3D Tiles tileset represented by an Entity.
#[derive(Clone, Default)]
pub struct Cesium3DTilesetGraphics {
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub uri: Option<Rc<dyn DynProperty<String>>>,
    pub maximum_screen_space_error: Option<Rc<dyn DynProperty<f64>>>,
    definition_changed: Event<()>,
}

impl Mergeable for Cesium3DTilesetGraphics {
    fn merge(&mut self, source: &Self) {
        self.merge(source);
    }
}

impl Cesium3DTilesetGraphics {
    /// Creates a new empty Cesium3DTilesetGraphics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a Cesium3DTilesetGraphics with provided options.
    pub fn from_options(options: Cesium3DTilesetGraphicsOptions) -> Self {
        Self {
            show: options.show,
            uri: options.uri,
            maximum_screen_space_error: options.maximum_screen_space_error,
            definition_changed: Event::new(),
        }
    }

    /// Creates a Cesium3DTilesetGraphics with constant values.
    pub fn from_constants(
        show: Option<bool>,
        uri: Option<String>,
        maximum_screen_space_error: Option<f64>,
    ) -> Self {
        Self {
            show: show.map(|v| Rc::new(ConstantProperty::new(Some(v))) as Rc<dyn DynProperty<bool>>),
            uri: uri.map(|v| Rc::new(ConstantProperty::new(Some(v))) as Rc<dyn DynProperty<String>>),
            maximum_screen_space_error: maximum_screen_space_error
                .map(|v| Rc::new(ConstantProperty::new(Some(v))) as Rc<dyn DynProperty<f64>>),
            definition_changed: Event::new(),
        }
    }

    /// Gets the event that is raised whenever a property is changed or modified.
    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    /// Duplicates into a provided result reference.
    pub fn clone_to(&self, result: &mut Self) {
        result.show = self.show.clone();
        result.uri = self.uri.clone();
        result.maximum_screen_space_error = self.maximum_screen_space_error.clone();
    }

    /// Assigns each unassigned property on this object to the value of the same property on the provided source.
    pub fn merge(&mut self, source: &Self) {
        if self.show.is_none() {
            self.show = source.show.clone();
        }
        if self.uri.is_none() {
            self.uri = source.uri.clone();
        }
        if self.maximum_screen_space_error.is_none() {
            self.maximum_screen_space_error = source.maximum_screen_space_error.clone();
        }
    }

    /// Merges with error checking.
    pub fn try_merge(&mut self, source: Option<&Self>) -> Result<(), DeveloperError> {
        let source = source.ok_or_else(|| DeveloperError::new("source is required."))?;
        self.merge(source);
        Ok(())
    }

    /// Evaluates all properties at the given time.
    pub fn get_value(&self, time: Option<&JulianDate>) -> Cesium3DTilesetGraphicsValues {
        Cesium3DTilesetGraphicsValues {
            show: self.show.as_ref().and_then(|p| p.get_value(time)),
            uri: self.uri.as_ref().and_then(|p| p.get_value(time)),
            maximum_screen_space_error: self
                .maximum_screen_space_error
                .as_ref()
                .and_then(|p| p.get_value(time)),
        }
    }
}
