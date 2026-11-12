//! GlobeSurfaceTileProvider matching CesiumJS `Scene/GlobeSurfaceTileProvider.js`.

use cesium_math::{Cartesian3, Color, DeveloperError, SceneMode, SplitDirection};

use crate::commands::{DrawCommand, DrawCommandOptions, Pass};
use crate::globe_surface_tile::GlobeSurfaceTile;
use crate::quadtree_tile_load_state::QuadtreeTileLoadState;

/// Options for constructing a `GlobeSurfaceTileProvider`.
#[derive(Clone, Debug, PartialEq)]
pub struct GlobeSurfaceTileProviderOptions {
    pub terrain_provider_name: Option<String>,
    pub imagery_layers_present: bool,
    pub surface_shader_set_present: bool,
    pub vector_provider_present: bool,
    pub base_color: Color,
    pub show_skirts: bool,
    pub back_face_culling: bool,
    pub enable_lighting: bool,
    pub has_water_mask: bool,
    pub show_water_effect: bool,
    pub fill_highlight_color: Option<Color>,
    pub hue_shift: f64,
    pub saturation_shift: f64,
    pub brightness_shift: f64,
}

impl Default for GlobeSurfaceTileProviderOptions {
    fn default() -> Self {
        Self {
            terrain_provider_name: Some("EllipsoidTerrainProvider".to_string()),
            imagery_layers_present: true,
            surface_shader_set_present: true,
            vector_provider_present: true,
            base_color: Color::new(0.0, 0.0, 0.5, 1.0),
            show_skirts: true,
            back_face_culling: true,
            enable_lighting: false,
            has_water_mask: false,
            show_water_effect: false,
            fill_highlight_color: None,
            hue_shift: 0.0,
            saturation_shift: 0.0,
            brightness_shift: 0.0,
        }
    }
}

/// Lightweight descriptor for an imagery layer tracked by `GlobeSurfaceTileProvider`.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageryLayerRecord {
    pub id: u64,
    pub show: bool,
    pub alpha: f64,
    pub split_direction: SplitDirection,
}

impl ImageryLayerRecord {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            show: true,
            alpha: 1.0,
            split_direction: SplitDirection::None,
        }
    }
}

/// A tile entry managed by `GlobeSurfaceTileProvider`.
#[derive(Clone, Debug)]
pub struct SurfaceTileEntry {
    pub tile: GlobeSurfaceTile,
    pub imagery_layer_ids: Vec<u64>,
    pub visible: bool,
    pub center: Cartesian3,
}

impl SurfaceTileEntry {
    pub fn new(tile: GlobeSurfaceTile, center: Cartesian3) -> Self {
        Self {
            tile,
            imagery_layer_ids: Vec::new(),
            visible: true,
            center,
        }
    }
}

/// Provides quadtree tiles representing the surface of the globe.
#[derive(Clone, Debug)]
pub struct GlobeSurfaceTileProvider {
    pub terrain_provider_name: String,
    pub base_color: Color,
    pub show_skirts: bool,
    pub back_face_culling: bool,
    pub enable_lighting: bool,
    pub has_water_mask: bool,
    pub show_water_effect: bool,
    pub fill_highlight_color: Option<Color>,
    pub hue_shift: f64,
    pub saturation_shift: f64,
    pub brightness_shift: f64,
    pub lighting_fade_out_distance: f64,
    pub lighting_fade_in_distance: f64,
    pub zoomed_out_ocean_specular_intensity: f64,
    pub vertex_shadow_darkness: f64,
    imagery_layers: Vec<ImageryLayerRecord>,
    tiles: Vec<SurfaceTileEntry>,
    tiles_to_render_by_texture_count: Vec<Vec<usize>>,
    draw_commands: Vec<DrawCommand>,
    layer_order_changed: bool,
    destroyed: bool,
}

impl GlobeSurfaceTileProvider {
    /// Creates a new `GlobeSurfaceTileProvider`, validating required dependencies.
    pub fn new(options: GlobeSurfaceTileProviderOptions) -> Result<Self, DeveloperError> {
        let Some(terrain_provider_name) = options.terrain_provider_name else {
            return Err(DeveloperError::new("options.terrainProvider is required."));
        };
        if !options.imagery_layers_present {
            return Err(DeveloperError::new("options.imageryLayers is required."));
        }
        if !options.surface_shader_set_present {
            return Err(DeveloperError::new("options.surfaceShaderSet is required."));
        }
        if !options.vector_provider_present {
            return Err(DeveloperError::new("options.vectorProvider is required."));
        }

        Ok(Self {
            terrain_provider_name,
            base_color: options.base_color,
            show_skirts: options.show_skirts,
            back_face_culling: options.back_face_culling,
            enable_lighting: options.enable_lighting,
            has_water_mask: options.has_water_mask,
            show_water_effect: options.show_water_effect,
            fill_highlight_color: options.fill_highlight_color,
            hue_shift: options.hue_shift,
            saturation_shift: options.saturation_shift,
            brightness_shift: options.brightness_shift,
            lighting_fade_out_distance: 6_500_000.0,
            lighting_fade_in_distance: 9_000_000.0,
            zoomed_out_ocean_specular_intensity: 0.5,
            vertex_shadow_darkness: 0.3,
            imagery_layers: Vec::new(),
            tiles: Vec::new(),
            tiles_to_render_by_texture_count: Vec::new(),
            draw_commands: Vec::new(),
            layer_order_changed: false,
            destroyed: false,
        })
    }

    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    pub fn imagery_layers(&self) -> &[ImageryLayerRecord] {
        &self.imagery_layers
    }

