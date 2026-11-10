//! SelectedFeatureIdPipelineStage matching CesiumJS `Scene/Model/SelectedFeatureIdPipelineStage.js` (Apache-2.0).

use std::collections::HashMap;

use crate::model_components::{FeatureId, Node, Primitive};
use crate::model_render_resources::ModelUniformCallback;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};

/// Common GLSL shader code for selected feature ID stage.
pub const SELECTED_FEATURE_ID_STAGE_COMMON: &str = r#"vec2 computeSt(float featureId)
{
    float stepX = model_textureStep.x;
    float centerX = model_textureStep.y;

    #ifdef MULTILINE_BATCH_TEXTURE
    float stepY = model_textureStep.z;
    float centerY = model_textureStep.w;

    float xId = mod(featureId, model_textureDimensions.x); 
    float yId = floor(featureId / model_textureDimensions.x);
    
    return vec2(centerX + (xId * stepX), centerY + (yId * stepY));
    #else
    return vec2(centerX + (featureId * stepX), 0.5);
    #endif
}

void selectedFeatureIdStage(out SelectedFeature feature, FeatureIds featureIds)
{   
    int featureId = featureIds.SELECTED_FEATURE_ID;

    if (featureId < model_featuresLength)
    {
        vec2 featureSt = computeSt(float(featureId));

        feature.id = featureId;
        feature.st = featureSt;
        feature.color = texture(model_batchTexture, featureSt);
    }
    else
    {
        feature.id = model_featuresLength + 1;
        feature.st = vec2(0.0);
        feature.color = vec4(1.0);
    }

    #ifdef HAS_NULL_FEATURE_ID
    if (featureId == model_nullFeatureId) {
        feature.id = featureId;
        feature.st = vec2(0.0);
        feature.color = vec4(1.0);
    }
    #endif
}"#;

/// Standalone model state for SelectedFeatureIdPipelineStage.
#[derive(Debug, Default)]
pub struct MockSelectedFeatureIdModel {
    pub feature_id_label: Option<String>,
    pub instance_feature_id_label: Option<String>,
}

/// Standalone runtime node state for SelectedFeatureIdPipelineStage.
#[derive(Debug, Default)]
pub struct MockSelectedFeatureIdRuntimeNode {
    pub node: Node,
}

/// Standalone render resources for SelectedFeatureIdPipelineStage.
#[derive(Default)]
pub struct MockSelectedFeatureIdRenderResources {
    pub shader_builder: ShaderBuilder,
    pub model: MockSelectedFeatureIdModel,
    pub runtime_node: MockSelectedFeatureIdRuntimeNode,
    pub uniform_map: HashMap<String, ModelUniformCallback>,
    pub has_property_table: bool,
    pub bound_null_feature_id: Option<i64>,
}

impl std::fmt::Debug for MockSelectedFeatureIdRenderResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockSelectedFeatureIdRenderResources")
            .field("shader_builder", &self.shader_builder)
            .field("model", &self.model)
            .field("runtime_node", &self.runtime_node)
            .field("has_property_table", &self.has_property_table)
            .field("bound_null_feature_id", &self.bound_null_feature_id)
            .finish()
    }
}

/// Trait abstracting render resources required by [`SelectedFeatureIdPipelineStage`].
pub trait SelectedFeatureIdRenderResources {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder;
    fn set_has_property_table(&mut self, val: bool);
    fn feature_id_label(&self) -> Option<&str>;
    fn instance_feature_id_label(&self) -> Option<&str>;
    fn set_null_feature_id_uniform(&mut self, null_feature_id: i64);
}

impl SelectedFeatureIdRenderResources for MockSelectedFeatureIdRenderResources {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder {
        &mut self.shader_builder
    }

    fn set_has_property_table(&mut self, val: bool) {
        self.has_property_table = val;
    }

    fn feature_id_label(&self) -> Option<&str> {
        self.model.feature_id_label.as_deref()
    }

    fn instance_feature_id_label(&self) -> Option<&str> {
        self.model.instance_feature_id_label.as_deref()
    }

    fn set_null_feature_id_uniform(&mut self, null_feature_id: i64) {
        self.bound_null_feature_id = Some(null_feature_id);
        self.uniform_map.insert(
            "model_nullFeatureId".to_string(),
            Box::new(move || null_feature_id as f64),
        );
    }
}

/// The selected feature ID pipeline stage is responsible for handling the
/// set of feature IDs selected for styling/picking.
pub struct SelectedFeatureIdPipelineStage;

impl SelectedFeatureIdPipelineStage {
    pub const NAME: &'static str = "SelectedFeatureIdPipelineStage";
    pub const STRUCT_ID_SELECTED_FEATURE: &'static str = "SelectedFeature";
    pub const STRUCT_NAME_SELECTED_FEATURE: &'static str = "SelectedFeature";

