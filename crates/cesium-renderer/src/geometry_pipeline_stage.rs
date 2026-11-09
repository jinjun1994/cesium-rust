//! GeometryPipelineStage matching CesiumJS `Scene/Model/GeometryPipelineStage.js` (Apache-2.0).

use crate::buffer::Buffer;
use crate::classification_pipeline_stage::VertexAttributeSemantic;
use crate::model_components::{Attribute, Primitive};
use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use cesium_math::{AttributeType, ComponentDatatype, DeveloperError, PrimitiveType, SceneMode};

/// Processed vertex attribute added to render resources.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryStageVertexAttribute {
    pub index: u32,
    pub value: Option<Vec<f64>>,
    pub vertex_buffer: Option<Buffer>,
    pub count: usize,
    pub components_per_attribute: usize,
    pub component_datatype: ComponentDatatype,
    pub offset_in_bytes: usize,
    pub stride_in_bytes: Option<usize>,
    pub normalize: bool,
}

/// Model options relevant for GeometryPipelineStage.
#[derive(Clone, Debug, Default)]
pub struct GeometryModelOptions {
    pub model_type: Option<String>,
    pub project_to_2d: bool,
}

/// Runtime primitive options relevant for 2D.
#[derive(Clone, Debug, Default)]
pub struct GeometryRuntimePrimitive {
    pub position_buffer_2d: Option<Buffer>,
}

/// Runtime node options relevant for instancing.
#[derive(Clone, Debug, Default)]
pub struct GeometryRuntimeNode {
    pub is_instanced: bool,
}

/// Render resources for GeometryPipelineStage.
pub struct GeometryRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub attributes: Vec<GeometryStageVertexAttribute>,
    pub attribute_index: u32,
    pub count: usize,
    pub model: GeometryModelOptions,
    pub runtime_node: GeometryRuntimeNode,
    pub runtime_primitive: GeometryRuntimePrimitive,
    pub feature_id_vertex_attribute_set_index: usize,
}

/// Frame state for GeometryPipelineStage.
#[derive(Clone, Debug)]
pub struct GeometryFrameState {
    pub mode: SceneMode,
    pub scene_3d_only: bool,
    pub webgl2: bool,
}

impl Default for GeometryFrameState {
    fn default() -> Self {
        Self {
            mode: SceneMode::Scene3D,
            scene_3d_only: false,
            webgl2: true,
        }
    }
}

/// Attribute info computed for processing an attribute.
#[derive(Clone, Debug)]
pub struct AttributeInfo<'a> {
    pub attribute: &'a Attribute,
    pub is_quantized: bool,
    pub variable_name: String,
    pub has_semantic: bool,
    pub glsl_type: &'static str,
    pub quantized_glsl_type: Option<&'static str>,
}

pub struct GeometryPipelineStage;

impl GeometryPipelineStage {
    pub const NAME: &'static str = "GeometryPipelineStage";
    pub const STRUCT_ID_PROCESSED_ATTRIBUTES_VS: &'static str = "ProcessedAttributesVS";
    pub const STRUCT_ID_PROCESSED_ATTRIBUTES_FS: &'static str = "ProcessedAttributesFS";
    pub const STRUCT_NAME_PROCESSED_ATTRIBUTES: &'static str = "ProcessedAttributes";
    pub const STRUCT_ID_SELECTED_FEATURE: &'static str = "SelectedFeature";
    pub const STRUCT_NAME_SELECTED_FEATURE: &'static str = "SelectedFeature";
    pub const FUNCTION_ID_INITIALIZE_ATTRIBUTES: &'static str = "initializeAttributes";
    pub const FUNCTION_SIGNATURE_INITIALIZE_ATTRIBUTES: &'static str =
        "void initializeAttributes(out ProcessedAttributes attributes)";
    pub const FUNCTION_ID_SET_DYNAMIC_VARYINGS_VS: &'static str = "setDynamicVaryingsVS";
    pub const FUNCTION_ID_SET_DYNAMIC_VARYINGS_FS: &'static str = "setDynamicVaryingsFS";
    pub const FUNCTION_SIGNATURE_SET_DYNAMIC_VARYINGS: &'static str =
        "void setDynamicVaryings(inout ProcessedAttributes attributes)";

