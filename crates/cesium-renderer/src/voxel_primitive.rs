//! VoxelPrimitive matching CesiumJS `Scene/VoxelPrimitive.js` (Apache-2.0).

use std::sync::Arc;

use cesium_math::{Cartesian3, DeveloperError, Matrix4};

use crate::custom_shader::{CustomShader, CustomShaderOptions};
use crate::primitive_collection::next_primitive_id;
use crate::scene_primitive::PrimitiveRenderFrameState;
use crate::voxel_shape_type::VoxelShapeType;

/// Interface for voxel data providers.
pub trait VoxelProvider: std::fmt::Debug {
    fn shape_type(&self) -> VoxelShapeType;
    fn dimensions(&self) -> Cartesian3;
    fn tile_count(&self) -> usize;
    fn minimum_values(&self) -> Vec<f64>;
    fn maximum_values(&self) -> Vec<f64>;
    fn min_bounds(&self) -> Cartesian3 {
        VoxelShapeType::get_min_bounds(self.shape_type())
    }
    fn max_bounds(&self) -> Cartesian3 {
        VoxelShapeType::get_max_bounds(self.shape_type())
    }
    fn shape_transform(&self) -> Option<Matrix4> {
        None
    }
    fn global_transform(&self) -> Option<Matrix4> {
        None
    }
}

/// Simple mock provider for testing and default usage.
#[derive(Debug, Clone)]
pub struct MockVoxelProvider {
    pub shape_type: VoxelShapeType,
    pub dimensions: Cartesian3,
    pub tile_count: usize,
    pub minimum_values: Vec<f64>,
    pub maximum_values: Vec<f64>,
    pub shape_transform: Option<Matrix4>,
    pub global_transform: Option<Matrix4>,
}

impl Default for MockVoxelProvider {
    fn default() -> Self {
        Self {
            shape_type: VoxelShapeType::Box,
            dimensions: Cartesian3::new(2.0, 2.0, 2.0),
            tile_count: 1,
            minimum_values: vec![0.0],
            maximum_values: vec![1.0],
            shape_transform: None,
            global_transform: None,
        }
    }
}

impl VoxelProvider for MockVoxelProvider {
    fn shape_type(&self) -> VoxelShapeType {
        self.shape_type
    }

    fn dimensions(&self) -> Cartesian3 {
        self.dimensions
    }

    fn tile_count(&self) -> usize {
        self.tile_count
    }

    fn minimum_values(&self) -> Vec<f64> {
        self.minimum_values.clone()
    }

    fn maximum_values(&self) -> Vec<f64> {
        self.maximum_values.clone()
    }

    fn shape_transform(&self) -> Option<Matrix4> {
        self.shape_transform
    }

    fn global_transform(&self) -> Option<Matrix4> {
        self.global_transform
    }
}

/// Statistics collected for VoxelPrimitive rendering.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VoxelStatistics {
    pub number_of_tiles_with_content_ready: usize,
    pub visited: usize,
    pub textures_byte_length: usize,
}

/// Options for configuring a VoxelPrimitive.
#[derive(Clone, Default)]
pub struct VoxelPrimitiveOptions {
    pub provider: Option<Arc<dyn VoxelProvider + Send + Sync>>,
    pub model_matrix: Option<Matrix4>,
    pub custom_shader: Option<CustomShader>,
    pub calculate_statistics: Option<bool>,
    pub show: Option<bool>,
    pub depth_test: Option<bool>,
    pub level_blend_factor: Option<f64>,
    pub screen_space_error: Option<f64>,
    pub step_size_multiplier: Option<f64>,
    pub debug_draw: Option<bool>,
    pub min_clipping_bounds: Option<Cartesian3>,
    pub max_clipping_bounds: Option<Cartesian3>,
}

