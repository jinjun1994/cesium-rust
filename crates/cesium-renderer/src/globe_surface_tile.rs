//! GlobeSurfaceTile matching CesiumJS `Scene/GlobeSurfaceTile.js` and `Scene/TerrainState.js`.

use cesium_math::Cartesian4;

use crate::imagery_pipeline_stage::ImageryState;
use crate::quadtree_tile_load_state::QuadtreeTileLoadState;

/// Terrain loading pipeline state matching CesiumJS `Scene/TerrainState.js`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[repr(u8)]
pub enum TerrainState {
    Failed = 0,
    #[default]
    Unloaded = 1,
    Receiving = 2,
    Received = 3,
    Transforming = 4,
    Transformed = 5,
    Ready = 6,
}

/// Water mask payload attached to terrain tile data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaterMaskData {
    AllLand,
    AllWater,
    MultiByte {
        width: u32,
        height: u32,
        data: Vec<u8>,
    },
}

/// Texture descriptor for a tile's water mask.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaterMaskTexture {
    pub id: u64,
    pub is_shared_all_water: bool,
    pub width: u32,
    pub height: u32,
}

/// Terrain geometry payload for a `GlobeSurfaceTile`.
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceTerrainData {
    pub created_by_upsampling: bool,
    pub water_mask: Option<WaterMaskData>,
    pub vertex_count: usize,
    pub minimum_height: f64,
    pub maximum_height: f64,
}

impl SurfaceTerrainData {
    pub fn was_created_by_upsampling(&self) -> bool {
        self.created_by_upsampling
    }
}

/// Imagery attachment state on a `GlobeSurfaceTile`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceImageryAttachment {
    pub state: ImageryState,
    pub upsampled_from_parent: bool,
}

/// Outcome of requesting terrain geometry for a tile.
#[derive(Clone, Debug, PartialEq)]
pub enum TerrainRequestOutcome {
    Pending,
    Loaded(SurfaceTerrainData),
    Failed(String),
}

/// Outcome of building a terrain mesh for a tile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MeshCreationOutcome {
    Pending,
    Ready { vertex_count: usize },
    Failed(String),
}

/// Terrain provider operations consumed by `GlobeSurfaceTile::process_state_machine`.
pub trait TerrainProviderActions {
    fn get_tile_data_available(&self, x: u32, y: u32, level: u32) -> Option<bool>;
    fn request_tile_geometry(&mut self, x: u32, y: u32, level: u32) -> TerrainRequestOutcome;
    fn create_mesh(
        &mut self,
        x: u32,
        y: u32,
        level: u32,
        data: &SurfaceTerrainData,
    ) -> MeshCreationOutcome;
    fn upsample(
        &mut self,
        parent_data: &SurfaceTerrainData,
        x: u32,
        y: u32,
        level: u32,
    ) -> Option<SurfaceTerrainData>;
}

/// Encapsulates terrain, water mask, and imagery loading state for a `QuadtreeTile`.
#[derive(Clone, Debug)]
pub struct GlobeSurfaceTile {
    pub x: u32,
    pub y: u32,
    pub level: u32,
    pub state: QuadtreeTileLoadState,
    pub terrain_state: TerrainState,
    pub upsampled_from_parent: bool,
    pub terrain_data: Option<SurfaceTerrainData>,
    pub water_mask_texture: Option<WaterMaskTexture>,
    pub water_mask_translation_and_scale: Cartesian4,
    pub imagery: Vec<SurfaceImageryAttachment>,
    pub mesh_vertex_count: usize,
    pub last_error: Option<String>,
}

