//! ModelRuntimePrimitive matching CesiumJS `Scene/Model/ModelRuntimePrimitive.js` (Apache-2.0).

use cesium_math::{BoundingSphere, DeveloperError, PrimitiveType};

use crate::custom_shader::{CustomShaderMode, CustomShaderOptions};
use crate::model_components::{FeatureId, Node, Primitive};
use crate::model_draw_command::ModelDrawCommand;
use crate::model_type::ModelType;

/// Scene mode for rendering frame state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SceneMode {
    #[default]
    Scene3D,
    Scene2D,
    ColumbusView,
}

/// Point cloud shading configuration.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PointCloudShadingOptions {
    pub attenuation: bool,
    pub back_face_culling: bool,
}

/// Frame state passed to `configure_pipeline`.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimitiveFrameState {
    pub mode: SceneMode,
    pub scene_3d_only: bool,
    pub webgl2: bool,
    pub vertical_exaggeration: f64,
}

impl Default for PrimitiveFrameState {
    fn default() -> Self {
        Self {
            mode: SceneMode::Scene3D,
            scene_3d_only: false,
            webgl2: false,
            vertical_exaggeration: 1.0,
        }
    }
}

/// Classification type matching CesiumJS `ClassificationType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClassificationType {
    Terrain = 1,
    Cesium3DTile = 2,
    Both = 3,
}

/// Model properties required by `ModelRuntimePrimitive`.
#[derive(Clone, Debug)]
pub struct ModelRuntimePrimitiveModel {
    pub model_type: ModelType,
    pub allow_picking: bool,
    pub feature_id_label: Option<String>,
    pub instance_feature_id_label: Option<String>,
    pub custom_shader: Option<CustomShaderOptions>,
    pub point_cloud_shading: Option<PointCloudShadingOptions>,
    pub has_style: bool,
    pub has_vertical_exaggeration: bool,
    pub debug_wireframe: bool,
    pub enable_debug_wireframe: bool,
    pub project_to_2d: bool,
    pub enable_show_outline: bool,
    pub edge_geometry_needed: bool,
    pub classification_type: Option<ClassificationType>,
    pub has_imagery_layers: bool,
}

impl PartialEq for ModelRuntimePrimitiveModel {
    fn eq(&self, other: &Self) -> bool {
        self.model_type == other.model_type
            && self.allow_picking == other.allow_picking
            && self.feature_id_label == other.feature_id_label
            && self.instance_feature_id_label == other.instance_feature_id_label
            && self.has_style == other.has_style
            && self.has_vertical_exaggeration == other.has_vertical_exaggeration
            && self.debug_wireframe == other.debug_wireframe
            && self.enable_debug_wireframe == other.enable_debug_wireframe
            && self.project_to_2d == other.project_to_2d
            && self.enable_show_outline == other.enable_show_outline
            && self.edge_geometry_needed == other.edge_geometry_needed
            && self.classification_type == other.classification_type
            && self.has_imagery_layers == other.has_imagery_layers
            && self.point_cloud_shading == other.point_cloud_shading
    }
}

impl Default for ModelRuntimePrimitiveModel {
    fn default() -> Self {
        Self {
            model_type: ModelType::Gltf,
            allow_picking: true,
            feature_id_label: Some("featureId_0".to_string()),
            instance_feature_id_label: None,
            custom_shader: None,
            point_cloud_shading: None,
            has_style: false,
            has_vertical_exaggeration: true,
            debug_wireframe: false,
            enable_debug_wireframe: false,
            project_to_2d: false,
            enable_show_outline: false,
            edge_geometry_needed: false,
            classification_type: None,
            has_imagery_layers: false,
        }
    }
}

/// Pipeline stage enum for primitive level rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimitivePipelineStage {
    SceneMode2D,
    Geometry,
    Wireframe,
    Classification,
    MorphTargets,
    Skinning,
    PointCloudStyling,
    Dequantization,
    Imagery,
    Material,
    FeatureId,
    Metadata,
    MetadataPicking,
    SelectedFeatureId,
    BatchTexture,
    CPUStyling,
    VerticalExaggeration,
    CustomShader,
    Lighting,
    Picking,
    PrimitiveOutline,
    EdgeVisibility,
    EdgeDetection,
    Alpha,
    PrimitiveStatistics,
}

