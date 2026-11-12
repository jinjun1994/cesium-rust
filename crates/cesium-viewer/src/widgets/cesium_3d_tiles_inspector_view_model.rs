use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cesium_math::DeveloperError;

/// Debug configuration options mirroring 3D Tiles inspection properties.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TilesetDebugOptions {
    pub debug_colorize_tiles: bool,
    pub debug_wireframe: bool,
    pub debug_show_bounding_volume: bool,
    pub debug_show_content_bounding_volume: bool,
    pub debug_show_viewer_request_volume: bool,
    pub debug_picked_tile_label_only: bool,
    pub debug_show_geometric_error: bool,
    pub debug_show_rendering_statistics: bool,
    pub debug_show_memory_usage: bool,
    pub debug_show_url: bool,
    pub debug_freeze_frame: bool,
    pub maximum_screen_space_error: f64,
    pub dynamic_screen_space_error: bool,
    pub dynamic_screen_space_error_factor: f64,
    pub dynamic_screen_space_error_density: f64,
    pub point_cloud_attenuation: bool,
    pub point_cloud_geometric_error_scale: f64,
    pub point_cloud_maximum_attenuation: Option<f64>,
    pub point_cloud_base_resolution: Option<f64>,
    pub point_cloud_eye_dome_lighting: bool,
    pub point_cloud_eye_dome_lighting_strength: f64,
    pub point_cloud_eye_dome_lighting_radius: f64,
    pub properties: Vec<String>,
    pub style: Option<serde_json::Value>,
}

impl TilesetDebugOptions {
    pub fn new() -> Self {
        Self {
            maximum_screen_space_error: 16.0,
            dynamic_screen_space_error_factor: 4.0,
            dynamic_screen_space_error_density: 0.00005,
            point_cloud_geometric_error_scale: 1.0,
            point_cloud_eye_dome_lighting_strength: 1.0,
            point_cloud_eye_dome_lighting_radius: 1.0,
            properties: vec![
                "id".to_string(),
                "Longitude".to_string(),
                "Latitude".to_string(),
                "Height".to_string(),
            ],
            ..Default::default()
        }
    }
}

/// A target tileset representation for the 3D Tiles Inspector.
#[derive(Clone, Debug)]
pub struct InspectorTileset {
    pub options: Rc<RefCell<TilesetDebugOptions>>,
}

impl InspectorTileset {
    pub fn new() -> Self {
        Self {
            options: Rc::new(RefCell::new(TilesetDebugOptions::new())),
        }
    }
}

impl Default for InspectorTileset {
    fn default() -> Self {
        Self::new()
    }
}

/// View model for `Cesium3DTilesInspector`.
#[derive(Clone, Debug)]
pub struct Cesium3DTilesInspectorViewModel {
    scene_id: String,
    performance_container_id: String,
    tileset: RefCell<Option<InspectorTileset>>,
    style_string: RefCell<String>,
    is_destroyed: Cell<bool>,
}

