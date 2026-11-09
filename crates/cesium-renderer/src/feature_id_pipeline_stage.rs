//! FeatureIdPipelineStage for processing glTF feature ID attributes and textures.
//! Matches CesiumJS `Scene/Model/FeatureIdPipelineStage.js`.

use crate::model_components::{FeatureId, Instances, Primitive};
use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use std::collections::HashMap;

pub const SHADERS_FEATURE_ID_STAGE_VS: &str = r#"
void featureIdStageVS(out FeatureIds featureIds, ProcessedAttributes attributes) {
    initializeFeatureIds(featureIds, attributes);
    initializeFeatureIdAliases(featureIds);
    setFeatureIdVaryings();
}
"#;

pub const SHADERS_FEATURE_ID_STAGE_FS: &str = r#"
void featureIdStageFS(out FeatureIds featureIds, ProcessedAttributes attributes) {
    initializeFeatureIds(featureIds, attributes);
    initializeFeatureIdAliases(featureIds);
}
"#;

fn add_line(sb: &mut ShaderBuilder, fn_id: &str, line: &str) {
    sb.add_function_lines(fn_id, &[line]);
}

/// Render resources passed to FeatureIdPipelineStage.
pub struct FeatureIdRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub uniform_map: &'a mut HashMap<String, Box<dyn Fn() -> serde_json::Value + Send + Sync>>,
    pub attributes: &'a mut Vec<FeatureIdVertexAttribute>,
    pub statistics_geometry_bytes: &'a mut usize,
    pub statistics_textures_bytes: &'a mut usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FeatureIdVertexAttribute {
    pub index: usize,
    pub instance_divisor: Option<usize>,
    pub components_per_attribute: usize,
    pub stride_in_bytes: usize,
    pub offset_in_bytes: usize,
    pub size_in_bytes: usize,
}

pub struct FeatureIdPipelineStage;

impl FeatureIdPipelineStage {
    pub const NAME: &'static str = "FeatureIdPipelineStage";

    pub const STRUCT_ID_FEATURE_IDS_VS: &'static str = "FeatureIdsVS";
    pub const STRUCT_ID_FEATURE_IDS_FS: &'static str = "FeatureIdsFS";
    pub const STRUCT_NAME_FEATURE_IDS: &'static str = "FeatureIds";

    pub const FUNCTION_ID_INITIALIZE_FEATURE_IDS_VS: &'static str = "initializeFeatureIdsVS";
    pub const FUNCTION_ID_INITIALIZE_FEATURE_IDS_FS: &'static str = "initializeFeatureIdsFS";
    pub const FUNCTION_SIGNATURE_INITIALIZE_FEATURE_IDS: &'static str =
        "void initializeFeatureIds(out FeatureIds featureIds, ProcessedAttributes attributes)";

    pub const FUNCTION_ID_INITIALIZE_FEATURE_ID_ALIASES_VS: &'static str =
        "initializeFeatureIdAliasesVS";
    pub const FUNCTION_ID_INITIALIZE_FEATURE_ID_ALIASES_FS: &'static str =
        "initializeFeatureIdAliasesFS";
    pub const FUNCTION_SIGNATURE_INITIALIZE_FEATURE_ID_ALIASES: &'static str =
        "void initializeFeatureIdAliases(inout FeatureIds featureIds)";

    pub const FUNCTION_ID_SET_FEATURE_ID_VARYINGS: &'static str = "setFeatureIdVaryings";
    pub const FUNCTION_SIGNATURE_SET_FEATURE_ID_VARYINGS: &'static str =
        "void setFeatureIdVaryings()";