impl PrimitivePipelineStage {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SceneMode2D => "SceneMode2DPipelineStage",
            Self::Geometry => "GeometryPipelineStage",
            Self::Wireframe => "WireframePipelineStage",
            Self::Classification => "ClassificationPipelineStage",
            Self::MorphTargets => "MorphTargetsPipelineStage",
            Self::Skinning => "SkinningPipelineStage",
            Self::PointCloudStyling => "PointCloudStylingPipelineStage",
            Self::Dequantization => "DequantizationPipelineStage",
            Self::Imagery => "ImageryPipelineStage",
            Self::Material => "MaterialPipelineStage",
            Self::FeatureId => "FeatureIdPipelineStage",
            Self::Metadata => "MetadataPipelineStage",
            Self::MetadataPicking => "MetadataPickingPipelineStage",
            Self::SelectedFeatureId => "SelectedFeatureIdPipelineStage",
            Self::BatchTexture => "BatchTexturePipelineStage",
            Self::CPUStyling => "CPUStylingPipelineStage",
            Self::VerticalExaggeration => "VerticalExaggerationPipelineStage",
            Self::CustomShader => "CustomShaderPipelineStage",
            Self::Lighting => "LightingPipelineStage",
            Self::Picking => "PickingPipelineStage",
            Self::PrimitiveOutline => "PrimitiveOutlinePipelineStage",
            Self::EdgeVisibility => "EdgeVisibilityPipelineStage",
            Self::EdgeDetection => "EdgeDetectionPipelineStage",
            Self::Alpha => "AlphaPipelineStage",
            Self::PrimitiveStatistics => "PrimitiveStatisticsPipelineStage",
        }
    }
}

/// Builder arguments for constructing a `ModelRuntimePrimitive` with strict validation.
#[derive(Clone, Debug, Default)]
pub struct ModelRuntimePrimitiveArgs {
    pub primitive: Option<Primitive>,
    pub node: Option<Node>,
    pub model: Option<ModelRuntimePrimitiveModel>,
}

/// In memory representation of a single primitive matching CesiumJS `ModelRuntimePrimitive`.
#[derive(Clone, Debug, Default)]
pub struct ModelRuntimePrimitive {
    pub primitive: Option<Primitive>,
    pub node: Option<Node>,
    pub model: Option<ModelRuntimePrimitiveModel>,
    pub pipeline_stages: Vec<PrimitivePipelineStage>,
    pub draw_command: Option<ModelDrawCommand>,
    pub bounding_sphere: Option<BoundingSphere>,
    pub bounding_sphere_2d: Option<BoundingSphere>,
    pub position_buffer_2d: Option<Vec<u8>>,
    pub batch_lengths: Option<Vec<usize>>,
    pub batch_offsets: Option<Vec<usize>>,
}

impl PartialEq for ModelRuntimePrimitive {
    fn eq(&self, other: &Self) -> bool {
        self.pipeline_stages == other.pipeline_stages
            && self.draw_command == other.draw_command
            && self.bounding_sphere == other.bounding_sphere
            && self.bounding_sphere_2d == other.bounding_sphere_2d
            && self.batch_lengths == other.batch_lengths
            && self.batch_offsets == other.batch_offsets
    }
}

impl ModelRuntimePrimitive {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_command(
        draw_command: ModelDrawCommand,
        bounding_sphere: Option<BoundingSphere>,
    ) -> Self {
        Self {
            draw_command: Some(draw_command),
            bounding_sphere,
            bounding_sphere_2d: None,
            ..Default::default()
        }
    }

    /// Strict constructor checking that all required arguments are present.
    pub fn try_new(args: ModelRuntimePrimitiveArgs) -> Result<Self, DeveloperError> {
        let primitive = args
            .primitive
            .ok_or_else(|| DeveloperError::new("options.primitive is required."))?;
        let node = args
            .node
            .ok_or_else(|| DeveloperError::new("options.node is required."))?;
        let model = args
            .model
            .ok_or_else(|| DeveloperError::new("options.model is required."))?;

        Ok(Self {
            primitive: Some(primitive),
            node: Some(node),
            model: Some(model),
            pipeline_stages: Vec::new(),
            draw_command: None,
            bounding_sphere: None,
            bounding_sphere_2d: None,
            position_buffer_2d: None,
            batch_lengths: None,
            batch_offsets: None,
        })
    }