    pub const GEOMETRY_STAGE_VS: &'static str = r#"vec4 geometryStage(inout ProcessedAttributes attributes, mat4 modelView, mat3 normal)
{
    vec4 computedPosition;

    // Compute positions in different coordinate systems
    vec3 positionMC = attributes.positionMC;
    v_positionMC = positionMC;
    v_positionEC = (modelView * vec4(positionMC, 1.0)).xyz;

    #if defined(USE_2D_POSITIONS) || defined(USE_2D_INSTANCING)
    vec3 position2D = attributes.position2D;
    vec3 positionEC = (u_modelView2D * vec4(position2D, 1.0)).xyz;
    computedPosition = czm_projection * vec4(positionEC, 1.0);
    #else
    computedPosition = czm_projection * vec4(v_positionEC, 1.0);
    #endif

    // Sometimes the custom shader and/or style needs this
    #if defined(COMPUTE_POSITION_WC_CUSTOM_SHADER) || defined(COMPUTE_POSITION_WC_STYLE) || defined(COMPUTE_POSITION_WC_ATMOSPHERE) || defined(ENABLE_CLIPPING_POLYGONS)
    // Note that this is a 32-bit position which may result in jitter on small
    // scales.
    v_positionWC = (czm_model * vec4(positionMC, 1.0)).xyz;
    #endif

    #ifdef HAS_NORMALS
    v_normalEC = normalize(normal * attributes.normalMC);
    #endif

    #ifdef HAS_TANGENTS
    v_tangentEC = normalize(normal * attributes.tangentMC);
    #endif

    #ifdef HAS_BITANGENTS
    v_bitangentEC = normalize(normal * attributes.bitangentMC);
    #endif

    // All other varyings need to be dynamically generated in
    // GeometryPipelineStage
    setDynamicVaryings(attributes);

    return computedPosition;
}"#;

    pub const GEOMETRY_STAGE_FS: &'static str = r#"void geometryStage(out ProcessedAttributes attributes)
{
  attributes.positionMC = v_positionMC;
  attributes.positionEC = v_positionEC;

  #if defined(COMPUTE_POSITION_WC_CUSTOM_SHADER) || defined(COMPUTE_POSITION_WC_STYLE) || defined(COMPUTE_POSITION_WC_ATMOSPHERE)
  attributes.positionWC = v_positionWC;
  #endif

  #ifdef HAS_NORMALS
  // renormalize after interpolation
  attributes.normalEC = normalize(v_normalEC);
  #endif

  #ifdef HAS_TANGENTS
  attributes.tangentEC = normalize(v_tangentEC);
  #endif

  #ifdef HAS_BITANGENTS
  attributes.bitangentEC = normalize(v_bitangentEC);
  #endif

  // Everything else is dynamically generated in GeometryPipelineStage
  setDynamicVaryings(attributes);
}"#;