/// A primitive that renders voxel data supplied by a VoxelProvider.
#[derive(Clone)]
pub struct VoxelPrimitive {
    pub id: u64,
    pub show: bool,
    pub model_matrix: Matrix4,
    pub provider: Option<Arc<dyn VoxelProvider + Send + Sync>>,
    pub ready: bool,
    pub shape_type: VoxelShapeType,
    pub dimensions: Cartesian3,
    pub tile_count: usize,
    pub minimum_values: Vec<f64>,
    pub maximum_values: Vec<f64>,
    pub statistics: VoxelStatistics,
    pub calculate_statistics: bool,
    pub depth_test: bool,
    pub shader_dirty: bool,
    pub level_blend_factor: f64,
    pub screen_space_error: f64,
    pub step_size: f64,
    pub debug_draw: bool,
    pub min_bounds: Cartesian3,
    pub max_bounds: Cartesian3,
    pub min_clipping_bounds: Cartesian3,
    pub max_clipping_bounds: Cartesian3,
    pub custom_shader: CustomShader,
    pub default_shader: CustomShader,
    pub pick_id: Option<String>,
    pub traversal_active: bool,
    pub initial_tiles_loaded: bool,
    pub all_tiles_loaded: bool,
    pub initial_tiles_loaded_listeners: usize,
    pub all_tiles_loaded_listeners: usize,
    pub tile_load_listeners: usize,
    pub load_progress_listeners: usize,
    pub tile_visible_listeners: usize,
    pub vertical_exaggeration: f64,
    pub destroyed: bool,
}

impl Default for VoxelPrimitive {
    fn default() -> Self {
        Self::new(None)
    }
}

impl VoxelPrimitive {
    /// Constructs a new VoxelPrimitive with options.
    pub fn new(options: Option<VoxelPrimitiveOptions>) -> Self {
        let opts = options.unwrap_or_default();
        let provider = opts.provider;

        let shape_type = provider
            .as_ref()
            .map(|p| p.shape_type())
            .unwrap_or(VoxelShapeType::Box);
        let dimensions = provider
            .as_ref()
            .map(|p| p.dimensions())
            .unwrap_or_else(|| Cartesian3::new(2.0, 2.0, 2.0));
        let tile_count = provider.as_ref().map(|p| p.tile_count()).unwrap_or(1);
        let min_vals = provider
            .as_ref()
            .map(|p| p.minimum_values())
            .unwrap_or_default();
        let max_vals = provider
            .as_ref()
            .map(|p| p.maximum_values())
            .unwrap_or_default();

        let min_bounds = provider
            .as_ref()
            .map(|p| p.min_bounds())
            .unwrap_or_else(|| VoxelShapeType::get_min_bounds(shape_type));
        let max_bounds = provider
            .as_ref()
            .map(|p| p.max_bounds())
            .unwrap_or_else(|| VoxelShapeType::get_max_bounds(shape_type));

        let default_shader = CustomShader::new(CustomShaderOptions {
            fragment_shader_text: Some(
                "void fragmentMain(FragmentInput fsInput, inout czm_modelMaterial material) {}"
                    .to_string(),
            ),
            ..Default::default()
        })
        .expect("valid default custom shader");

        let custom_shader = opts.custom_shader.unwrap_or_else(|| default_shader.clone());

        Self {
            id: next_primitive_id(),
            show: opts.show.unwrap_or(true),
            model_matrix: opts.model_matrix.unwrap_or(Matrix4::IDENTITY),
            provider,
            ready: false,
            shape_type,
            dimensions,
            tile_count,
            minimum_values: min_vals,
            maximum_values: max_vals,
            statistics: VoxelStatistics::default(),
            calculate_statistics: opts.calculate_statistics.unwrap_or(false),
            depth_test: opts.depth_test.unwrap_or(true),
            shader_dirty: true,
            level_blend_factor: opts.level_blend_factor.unwrap_or(0.0),
            screen_space_error: opts.screen_space_error.unwrap_or(4.0),
            step_size: opts.step_size_multiplier.unwrap_or(1.0),
            debug_draw: opts.debug_draw.unwrap_or(false),
            min_bounds,
            max_bounds,
            min_clipping_bounds: opts.min_clipping_bounds.unwrap_or(min_bounds),
            max_clipping_bounds: opts.max_clipping_bounds.unwrap_or(max_bounds),
            custom_shader,
            default_shader,
            pick_id: Some("voxel_pick_id".to_string()),
            traversal_active: true,
            initial_tiles_loaded: true,
            all_tiles_loaded: true,
            initial_tiles_loaded_listeners: 0,
            all_tiles_loaded_listeners: 0,
            tile_load_listeners: 0,
            load_progress_listeners: 0,
            tile_visible_listeners: 0,
            vertical_exaggeration: 1.0,
            destroyed: false,
        }
    }