    /// Configures the primitive pipeline stages according to model and frame state.
    pub fn configure_pipeline(&mut self, frame_state: &PrimitiveFrameState) {
        self.pipeline_stages.clear();

        let primitive = match &self.primitive {
            Some(p) => p,
            None => return,
        };
        let node = match &self.node {
            Some(n) => n,
            None => return,
        };
        let model = match &self.model {
            Some(m) => m,
            None => return,
        };

        let use_webgl2 = frame_state.webgl2;
        let mode = frame_state.mode;
        let use_2d = mode != SceneMode::Scene3D && !frame_state.scene_3d_only && model.project_to_2d;
        let has_vertical_exaggeration =
            (frame_state.vertical_exaggeration - 1.0).abs() > 1e-6 && model.has_vertical_exaggeration;

        let has_morph_targets = !primitive.morph_targets.is_empty();
        let has_skinning = node.skin.is_some();

        let has_imagery_layers = model.has_imagery_layers;

        let custom_shader = &model.custom_shader;
        let has_custom_shader = custom_shader.is_some();
        let has_custom_fragment_shader =
            custom_shader.as_ref().is_some_and(|cs| cs.fragment_shader_text.is_some());
        let materials_enabled = !has_custom_fragment_shader
            || custom_shader.as_ref().is_none_or(|cs| cs.mode != Some(CustomShaderMode::ReplaceMaterial));

        let has_quantization = primitive.attributes.iter().any(|attr| attr.quantization.is_some());

        let is_triangles = primitive.primitive_type == Some(PrimitiveType::Triangles);
        let generate_wireframe_indices = model.debug_wireframe
            && is_triangles
            && (model.enable_debug_wireframe || use_webgl2);

        let point_cloud_shading = &model.point_cloud_shading;
        let has_attenuation = point_cloud_shading.as_ref().is_some_and(|pcs| pcs.attenuation);
        let has_point_cloud_back_face_culling =
            point_cloud_shading.as_ref().is_some_and(|pcs| pcs.back_face_culling);
        let has_point_cloud_style = primitive.primitive_type == Some(PrimitiveType::Points)
            && (model.has_style || has_attenuation || has_point_cloud_back_face_culling);

        let has_outlines = model.enable_show_outline && primitive.outline_coordinates.is_some();
        let has_edge_visibility = model.edge_geometry_needed;

        let feature_id_flags = self.inspect_feature_ids();
        let has_classification = model.classification_type.is_some();

        // 1. SceneMode2D
        if use_2d {
            self.pipeline_stages.push(PrimitivePipelineStage::SceneMode2D);
        }

        // 2. Geometry
        self.pipeline_stages.push(PrimitivePipelineStage::Geometry);

        // 3. Wireframe
        if generate_wireframe_indices {
            self.pipeline_stages.push(PrimitivePipelineStage::Wireframe);
        }

        // 4. Classification
        if has_classification {
            self.pipeline_stages.push(PrimitivePipelineStage::Classification);
        }

        // 5. MorphTargets
        if has_morph_targets {
            self.pipeline_stages.push(PrimitivePipelineStage::MorphTargets);
        }

        // 6. Skinning
        if has_skinning {
            self.pipeline_stages.push(PrimitivePipelineStage::Skinning);
        }

        // 7. PointCloudStyling
        if has_point_cloud_style {
            self.pipeline_stages.push(PrimitivePipelineStage::PointCloudStyling);
        }

        // 8. Dequantization
        if has_quantization {
            self.pipeline_stages.push(PrimitivePipelineStage::Dequantization);
        }

        // 9. Imagery
        if has_imagery_layers && !has_outlines {
            self.pipeline_stages.push(PrimitivePipelineStage::Imagery);
        }

        // 10. Material
        if materials_enabled {
            self.pipeline_stages.push(PrimitivePipelineStage::Material);
        }

        // 11-13. Struct declarations
        self.pipeline_stages.push(PrimitivePipelineStage::FeatureId);
        self.pipeline_stages.push(PrimitivePipelineStage::Metadata);
        self.pipeline_stages.push(PrimitivePipelineStage::MetadataPicking);

        // 14-16. Property table
        if feature_id_flags.has_property_table {
            self.pipeline_stages.push(PrimitivePipelineStage::SelectedFeatureId);
            self.pipeline_stages.push(PrimitivePipelineStage::BatchTexture);
            self.pipeline_stages.push(PrimitivePipelineStage::CPUStyling);
        }

        // 17. VerticalExaggeration
        if has_vertical_exaggeration {
            self.pipeline_stages.push(PrimitivePipelineStage::VerticalExaggeration);
        }

        // 18. CustomShader
        if has_custom_shader {
            self.pipeline_stages.push(PrimitivePipelineStage::CustomShader);
        }

        // 19. Lighting
        self.pipeline_stages.push(PrimitivePipelineStage::Lighting);

        // 20. Picking
        if model.allow_picking {
            self.pipeline_stages.push(PrimitivePipelineStage::Picking);
        }

        // 21. Outline
        if has_outlines {
            self.pipeline_stages.push(PrimitivePipelineStage::PrimitiveOutline);
        }

        // 22-23. Edge visibility and detection
        if has_edge_visibility {
            self.pipeline_stages.push(PrimitivePipelineStage::EdgeVisibility);
            self.pipeline_stages.push(PrimitivePipelineStage::EdgeDetection);
        }

        // 24. Alpha
        self.pipeline_stages.push(PrimitivePipelineStage::Alpha);

        // 25. Statistics
        self.pipeline_stages.push(PrimitivePipelineStage::PrimitiveStatistics);
    }

