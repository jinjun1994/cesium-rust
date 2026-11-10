//! TilesetPipelineStage matching CesiumJS `Scene/Model/TilesetPipelineStage.js` (Apache-2.0).

use std::collections::HashMap;
use cesium_math::Cartesian2;

use crate::classification_model_draw_command::StencilConstants;
use crate::model_render_resources::ModelUniformCallback;
use crate::render_state::RenderState;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};

/// Model state providing 3D Tiles skipLevelOfDetail configuration.
#[derive(Debug, Default, Clone)]
pub struct MockTilesetModel {
    pub has_skip_level_of_detail: bool,
}

/// Standalone render resources for TilesetPipelineStage.
#[derive(Default)]
pub struct MockTilesetRenderResources {
    pub shader_builder: ShaderBuilder,
    pub uniform_map: HashMap<String, ModelUniformCallback>,
    pub model: MockTilesetModel,
    pub render_state_options: RenderState,
    pub has_skip_level_of_detail: bool,
    pub u_polygon_offset: Option<Cartesian2>,
}

impl std::fmt::Debug for MockTilesetRenderResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockTilesetRenderResources")
            .field("shader_builder", &self.shader_builder)
            .field("model", &self.model)
            .field("render_state_options", &self.render_state_options)
            .field("has_skip_level_of_detail", &self.has_skip_level_of_detail)
            .field("u_polygon_offset", &self.u_polygon_offset)
            .finish()
    }
}

/// Trait abstracting render resources required by [`TilesetPipelineStage`].
pub trait TilesetRenderResources {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder;
    fn render_state_options_mut(&mut self) -> &mut RenderState;
    fn set_has_skip_level_of_detail(&mut self, val: bool);
    fn set_polygon_offset_uniform(&mut self, offset: Cartesian2);
}

impl TilesetRenderResources for MockTilesetRenderResources {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder {
        &mut self.shader_builder
    }

    fn render_state_options_mut(&mut self) -> &mut RenderState {
        &mut self.render_state_options
    }

    fn set_has_skip_level_of_detail(&mut self, val: bool) {
        self.has_skip_level_of_detail = val;
    }

    fn set_polygon_offset_uniform(&mut self, offset: Cartesian2) {
        self.u_polygon_offset = Some(offset);
        self.uniform_map.insert(
            "u_polygonOffset".to_string(),
            Box::new(move || 0.0),
        );
    }
}

/// Provider trait for models that support 3D Tiles behavior.
pub trait TilesetModelProvider {
    fn has_skip_level_of_detail(&self) -> bool;
}

impl TilesetModelProvider for MockTilesetModel {
    fn has_skip_level_of_detail(&self) -> bool {
        self.has_skip_level_of_detail
    }
}

/// The tileset pipeline stage is responsible for updating the model with behavior
/// specific to 3D Tiles.
pub struct TilesetPipelineStage;

impl TilesetPipelineStage {
    pub const NAME: &'static str = "TilesetPipelineStage";

    /// Process a model for 3D Tiles specific rendering.
    pub fn process<R: TilesetRenderResources, M: TilesetModelProvider>(
        render_resources: &mut R,
        model: &M,
    ) {
        if model.has_skip_level_of_detail() {
            let shader_builder = render_resources.shader_builder_mut();
            shader_builder.add_define(
                "POLYGON_OFFSET",
                None,
                Some(ShaderDestination::FRAGMENT),
            );

            render_resources.set_polygon_offset_uniform(Cartesian2::ZERO);
            render_resources.set_has_skip_level_of_detail(true);
        }

        let render_state_options = render_resources.render_state_options_mut();
        render_state_options.stencil_test = StencilConstants::set_cesium_3d_tile_bit();
        render_state_options.stencil_mask = StencilConstants::CESIUM_3D_TILE_MASK;
    }
}