impl Cesium3DTilesInspectorViewModel {
    /// Creates a new `Cesium3DTilesInspectorViewModel`.
    pub fn new(
        scene_id: Option<impl Into<String>>,
        performance_container_id: Option<impl Into<String>>,
    ) -> Result<Self, DeveloperError> {
        let scene_id = scene_id
            .map(Into::into)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| DeveloperError::new("scene is required."))?;
        let perf_id = performance_container_id
            .map(Into::into)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| DeveloperError::new("performanceContainer is required."))?;

        Ok(Self {
            scene_id,
            performance_container_id: perf_id,
            tileset: RefCell::new(None),
            style_string: RefCell::new(String::new()),
            is_destroyed: Cell::new(false),
        })
    }

    /// Gets scene identifier.
    pub fn scene_id(&self) -> &str {
        &self.scene_id
    }

    /// Gets performance container identifier.
    pub fn performance_container_id(&self) -> &str {
        &self.performance_container_id
    }

    /// Gets current inspected tileset.
    pub fn tileset(&self) -> Option<InspectorTileset> {
        self.tileset.borrow().clone()
    }

    /// Sets current inspected tileset and synchronizes style string.
    pub fn set_tileset(&self, tileset: Option<InspectorTileset>) {
        if let Some(ref t) = tileset {
            if let Some(ref style) = t.options.borrow().style {
                *self.style_string.borrow_mut() = style.to_string();
            }
        }
        *self.tileset.borrow_mut() = tileset;
    }

    /// Gets properties defined in the current tileset.
    pub fn properties(&self) -> Vec<String> {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().properties.clone())
            .unwrap_or_default()
    }

    /// Colorize tiles debug display.
    pub fn colorize(&self) -> bool {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().debug_colorize_tiles)
            .unwrap_or(false)
    }

    pub fn set_colorize(&self, value: bool) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().debug_colorize_tiles = value;
        }
    }

    /// Wireframe debug display.
    pub fn wireframe(&self) -> bool {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().debug_wireframe)
            .unwrap_or(false)
    }

    pub fn set_wireframe(&self, value: bool) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().debug_wireframe = value;
        }
    }

    /// Show bounding volumes.
    pub fn show_bounding_volumes(&self) -> bool {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().debug_show_bounding_volume)
            .unwrap_or(false)
    }

    pub fn set_show_bounding_volumes(&self, value: bool) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().debug_show_bounding_volume = value;
        }
    }

    /// Show content bounding volumes.
    pub fn show_content_bounding_volumes(&self) -> bool {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().debug_show_content_bounding_volume)
            .unwrap_or(false)
    }

    pub fn set_show_content_bounding_volumes(&self, value: bool) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().debug_show_content_bounding_volume = value;
        }
    }

    /// Show viewer request volumes.
    pub fn show_request_volumes(&self) -> bool {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().debug_show_viewer_request_volume)
            .unwrap_or(false)
    }

    pub fn set_show_request_volumes(&self, value: bool) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().debug_show_viewer_request_volume = value;
        }
    }

    /// Show debug label for picked tile only.
    pub fn show_only_picked_tile_debug_label(&self) -> bool {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().debug_picked_tile_label_only)
            .unwrap_or(false)
    }

    pub fn set_show_only_picked_tile_debug_label(&self, value: bool) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().debug_picked_tile_label_only = value;
        }
    }

    /// Show geometric error.
    pub fn show_geometric_error(&self) -> bool {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().debug_show_geometric_error)
            .unwrap_or(false)
    }

    pub fn set_show_geometric_error(&self, value: bool) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().debug_show_geometric_error = value;
        }
    }

    /// Show rendering statistics.
    pub fn show_rendering_statistics(&self) -> bool {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().debug_show_rendering_statistics)
            .unwrap_or(false)
    }

    pub fn set_show_rendering_statistics(&self, value: bool) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().debug_show_rendering_statistics = value;
        }
    }

    /// Show memory usage.
    pub fn show_memory_usage(&self) -> bool {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().debug_show_memory_usage)
            .unwrap_or(false)
    }

    pub fn set_show_memory_usage(&self, value: bool) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().debug_show_memory_usage = value;
        }
    }

    /// Show tile URL.
    pub fn show_url(&self) -> bool {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().debug_show_url)
            .unwrap_or(false)
    }

    pub fn set_show_url(&self, value: bool) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().debug_show_url = value;
        }
    }

    /// Point cloud shading attenuation.
    pub fn point_cloud_shading(&self) -> bool {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().point_cloud_attenuation)
            .unwrap_or(false)
    }

    pub fn set_point_cloud_shading(&self, value: bool) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().point_cloud_attenuation = value;
        }
    }

    /// Point cloud geometric error scale.
    pub fn geometric_error_scale(&self) -> f64 {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().point_cloud_geometric_error_scale)
            .unwrap_or(1.0)
    }

    pub fn set_geometric_error_scale(&self, value: f64) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().point_cloud_geometric_error_scale = value;
        }
    }

    /// Point cloud maximum attenuation.
    pub fn maximum_attenuation(&self) -> Option<f64> {
        self.tileset
            .borrow()
            .as_ref()
            .and_then(|t| t.options.borrow().point_cloud_maximum_attenuation)
    }

    pub fn set_maximum_attenuation(&self, value: f64) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().point_cloud_maximum_attenuation =
                if value > 0.0 { Some(value) } else { None };
        }
    }

    /// Point cloud base resolution.
    pub fn base_resolution(&self) -> Option<f64> {
        self.tileset
            .borrow()
            .as_ref()
            .and_then(|t| t.options.borrow().point_cloud_base_resolution)
    }

    pub fn set_base_resolution(&self, value: f64) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().point_cloud_base_resolution =
                if value > 0.0 { Some(value) } else { None };
        }
    }

    /// Point cloud EDL.
    pub fn eye_dome_lighting(&self) -> bool {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().point_cloud_eye_dome_lighting)
            .unwrap_or(false)
    }

    pub fn set_eye_dome_lighting(&self, value: bool) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().point_cloud_eye_dome_lighting = value;
        }
    }

    /// Point cloud EDL strength.
    pub fn eye_dome_lighting_strength(&self) -> f64 {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().point_cloud_eye_dome_lighting_strength)
            .unwrap_or(1.0)
    }

    pub fn set_eye_dome_lighting_strength(&self, value: f64) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().point_cloud_eye_dome_lighting_strength = value;
        }
    }

    /// Point cloud EDL radius.
    pub fn eye_dome_lighting_radius(&self) -> f64 {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().point_cloud_eye_dome_lighting_radius)
            .unwrap_or(1.0)
    }

    pub fn set_eye_dome_lighting_radius(&self, value: f64) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().point_cloud_eye_dome_lighting_radius = value;
        }
    }

    /// Freeze frame debug option.
    pub fn freeze_frame(&self) -> bool {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().debug_freeze_frame)
            .unwrap_or(false)
    }

    pub fn set_freeze_frame(&self, value: bool) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().debug_freeze_frame = value;
        }
    }

    /// Maximum screen space error.
    pub fn maximum_screen_space_error(&self) -> f64 {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().maximum_screen_space_error)
            .unwrap_or(16.0)
    }

    pub fn set_maximum_screen_space_error(&self, value: f64) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().maximum_screen_space_error = value;
        }
    }

    /// Dynamic screen space error toggle.
    pub fn dynamic_screen_space_error(&self) -> bool {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().dynamic_screen_space_error)
            .unwrap_or(false)
    }

    pub fn set_dynamic_screen_space_error(&self, value: bool) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().dynamic_screen_space_error = value;
        }
    }

    /// Dynamic screen space error factor.
    pub fn dynamic_screen_space_error_factor(&self) -> f64 {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().dynamic_screen_space_error_factor)
            .unwrap_or(4.0)
    }

    pub fn set_dynamic_screen_space_error_factor(&self, value: f64) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().dynamic_screen_space_error_factor = value;
        }
    }

    /// Dynamic screen space error density.
    pub fn dynamic_screen_space_error_density(&self) -> f64 {
        self.tileset
            .borrow()
            .as_ref()
            .map(|t| t.options.borrow().dynamic_screen_space_error_density)
            .unwrap_or(0.00005)
    }

    pub fn set_dynamic_screen_space_error_density(&self, value: f64) {
        if let Some(ref t) = *self.tileset.borrow() {
            t.options.borrow_mut().dynamic_screen_space_error_density = value;
        }
    }

    /// Dynamic screen space error density slider value (exponential mapping: density = slider^6).
    pub fn dynamic_screen_space_error_density_slider_value(&self) -> f64 {
        self.dynamic_screen_space_error_density().powf(1.0 / 6.0)
    }

    pub fn set_dynamic_screen_space_error_density_slider_value(&self, slider_value: f64) {
        let density = slider_value.powi(6);
        self.set_dynamic_screen_space_error_density(density);
    }

    /// Gets style string.
    pub fn style_string(&self) -> String {
        self.style_string.borrow().clone()
    }

    /// Sets style string.
    pub fn set_style_string(&self, s: impl Into<String>) {
        *self.style_string.borrow_mut() = s.into();
    }

    /// Compiles style string into the tileset style.
    pub fn compile_style(&self) {
        let text = self.style_string.borrow();
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(ref t) = *self.tileset.borrow() {
                t.options.borrow_mut().style = Some(json);
            }
        }
    }

    /// Synchronizes viewModel from tileset.
    pub fn update(&self) {
        if let Some(ref t) = *self.tileset.borrow() {
            if let Some(ref style) = t.options.borrow().style {
                *self.style_string.borrow_mut() = style.to_string();
            }
        }
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