    /// Processes a primitive's vertex attributes and injects geometry stage shaders.
    pub fn process(
        render_resources: &mut GeometryRenderResources<'_>,
        primitive: &Primitive,
        frame_state: &GeometryFrameState,
    ) -> Result<(), DeveloperError> {
        let shader_builder = &mut render_resources.shader_builder;

        // ProcessedAttributes structs
        shader_builder.add_struct(
            Self::STRUCT_ID_PROCESSED_ATTRIBUTES_VS,
            Self::STRUCT_NAME_PROCESSED_ATTRIBUTES,
            ShaderDestination::VERTEX,
        );
        shader_builder.add_struct(
            Self::STRUCT_ID_PROCESSED_ATTRIBUTES_FS,
            Self::STRUCT_NAME_PROCESSED_ATTRIBUTES,
            ShaderDestination::FRAGMENT,
        );

        // Feature struct
        shader_builder.add_struct(
            Self::STRUCT_ID_SELECTED_FEATURE,
            Self::STRUCT_NAME_SELECTED_FEATURE,
            ShaderDestination::BOTH,
        );

        // initializeAttributes function in VS
        shader_builder.add_function(
            Self::FUNCTION_ID_INITIALIZE_ATTRIBUTES,
            Self::FUNCTION_SIGNATURE_INITIALIZE_ATTRIBUTES,
            ShaderDestination::VERTEX,
        );

        // Position varyings and struct fields
        shader_builder.add_varying("vec3", "v_positionWC", None);
        shader_builder.add_varying("vec3", "v_positionEC", None);
        shader_builder.add_struct_field(
            Self::STRUCT_ID_PROCESSED_ATTRIBUTES_FS,
            "vec3",
            "positionWC",
        );
        shader_builder.add_struct_field(
            Self::STRUCT_ID_PROCESSED_ATTRIBUTES_FS,
            "vec3",
            "positionEC",
        );

        // Dynamic varyings functions
        shader_builder.add_function(
            Self::FUNCTION_ID_SET_DYNAMIC_VARYINGS_VS,
            Self::FUNCTION_SIGNATURE_SET_DYNAMIC_VARYINGS,
            ShaderDestination::VERTEX,
        );
        shader_builder.add_function(
            Self::FUNCTION_ID_SET_DYNAMIC_VARYINGS_FS,
            Self::FUNCTION_SIGNATURE_SET_DYNAMIC_VARYINGS,
            ShaderDestination::FRAGMENT,
        );

        // sRGB color for PNTS
        if render_resources.model.model_type.as_deref() == Some("TILE_PNTS") {
            shader_builder.add_define(
                "HAS_SRGB_COLOR",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }

        // 2D check
        let use_2d = frame_state.mode != SceneMode::Scene3D
            && !frame_state.scene_3d_only
            && render_resources.model.project_to_2d;

        let instanced = render_resources.runtime_node.is_instanced;
        let increment_index_for_2d = use_2d && !instanced;

        for attribute in &primitive.attributes {
            if attribute.buffer.is_none() && attribute.constant.is_none() {
                return Err(DeveloperError::new(
                    "Attributes must be provided as a Buffer or constant value",
                ));
            }

            let attr_type = attribute.attribute_type.unwrap_or(AttributeType::Scalar);
            let attribute_location_count = attr_type.get_attribute_location_count();

            let is_position_attribute =
                attribute.semantic.as_deref() == Some(VertexAttributeSemantic::POSITION);

            let index = if attribute_location_count > 1 {
                let idx = render_resources.attribute_index;
                render_resources.attribute_index += attribute_location_count as u32;
                idx
            } else if is_position_attribute && !increment_index_for_2d {
                0
            } else {
                let idx = render_resources.attribute_index;
                render_resources.attribute_index += 1;
                idx
            };

            let interpolation_qualifier = if !is_position_attribute
                && primitive.primitive_type == Some(PrimitiveType::Points)
                && frame_state.webgl2
            {
                Some("flat")
            } else {
                None
            };

            process_attribute(
                render_resources,
                attribute,
                index,
                attribute_location_count,
                use_2d,
                instanced,
                interpolation_qualifier,
            )?;
        }

        handle_bitangents(render_resources.shader_builder, &primitive.attributes);

        if primitive.primitive_type == Some(PrimitiveType::Points) {
            render_resources
                .shader_builder
                .add_define("PRIMITIVE_TYPE_POINTS", None, None);
        }

        render_resources
            .shader_builder
            .add_vertex_lines(&[Self::GEOMETRY_STAGE_VS]);
        render_resources
            .shader_builder
            .add_fragment_lines(&[Self::GEOMETRY_STAGE_FS]);

        Ok(())
    }
}

pub fn get_attribute_info(attribute: &Attribute) -> Result<AttributeInfo<'_>, DeveloperError> {
    let semantic = attribute.semantic.as_deref();
    let set_index = attribute.set_index;

    let (variable_name, has_semantic) = if let Some(sem) = semantic {
        (get_semantic_variable_name(sem, set_index), true)
    } else if let Some(ref name) = attribute.name {
        let trimmed = name.strip_prefix('_').unwrap_or(name);
        (trimmed.to_lowercase(), false)
    } else {
        return Err(DeveloperError::new("Attribute must have a semantic or name"));
    };

    let is_vertex_color = variable_name.starts_with("color_") || variable_name == "color";
    let attr_type = attribute.attribute_type.unwrap_or(AttributeType::Scalar);
    let glsl_type = if is_vertex_color {
        "vec4"
    } else {
        attr_type.get_glsl_type()
    };

    let is_quantized = attribute.quantization.is_some();
    let quantized_glsl_type = if let Some(ref q) = attribute.quantization {
        if is_vertex_color {
            Some("vec4")
        } else {
            Some(q.attribute_type.unwrap_or(attr_type).get_glsl_type())
        }
    } else {
        None
    };

    Ok(AttributeInfo {
        attribute,
        is_quantized,
        variable_name,
        has_semantic,
        glsl_type,
        quantized_glsl_type,
    })
}

