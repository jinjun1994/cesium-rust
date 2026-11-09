//! CustomShaderPipelineStage matching CesiumJS `Scene/Model/CustomShaderPipelineStage.js` (Apache-2.0).

use crate::alpha_pipeline_stage::ModelAlphaOptions;
use crate::commands::Pass;
use crate::custom_shader::{
    CustomShader, CustomShaderTranslucencyMode, LightingModel,
};
use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use std::collections::{BTreeSet, HashMap};

pub const CUSTOM_SHADER_STAGE_VS: &str = r#"
void customShaderStage(
    inout czm_modelVertexOutput vsOutput, 
    inout ProcessedAttributes attributes, 
    FeatureIds featureIds,
    Metadata metadata,
    MetadataClass metadataClass,
    MetadataStatistics metadataStatistics
) {
    VertexInput vsInput;
    initializeInputStruct(vsInput, attributes);
    vsInput.featureIds = featureIds;
    vsInput.metadata = metadata;
    vsInput.metadataClass = metadataClass;
    vsInput.metadataStatistics = metadataStatistics;
    vertexMain(vsInput, vsOutput);
    attributes.positionMC = vsOutput.positionMC;
}
"#;

pub const CUSTOM_SHADER_STAGE_FS: &str = r#"
void customShaderStage(
    inout czm_modelMaterial material,
    ProcessedAttributes attributes,
    FeatureIds featureIds,
    Metadata metadata,
    MetadataClass metadataClass,
    MetadataStatistics metadataStatistics
) {
    FragmentInput fsInput;
    initializeInputStruct(fsInput, attributes);
    fsInput.featureIds = featureIds;
    fsInput.metadata = metadata;
    fsInput.metadataClass = metadataClass;
    fsInput.metadataStatistics = metadataStatistics;
    fragmentMain(fsInput, material);
}
"#;

/// Primitive attribute information used for custom shader stage.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomShaderAttributeInfo {
    pub variable_name: String,
    pub glsl_type: String,
}

/// Primitive descriptor for CustomShaderPipelineStage.
#[derive(Clone, Debug, Default)]
pub struct CustomShaderPrimitive {
    pub attributes: Vec<CustomShaderAttributeInfo>,
    pub property_ids: BTreeSet<String>,
}

/// Render resources for CustomShaderPipelineStage.
pub struct CustomShaderRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub lighting_model: &'a mut Option<LightingModel>,
    pub alpha_options: &'a mut ModelAlphaOptions,
    pub custom_shader: &'a CustomShader,
    pub uniform_map: &'a mut HashMap<String, Box<dyn Fn() -> serde_json::Value + Send + Sync>>,
}

/// Pipeline stage inserting custom shader GLSL callbacks into vertex and fragment shaders.
pub struct CustomShaderPipelineStage;

impl CustomShaderPipelineStage {
    pub const NAME: &'static str = "CustomShaderPipelineStage";

    pub const STRUCT_ID_ATTRIBUTES_VS: &'static str = "AttributesVS";
    pub const STRUCT_ID_ATTRIBUTES_FS: &'static str = "AttributesFS";
    pub const STRUCT_NAME_ATTRIBUTES: &'static str = "Attributes";
    pub const STRUCT_ID_VERTEX_INPUT: &'static str = "VertexInput";
    pub const STRUCT_NAME_VERTEX_INPUT: &'static str = "VertexInput";
    pub const STRUCT_ID_FRAGMENT_INPUT: &'static str = "FragmentInput";
    pub const STRUCT_NAME_FRAGMENT_INPUT: &'static str = "FragmentInput";
    pub const FUNCTION_ID_INITIALIZE_INPUT_STRUCT_VS: &'static str = "initializeInputStructVS";
    pub const FUNCTION_SIGNATURE_INITIALIZE_INPUT_STRUCT_VS: &'static str =
        "void initializeInputStruct(out VertexInput vsInput, ProcessedAttributes attributes)";
    pub const FUNCTION_ID_INITIALIZE_INPUT_STRUCT_FS: &'static str = "initializeInputStructFS";
    pub const FUNCTION_SIGNATURE_INITIALIZE_INPUT_STRUCT_FS: &'static str =
        "void initializeInputStruct(out FragmentInput fsInput, ProcessedAttributes attributes)";