    pub fn process(
        render_resources: &mut FeatureIdRenderResources<'_>,
        primitive: &Primitive,
        instances: Option<&Instances>,
        primitive_feature_ids: &[FeatureId],
    ) {
        let sb = &mut render_resources.shader_builder;

        // 1. Declare structs
        sb.add_struct(
            Self::STRUCT_ID_FEATURE_IDS_VS,
            Self::STRUCT_NAME_FEATURE_IDS,
            ShaderDestination::VERTEX,
        );
        sb.add_struct(
            Self::STRUCT_ID_FEATURE_IDS_FS,
            Self::STRUCT_NAME_FEATURE_IDS,
            ShaderDestination::FRAGMENT,
        );

        // 2. Declare functions
        sb.add_function(
            Self::FUNCTION_ID_INITIALIZE_FEATURE_IDS_VS,
            Self::FUNCTION_SIGNATURE_INITIALIZE_FEATURE_IDS,
            ShaderDestination::VERTEX,
        );
        sb.add_function(
            Self::FUNCTION_ID_INITIALIZE_FEATURE_IDS_FS,
            Self::FUNCTION_SIGNATURE_INITIALIZE_FEATURE_IDS,
            ShaderDestination::FRAGMENT,
        );

        sb.add_function(
            Self::FUNCTION_ID_INITIALIZE_FEATURE_ID_ALIASES_VS,
            Self::FUNCTION_SIGNATURE_INITIALIZE_FEATURE_ID_ALIASES,
            ShaderDestination::VERTEX,
        );
        sb.add_function(
            Self::FUNCTION_ID_INITIALIZE_FEATURE_ID_ALIASES_FS,
            Self::FUNCTION_SIGNATURE_INITIALIZE_FEATURE_ID_ALIASES,
            ShaderDestination::FRAGMENT,
        );

        sb.add_function(
            Self::FUNCTION_ID_SET_FEATURE_ID_VARYINGS,
            Self::FUNCTION_SIGNATURE_SET_FEATURE_ID_VARYINGS,
            ShaderDestination::VERTEX,
        );

        // 3. Process instance feature IDs
        if let Some(inst) = instances {
            Self::process_instances(render_resources, inst);
        }

        // 4. Process primitive feature IDs
        Self::process_primitive(render_resources, primitive, primitive_feature_ids);

        // 5. Add vertex & fragment lines
        render_resources
            .shader_builder
            .add_vertex_lines(&[SHADERS_FEATURE_ID_STAGE_VS]);
        render_resources
            .shader_builder
            .add_fragment_lines(&[SHADERS_FEATURE_ID_STAGE_FS]);
    }