fn get_semantic_variable_name(semantic: &str, set_index: Option<usize>) -> String {
    let base = match semantic {
        "POSITION" => "positionMC",
        "NORMAL" => "normalMC",
        "TANGENT" => "tangentMC",
        "TEXCOORD" => "texCoord",
        "COLOR" => "color",
        "JOINTS" => "joints",
        "WEIGHTS" => "weights",
        "_FEATURE_ID" | "FEATURE_ID" => "featureId",
        "SCALE" | "_SCALE" | "KHR_gaussian_splatting:SCALE" => "scale",
        "ROTATION" | "_ROTATION" | "KHR_gaussian_splatting:ROTATION" => "rotation",
        "CUMULATIVE_DISTANCE" | "BENTLEY_materials_line_style:CUMULATIVE_DISTANCE" => {
            "cumulativeDistance"
        }
        other => other,
    };

    if let Some(idx) = set_index {
        format!("{base}_{idx}")
    } else {
        base.to_string()
    }
}

fn process_attribute(
    render_resources: &mut GeometryRenderResources<'_>,
    attribute: &Attribute,
    attribute_index: u32,
    attribute_location_count: usize,
    use_2d: bool,
    instanced: bool,
    interpolation_qualifier: Option<&str>,
) -> Result<(), DeveloperError> {
    let attribute_info = get_attribute_info(attribute)?;
    let modify_for_2d = use_2d && !instanced;

    if attribute_location_count > 1 {
        add_matrix_attribute_to_render_resources(
            render_resources,
            attribute,
            attribute_index,
            attribute_location_count,
        )?;
    } else {
        add_attribute_to_render_resources(
            render_resources,
            attribute,
            attribute_index,
            modify_for_2d,
        )?;
    }

    add_attribute_declaration(
        render_resources.shader_builder,
        &attribute_info,
        modify_for_2d,
    )?;

    add_varying_declaration(
        render_resources.shader_builder,
        &attribute_info,
        interpolation_qualifier,
    );

    if attribute.semantic.is_some() {
        add_semantic_define(render_resources.shader_builder, attribute);
    }

    update_attributes_struct(render_resources.shader_builder, &attribute_info, use_2d);
    update_initialize_attributes_function(
        render_resources.shader_builder,
        &attribute_info,
        modify_for_2d,
    );
    update_set_dynamic_varyings_function(render_resources.shader_builder, &attribute_info);

    Ok(())
}

fn add_attribute_to_render_resources(
    render_resources: &mut GeometryRenderResources<'_>,
    attribute: &Attribute,
    attribute_index: u32,
    modify_for_2d: bool,
) -> Result<(), DeveloperError> {
    let (attr_type, component_datatype) = if let Some(ref q) = attribute.quantization {
        (
            q.attribute_type
                .or(attribute.attribute_type)
                .unwrap_or(AttributeType::Scalar),
            q.component_datatype
                .or(attribute.component_datatype)
                .unwrap_or(ComponentDatatype::Float),
        )
    } else {
        (
            attribute.attribute_type.unwrap_or(AttributeType::Scalar),
            attribute
                .component_datatype
                .unwrap_or(ComponentDatatype::Float),
        )
    };

    if let Some(ref sem) = attribute.semantic {
        if (sem == "_FEATURE_ID" || sem == "FEATURE_ID")
            && attribute.set_index.unwrap_or(0)
                >= render_resources.feature_id_vertex_attribute_set_index
        {
            render_resources.feature_id_vertex_attribute_set_index =
                attribute.set_index.unwrap_or(0) + 1;
        }
    }

    let is_position_attribute =
        attribute.semantic.as_deref() == Some(VertexAttributeSemantic::POSITION);
    let index = if is_position_attribute {
        0
    } else {
        attribute_index
    };
    let components_per_attribute = attr_type.get_number_of_components();

    let vertex_attribute = GeometryStageVertexAttribute {
        index,
        value: if attribute.buffer.is_some() {
            None
        } else {
            attribute.constant.clone()
        },
        vertex_buffer: attribute.buffer.clone(),
        count: attribute.count,
        components_per_attribute,
        component_datatype,
        offset_in_bytes: attribute.byte_offset,
        stride_in_bytes: attribute.byte_stride,
        normalize: attribute.normalized,
    };

    render_resources.attributes.push(vertex_attribute);

    if !is_position_attribute || !modify_for_2d {
        return Ok(());
    }

    // Add an additional attribute for projected 2D positions
    let buffer_2d = render_resources
        .runtime_primitive
        .position_buffer_2d
        .clone();
    let position_attribute_2d = GeometryStageVertexAttribute {
        index: attribute_index,
        value: None,
        vertex_buffer: buffer_2d,
        count: attribute.count,
        components_per_attribute,
        component_datatype: ComponentDatatype::Float,
        offset_in_bytes: 0,
        stride_in_bytes: None,
        normalize: attribute.normalized,
    };

    render_resources.attributes.push(position_attribute_2d);

    Ok(())
}