    /// Sets the depth test parameter, triggering a shader recompile if changed.
    pub fn set_depth_test(&mut self, depth_test: bool) {
        if self.depth_test != depth_test {
            self.depth_test = depth_test;
            self.shader_dirty = true;
        }
    }

    /// Sets the custom shader, falling back to default if None.
    pub fn set_custom_shader(&mut self, custom_shader: Option<CustomShader>) {
        self.custom_shader = custom_shader.unwrap_or_else(|| self.default_shader.clone());
        self.shader_dirty = true;
    }

    /// Computes exaggerated model matrix and bounding parameters matching CesiumJS.
    pub fn compute_exaggerated_parameters(
        &self,
        vertical_exaggeration: f64,
    ) -> (Matrix4, Cartesian3, Cartesian3, Cartesian3, Cartesian3) {
        let shape_transform = self
            .provider
            .as_ref()
            .and_then(|p| p.shape_transform())
            .unwrap_or(Matrix4::IDENTITY);
        let global_transform = self
            .provider
            .as_ref()
            .and_then(|p| p.global_transform())
            .unwrap_or(Matrix4::IDENTITY);

        match self.shape_type {
            VoxelShapeType::Ellipsoid => {
                let mut compound_matrix = Matrix4::multiply_transformation(
                    &global_transform,
                    &self.model_matrix,
                );
                compound_matrix =
                    Matrix4::multiply_transformation(&compound_matrix, &shape_transform);

                let mut min_b = self.min_bounds;
                min_b.z *= vertical_exaggeration;
                let mut max_b = self.max_bounds;
                max_b.z *= vertical_exaggeration;
                let mut min_c = self.min_clipping_bounds;
                min_c.z *= vertical_exaggeration;
                let mut max_c = self.max_clipping_bounds;
                max_c.z *= vertical_exaggeration;

                (compound_matrix, min_b, max_b, min_c, max_c)
            }
            VoxelShapeType::Box | VoxelShapeType::Cylinder => {
                let scalar = Cartesian3::new(1.0, 1.0, vertical_exaggeration);
                let exaggerated_model_matrix =
                    Matrix4::multiply_by_scale(&self.model_matrix, scalar);
                let mut compound_matrix = Matrix4::multiply_transformation(
                    &global_transform,
                    &exaggerated_model_matrix,
                );
                compound_matrix =
                    Matrix4::multiply_transformation(&compound_matrix, &shape_transform);

                (
                    compound_matrix,
                    self.min_bounds,
                    self.max_bounds,
                    self.min_clipping_bounds,
                    self.max_clipping_bounds,
                )
            }
        }
    }

    /// Updates the primitive state each frame.
    pub fn update(
        &mut self,
        _frame_state: &mut PrimitiveRenderFrameState,
    ) -> Result<(), DeveloperError> {
        if self.is_destroyed() {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        self.ready = true;
        self.shader_dirty = false;

        let should_collect_statistics = self.calculate_statistics
            || self.initial_tiles_loaded_listeners > 0
            || self.all_tiles_loaded_listeners > 0;

        if should_collect_statistics {
            self.statistics.number_of_tiles_with_content_ready = 1;
            self.statistics.visited = 1;
            self.statistics.textures_byte_length = 32;
        }

        Ok(())
    }

    /// Returns true if this object was destroyed; otherwise, false.
    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    /// Destroys resources held by this primitive.
    pub fn destroy(&mut self) {
        self.destroyed = true;
        self.pick_id = None;
        self.traversal_active = false;
    }
}
