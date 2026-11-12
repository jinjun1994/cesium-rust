//! Globe implementation matching CesiumJS `Scene/Globe.js` (Apache-2.0).

use cesium_math::{Cartesian3, Color, Ellipsoid, Event, GeographicTilingScheme, SceneMode};

use crate::commands::DrawCommand;
use crate::globe_surface_tile::GlobeSurfaceTile;
use crate::globe_surface_tile_provider::{
    GlobeSurfaceTileProvider, GlobeSurfaceTileProviderOptions, ImageryLayerRecord,
};
use crate::globe_translucency_state::GlobeTranslucencyConfig;

/// The globe rendered in the scene, managing surface tiles, terrain provider, and imagery layers.
#[derive(Clone, Debug)]
pub struct Globe {
    pub ellipsoid: Ellipsoid,
    pub show: bool,
    pub enable_lighting: bool,
    pub dynamic_atmosphere_lighting: bool,
    pub dynamic_atmosphere_lighting_from_sun: bool,
    pub show_water_effect: bool,
    pub lambert_diffuse_multiplier: f64,
    pub vertex_shadow_darkness: f64,
    pub brightness_shift: f64,
    pub base_color: Color,
    pub depth_test_against_terrain: bool,
    pub translucency: GlobeTranslucencyConfig,
    pub terrain_provider: String,
    pub terrain_provider_changed: Event<String>,
    pub imagery_layers_updated_event: Event<()>,
    pub tile_load_queue_high: usize,
    pub tile_load_queue_medium: usize,
    pub tile_load_queue_low: usize,
    surface: GlobeSurfaceTileProvider,
    draw_commands: Vec<DrawCommand>,
    destroyed: bool,
}

impl Default for Globe {
    fn default() -> Self {
        Self::new(Ellipsoid::default())
    }
}

impl Globe {
    /// Creates a new `Globe` with the specified ellipsoid and default providers.
    pub fn new(ellipsoid: Ellipsoid) -> Self {
        let surface = GlobeSurfaceTileProvider::new(GlobeSurfaceTileProviderOptions {
            terrain_provider_name: Some("EllipsoidTerrainProvider".to_string()),
            ..Default::default()
        })
        .expect("valid default surface tile provider options");

        Self {
            ellipsoid,
            show: true,
            enable_lighting: false,
            dynamic_atmosphere_lighting: false,
            dynamic_atmosphere_lighting_from_sun: false,
            show_water_effect: true,
            lambert_diffuse_multiplier: 0.0,
            vertex_shadow_darkness: 0.3,
            brightness_shift: 0.0,
            base_color: Color::new(0.0, 0.0, 0.5, 1.0),
            depth_test_against_terrain: false,
            translucency: GlobeTranslucencyConfig::default(),
            terrain_provider: "EllipsoidTerrainProvider".to_string(),
            terrain_provider_changed: Event::new(),
            imagery_layers_updated_event: Event::new(),
            tile_load_queue_high: 0,
            tile_load_queue_medium: 0,
            tile_load_queue_low: 0,
            surface,
            draw_commands: Vec::new(),
            destroyed: false,
        }
    }

    /// Returns whether the globe has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    /// Returns whether all tile load queues are empty and tiles are fully loaded.
    pub fn tiles_loaded(&self) -> bool {
        self.tile_load_queue_high == 0
            && self.tile_load_queue_medium == 0
            && self.tile_load_queue_low == 0
    }

    /// Returns the surface tile provider.
    pub fn surface(&self) -> &GlobeSurfaceTileProvider {
        &self.surface
    }

    /// Returns a mutable reference to the surface tile provider.
    pub fn surface_mut(&mut self) -> &mut GlobeSurfaceTileProvider {
        &mut self.surface
    }

    /// Returns the list of registered imagery layer descriptors.
    pub fn imagery_layers(&self) -> &[ImageryLayerRecord] {
        self.surface.imagery_layers()
    }

    /// Sets the terrain provider and fires `terrain_provider_changed`.
    pub fn set_terrain_provider(&mut self, provider_name: impl Into<String>) {
        let name = provider_name.into();
        self.terrain_provider = name.clone();
        self.surface.terrain_provider_name = name.clone();
        // Changing terrain provider invalidates loaded tiles until re-evaluated.
        self.tile_load_queue_medium = 1;
        self.terrain_provider_changed.raise_event(&name);
    }

    /// Adds an imagery layer to the globe and fires `imagery_layers_updated_event`.
    pub fn add_imagery_layer(&mut self, layer: ImageryLayerRecord) {
        self.surface.add_imagery_layer(layer);
        self.imagery_layers_updated_event.raise_event(&());
    }

    /// Removes an imagery layer from the globe and fires `imagery_layers_updated_event`.
    pub fn remove_imagery_layer(&mut self, layer_id: u64) {
        self.surface.remove_imagery_layer(layer_id);
        self.imagery_layers_updated_event.raise_event(&());
    }

    /// Raises an imagery layer to the top and fires `imagery_layers_updated_event`.
    pub fn raise_imagery_layer(&mut self, layer_id: u64) {
        self.surface.raise_layer_to_top(layer_id);
        self.imagery_layers_updated_event.raise_event(&());
    }

    /// Updates visibility for a registered imagery layer and fires `imagery_layers_updated_event`.
    pub fn set_layer_show(&mut self, layer_id: u64, show: bool) {
        self.surface.set_layer_show(layer_id, show);
        self.imagery_layers_updated_event.raise_event(&());
    }

    /// Adds a surface tile to the underlying surface provider.
    pub fn add_tile(&mut self, tile: GlobeSurfaceTile) -> usize {
        let center = GeographicTilingScheme::default()
            .tile_rectangle(tile.x, tile.y, tile.level)
            .and_then(|rect| self.ellipsoid.cartographic_to_cartesian(rect.center()))
            .map(Cartesian3::from)
            .unwrap_or(Cartesian3::ZERO);
        self.surface.add_tile(tile, center)
    }

    /// Updates surface state and produces draw commands for the current frame.
    pub fn update(&mut self, mode: SceneMode) -> &[DrawCommand] {
        self.draw_commands.clear();
        if self.destroyed || !self.show {
            return &self.draw_commands;
        }

        self.surface.base_color = self.base_color;
        self.surface.show_water_effect = self.show_water_effect;
        self.surface.enable_lighting = self.enable_lighting;
        self.surface.brightness_shift = self.brightness_shift;
        self.surface.vertex_shadow_darkness = self.vertex_shadow_darkness;

        self.surface.begin_update();
        for i in 0..self.surface.tiles().len() {
            self.surface.show_tile_this_frame(i);
        }
        let cmds = self.surface.end_update(mode);
        self.draw_commands.extend_from_slice(cmds);
        &self.draw_commands
    }

    /// Destroys the globe and releases underlying surface resources.
    pub fn destroy(&mut self) {
        self.surface.destroy();
        self.draw_commands.clear();
        self.destroyed = true;
    }
}