fn add_matrix_attribute_to_render_resources(
    render_resources: &mut GeometryRenderResources<'_>,
    attribute: &Attribute,
    attribute_index: u32,
    column_count: usize,
) -> Result<(), DeveloperError> {
    let (attr_type, component_datatype) = if let Some(ref q) = attribute.quantization {
        (
            q.attribute_type
                .or(attribute.attribute_type)
                .unwrap_or(AttributeType::Scalar),
            q.component_datatype
                .or(attribute.component_datatype)
                .unwrap_or(ComponentDatatype::Float),
        )
    } else {
        (
            attribute.attribute_type.unwrap_or(AttributeType::Scalar),
            attribute
                .component_datatype
                .unwrap_or(ComponentDatatype::Float),
        )
    };

    let component_count = attr_type.get_number_of_components();
    let components_per_column = component_count / column_count;
    let component_size_in_bytes = component_datatype.size_in_bytes();
    let column_length_in_bytes = components_per_column * component_size_in_bytes;
    let stride_in_bytes = attribute.byte_stride;

    for i in 0..column_count {
        let offset_in_bytes = attribute.byte_offset + i * column_length_in_bytes;

        let column_attribute = GeometryStageVertexAttribute {
            index: attribute_index + i as u32,
            value: if attribute.buffer.is_some() {
                None
            } else {
                attribute.constant.clone()
            },
            vertex_buffer: attribute.buffer.clone(),
            count: attribute.count,
            components_per_attribute: components_per_column,
            component_datatype,
            offset_in_bytes,
            stride_in_bytes,
            normalize: attribute.normalized,
        };

        render_resources.attributes.push(column_attribute);
    }

    Ok(())
}

fn add_attribute_declaration(
    shader_builder: &mut ShaderBuilder,
    attribute_info: &AttributeInfo<'_>,
    modify_for_2d: bool,
) -> Result<(), DeveloperError> {
    let is_position = attribute_info.attribute.semantic.as_deref()
        == Some(VertexAttributeSemantic::POSITION);
    let variable_name = &attribute_info.variable_name;

    let (attr_name, glsl_type) = if attribute_info.is_quantized {
        (
            format!("a_quantized_{variable_name}"),
            attribute_info.quantized_glsl_type.unwrap_or("vec4"),
        )
    } else {
        (format!("a_{variable_name}"), attribute_info.glsl_type)
    };

    if is_position {
        shader_builder.set_position_attribute(glsl_type, &attr_name)?;
    } else {
        shader_builder.add_attribute(glsl_type, &attr_name);
    }

    if is_position && modify_for_2d {
        shader_builder.add_attribute("vec3", "a_position2D");
    }

    Ok(())
}

fn add_varying_declaration(
    shader_builder: &mut ShaderBuilder,
    attribute_info: &AttributeInfo<'_>,
    interpolation_qualifier: Option<&str>,
) {
    let variable_name = &attribute_info.variable_name;
    let (varying_name, glsl_type) = if variable_name == "normalMC" {
        ("v_normalEC".to_string(), attribute_info.glsl_type)
    } else if variable_name == "tangentMC" {
        ("v_tangentEC".to_string(), "vec3")
    } else {
        (format!("v_{variable_name}"), attribute_info.glsl_type)
    };

    shader_builder.add_varying(glsl_type, &varying_name, interpolation_qualifier);
}

fn add_semantic_define(shader_builder: &mut ShaderBuilder, attribute: &Attribute) {
    let Some(ref semantic) = attribute.semantic else {
        return;
    };
    let set_index = attribute.set_index.unwrap_or(0);

    match semantic.as_str() {
        "NORMAL" => {
            shader_builder.add_define("HAS_NORMALS", None, None);
        }
        "TANGENT" => {
            shader_builder.add_define("HAS_TANGENTS", None, None);
        }
        "_FEATURE_ID" | "FEATURE_ID" => {
            if semantic.starts_with('_') {
                shader_builder.add_define(&format!("HAS{semantic}_{set_index}"), None, None);
            } else {
                shader_builder.add_define(&format!("HAS_{semantic}_{set_index}"), None, None);
            }
        }
        "TEXCOORD" | "COLOR" => {
            shader_builder.add_define(&format!("HAS_{semantic}_{set_index}"), None, None);
        }
        _ => {}
    }
}