    pub fn tiles(&self) -> &[SurfaceTileEntry] {
        &self.tiles
    }

    pub fn tiles_mut(&mut self) -> &mut [SurfaceTileEntry] {
        &mut self.tiles
    }

    pub fn draw_commands(&self) -> &[DrawCommand] {
        &self.draw_commands
    }

    pub fn tiles_to_render_by_texture_count(&self) -> &[Vec<usize>] {
        &self.tiles_to_render_by_texture_count
    }

    /// Adds a tile and creates skeletons for all currently registered imagery layers.
    pub fn add_tile(&mut self, tile: GlobeSurfaceTile, center: Cartesian3) -> usize {
        let mut entry = SurfaceTileEntry::new(tile, center);
        for layer in &self.imagery_layers {
            entry.imagery_layer_ids.push(layer.id);
        }
        self.tiles.push(entry);
        self.tiles.len() - 1
    }

    /// Adds an imagery layer and attaches its skeleton once to every existing tile.
    pub fn add_imagery_layer(&mut self, layer: ImageryLayerRecord) {
        if self.imagery_layers.iter().any(|l| l.id == layer.id) {
            return;
        }
        let layer_id = layer.id;
        self.imagery_layers.push(layer);
        for tile in &mut self.tiles {
            if !tile.imagery_layer_ids.contains(&layer_id) {
                tile.imagery_layer_ids.push(layer_id);
            }
        }
    }

    /// Removes an imagery layer and strips its instances from all tiles.
    pub fn remove_imagery_layer(&mut self, layer_id: u64) {
        self.imagery_layers.retain(|l| l.id != layer_id);
        for tile in &mut self.tiles {
            tile.imagery_layer_ids.retain(|&id| id != layer_id);
        }
    }

    /// Raises an imagery layer to the top of the stack and reorders tile imagery accordingly.
    pub fn raise_layer_to_top(&mut self, layer_id: u64) {
        if let Some(pos) = self.imagery_layers.iter().position(|l| l.id == layer_id) {
            let layer = self.imagery_layers.remove(pos);
            self.imagery_layers.push(layer);
            self.layer_order_changed = true;
            self.reorder_tile_imagery();
        }
    }

    /// Updates visibility for a registered imagery layer.
    pub fn set_layer_show(&mut self, layer_id: u64, show: bool) {
        if let Some(layer) = self.imagery_layers.iter_mut().find(|l| l.id == layer_id) {
            layer.show = show;
        }
    }

    fn reorder_tile_imagery(&mut self) {
        let order: Vec<u64> = self.imagery_layers.iter().map(|l| l.id).collect();
        for tile in &mut self.tiles {
            tile.imagery_layer_ids.sort_by_key(|id| {
                order
                    .iter()
                    .position(|o| o == id)
                    .unwrap_or(usize::MAX)
            });
        }
        self.layer_order_changed = false;
    }

    /// Clears per-frame render queues at the start of an update pass.
    pub fn begin_update(&mut self) {
        self.tiles_to_render_by_texture_count.clear();
        self.draw_commands.clear();
        if self.layer_order_changed {
            self.reorder_tile_imagery();
        }
    }

    /// Marks a tile to be rendered this frame, bucketed by active imagery texture count.
    pub fn show_tile_this_frame(&mut self, tile_index: usize) {
        let Some(entry) = self.tiles.get(tile_index) else {
            return;
        };
        if !entry.visible
            || !matches!(
                entry.tile.state,
                QuadtreeTileLoadState::Loaded | QuadtreeTileLoadState::Done
            )
        {
            return;
        }

        let texture_count = entry
            .imagery_layer_ids
            .iter()
            .filter(|&&id| {
                self.imagery_layers
                    .iter()
                    .any(|l| l.id == id && l.show && l.alpha > 0.0)
            })
            .count();

        if self.tiles_to_render_by_texture_count.len() <= texture_count {
            self.tiles_to_render_by_texture_count
                .resize_with(texture_count + 1, Vec::new);
        }
        self.tiles_to_render_by_texture_count[texture_count].push(tile_index);
    }

    /// Finalizes the frame by emitting `DrawCommand`s for all queued tiles.
    pub fn end_update(&mut self, mode: SceneMode) -> &[DrawCommand] {
        if self.destroyed || mode == SceneMode::Morphing {
            return &self.draw_commands;
        }

        let pass = if self.base_color.alpha < 1.0 {
            Pass::Translucent
        } else {
            Pass::Globe
        };

        for bucket in &self.tiles_to_render_by_texture_count {
            for &tile_index in bucket {
                let entry = &self.tiles[tile_index];
                self.draw_commands.push(DrawCommand::new(DrawCommandOptions {
                    count: Some(entry.tile.mesh_vertex_count),
                    pass: Some(pass),
                    cull: Some(self.back_face_culling),
                    owner: Some("GlobeSurfaceTileProvider".to_string()),
                    ..Default::default()
                }));
            }
        }

        &self.draw_commands
    }

    /// Computes distance from the camera to a tile center.
    pub fn compute_distance_to_tile(
        &self,
        tile_center: Cartesian3,
        camera_position: Cartesian3,
    ) -> f64 {
        (tile_center - camera_position).magnitude()
    }

    /// Destroys the provider and releases all tile and layer state.
    pub fn destroy(&mut self) {
        self.tiles.clear();
        self.imagery_layers.clear();
        self.tiles_to_render_by_texture_count.clear();
        self.draw_commands.clear();
        self.destroyed = true;
    }
}
