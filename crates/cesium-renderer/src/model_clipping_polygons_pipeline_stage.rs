//! ModelClippingPolygonsPipelineStage matching CesiumJS `Scene/Model/ModelClippingPolygonsPipelineStage.js` (Apache-2.0).

use std::sync::Arc;

use cesium_math::{negative_pi_to_pi, Cartesian2, Cartesian3, Cartographic, Rectangle};
use crate::clipping_polygon_collection::ClippingPolygonCollection;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};

pub const SHADERS_MODEL_CLIPPING_POLYGONS_STAGE_VS: &str = r#"
void modelClippingPolygonsStage()
{
    vec2 delta = czm_eyeToCartographicDelta(v_positionEC).xy;
    v_clippingUv = u_clippingCameraUv + delta * u_clippingRectangleInverseSize;
}
"#;

pub const SHADERS_VECTOR_COMMON: &str = r#"
uniform highp sampler2D u_clippingEdgeTexture;
uniform highp sampler2D u_clippingEdgePrimitiveIndicesTexture;
uniform highp sampler2D u_clippingGridCellIndicesTexture;

bool vectorClip(vec2 uv)
{
    return false;
}
"#;

pub const SHADERS_MODEL_CLIPPING_POLYGONS_STAGE_FS: &str = r#"
void modelClippingPolygonsStage()
{
    bool insideAny = vectorClip(v_clippingUv);

#ifdef CLIPPING_INVERSE
    if (!insideAny)
    {
        discard;
    }
#else
    if (insideAny)
    {
        discard;
    }
#endif
}
"#;

/// Vector clipping polygon metadata on a model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelClippingPolygonData {
    pub rectangle: Option<Rectangle>,
    pub polygon_edge_texture: Option<u32>,
    pub polygon_edge_primitive_indices_texture: Option<u32>,
    pub polygon_grid_cell_indices_texture: Option<u32>,
}

/// Model descriptor for clipping polygons processing.
#[derive(Clone, Debug)]
pub struct ModelClippingPolygonsModel {
    pub clipping_polygons: ClippingPolygonCollection,
    pub clipping_polygon_data: Option<ModelClippingPolygonData>,
}

impl ModelClippingPolygonsModel {
    pub fn new(clipping_polygons: ClippingPolygonCollection) -> Self {
        Self {
            clipping_polygons,
            clipping_polygon_data: None,
        }
    }
}

/// Uniform state for camera eye cartographic.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelClippingPolygonsUniformState {
    pub eye_cartographic: Cartesian3,
}

impl Default for ModelClippingPolygonsUniformState {
    fn default() -> Self {
        Self {
            eye_cartographic: Cartesian3::ZERO,
        }
    }
}

/// Context descriptor for clipping polygons processing.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ModelClippingPolygonsContext {
    pub default_texture: Option<u32>,
    pub uniform_state: ModelClippingPolygonsUniformState,
}

/// Camera descriptor for clipping polygons processing.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ModelClippingPolygonsCamera {
    pub position_cartographic: Cartographic,
}

/// Frame state containing camera and context for clipping polygons processing.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ModelClippingPolygonsFrameState {
    pub camera: ModelClippingPolygonsCamera,
    pub context: ModelClippingPolygonsContext,
}

pub type CameraUvGetter = Arc<dyn Fn() -> Cartesian2 + Send + Sync>;
pub type InverseSizeGetter = Arc<dyn Fn() -> Cartesian2 + Send + Sync>;
pub type TextureGetter = Arc<dyn Fn() -> Option<u32> + Send + Sync>;

/// Uniform map populated by `ModelClippingPolygonsPipelineStage`.
#[derive(Clone, Default)]
pub struct ModelClippingPolygonsUniformMap {
    pub u_clipping_camera_uv: Option<CameraUvGetter>,
    pub u_clipping_rectangle_inverse_size: Option<InverseSizeGetter>,
    pub u_clipping_edge_texture: Option<TextureGetter>,
    pub u_clipping_edge_primitive_indices_texture: Option<TextureGetter>,
    pub u_clipping_grid_cell_indices_texture: Option<TextureGetter>,
}

impl ModelClippingPolygonsUniformMap {
    pub fn u_clipping_camera_uv(&self) -> Cartesian2 {
        self.u_clipping_camera_uv
            .as_ref()
            .map(|cb| cb())
            .unwrap_or(Cartesian2::ZERO)
    }

    pub fn u_clipping_rectangle_inverse_size(&self) -> Cartesian2 {
        self.u_clipping_rectangle_inverse_size
            .as_ref()
            .map(|cb| cb())
            .unwrap_or(Cartesian2::ZERO)
    }