fn update_attributes_struct(
    shader_builder: &mut ShaderBuilder,
    attribute_info: &AttributeInfo<'_>,
    use_2d: bool,
) {
    let vs_struct_id = GeometryPipelineStage::STRUCT_ID_PROCESSED_ATTRIBUTES_VS;
    let fs_struct_id = GeometryPipelineStage::STRUCT_ID_PROCESSED_ATTRIBUTES_FS;
    let variable_name = &attribute_info.variable_name;
    let glsl_type = attribute_info.glsl_type;

    if variable_name == "tangentMC" {
        shader_builder.add_struct_field(vs_struct_id, "vec3", "tangentMC");
        shader_builder.add_struct_field(vs_struct_id, "float", "tangentSignMC");
        shader_builder.add_struct_field(fs_struct_id, "vec3", "tangentEC");
    } else if variable_name == "normalMC" {
        shader_builder.add_struct_field(vs_struct_id, "vec3", "normalMC");
        shader_builder.add_struct_field(fs_struct_id, "vec3", "normalEC");
    } else {
        shader_builder.add_struct_field(vs_struct_id, glsl_type, variable_name);
        shader_builder.add_struct_field(fs_struct_id, glsl_type, variable_name);
    }

    if variable_name == "positionMC" && use_2d {
        shader_builder.add_struct_field(vs_struct_id, "vec3", "position2D");
    }
}

fn update_initialize_attributes_function(
    shader_builder: &mut ShaderBuilder,
    attribute_info: &AttributeInfo<'_>,
    modify_for_2d: bool,
) {
    let function_id = GeometryPipelineStage::FUNCTION_ID_INITIALIZE_ATTRIBUTES;
    let variable_name = &attribute_info.variable_name;

    let use_2d_position = variable_name == "positionMC" && modify_for_2d;
    if use_2d_position {
        shader_builder.add_function_lines(function_id, &["attributes.position2D = a_position2D;"]);
    }

    if attribute_info.is_quantized {
        return;
    }

    if variable_name == "tangentMC" {
        shader_builder.add_function_lines(
            function_id,
            &[
                "attributes.tangentMC = a_tangentMC.xyz;",
                "attributes.tangentSignMC = a_tangentMC.w;",
            ],
        );
    } else {
        let line = format!("attributes.{variable_name} = a_{variable_name};");
        shader_builder.add_function_lines(function_id, &[&line]);
    }
}

fn update_set_dynamic_varyings_function(
    shader_builder: &mut ShaderBuilder,
    attribute_info: &AttributeInfo<'_>,
) {
    let semantic = attribute_info.attribute.semantic.as_deref();
    let set_index = attribute_info.attribute.set_index;

    if semantic.is_some() && set_index.is_none() {
        return;
    }

    let variable_name = &attribute_info.variable_name;

    let vs_line = format!("v_{variable_name} = attributes.{variable_name};");
    shader_builder.add_function_lines(
        GeometryPipelineStage::FUNCTION_ID_SET_DYNAMIC_VARYINGS_VS,
        &[&vs_line],
    );

    let fs_line = format!("attributes.{variable_name} = v_{variable_name};");
    shader_builder.add_function_lines(
        GeometryPipelineStage::FUNCTION_ID_SET_DYNAMIC_VARYINGS_FS,
        &[&fs_line],
    );
}

fn handle_bitangents(shader_builder: &mut ShaderBuilder, attributes: &[Attribute]) {
    let mut has_normals = false;
    let mut has_tangents = false;
    for attr in attributes {
        match attr.semantic.as_deref() {
            Some(VertexAttributeSemantic::NORMAL) => has_normals = true,
            Some(VertexAttributeSemantic::TANGENT) => has_tangents = true,
            _ => {}
        }
    }

    if !has_normals || !has_tangents {
        return;
    }

    shader_builder.add_define("HAS_BITANGENTS", None, None);
    shader_builder.add_varying("vec3", "v_bitangentEC", None);
    shader_builder.add_struct_field(
        GeometryPipelineStage::STRUCT_ID_PROCESSED_ATTRIBUTES_VS,
        "vec3",
        "bitangentMC",
    );
    shader_builder.add_struct_field(
        GeometryPipelineStage::STRUCT_ID_PROCESSED_ATTRIBUTES_FS,
        "vec3",
        "bitangentEC",
    );
}