    /// Processes a primitive with a CustomShader.
    pub fn process(
        render_resources: &mut CustomShaderRenderResources<'_>,
        primitive: &CustomShaderPrimitive,
    ) {
        let custom_shader = render_resources.custom_shader;

        if let Some(lighting_model) = custom_shader.lighting_model {
            *render_resources.lighting_model = Some(lighting_model);
        }

        match custom_shader.translucency_mode {
            CustomShaderTranslucencyMode::Translucent => {
                render_resources.alpha_options.pass = Some(Pass::Translucent);
            }
            CustomShaderTranslucencyMode::Opaque => {
                render_resources.alpha_options.pass = None;
            }
            CustomShaderTranslucencyMode::Inherit => {}
        }

        let attributes_by_name: HashMap<String, String> = primitive
            .attributes
            .iter()
            .map(|a| (a.variable_name.clone(), a.glsl_type.clone()))
            .collect();

        let generated_code = generate_shader_lines(
            custom_shader,
            &attributes_by_name,
            &primitive.property_ids,
        );

        if !generated_code.custom_shader_enabled {
            return;
        }

        let sb = &mut render_resources.shader_builder;

        add_lines_to_shader(sb, custom_shader, &generated_code);

        if generated_code.should_compute_position_wc {
            sb.add_define(
                "COMPUTE_POSITION_WC_CUSTOM_SHADER",
                None,
                Some(ShaderDestination::BOTH),
            );
        }

        if custom_shader.vertex_shader_text.is_some() && generated_code.vertex_lines.enabled {
            sb.add_define(
                "HAS_CUSTOM_VERTEX_SHADER",
                None,
                Some(ShaderDestination::VERTEX),
            );
        }

        if custom_shader.fragment_shader_text.is_some() && generated_code.fragment_lines.enabled {
            sb.add_define(
                "HAS_CUSTOM_FRAGMENT_SHADER",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
            sb.add_define(
                custom_shader.mode.get_define_name(),
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }

        let uniforms_guard = custom_shader.uniforms.read().unwrap();
        for (uniform_name, spec) in uniforms_guard.iter() {
            sb.add_uniform(
                spec.uniform_type.glsl_type(),
                uniform_name,
                Some(ShaderDestination::BOTH),
            );
        }

        for (varying_name, vtype) in &custom_shader.varyings {
            sb.add_varying(vtype.glsl_type(), varying_name, None);
        }

        for uniform_name in uniforms_guard.keys() {
            let name = uniform_name.clone();
            let uniforms_ref = std::sync::Arc::clone(&custom_shader.uniforms);
            render_resources.uniform_map.insert(
                name.clone(),
                Box::new(move || {
                    let guard = uniforms_ref.read().unwrap();
                    let val = &guard.get(&name).unwrap().value;
                    serde_json::json!(format!("{val:?}"))
                }),
            );
        }
    }
}

struct GeneratedStageLines {
    enabled: bool,
    attribute_fields: Vec<(String, String)>,
    initialization_lines: Vec<String>,
}

struct GeneratedCode {
    vertex_lines: GeneratedStageLines,
    fragment_lines: GeneratedStageLines,
    custom_shader_enabled: bool,
    should_compute_position_wc: bool,
}

fn infer_attribute_defaults(name: &str) -> Option<(&'static str, &'static str)> {
    let mut trimmed = name;
    if let Some(pos) = trimmed.rfind('_') {
        if trimmed[pos + 1..].chars().all(|c| c.is_ascii_digit()) {
            trimmed = &trimmed[..pos];
        }
    }
    if trimmed.ends_with("MC") || trimmed.ends_with("EC") {
        trimmed = &trimmed[..trimmed.len() - 2];
    }

    match trimmed {
        "position" => Some(("vec3", "vec3(0.0)")),
        "normal" => Some(("vec3", "vec3(0.0, 0.0, 1.0)")),
        "tangent" => Some(("vec3", "vec3(1.0, 0.0, 0.0)")),
        "bitangent" => Some(("vec3", "vec3(0.0, 1.0, 0.0)")),
        "texCoord" => Some(("vec2", "vec2(0.0)")),
        "color" => Some(("vec4", "vec4(1.0)")),
        "joints" => Some(("ivec4", "ivec4(0)")),
        "weights" => Some(("vec4", "vec4(0.0)")),
        _ => None,
    }
}

fn check_metadata_compatibility(
    used_metadata: &BTreeSet<String>,
    primitive_properties: &BTreeSet<String>,
) -> bool {
    for prop in used_metadata {
        if !primitive_properties.contains(prop) {
            return false;
        }
    }
    true
}

fn generate_vertex_shader_lines(
    custom_shader: &CustomShader,
    attributes_by_name: &HashMap<String, String>,
    primitive_properties: &BTreeSet<String>,
) -> GeneratedStageLines {
    if custom_shader.vertex_shader_text.is_none() {
        return GeneratedStageLines {
            enabled: false,
            attribute_fields: Vec::new(),
            initialization_lines: Vec::new(),
        };
    }

    let used_vars = &custom_shader.used_variables_vertex;
    if !check_metadata_compatibility(&used_vars.metadata_set, primitive_properties) {
        return GeneratedStageLines {
            enabled: false,
            attribute_fields: Vec::new(),
            initialization_lines: Vec::new(),
        };
    }

    let mut attribute_fields = Vec::new();
    let mut initialization_lines = Vec::new();

    for var_name in &used_vars.attribute_set {
        if let Some(glsl_type) = attributes_by_name.get(var_name) {
            attribute_fields.push((glsl_type.clone(), var_name.clone()));
            initialization_lines.push(format!("vsInput.attributes.{var_name} = attributes.{var_name};"));
        } else if let Some((def_type, def_val)) = infer_attribute_defaults(var_name) {
            attribute_fields.push((def_type.to_string(), var_name.clone()));
            initialization_lines.push(format!("vsInput.attributes.{var_name} = {def_val};"));
        } else {
            return GeneratedStageLines {
                enabled: false,
                attribute_fields: Vec::new(),
                initialization_lines: Vec::new(),
            };
        }
    }

    GeneratedStageLines {
        enabled: true,
        attribute_fields,
        initialization_lines,
    }
}

fn generate_fragment_shader_lines(
    custom_shader: &CustomShader,
    attributes_by_name: &HashMap<String, String>,
    primitive_properties: &BTreeSet<String>,
) -> GeneratedStageLines {
    if custom_shader.fragment_shader_text.is_none() {
        return GeneratedStageLines {
            enabled: false,
            attribute_fields: Vec::new(),
            initialization_lines: Vec::new(),
        };
    }

    let used_vars = &custom_shader.used_variables_fragment;
    if !check_metadata_compatibility(&used_vars.metadata_set, primitive_properties) {
        return GeneratedStageLines {
            enabled: false,
            attribute_fields: Vec::new(),
            initialization_lines: Vec::new(),
        };
    }

    let mut attribute_fields = Vec::new();
    let mut initialization_lines = Vec::new();

    for var_name in &used_vars.attribute_set {
        if var_name == "positionWC" {
            attribute_fields.push(("vec3".to_string(), "positionWC".to_string()));
            initialization_lines.push("fsInput.attributes.positionWC = attributes.positionWC;".to_string());
        } else if var_name == "positionEC" {
            attribute_fields.push(("vec3".to_string(), "positionEC".to_string()));
            initialization_lines.push("fsInput.attributes.positionEC = attributes.positionEC;".to_string());
        } else if let Some(glsl_type) = attributes_by_name.get(var_name) {
            attribute_fields.push((glsl_type.clone(), var_name.clone()));
            initialization_lines.push(format!("fsInput.attributes.{var_name} = attributes.{var_name};"));
        } else if let Some((def_type, def_val)) = infer_attribute_defaults(var_name) {
            attribute_fields.push((def_type.to_string(), var_name.clone()));
            initialization_lines.push(format!("fsInput.attributes.{var_name} = {def_val};"));
        } else {
            return GeneratedStageLines {
                enabled: false,
                attribute_fields: Vec::new(),
                initialization_lines: Vec::new(),
            };
        }
    }

    GeneratedStageLines {
        enabled: true,
        attribute_fields,
        initialization_lines,
    }
}

fn generate_shader_lines(
    custom_shader: &CustomShader,
    attributes_by_name: &HashMap<String, String>,
    primitive_properties: &BTreeSet<String>,
) -> GeneratedCode {
    let vertex_lines = generate_vertex_shader_lines(
        custom_shader,
        attributes_by_name,
        primitive_properties,
    );
    let fragment_lines = generate_fragment_shader_lines(
        custom_shader,
        attributes_by_name,
        primitive_properties,
    );

    let should_compute_position_wc = custom_shader
        .used_variables_fragment
        .attribute_set
        .contains("positionWC")
        && fragment_lines.enabled;

    let custom_shader_enabled = vertex_lines.enabled || fragment_lines.enabled;

    GeneratedCode {
        vertex_lines,
        fragment_lines,
        custom_shader_enabled,
        should_compute_position_wc,
    }
}

fn add_vertex_lines_to_shader(sb: &mut ShaderBuilder, lines: &GeneratedStageLines) {
    let struct_id = CustomShaderPipelineStage::STRUCT_ID_ATTRIBUTES_VS;
    sb.add_struct(
        struct_id,
        CustomShaderPipelineStage::STRUCT_NAME_ATTRIBUTES,
        ShaderDestination::VERTEX,
    );

    for (glsl_type, var_name) in &lines.attribute_fields {
        sb.add_struct_field(struct_id, glsl_type, var_name);
    }

    let input_id = CustomShaderPipelineStage::STRUCT_ID_VERTEX_INPUT;
    sb.add_struct(
        input_id,
        CustomShaderPipelineStage::STRUCT_NAME_VERTEX_INPUT,
        ShaderDestination::VERTEX,
    );
    sb.add_struct_field(input_id, CustomShaderPipelineStage::STRUCT_NAME_ATTRIBUTES, "attributes");
    sb.add_struct_field(input_id, "FeatureIds", "featureIds");
    sb.add_struct_field(input_id, "Metadata", "metadata");
    sb.add_struct_field(input_id, "MetadataClass", "metadataClass");
    sb.add_struct_field(input_id, "MetadataStatistics", "metadataStatistics");

    let func_id = CustomShaderPipelineStage::FUNCTION_ID_INITIALIZE_INPUT_STRUCT_VS;
    sb.add_function(
        func_id,
        CustomShaderPipelineStage::FUNCTION_SIGNATURE_INITIALIZE_INPUT_STRUCT_VS,
        ShaderDestination::VERTEX,
    );
    let init_lines: Vec<&str> = lines.initialization_lines.iter().map(|s| s.as_str()).collect();
    sb.add_function_lines(func_id, &init_lines);
}

fn add_fragment_lines_to_shader(sb: &mut ShaderBuilder, lines: &GeneratedStageLines) {
    let struct_id = CustomShaderPipelineStage::STRUCT_ID_ATTRIBUTES_FS;
    sb.add_struct(
        struct_id,
        CustomShaderPipelineStage::STRUCT_NAME_ATTRIBUTES,
        ShaderDestination::FRAGMENT,
    );

    for (glsl_type, var_name) in &lines.attribute_fields {
        sb.add_struct_field(struct_id, glsl_type, var_name);
    }

    let input_id = CustomShaderPipelineStage::STRUCT_ID_FRAGMENT_INPUT;
    sb.add_struct(
        input_id,
        CustomShaderPipelineStage::STRUCT_NAME_FRAGMENT_INPUT,
        ShaderDestination::FRAGMENT,
    );
    sb.add_struct_field(input_id, CustomShaderPipelineStage::STRUCT_NAME_ATTRIBUTES, "attributes");
    sb.add_struct_field(input_id, "FeatureIds", "featureIds");
    sb.add_struct_field(input_id, "Metadata", "metadata");
    sb.add_struct_field(input_id, "MetadataClass", "metadataClass");
    sb.add_struct_field(input_id, "MetadataStatistics", "metadataStatistics");

    let func_id = CustomShaderPipelineStage::FUNCTION_ID_INITIALIZE_INPUT_STRUCT_FS;
    sb.add_function(
        func_id,
        CustomShaderPipelineStage::FUNCTION_SIGNATURE_INITIALIZE_INPUT_STRUCT_FS,
        ShaderDestination::FRAGMENT,
    );
    let init_lines: Vec<&str> = lines.initialization_lines.iter().map(|s| s.as_str()).collect();
    sb.add_function_lines(func_id, &init_lines);
}

fn add_lines_to_shader(
    sb: &mut ShaderBuilder,
    custom_shader: &CustomShader,
    generated_code: &GeneratedCode,
) {
    if generated_code.vertex_lines.enabled {
        add_vertex_lines_to_shader(sb, &generated_code.vertex_lines);

        let vs_text = custom_shader.vertex_shader_text.as_deref().unwrap_or_default();
        sb.add_vertex_lines(&["#line 0", vs_text, CUSTOM_SHADER_STAGE_VS]);
    }

    if generated_code.fragment_lines.enabled {
        add_fragment_lines_to_shader(sb, &generated_code.fragment_lines);

        let fs_text = custom_shader.fragment_shader_text.as_deref().unwrap_or_default();
        sb.add_fragment_lines(&["#line 0", fs_text, CUSTOM_SHADER_STAGE_FS]);
    }
}
