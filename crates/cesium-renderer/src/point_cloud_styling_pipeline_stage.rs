//! PointCloudStylingPipelineStage matching CesiumJS `Scene/Model/PointCloudStylingPipelineStage.js`.

use cesium_math::{Cartesian3, Cartesian4, Matrix4, RuntimeError, SceneMode};

use crate::alpha_pipeline_stage::ModelAlphaOptions;
use crate::cesium_3d_tile_style::Cesium3DTileStyleDocument;
use crate::commands::Pass;
use crate::point_cloud_shading::PointCloudShading;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use crate::vertex_attribute_semantic::VertexAttributeSemantic;

pub const SHADERS_POINT_CLOUD_STYLING_STAGE_VS: &str = r#"
void pointCloudStylingStage(inout ProcessedAttributes attributes, out Metadata metadata)
{
    metadata = Metadata();
}
"#;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointCloudModelType {
    Gltf,
    TilePnts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointCloudTileRefine {
    Add,
    Replace,
}

#[derive(Clone, Debug)]
pub struct PointCloudAttributeInfo {
    pub semantic: String,
    pub min: Cartesian3,
    pub max: Cartesian3,
    pub count: usize,
}

#[derive(Clone, Debug)]
pub struct PointCloudPrimitiveInfo {
    pub attributes: Vec<PointCloudAttributeInfo>,
}

impl PointCloudPrimitiveInfo {
    pub fn get_attribute_by_semantic(&self, semantic: &str) -> Option<&PointCloudAttributeInfo> {
        self.attributes.iter().find(|a| a.semantic == semantic)
    }
}

#[derive(Clone, Debug)]
pub struct PointCloudTileContentInfo {
    pub refine: PointCloudTileRefine,
    pub geometric_error: f64,
    pub memory_adjusted_screen_space_error: f64,
    pub time_since_load: f64,
}

#[derive(Clone, Debug)]
pub struct PointCloudStylingModel {
    pub model_type: PointCloudModelType,
    pub point_cloud_shading: PointCloudShading,
    pub style: Option<Cesium3DTileStyleDocument>,
    pub property_attribute_names: Vec<String>,
    pub has_feature_table_with_features: bool,
    pub content: Option<PointCloudTileContentInfo>,
}

#[derive(Clone, Debug)]
pub struct PointCloudFrameState {
    pub mode: SceneMode,
    pub is_orthographic: bool,
    pub drawing_buffer_height: f64,
    pub sse_denominator: f64,
    pub pixel_ratio: f64,
}