    pub fn u_clipping_edge_texture(&self) -> Option<u32> {
        self.u_clipping_edge_texture.as_ref().and_then(|cb| cb())
    }

    pub fn u_clipping_edge_primitive_indices_texture(&self) -> Option<u32> {
        self.u_clipping_edge_primitive_indices_texture
            .as_ref()
            .and_then(|cb| cb())
    }

    pub fn u_clipping_grid_cell_indices_texture(&self) -> Option<u32> {
        self.u_clipping_grid_cell_indices_texture
            .as_ref()
            .and_then(|cb| cb())
    }
}

/// Render resources configured by `ModelClippingPolygonsPipelineStage`.
pub struct ModelClippingPolygonsRenderResources {
    pub shader_builder: ShaderBuilder,
    pub uniform_map: ModelClippingPolygonsUniformMap,
    pub model: ModelClippingPolygonsModel,
}

impl ModelClippingPolygonsRenderResources {
    pub fn new(model: ModelClippingPolygonsModel) -> Self {
        Self {
            shader_builder: ShaderBuilder::new(),
            uniform_map: ModelClippingPolygonsUniformMap::default(),
            model,
        }
    }
}

/// Pipeline stage for applying clipping polygons to a 3D model.
pub struct ModelClippingPolygonsPipelineStage;

impl ModelClippingPolygonsPipelineStage {
    pub const NAME: &'static str = "ModelClippingPolygonsPipelineStage";

    /// Modifies the shader builder and uniform map to configure clipping polygons rendering.
    pub fn process(
        render_resources: &mut ModelClippingPolygonsRenderResources,
        model: &ModelClippingPolygonsModel,
        frame_state: &ModelClippingPolygonsFrameState,
    ) {
        let clipping_polygons = &model.clipping_polygons;
        let shader_builder = &mut render_resources.shader_builder;

        shader_builder.add_define(
            "ENABLE_CLIPPING_POLYGONS",
            None,
            Some(ShaderDestination::BOTH),
        );

        if clipping_polygons.inverse {
            shader_builder.add_define(
                "CLIPPING_INVERSE",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }

        shader_builder.add_varying("vec2", "v_clippingUv", None);

        shader_builder.add_uniform(
            "vec2",
            "u_clippingCameraUv",
            Some(ShaderDestination::VERTEX),
        );

        shader_builder.add_uniform(
            "vec2",
            "u_clippingRectangleInverseSize",
            Some(ShaderDestination::VERTEX),
        );

        shader_builder.add_vertex_lines(&[SHADERS_MODEL_CLIPPING_POLYGONS_STAGE_VS]);
        shader_builder.add_fragment_lines(&[
            SHADERS_VECTOR_COMMON,
            SHADERS_MODEL_CLIPPING_POLYGONS_STAGE_FS,
        ]);

        let rectangle = model
            .clipping_polygon_data
            .as_ref()
            .and_then(|d| d.rectangle)
            .unwrap_or(Rectangle::MAXIMUM);

        let default_texture = frame_state.context.default_texture;
        let edge_texture = model
            .clipping_polygon_data
            .as_ref()
            .and_then(|d| d.polygon_edge_texture)
            .or(default_texture);
        let edge_prim_texture = model
            .clipping_polygon_data
            .as_ref()
            .and_then(|d| d.polygon_edge_primitive_indices_texture)
            .or(default_texture);
        let grid_texture = model
            .clipping_polygon_data
            .as_ref()
            .and_then(|d| d.polygon_grid_cell_indices_texture)
            .or(default_texture);

        let uniform_state = frame_state.context.uniform_state.clone();

        render_resources.uniform_map.u_clipping_camera_uv = Some(Arc::new(move || {
            let half_width = rectangle.width() * 0.5;
            let center_longitude = rectangle.west + half_width;
            let eye = uniform_state.eye_cartographic;

            let longitude_offset =
                negative_pi_to_pi(eye.x - center_longitude) + half_width;
            Cartesian2::new(
                longitude_offset / rectangle.width(),
                (eye.y - rectangle.south) / rectangle.height(),
            )
        }));

        render_resources
            .uniform_map
            .u_clipping_rectangle_inverse_size = Some(Arc::new(move || {
            Cartesian2::new(1.0 / rectangle.width(), 1.0 / rectangle.height())
        }));

        render_resources.uniform_map.u_clipping_edge_texture =
            Some(Arc::new(move || edge_texture));
        render_resources
            .uniform_map
            .u_clipping_edge_primitive_indices_texture =
            Some(Arc::new(move || edge_prim_texture));
        render_resources
            .uniform_map
            .u_clipping_grid_cell_indices_texture =
            Some(Arc::new(move || grid_texture));
    }
}