    /// Process a primitive to configure selected feature IDs.
    pub fn process<R: SelectedFeatureIdRenderResources>(
        render_resources: &mut R,
        node: &Node,
        primitive: &Primitive,
    ) {
        render_resources.set_has_property_table(true);

        let selected = get_selected_feature_ids(
            render_resources.feature_id_label(),
            render_resources.instance_feature_id_label(),
            node,
            primitive,
        );

        let Some(selected) = selected else {
            return;
        };

        let shader_destination = selected.shader_destination;
        let shader_builder = render_resources.shader_builder_mut();

        shader_builder.add_define(
            "HAS_SELECTED_FEATURE_ID",
            None,
            Some(shader_destination),
        );

        shader_builder.add_define(
            "SELECTED_FEATURE_ID",
            Some(&selected.variable_name),
            Some(shader_destination),
        );

        shader_builder.add_define(
            selected.feature_id_define,
            None,
            Some(shader_destination),
        );

        update_feature_struct(shader_builder);

        if let Some(null_feature_id) = selected.null_feature_id {
            let shader_builder = render_resources.shader_builder_mut();
            shader_builder.add_define(
                "HAS_NULL_FEATURE_ID",
                None,
                Some(shader_destination),
            );
            shader_builder.add_uniform(
                "int",
                "model_nullFeatureId",
                Some(shader_destination),
            );
            render_resources.set_null_feature_id_uniform(null_feature_id);
        }

        let shader_builder = render_resources.shader_builder_mut();
        if shader_destination == ShaderDestination::BOTH {
            shader_builder.add_vertex_lines(&[SELECTED_FEATURE_ID_STAGE_COMMON]);
        }
        shader_builder.add_fragment_lines(&[SELECTED_FEATURE_ID_STAGE_COMMON]);
    }
}

struct SelectedFeatureInfo {
    variable_name: String,
    shader_destination: ShaderDestination,
    feature_id_define: &'static str,
    null_feature_id: Option<i64>,
}

fn get_selected_feature_ids(
    feature_id_label: Option<&str>,
    instance_feature_id_label: Option<&str>,
    node: &Node,
    primitive: &Primitive,
) -> Option<SelectedFeatureInfo> {
    if let Some(ref instances) = node.instances {
        if let Some(fid) = find_feature_id(&instances.feature_ids, instance_feature_id_label) {
            return Some(make_feature_info(fid));
        }
    }

    if let Some(fid) = find_feature_id(&primitive.feature_ids, feature_id_label) {
        return Some(make_feature_info(fid));
    }

    None
}

fn find_feature_id<'a>(feature_ids: &'a [FeatureId], label: Option<&str>) -> Option<&'a FeatureId> {
    if let Some(label) = label {
        for fid in feature_ids {
            match fid {
                FeatureId::Attribute(a) => {
                    if a.label.as_deref() == Some(label) || a.positional_label.as_deref() == Some(label) {
                        return Some(fid);
                    }
                }
                FeatureId::Texture(t) => {
                    if t.label.as_deref() == Some(label) || t.positional_label.as_deref() == Some(label) {
                        return Some(fid);
                    }
                }
                FeatureId::ImplicitRange(r) => {
                    if r.label.as_deref() == Some(label) || r.positional_label.as_deref() == Some(label) {
                        return Some(fid);
                    }
                }
            }
        }
    }

    feature_ids.first()
}

fn make_feature_info(fid: &FeatureId) -> SelectedFeatureInfo {
    match fid {
        FeatureId::Texture(t) => {
            let variable_name = t
                .label
                .clone()
                .or_else(|| t.positional_label.clone())
                .unwrap_or_else(|| "featureId_0".to_string());
            SelectedFeatureInfo {
                variable_name,
                shader_destination: ShaderDestination::FRAGMENT,
                feature_id_define: "HAS_SELECTED_FEATURE_ID_TEXTURE",
                null_feature_id: t.null_feature_id,
            }
        }
        FeatureId::Attribute(a) => {
            let variable_name = a
                .label
                .clone()
                .or_else(|| a.positional_label.clone())
                .unwrap_or_else(|| "featureId_0".to_string());
            SelectedFeatureInfo {
                variable_name,
                shader_destination: ShaderDestination::BOTH,
                feature_id_define: "HAS_SELECTED_FEATURE_ID_ATTRIBUTE",
                null_feature_id: a.null_feature_id,
            }
        }
        FeatureId::ImplicitRange(r) => {
            let variable_name = r
                .label
                .clone()
                .or_else(|| r.positional_label.clone())
                .unwrap_or_else(|| "featureId_0".to_string());
            SelectedFeatureInfo {
                variable_name,
                shader_destination: ShaderDestination::BOTH,
                feature_id_define: "HAS_SELECTED_FEATURE_ID_ATTRIBUTE",
                null_feature_id: r.null_feature_id,
            }
        }
    }
}

fn update_feature_struct(shader_builder: &mut ShaderBuilder) {
    shader_builder.add_struct_field(
        SelectedFeatureIdPipelineStage::STRUCT_ID_SELECTED_FEATURE,
        "int",
        "id",
    );
    shader_builder.add_struct_field(
        SelectedFeatureIdPipelineStage::STRUCT_ID_SELECTED_FEATURE,
        "vec2",
        "st",
    );
    shader_builder.add_struct_field(
        SelectedFeatureIdPipelineStage::STRUCT_ID_SELECTED_FEATURE,
        "vec4",
        "color",
    );
}