impl GlobeSurfaceTile {
    /// Creates a new `GlobeSurfaceTile` in `Start` / `Unloaded` state.
    pub fn new(x: u32, y: u32, level: u32) -> Self {
        Self {
            x,
            y,
            level,
            state: QuadtreeTileLoadState::Start,
            terrain_state: TerrainState::Unloaded,
            upsampled_from_parent: false,
            terrain_data: None,
            water_mask_texture: None,
            water_mask_translation_and_scale: Cartesian4::new(0.0, 0.0, 1.0, 1.0),
            imagery: Vec::new(),
            mesh_vertex_count: 0,
            last_error: None,
        }
    }

    /// Returns whether the tile can be safely unloaded without interrupting in-flight work.
    pub fn eligible_for_unloading(&self) -> bool {
        if matches!(
            self.terrain_state,
            TerrainState::Receiving | TerrainState::Transforming
        ) {
            return false;
        }
        !self
            .imagery
            .iter()
            .any(|img| img.state == ImageryState::Transitioning)
    }

    /// Releases terrain, mesh, water mask, and imagery resources and resets state.
    pub fn free_resources(&mut self) {
        self.water_mask_texture = None;
        self.water_mask_translation_and_scale = Cartesian4::new(0.0, 0.0, 1.0, 1.0);
        self.terrain_data = None;
        self.imagery.clear();
        self.mesh_vertex_count = 0;
        self.terrain_state = TerrainState::Unloaded;
        self.state = QuadtreeTileLoadState::Start;
        self.upsampled_from_parent = false;
        self.last_error = None;
    }

    /// Advances the tile's terrain and imagery state machine by one step.
    pub fn process_state_machine<P: TerrainProviderActions>(
        &mut self,
        mut parent: Option<&mut GlobeSurfaceTile>,
        provider: &mut P,
        shared_all_water_texture_id: u64,
    ) {
        if self.state == QuadtreeTileLoadState::Start {
            self.prepare_new_tile(provider);
        }

        if self.state == QuadtreeTileLoadState::Loading {
            self.process_terrain_state_machine(parent.as_deref_mut(), provider, shared_all_water_texture_id);
        }

        self.update_load_state(parent.as_deref());
    }

    fn prepare_new_tile<P: TerrainProviderActions>(&mut self, provider: &P) {
        let available = provider.get_tile_data_available(self.x, self.y, self.level);
        if available == Some(false) {
            self.terrain_state = TerrainState::Failed;
        }
        self.state = QuadtreeTileLoadState::Loading;
    }

    fn process_terrain_state_machine<P: TerrainProviderActions>(
        &mut self,
        mut parent: Option<&mut GlobeSurfaceTile>,
        provider: &mut P,
        shared_all_water_texture_id: u64,
    ) {
        if self.terrain_state == TerrainState::Unloaded {
            self.terrain_state = TerrainState::Receiving;
            match provider.request_tile_geometry(self.x, self.y, self.level) {
                TerrainRequestOutcome::Pending => {}
                TerrainRequestOutcome::Loaded(data) => {
                    self.terrain_data = Some(data);
                    self.terrain_state = TerrainState::Received;
                }
                TerrainRequestOutcome::Failed(err) => {
                    self.last_error = Some(err);
                    self.terrain_state = TerrainState::Failed;
                }
            }
        }

        if self.terrain_state == TerrainState::Failed {
            // Try upsampling from parent if available.
            if let Some(parent_tile) = parent.as_deref_mut() {
                if parent_tile.terrain_data.is_none()
                    && parent_tile.terrain_state != TerrainState::Failed
                {
                    parent_tile.process_terrain_state_machine(
                        None,
                        provider,
                        shared_all_water_texture_id,
                    );
                }
                if let Some(parent_data) = &parent_tile.terrain_data {
                    if let Some(upsampled) =
                        provider.upsample(parent_data, self.x, self.y, self.level)
                    {
                        self.terrain_data = Some(upsampled);
                        self.terrain_state = TerrainState::Received;
                    }
                }
            }
        }

        if self.terrain_state == TerrainState::Received {
            if let Some(data) = self.terrain_data.clone() {
                self.terrain_state = TerrainState::Transforming;
                match provider.create_mesh(self.x, self.y, self.level, &data) {
                    MeshCreationOutcome::Pending => {}
                    MeshCreationOutcome::Ready { vertex_count } => {
                        self.mesh_vertex_count = vertex_count;
                        self.terrain_state = TerrainState::Transformed;
                    }
                    MeshCreationOutcome::Failed(err) => {
                        self.last_error = Some(err);
                        self.terrain_state = TerrainState::Failed;
                    }
                }
            }
        }

        if self.terrain_state == TerrainState::Transformed {
            self.process_water_mask(parent.as_deref(), shared_all_water_texture_id);
            self.terrain_state = TerrainState::Ready;
        }
    }