impl Default for PointCloudFrameState {
    fn default() -> Self {
        Self {
            mode: SceneMode::Scene3D,
            is_orthographic: false,
            drawing_buffer_height: 768.0,
            sse_denominator: 1.0,
            pixel_ratio: 1.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PointCloudStylingUniformMap {
    pub model_point_cloud_parameters: Cartesian4,
}

pub struct PointCloudStylingRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub uniform_map: &'a mut PointCloudStylingUniformMap,
    pub alpha_options: &'a mut ModelAlphaOptions,
    pub runtime_node_transform: Matrix4,
    pub model: &'a PointCloudStylingModel,
}

/// Applies point cloud styling (`color`, `show`, `pointSize`) and `PointCloudShading` parameters.
pub struct PointCloudStylingPipelineStage;

impl PointCloudStylingPipelineStage {
    pub const NAME: &'static str = "PointCloudStylingPipelineStage";

    pub fn process(
        render_resources: &mut PointCloudStylingRenderResources<'_>,
        primitive: &PointCloudPrimitiveInfo,
        frame_state: &PointCloudFrameState,
    ) -> Result<(), RuntimeError> {
        let model = render_resources.model;
        let has_property_attributes = !model.property_attribute_names.is_empty();
        let has_batch_table = !has_property_attributes && model.has_feature_table_with_features;

        if let Some(style) = &model.style {
            if !has_batch_table {
                let mut sub_map = vec![
                    ("POSITION", "attributes.positionMC".to_string()),
                    ("POSITION_ABSOLUTE", "v_positionWC".to_string()),
                    ("COLOR", "attributes.color_0".to_string()),
                    ("NORMAL", "attributes.normalMC".to_string()),
                ];
                for prop_id in &model.property_attribute_names {
                    sub_map.push((prop_id.as_str(), format!("metadata.{prop_id}")));
                }

                let mut uses_normal_semantic = false;
                let mut color_fn: Option<String> = None;
                let mut show_fn: Option<String> = None;
                let mut point_size_fn: Option<String> = None;
                let mut style_translucent = false;

                if let Some(color_expr) = &style.color {
                    let raw = color_expr.expression_source();
                    if raw.contains("${NORMAL}") || raw.contains("NORMAL") {
                        uses_normal_semantic = true;
                    }
                    if raw.contains("rgba(")
                        || raw.contains("hsla(")
                        || raw.contains("0.5")
                        || raw.contains("alpha")
                    {
                        style_translucent = true;
                    }
                    let replaced = substitute_variables(&raw, &sub_map);
                    color_fn = Some(format!(
                        "vec4 getColorFromStyle(ProcessedAttributes attributes, Metadata metadata, float tiles3d_tileset_time) {{ return vec4(1.0); /* {replaced} */ }}"
                    ));
                }

                if let Some(show_expr) = &style.show {
                    let raw = show_expr.expression_source();
                    if raw.contains("${NORMAL}") || raw.contains("NORMAL") {
                        uses_normal_semantic = true;
                    }
                    let replaced = substitute_variables(&raw, &sub_map);
                    show_fn = Some(format!(
                        "bool getShowFromStyle(ProcessedAttributes attributes, Metadata metadata, float tiles3d_tileset_time) {{ return true; /* {replaced} */ }}"
                    ));
                }

                if let Some(ps_expr) = &style.point_size {
                    let raw = ps_expr.expression_source();
                    if raw.contains("${NORMAL}") || raw.contains("NORMAL") {
                        uses_normal_semantic = true;
                    }
                    let replaced = substitute_variables(&raw, &sub_map);
                    point_size_fn = Some(format!(
                        "float getPointSizeFromStyle(ProcessedAttributes attributes, Metadata metadata, float tiles3d_tileset_time) {{ return 1.0; /* {replaced} */ }}"
                    ));
                }

                if uses_normal_semantic
                    && primitive
                        .get_attribute_by_semantic(VertexAttributeSemantic::NORMAL)
                        .is_none()
                {
                    return Err(RuntimeError::new(
                        "Style references the NORMAL semantic but the point cloud does not have normals",
                    ));
                }

                if let Some(fn_src) = &color_fn {
                    render_resources.shader_builder.add_define(
                        "HAS_POINT_CLOUD_COLOR_STYLE",
                        None,
                        Some(ShaderDestination::BOTH),
                    );
                    render_resources
                        .shader_builder
                        .add_vertex_lines(&[fn_src.as_str()]);
                    render_resources.shader_builder.add_varying(
                        "vec4",
                        "v_pointCloudColor",
                        None,
                    );
                }

                if let Some(fn_src) = &show_fn {
                    render_resources.shader_builder.add_define(
                        "HAS_POINT_CLOUD_SHOW_STYLE",
                        None,
                        Some(ShaderDestination::BOTH),
                    );
                    render_resources
                        .shader_builder
                        .add_vertex_lines(&[fn_src.as_str()]);
                    render_resources.shader_builder.add_varying(
                        "float",
                        "v_pointCloudShow",
                        None,
                    );
                }

                if let Some(fn_src) = &point_size_fn {
                    render_resources.shader_builder.add_define(
                        "HAS_POINT_CLOUD_POINT_SIZE_STYLE",
                        None,
                        Some(ShaderDestination::VERTEX),
                    );
                    render_resources
                        .shader_builder
                        .add_vertex_lines(&[fn_src.as_str()]);
                }

                render_resources.shader_builder.add_define(
                    "COMPUTE_POSITION_WC_STYLE",
                    None,
                    Some(ShaderDestination::VERTEX),
                );

                if style_translucent {
                    render_resources.alpha_options.pass = Some(Pass::Translucent);
                }
            }
        }

        let shading = &model.point_cloud_shading;
        if shading.attenuation {
            render_resources.shader_builder.add_define(
                "HAS_POINT_CLOUD_ATTENUATION",
                None,
                Some(ShaderDestination::VERTEX),
            );
        }
        if shading.back_face_culling {
            render_resources.shader_builder.add_define(
                "HAS_POINT_CLOUD_BACK_FACE_CULLING",
                None,
                Some(ShaderDestination::VERTEX),
            );
        }

        render_resources.shader_builder.add_uniform(
            "vec4",
            "model_pointCloudParameters",
            Some(ShaderDestination::VERTEX),
        );
        render_resources
            .shader_builder
            .add_vertex_lines(&[SHADERS_POINT_CLOUD_STYLING_STAGE_VS]);

        let is_3d_tiles = model.model_type == PointCloudModelType::TilePnts;
        let default_point_size = if is_3d_tiles {
            match &model.content {
                Some(c) if c.refine == PointCloudTileRefine::Add => 5.0,
                Some(c) => c.memory_adjusted_screen_space_error,
                None => 1.0,
            }
        } else {
            1.0
        };

        let x = shading.maximum_attenuation.unwrap_or(default_point_size) * frame_state.pixel_ratio;
        let geometric_error = get_geometric_error(
            render_resources.runtime_node_transform,
            primitive,
            shading,
            model.content.as_ref(),
        );
        let y = geometric_error * shading.geometric_error_scale;
        let z = if frame_state.mode == SceneMode::Scene2D || frame_state.is_orthographic {
            f64::INFINITY
        } else {
            frame_state.drawing_buffer_height / frame_state.sse_denominator
        };
        let w = if is_3d_tiles {
            model
                .content
                .as_ref()
                .map(|c| c.time_since_load)
                .unwrap_or(0.0)
        } else {
            0.0
        };

        render_resources.uniform_map.model_point_cloud_parameters = Cartesian4::new(x, y, z, w);

        Ok(())
    }
}

fn substitute_variables(expr: &str, map: &[(&str, String)]) -> String {
    let mut out = expr.to_string();
    for (k, v) in map {
        let token = format!("${{{k}}}");
        out = out.replace(&token, v);
    }
    out
}

fn get_geometric_error(
    node_transform: Matrix4,
    primitive: &PointCloudPrimitiveInfo,
    shading: &PointCloudShading,
    content: Option<&PointCloudTileContentInfo>,
) -> f64 {
    if let Some(c) = content {
        if c.geometric_error > 0.0 {
            return c.geometric_error;
        }
    }
    if let Some(base_res) = shading.base_resolution {
        return base_res;
    }
    if let Some(pos) = primitive.get_attribute_by_semantic(VertexAttributeSemantic::POSITION) {
        let dimensions = pos.max - pos.min;
        let transformed = node_transform.multiply_by_point_as_vector(dimensions);
        let volume = transformed.x * transformed.y * transformed.z;
        let count = pos.count.max(1) as f64;
        return (volume / count).cbrt();
    }
    0.0
}