    fn process_instances(render_resources: &mut FeatureIdRenderResources<'_>, instances: &Instances) {
        let sb = &mut render_resources.shader_builder;
        for (i, feature_id) in instances.feature_ids.iter().enumerate() {
            let fid_name = format!("instanceFeatureId_{}", i);
            sb.add_struct_field(
                Self::STRUCT_ID_FEATURE_IDS_VS,
                "int",
                &fid_name,
            );
            sb.add_struct_field(
                Self::STRUCT_ID_FEATURE_IDS_FS,
                "int",
                &fid_name,
            );

            match feature_id {
                FeatureId::Attribute(attr) => {
                    let set_idx = attr.set_index.unwrap_or(0);
                    let attr_name = format!("a_instanceFeatureId_{}", set_idx);
                    let var_name = format!("v_instanceFeatureId_{}", set_idx);

                    sb.add_varying("float", &var_name, None);

                    let init_vs = format!(
                        "    featureIds.{} = int(czm_round({}));",
                        fid_name, attr_name
                    );
                    let init_fs = format!(
                        "    featureIds.{} = int(czm_round({}));",
                        fid_name, var_name
                    );
                    add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_IDS_VS, &init_vs);
                    add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_IDS_FS, &init_fs);

                    let set_var = format!("    {} = {};", var_name, attr_name);
                    add_line(sb, Self::FUNCTION_ID_SET_FEATURE_ID_VARYINGS, &set_var);

                    if let Some(ref label) = attr.label {
                        sb.add_struct_field(
                            Self::STRUCT_ID_FEATURE_IDS_VS,
                            "int",
                            label,
                        );
                        sb.add_struct_field(
                            Self::STRUCT_ID_FEATURE_IDS_FS,
                            "int",
                            label,
                        );
                        let alias_line = format!("    featureIds.{} = featureIds.{};", label, fid_name);
                        add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_ID_ALIASES_VS, &alias_line);
                        add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_ID_ALIASES_FS, &alias_line);
                    }
                }
                FeatureId::ImplicitRange(range) => {
                    let attr_name = format!("a_implicit_{}", fid_name);
                    let var_name = format!("v_implicit_{}", fid_name);

                    sb.add_attribute("float", &attr_name);
                    sb.add_varying("float", &var_name, None);

                    let init_vs = format!(
                        "    featureIds.{} = int(czm_round({}));",
                        fid_name, attr_name
                    );
                    let init_fs = format!(
                        "    featureIds.{} = int(czm_round({}));",
                        fid_name, var_name
                    );
                    add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_IDS_VS, &init_vs);
                    add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_IDS_FS, &init_fs);

                    let set_var = format!("    {} = {};", var_name, attr_name);
                    add_line(sb, Self::FUNCTION_ID_SET_FEATURE_ID_VARYINGS, &set_var);

                    let attr_idx = render_resources.attributes.len();
                    let byte_len = 64; // instance buffer size
                    render_resources.attributes.push(FeatureIdVertexAttribute {
                        index: attr_idx,
                        instance_divisor: Some(1),
                        components_per_attribute: 1,
                        stride_in_bytes: 4,
                        offset_in_bytes: 0,
                        size_in_bytes: byte_len,
                    });
                    *render_resources.statistics_geometry_bytes += byte_len;

                    if let Some(ref label) = range.label {
                        sb.add_struct_field(
                            Self::STRUCT_ID_FEATURE_IDS_VS,
                            "int",
                            label,
                        );
                        sb.add_struct_field(
                            Self::STRUCT_ID_FEATURE_IDS_FS,
                            "int",
                            label,
                        );
                        let alias_line = format!("    featureIds.{} = featureIds.{};", label, fid_name);
                        add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_ID_ALIASES_VS, &alias_line);
                        add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_ID_ALIASES_FS, &alias_line);
                    }
                }
                _ => {}
            }
        }
    }

    fn process_primitive(
        render_resources: &mut FeatureIdRenderResources<'_>,
        _primitive: &Primitive,
        feature_ids: &[FeatureId],
    ) {
        let sb = &mut render_resources.shader_builder;

        for (i, fid) in feature_ids.iter().enumerate() {
            let fid_name = format!("featureId_{}", i);

            match fid {
                FeatureId::Attribute(attr) => {
                    sb.add_struct_field(
                        Self::STRUCT_ID_FEATURE_IDS_VS,
                        "int",
                        &fid_name,
                    );
                    sb.add_struct_field(
                        Self::STRUCT_ID_FEATURE_IDS_FS,
                        "int",
                        &fid_name,
                    );

                    let set_idx = attr.set_index.unwrap_or(0);
                    let line_vs = format!(
                        "    featureIds.{} = int(czm_round(attributes.featureId_{}));",
                        fid_name, set_idx
                    );
                    let line_fs = format!(
                        "    featureIds.{} = int(czm_round(attributes.featureId_{}));",
                        fid_name, set_idx
                    );

                    add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_IDS_VS, &line_vs);
                    add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_IDS_FS, &line_fs);

                    if let Some(ref label) = attr.label {
                        sb.add_struct_field(
                            Self::STRUCT_ID_FEATURE_IDS_VS,
                            "int",
                            label,
                        );
                        sb.add_struct_field(
                            Self::STRUCT_ID_FEATURE_IDS_FS,
                            "int",
                            label,
                        );
                        let alias = format!("    featureIds.{} = featureIds.{};", label, fid_name);
                        add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_ID_ALIASES_VS, &alias);
                        add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_ID_ALIASES_FS, &alias);
                    }
                }
                FeatureId::ImplicitRange(range) => {
                    sb.add_struct_field(
                        Self::STRUCT_ID_FEATURE_IDS_VS,
                        "int",
                        &fid_name,
                    );
                    sb.add_struct_field(
                        Self::STRUCT_ID_FEATURE_IDS_FS,
                        "int",
                        &fid_name,
                    );

                    let attr_name = format!("a_implicit_{}", fid_name);
                    let var_name = format!("v_implicit_{}", fid_name);

                    sb.add_attribute("float", &attr_name);
                    sb.add_varying("float", &var_name, None);

                    let line_vs = format!(
                        "    featureIds.{} = int(czm_round({}));",
                        fid_name, attr_name
                    );
                    let line_fs = format!(
                        "    featureIds.{} = int(czm_round({}));",
                        fid_name, var_name
                    );

                    add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_IDS_VS, &line_vs);
                    add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_IDS_FS, &line_fs);

                    let set_var = format!("    {} = {};", var_name, attr_name);
                    add_line(sb, Self::FUNCTION_ID_SET_FEATURE_ID_VARYINGS, &set_var);

                    let attr_idx = render_resources.attributes.len();
                    let byte_len = 16;
                    render_resources.attributes.push(FeatureIdVertexAttribute {
                        index: attr_idx,
                        instance_divisor: None,
                        components_per_attribute: 1,
                        stride_in_bytes: 4,
                        offset_in_bytes: 0,
                        size_in_bytes: byte_len,
                    });
                    *render_resources.statistics_geometry_bytes += byte_len;

                    if let Some(ref label) = range.label {
                        sb.add_struct_field(
                            Self::STRUCT_ID_FEATURE_IDS_VS,
                            "int",
                            label,
                        );
                        sb.add_struct_field(
                            Self::STRUCT_ID_FEATURE_IDS_FS,
                            "int",
                            label,
                        );
                        let alias = format!("    featureIds.{} = featureIds.{};", label, fid_name);
                        add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_ID_ALIASES_VS, &alias);
                        add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_ID_ALIASES_FS, &alias);
                    }
                }
                FeatureId::Texture(tex) => {
                    sb.add_struct_field(
                        Self::STRUCT_ID_FEATURE_IDS_FS,
                        "int",
                        &fid_name,
                    );

                    let uniform_name = format!("u_featureIdTexture_{}", i);
                    sb.add_uniform(
                        "sampler2D",
                        &uniform_name,
                        Some(ShaderDestination::FRAGMENT),
                    );

                    let reader = tex.texture_reader.as_ref();
                    let tex_coord = reader.map(|r| r.tex_coord).unwrap_or(0);
                    let channels = reader
                        .and_then(|r| r.channels.as_deref())
                        .unwrap_or("r");

                    let has_transform = reader.and_then(|r| r.transform).is_some();
                    let line_fs = if has_transform {
                        let transform_uniform = format!("u_featureIdTexture_{}Transform", i);
                        sb.add_uniform(
                            "mat3",
                            &transform_uniform,
                            Some(ShaderDestination::FRAGMENT),
                        );
                        format!(
                            "    featureIds.{} = czm_unpackUint(texture({}, vec2({} * vec3(v_texCoord_{}, 1.0))).{});",
                            fid_name, uniform_name, transform_uniform, tex_coord, channels
                        )
                    } else {
                        format!(
                            "    featureIds.{} = czm_unpackUint(texture({}, v_texCoord_{}).{});",
                            fid_name, uniform_name, tex_coord, channels
                        )
                    };

                    add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_IDS_FS, &line_fs);

                    render_resources
                        .uniform_map
                        .insert(uniform_name, Box::new(|| serde_json::json!("texture_0")));

                    if let Some(ref label) = tex.label {
                        sb.add_struct_field(
                            Self::STRUCT_ID_FEATURE_IDS_FS,
                            "int",
                            label,
                        );
                        let alias = format!("    featureIds.{} = featureIds.{};", label, fid_name);
                        add_line(sb, Self::FUNCTION_ID_INITIALIZE_FEATURE_ID_ALIASES_FS, &alias);
                    }
                }
            }
        }
    }
}