    fn process_water_mask(
        &mut self,
        parent: Option<&GlobeSurfaceTile>,
        shared_all_water_texture_id: u64,
    ) {
        let mask = self
            .terrain_data
            .as_ref()
            .and_then(|d| d.water_mask.as_ref());

        match mask {
            Some(WaterMaskData::AllLand) => {
                self.water_mask_texture = None;
            }
            Some(WaterMaskData::AllWater) => {
                self.water_mask_texture = Some(WaterMaskTexture {
                    id: shared_all_water_texture_id,
                    is_shared_all_water: true,
                    width: 1,
                    height: 1,
                });
                self.water_mask_translation_and_scale = Cartesian4::new(0.0, 0.0, 1.0, 1.0);
            }
            Some(WaterMaskData::MultiByte { width, height, .. }) => {
                let unique_id = ((self.level as u64 + 1) << 40)
                    | ((self.x as u64) << 20)
                    | (self.y as u64);
                self.water_mask_texture = Some(WaterMaskTexture {
                    id: unique_id,
                    is_shared_all_water: false,
                    width: *width,
                    height: *height,
                });
                self.water_mask_translation_and_scale = Cartesian4::new(0.0, 0.0, 1.0, 1.0);
            }
            None => {
                // Upsample water mask from parent if parent has one.
                if let Some(p) = parent {
                    if let Some(p_tex) = &p.water_mask_texture {
                        self.water_mask_texture = Some(p_tex.clone());
                        let p_ts = p.water_mask_translation_and_scale;
                        let half_scale_x = p_ts.z * 0.5;
                        let half_scale_y = p_ts.w * 0.5;
                        let offset_x = if self.x % 2 == 1 { half_scale_x } else { 0.0 };
                        // In TMS/quadtree y=0 is north, y=1 is south (bottom in GL UV).
                        let offset_y = if self.y.is_multiple_of(2) { half_scale_y } else { 0.0 };
                        self.water_mask_translation_and_scale = Cartesian4::new(
                            p_ts.x + offset_x,
                            p_ts.y + offset_y,
                            half_scale_x,
                            half_scale_y,
                        );
                    }
                }
            }
        }
    }

    fn update_load_state(&mut self, _parent: Option<&GlobeSurfaceTile>) {
        if self.terrain_state == TerrainState::Failed {
            self.state = QuadtreeTileLoadState::Failed;
            return;
        }

        let all_imagery_done = self
            .imagery
            .iter()
            .all(|img| matches!(img.state, ImageryState::Ready | ImageryState::Failed | ImageryState::Invalid));

        if self.terrain_state == TerrainState::Ready && all_imagery_done {
            let terrain_upsampled = self
                .terrain_data
                .as_ref()
                .is_some_and(SurfaceTerrainData::was_created_by_upsampling);
            let any_fresh_imagery = self
                .imagery
                .iter()
                .any(|img| img.state == ImageryState::Ready && !img.upsampled_from_parent);

            self.upsampled_from_parent = terrain_upsampled && !any_fresh_imagery;
            self.state = QuadtreeTileLoadState::Done;
        } else if self.terrain_state == TerrainState::Ready {
            self.state = QuadtreeTileLoadState::Loaded;
        }
    }
}