    /// Inspects feature IDs from node instances or primitive.
    fn inspect_feature_ids(&self) -> FeatureIdFlags {
        let primitive = match &self.primitive {
            Some(p) => p,
            None => return FeatureIdFlags::default(),
        };
        let node = match &self.node {
            Some(n) => n,
            None => return FeatureIdFlags::default(),
        };
        let model = match &self.model {
            Some(m) => m,
            None => return FeatureIdFlags::default(),
        };

        // Check instances first
        if let Some(ref instances) = node.instances {
            let label = model
                .instance_feature_id_label
                .as_deref()
                .unwrap_or("instanceFeatureId_0");
            if let Some((_, has_pt)) = Self::find_feature_id(&instances.feature_ids, label) {
                return FeatureIdFlags {
                    has_feature_ids: true,
                    has_property_table: has_pt,
                };
            }
        }

        // Check primitive feature IDs
        let label = model.feature_id_label.as_deref().unwrap_or("featureId_0");
        if let Some((_, has_pt)) = Self::find_feature_id(&primitive.feature_ids, label) {
            return FeatureIdFlags {
                has_feature_ids: true,
                has_property_table: has_pt,
            };
        }

        FeatureIdFlags::default()
    }

    fn find_feature_id<'a>(
        feature_ids: &'a [FeatureId],
        label: &str,
    ) -> Option<(&'a FeatureId, bool)> {
        for feat in feature_ids {
            match feat {
                FeatureId::Attribute(attr) => {
                    let matches = attr.label.as_deref() == Some(label)
                        || attr.positional_label.as_deref() == Some(label);
                    if matches {
                        return Some((feat, attr.property_table_id.is_some()));
                    }
                }
                FeatureId::Texture(tex) => {
                    let matches = tex.label.as_deref() == Some(label)
                        || tex.positional_label.as_deref() == Some(label);
                    if matches {
                        return Some((feat, tex.property_table_id.is_some()));
                    }
                }
                FeatureId::ImplicitRange(range) => {
                    let matches = range.label.as_deref() == Some(label)
                        || range.positional_label.as_deref() == Some(label);
                    if matches {
                        return Some((feat, range.property_table_id.is_some()));
                    }
                }
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct FeatureIdFlags {
    pub has_feature_ids: bool,
    pub has_property_table: bool,
}
