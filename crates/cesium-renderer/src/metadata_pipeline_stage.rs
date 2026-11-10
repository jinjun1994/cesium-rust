//! MetadataPipelineStage matching CesiumJS `Scene/Model/MetadataPipelineStage.js` (Apache-2.0).

use std::collections::{BTreeMap, HashMap, HashSet};
use serde_json::Value;

use cesium_math::Matrix3;
use crate::custom_shader::CustomShader;
use crate::model_components::{ModelTextureReader, Primitive};
pub use crate::metadata_class_property::MetadataClassProperty;
pub use crate::metadata_type::MetadataType;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};

pub const STRUCT_ID_METADATA_VS: &str = "MetadataVS";
pub const STRUCT_ID_METADATA_FS: &str = "MetadataFS";
pub const STRUCT_NAME_METADATA: &str = "Metadata";

pub const STRUCT_ID_METADATA_CLASS_VS: &str = "MetadataClassVS";
pub const STRUCT_ID_METADATA_CLASS_FS: &str = "MetadataClassFS";
pub const STRUCT_NAME_METADATA_CLASS: &str = "MetadataClass";

pub const STRUCT_ID_METADATA_STATISTICS_VS: &str = "MetadataStatisticsVS";
pub const STRUCT_ID_METADATA_STATISTICS_FS: &str = "MetadataStatisticsFS";
pub const STRUCT_NAME_METADATA_STATISTICS: &str = "MetadataStatistics";

pub const FUNCTION_ID_INITIALIZE_METADATA_VS: &str = "initializeMetadataVS";
pub const FUNCTION_ID_INITIALIZE_METADATA_FS: &str = "initializeMetadataFS";
pub const FUNCTION_SIGNATURE_INITIALIZE_METADATA: &str =
    "void initializeMetadata(FeatureIds featureIds, out Metadata metadata, out MetadataClass metadataClass, out MetadataStatistics metadataStatistics, ProcessedAttributes attributes)";
pub const FUNCTION_ID_SET_METADATA_VARYINGS: &str = "setMetadataVaryings";
pub const FUNCTION_SIGNATURE_SET_METADATA_VARYINGS: &str = "void setMetadataVaryings()";

pub const SHADERS_METADATA_STAGE_VS: &str = r#"void metadataStage(
  FeatureIds featureIds,
  out Metadata metadata,
  out MetadataClass metadataClass,
  out MetadataStatistics metadataStatistics,
  ProcessedAttributes attributes
  )
{
  initializeMetadata(featureIds, metadata, metadataClass, metadataStatistics, attributes);
  setMetadataVaryings();
}"#;

pub const SHADERS_METADATA_STAGE_FS: &str = r#"void metadataStage(
  FeatureIds featureIds,
  out Metadata metadata,
  out MetadataClass metadataClass,
  out MetadataStatistics metadataStatistics,
  ProcessedAttributes attributes
  )
{
  initializeMetadata(featureIds, metadata, metadataClass, metadataStatistics, attributes);
}"#;

/// Formats a serde_json::Value as a flattened comma-separated GLSL constructor argument string.
pub fn format_glsl_value(val: &Value) -> String {
    match val {
        Value::Array(arr) => {
            let parts: Vec<String> = arr.iter().map(format_glsl_value).collect();
            parts.join(",")
        }
        Value::Number(num) => num.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        _ => String::new(),
    }
}

/// Metadata class field mapping.
#[derive(Debug, Clone, Copy)]
pub struct MetadataClassField {
    pub spec_name: &'static str,
    pub shader_name: &'static str,
}

pub const METADATA_CLASS_FIELDS: [MetadataClassField; 4] = [
    MetadataClassField { spec_name: "noData", shader_name: "noData" },
    MetadataClassField { spec_name: "default", shader_name: "defaultValue" },
    MetadataClassField { spec_name: "min", shader_name: "minValue" },
    MetadataClassField { spec_name: "max", shader_name: "maxValue" },
];

/// Metadata statistics field mapping.
#[derive(Debug, Clone, Copy)]
pub struct MetadataStatisticsField {
    pub spec_name: &'static str,
    pub shader_name: &'static str,
    pub is_float: bool,
}

pub const METADATA_STATISTICS_FIELDS: [MetadataStatisticsField; 7] = [
    MetadataStatisticsField { spec_name: "min", shader_name: "minValue", is_float: false },
    MetadataStatisticsField { spec_name: "max", shader_name: "maxValue", is_float: false },
    MetadataStatisticsField { spec_name: "mean", shader_name: "mean", is_float: true },
    MetadataStatisticsField { spec_name: "median", shader_name: "median", is_float: false },
    MetadataStatisticsField { spec_name: "standardDeviation", shader_name: "standardDeviation", is_float: true },
    MetadataStatisticsField { spec_name: "variance", shader_name: "variance", is_float: true },
    MetadataStatisticsField { spec_name: "sum", shader_name: "sum", is_float: false },
];

/// Property in a PropertyAttribute.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyAttributeProperty {
    pub attribute: String,
    pub glsl_type: String,
    pub metadata_type: MetadataType,
    pub offset: Option<Value>,
    pub scale: Option<Value>,
    pub has_value_transform: bool,
    pub no_data: Option<Value>,
    pub default_value: Option<Value>,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
}

impl Default for PropertyAttributeProperty {
    fn default() -> Self {
        Self {
            attribute: String::new(),
            glsl_type: "float".to_string(),
            metadata_type: MetadataType::Scalar,
            offset: None,
            scale: None,
            has_value_transform: false,
            no_data: None,
            default_value: None,
            min_value: None,
            max_value: None,
        }
    }
}

/// Property attribute grouping properties by class.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PropertyAttribute {
    pub class_id: String,
    pub properties: BTreeMap<String, PropertyAttributeProperty>,
}

/// Property in a PropertyTexture.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyTextureProperty {
    pub texture_reader: ModelTextureReader,
    pub channels: String,
    pub glsl_type: String,
    pub metadata_type: MetadataType,
    pub component_count: usize,
    pub normalized: bool,
    pub is_float_scalar: bool,
    pub is_int_scalar: bool,
    pub offset: Option<Value>,
    pub scale: Option<Value>,
    pub has_value_transform: bool,
    pub no_data: Option<Value>,
    pub default_value: Option<Value>,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
    pub is_gpu_compatible: bool,
}

impl Default for PropertyTextureProperty {
    fn default() -> Self {
        Self {
            texture_reader: ModelTextureReader {
                index: 0,
                texture: None,
                tex_coord: 0,
                scale: None,
                transform: None,
                channels: None,
                constant_lod: None,
            },
            channels: "r".to_string(),
            glsl_type: "float".to_string(),
            metadata_type: MetadataType::Scalar,
            component_count: 1,
            normalized: false,
            is_float_scalar: false,
            is_int_scalar: false,
            offset: None,
            scale: None,
            has_value_transform: false,
            no_data: None,
            default_value: None,
            min_value: None,
            max_value: None,
            is_gpu_compatible: true,
        }
    }
}

/// Property texture object in structural metadata.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PropertyTexture {
    pub id: usize,
    pub class_id: String,
    pub properties: BTreeMap<String, PropertyTextureProperty>,
}

/// Statistics for a single property.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PropertyStatistics {
    pub min: Option<Value>,
    pub max: Option<Value>,
    pub mean: Option<Value>,
    pub median: Option<Value>,
    pub standard_deviation: Option<Value>,
    pub variance: Option<Value>,
    pub sum: Option<Value>,
}

/// Statistics grouped by class.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetadataStatistics {
    pub classes: HashMap<String, HashMap<String, PropertyStatistics>>,
}

/// Structural metadata for a model.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StructuralMetadata {
    pub property_attributes: Vec<PropertyAttribute>,
    pub property_textures: Vec<PropertyTexture>,
}

impl StructuralMetadata {
    pub fn get_property_texture(&self, index: usize) -> Option<&PropertyTexture> {
        self.property_textures.get(index)
    }
}

/// Point cloud style variables for styling shader injection.
#[derive(Debug, Clone, Default)]
pub struct PointCloudStyle {
    pub color_variables: Vec<String>,
    pub show_variables: Vec<String>,
    pub point_size_variables: Vec<String>,
}

/// Model options relevant to MetadataPipelineStage.
#[derive(Debug, Clone, Default)]
pub struct MetadataModel {
    pub structural_metadata: Option<StructuralMetadata>,
    pub custom_shader: Option<CustomShader>,
    pub statistics: Option<MetadataStatistics>,
    pub style: Option<PointCloudStyle>,
}

/// Render resources passed to MetadataPipelineStage.
pub struct MetadataRenderResources<'a> {
    pub shader_builder: &'a mut ShaderBuilder,
    pub model: &'a mut MetadataModel,
    pub uniform_map: &'a mut HashMap<String, Box<dyn Fn() -> Value + Send + Sync>>,
}

/// Context for frame state.
#[derive(Debug, Clone, Copy)]
pub struct MetadataFrameContext {
    pub webgl2: bool,
}

impl Default for MetadataFrameContext {
    fn default() -> Self {
        Self { webgl2: true }
    }
}

/// Frame state passed to MetadataPipelineStage.
#[derive(Debug, Clone, Copy, Default)]
pub struct MetadataFrameState {
    pub context: MetadataFrameContext,
}

/// Pipeline stage for processing EXT_structural_metadata.
pub struct MetadataPipelineStage;

impl MetadataPipelineStage {
    pub const NAME: &'static str = "MetadataPipelineStage";

    pub fn sanitize_glsl_identifier(id: &str) -> String {
        let mut result = String::new();
        for (i, c) in id.chars().enumerate() {
            if c.is_ascii_alphanumeric() || c == '_' {
                if i == 0 && c.is_ascii_digit() {
                    result.push('_');
                }
                result.push(c);
            } else {
                result.push('_');
            }
        }
        result
    }

    /// Process a primitive to add structural metadata to the shader.
    pub fn process(
        render_resources: &mut MetadataRenderResources<'_>,
        primitive: &Primitive,
        frame_state: &MetadataFrameState,
    ) {
        let structural_metadata = render_resources
            .model
            .structural_metadata
            .clone()
            .unwrap_or_default();
        let statistics = render_resources.model.statistics.clone();
        let webgl2 = frame_state.context.webgl2;

        let mut class_types = HashSet::new();
        let mut statistics_types = HashSet::new();

        for attr in &structural_metadata.property_attributes {
            for (prop_id, prop) in &attr.properties {
                class_types.insert(prop.glsl_type.clone());
                if let Some(stats) = &statistics {
                    if let Some(class_stats) = stats.classes.get(&attr.class_id) {
                        if class_stats.contains_key(prop_id) && prop.metadata_type != MetadataType::Enum {
                            statistics_types.insert(prop.glsl_type.clone());
                        }
                    }
                }
            }
        }

        for tex in &structural_metadata.property_textures {
            for (prop_id, prop) in &tex.properties {
                if prop.is_gpu_compatible {
                    class_types.insert(prop.glsl_type.clone());
                    if let Some(stats) = &statistics {
                        if let Some(class_stats) = stats.classes.get(&tex.class_id) {
                            if class_stats.contains_key(prop_id) && prop.metadata_type != MetadataType::Enum {
                                statistics_types.insert(prop.glsl_type.clone());
                            }
                        }
                    }
                }
            }
        }

        for glsl_type in &class_types {
            let struct_name = format!("{glsl_type}MetadataClass");
            render_resources.shader_builder.add_struct(
                &struct_name,
                &struct_name,
                ShaderDestination::BOTH,
            );
            for field in &METADATA_CLASS_FIELDS {
                render_resources.shader_builder.add_struct_field(
                    &struct_name,
                    glsl_type,
                    field.shader_name,
                );
            }
        }

        for glsl_type in &statistics_types {
            let struct_name = format!("{glsl_type}MetadataStatistics");
            render_resources.shader_builder.add_struct(
                &struct_name,
                &struct_name,
                ShaderDestination::BOTH,
            );
            for field in &METADATA_STATISTICS_FIELDS {
                let field_type = if field.is_float {
                    match glsl_type.as_str() {
                        "int" => "float",
                        "ivec2" => "vec2",
                        "ivec3" => "vec3",
                        "ivec4" => "vec4",
                        other => other,
                    }
                } else {
                    glsl_type.as_str()
                };
                render_resources.shader_builder.add_struct_field(
                    &struct_name,
                    field_type,
                    field.shader_name,
                );
            }
        }

        Self::declare_structs_and_functions(render_resources.shader_builder);
        render_resources
            .shader_builder
            .add_vertex_lines(&[SHADERS_METADATA_STAGE_VS]);
        render_resources
            .shader_builder
            .add_fragment_lines(&[SHADERS_METADATA_STAGE_FS]);

        for property_attribute in &structural_metadata.property_attributes {
            Self::process_property_attribute(
                render_resources,
                property_attribute,
                primitive,
                statistics.as_ref(),
            );
        }

        for property_texture in &structural_metadata.property_textures {
            Self::process_property_texture(
                render_resources,
                property_texture,
                statistics.as_ref(),
                webgl2,
            );
        }
    }

    fn declare_structs_and_functions(shader_builder: &mut ShaderBuilder) {
        shader_builder.add_struct(
            STRUCT_ID_METADATA_VS,
            STRUCT_NAME_METADATA,
            ShaderDestination::VERTEX,
        );
        shader_builder.add_struct(
            STRUCT_ID_METADATA_FS,
            STRUCT_NAME_METADATA,
            ShaderDestination::FRAGMENT,
        );

        shader_builder.add_struct(
            STRUCT_ID_METADATA_CLASS_VS,
            STRUCT_NAME_METADATA_CLASS,
            ShaderDestination::VERTEX,
        );
        shader_builder.add_struct(
            STRUCT_ID_METADATA_CLASS_FS,
            STRUCT_NAME_METADATA_CLASS,
            ShaderDestination::FRAGMENT,
        );

        shader_builder.add_struct(
            STRUCT_ID_METADATA_STATISTICS_VS,
            STRUCT_NAME_METADATA_STATISTICS,
            ShaderDestination::VERTEX,
        );
        shader_builder.add_struct(
            STRUCT_ID_METADATA_STATISTICS_FS,
            STRUCT_NAME_METADATA_STATISTICS,
            ShaderDestination::FRAGMENT,
        );

        shader_builder.add_function(
            FUNCTION_ID_INITIALIZE_METADATA_VS,
            FUNCTION_SIGNATURE_INITIALIZE_METADATA,
            ShaderDestination::VERTEX,
        );
        shader_builder.add_function(
            FUNCTION_ID_INITIALIZE_METADATA_FS,
            FUNCTION_SIGNATURE_INITIALIZE_METADATA,
            ShaderDestination::FRAGMENT,
        );

        shader_builder.add_function(
            FUNCTION_ID_SET_METADATA_VARYINGS,
            FUNCTION_SIGNATURE_SET_METADATA_VARYINGS,
            ShaderDestination::VERTEX,
        );
    }

    fn process_property_attribute(
        render_resources: &mut MetadataRenderResources<'_>,
        property_attribute: &PropertyAttribute,
        primitive: &Primitive,
        statistics: Option<&MetadataStatistics>,
    ) {
        let class_stats = statistics.and_then(|s| s.classes.get(&property_attribute.class_id));

        for (property_id, property) in &property_attribute.properties {
            let metadata_var = Self::sanitize_glsl_identifier(property_id);
            let glsl_type = &property.glsl_type;

            let model_attr = primitive
                .attributes
                .iter()
                .find(|a| a.name.as_deref() == Some(&property.attribute));

            let attr_var_name = model_attr
                .and_then(|a| a.name.clone())
                .unwrap_or_else(|| property.attribute.clone());

            let mut value_expr = format!("attributes.{attr_var_name}");
            if property.has_value_transform {
                let offset_name = format!("u_{metadata_var}_offset");
                let scale_name = format!("u_{metadata_var}_scale");

                render_resources.shader_builder.add_uniform(
                    glsl_type,
                    &offset_name,
                    Some(ShaderDestination::BOTH),
                );
                render_resources.shader_builder.add_uniform(
                    glsl_type,
                    &scale_name,
                    Some(ShaderDestination::BOTH),
                );

                let offset_val = property.offset.clone().unwrap_or(Value::from(0.0));
                let scale_val = property.scale.clone().unwrap_or(Value::from(1.0));
                render_resources.uniform_map.insert(
                    offset_name.clone(),
                    Box::new(move || offset_val.clone()),
                );
                render_resources.uniform_map.insert(
                    scale_name.clone(),
                    Box::new(move || scale_val.clone()),
                );

                value_expr = format!("czm_valueTransform({offset_name}, {scale_name}, {value_expr})");
            }

            render_resources.shader_builder.add_struct_field(
                STRUCT_ID_METADATA_VS,
                glsl_type,
                &metadata_var,
            );
            render_resources.shader_builder.add_struct_field(
                STRUCT_ID_METADATA_FS,
                glsl_type,
                &metadata_var,
            );

            let init_line = format!("metadata.{metadata_var} = {value_expr};");
            render_resources.shader_builder.add_function_lines(
                FUNCTION_ID_INITIALIZE_METADATA_VS,
                &[&init_line],
            );
            render_resources.shader_builder.add_function_lines(
                FUNCTION_ID_INITIALIZE_METADATA_FS,
                &[&init_line],
            );

            let class_struct_name = format!("{glsl_type}MetadataClass");
            render_resources.shader_builder.add_struct_field(
                STRUCT_ID_METADATA_CLASS_VS,
                &class_struct_name,
                &metadata_var,
            );
            render_resources.shader_builder.add_struct_field(
                STRUCT_ID_METADATA_CLASS_FS,
                &class_struct_name,
                &metadata_var,
            );

            let mut class_assignments = Vec::new();
            if let Some(nd) = &property.no_data {
                class_assignments.push(format!("metadataClass.{metadata_var}.noData = {glsl_type}({});", format_glsl_value(nd)));
            }
            if let Some(def) = &property.default_value {
                class_assignments.push(format!("metadataClass.{metadata_var}.defaultValue = {glsl_type}({});", format_glsl_value(def)));
            }
            if let Some(min) = &property.min_value {
                class_assignments.push(format!("metadataClass.{metadata_var}.minValue = {glsl_type}({});", format_glsl_value(min)));
            }
            if let Some(max) = &property.max_value {
                class_assignments.push(format!("metadataClass.{metadata_var}.maxValue = {glsl_type}({});", format_glsl_value(max)));
            }

            let class_assign_refs: Vec<&str> = class_assignments.iter().map(|s| s.as_str()).collect();
            render_resources.shader_builder.add_function_lines(
                FUNCTION_ID_INITIALIZE_METADATA_VS,
                &class_assign_refs,
            );
            render_resources.shader_builder.add_function_lines(
                FUNCTION_ID_INITIALIZE_METADATA_FS,
                &class_assign_refs,
            );

            if let Some(c_stats) = class_stats {
                if let Some(p_stats) = c_stats.get(property_id) {
                    if property.metadata_type != MetadataType::Enum {
                        let stats_struct_name = format!("{glsl_type}MetadataStatistics");
                        render_resources.shader_builder.add_struct_field(
                            STRUCT_ID_METADATA_STATISTICS_VS,
                            &stats_struct_name,
                            &metadata_var,
                        );
                        render_resources.shader_builder.add_struct_field(
                            STRUCT_ID_METADATA_STATISTICS_FS,
                            &stats_struct_name,
                            &metadata_var,
                        );

                        let mut stats_lines = Vec::new();
                        for field in &METADATA_STATISTICS_FIELDS {
                            let val_opt = match field.spec_name {
                                "min" => &p_stats.min,
                                "max" => &p_stats.max,
                                "mean" => &p_stats.mean,
                                "median" => &p_stats.median,
                                "standardDeviation" => &p_stats.standard_deviation,
                                "variance" => &p_stats.variance,
                                "sum" => &p_stats.sum,
                                _ => &None,
                            };
                            if let Some(val) = val_opt {
                                let f_type = if field.is_float {
                                    match glsl_type.as_str() {
                                        "int" => "float",
                                        "ivec2" => "vec2",
                                        "ivec3" => "vec3",
                                        "ivec4" => "vec4",
                                        other => other,
                                    }
                                } else {
                                    glsl_type.as_str()
                                };
                                stats_lines.push(format!(
                                    "metadataStatistics.{metadata_var}.{} = {f_type}({});",
                                    field.shader_name,
                                    format_glsl_value(val)
                                ));
                            }
                        }
                        let stats_refs: Vec<&str> = stats_lines.iter().map(|s| s.as_str()).collect();
                        render_resources.shader_builder.add_function_lines(
                            FUNCTION_ID_INITIALIZE_METADATA_VS,
                            &stats_refs,
                        );
                        render_resources.shader_builder.add_function_lines(
                            FUNCTION_ID_INITIALIZE_METADATA_FS,
                            &stats_refs,
                        );
                    }
                }
            }
        }
    }

    fn process_property_texture(
        render_resources: &mut MetadataRenderResources<'_>,
        property_texture: &PropertyTexture,
        statistics: Option<&MetadataStatistics>,
        webgl2: bool,
    ) {
        let class_stats = statistics.and_then(|s| s.classes.get(&property_texture.class_id));

        for (property_id, property) in &property_texture.properties {
            if !property.is_gpu_compatible {
                continue;
            }

            let metadata_var = Self::sanitize_glsl_identifier(property_id);
            let glsl_type = &property.glsl_type;

            let reader = &property.texture_reader;
            let tex_uniform_name = format!("u_propertyTexture_{}", reader.index);

            if !render_resources.uniform_map.contains_key(&tex_uniform_name) {
                render_resources.shader_builder.add_uniform(
                    "sampler2D",
                    &tex_uniform_name,
                    Some(ShaderDestination::FRAGMENT),
                );
                let texture_val = Value::from(reader.index);
                render_resources.uniform_map.insert(
                    tex_uniform_name.clone(),
                    Box::new(move || texture_val.clone()),
                );
            }

            render_resources.shader_builder.add_struct_field(
                STRUCT_ID_METADATA_FS,
                glsl_type,
                &metadata_var,
            );

            let tex_coord_var = format!("attributes.texCoord_{}", reader.tex_coord);
            let mut tex_coord_expr = tex_coord_var.clone();

            if let Some(transform) = reader.transform {
                if transform != Matrix3::IDENTITY {
                    let transform_uniform = format!("{tex_uniform_name}Transform");
                    render_resources.shader_builder.add_uniform(
                        "mat3",
                        &transform_uniform,
                        Some(ShaderDestination::FRAGMENT),
                    );
                    render_resources.uniform_map.insert(
                        transform_uniform.clone(),
                        Box::new(move || Value::from(format!("{transform:?}"))),
                    );
                    tex_coord_expr = format!("vec2({transform_uniform} * vec3({tex_coord_var}, 1.0))");
                }
            }

            let mut init_lines = Vec::new();
            let raw_channels = format!("texture({tex_uniform_name}, {tex_coord_expr}).{}", property.channels);

            let unpacked_var = if webgl2 {
                let num_channels = property.channels.len();
                let component_count = property.component_count.max(1);
                let channels_per_component = (num_channels / component_count).max(1);

                let raw_channels_name = format!("{metadata_var}_rawChannels");
                let raw_bits_name = format!("{metadata_var}_rawBits");
                let unpacked_value_name = format!("{metadata_var}_unpackedValue");

                init_lines.push(format!("{glsl_type} {unpacked_value_name};"));
                init_lines.push(format!("uint {raw_bits_name};"));

                let float_type = match num_channels {
                    1 => "float",
                    2 => "vec2",
                    3 => "vec3",
                    4 => "vec4",
                    _ => "vec4",
                };
                init_lines.push(format!("{float_type} {raw_channels_name} = {raw_channels};"));

                let cast_fn = if property.is_float_scalar {
                    "uintBitsToFloat"
                } else if property.is_int_scalar {
                    "int"
                } else {
                    ""
                };

                let has_multiple_components = component_count > 1;

                for i in 0..component_count {
                    let sub_channels = if num_channels > 1 {
                        let start = i * channels_per_component;
                        let end = (i + 1) * channels_per_component;
                        let slice = match (start, end) {
                            (0, 1) => "r",
                            (1, 2) => "g",
                            (2, 3) => "b",
                            (3, 4) => "a",
                            (0, 2) => "rg",
                            (2, 4) => "ba",
                            (0, 3) => "rgb",
                            (0, 4) => "rgba",
                            _ => "r",
                        };
                        format!(".{slice}")
                    } else {
                        String::new()
                    };

                    init_lines.push(format!("{raw_bits_name} = czm_unpackTexture({raw_channels_name}{sub_channels});"));

                    let index_expr = if has_multiple_components {
                        format!("[{i}]")
                    } else {
                        String::new()
                    };

                    let assign_line = if property.normalized {
                        format!("{unpacked_value_name}{index_expr} = float({cast_fn}({raw_bits_name})) * 0.00392156862745098;")
                    } else if property.is_float_scalar {
                        format!("{unpacked_value_name}{index_expr} = ({cast_fn}({raw_bits_name}));")
                    } else {
                        format!("{unpacked_value_name}{index_expr} = (({raw_bits_name}));")
                    };
                    init_lines.push(assign_line);
                }

                unpacked_value_name
            } else if property.normalized {
                raw_channels
            } else {
                format!("{glsl_type}(255.0 * {raw_channels})")
            };

            let mut final_expr = unpacked_var;
            if property.has_value_transform {
                let offset_name = format!("u_{metadata_var}_offset");
                let scale_name = format!("u_{metadata_var}_scale");

                render_resources.shader_builder.add_uniform(
                    glsl_type,
                    &offset_name,
                    Some(ShaderDestination::FRAGMENT),
                );
                render_resources.shader_builder.add_uniform(
                    glsl_type,
                    &scale_name,
                    Some(ShaderDestination::FRAGMENT),
                );

                let offset_val = property.offset.clone().unwrap_or(Value::from(0.0));
                let scale_val = property.scale.clone().unwrap_or(Value::from(1.0));
                render_resources.uniform_map.insert(
                    offset_name.clone(),
                    Box::new(move || offset_val.clone()),
                );
                render_resources.uniform_map.insert(
                    scale_name.clone(),
                    Box::new(move || scale_val.clone()),
                );

                final_expr = format!("czm_valueTransform({offset_name}, {scale_name}, {final_expr})");
            }

            init_lines.push(format!("metadata.{metadata_var} = {final_expr};"));

            if let Some(def) = &property.default_value {
                init_lines.push(format!("metadataClass.{metadata_var}.defaultValue = {glsl_type}({});", format_glsl_value(def)));
            }
            if let Some(nd) = &property.no_data {
                init_lines.push(format!("metadataClass.{metadata_var}.noData = {glsl_type}({});", format_glsl_value(nd)));
            }
            if let Some(min) = &property.min_value {
                init_lines.push(format!("metadataClass.{metadata_var}.minValue = {glsl_type}({});", format_glsl_value(min)));
            }
            if let Some(max) = &property.max_value {
                init_lines.push(format!("metadataClass.{metadata_var}.maxValue = {glsl_type}({});", format_glsl_value(max)));
            }

            let init_refs: Vec<&str> = init_lines.iter().map(|s| s.as_str()).collect();
            render_resources.shader_builder.add_function_lines(
                FUNCTION_ID_INITIALIZE_METADATA_FS,
                &init_refs,
            );

            let class_struct_name = format!("{glsl_type}MetadataClass");
            render_resources.shader_builder.add_struct_field(
                STRUCT_ID_METADATA_CLASS_FS,
                &class_struct_name,
                &metadata_var,
            );

            if let Some(c_stats) = class_stats {
                if let Some(p_stats) = c_stats.get(property_id) {
                    if property.metadata_type != MetadataType::Enum {
                        let stats_struct_name = format!("{glsl_type}MetadataStatistics");
                        render_resources.shader_builder.add_struct_field(
                            STRUCT_ID_METADATA_STATISTICS_FS,
                            &stats_struct_name,
                            &metadata_var,
                        );

                        let mut stats_lines = Vec::new();
                        for field in &METADATA_STATISTICS_FIELDS {
                            let val_opt = match field.spec_name {
                                "min" => &p_stats.min,
                                "max" => &p_stats.max,
                                "mean" => &p_stats.mean,
                                "median" => &p_stats.median,
                                "standardDeviation" => &p_stats.standard_deviation,
                                "variance" => &p_stats.variance,
                                "sum" => &p_stats.sum,
                                _ => &None,
                            };
                            if let Some(val) = val_opt {
                                let f_type = if field.is_float {
                                    match glsl_type.as_str() {
                                        "int" => "float",
                                        "ivec2" => "vec2",
                                        "ivec3" => "vec3",
                                        "ivec4" => "vec4",
                                        other => other,
                                    }
                                } else {
                                    glsl_type.as_str()
                                };
                                stats_lines.push(format!(
                                    "metadataStatistics.{metadata_var}.{} = {f_type}({});",
                                    field.shader_name,
                                    format_glsl_value(val)
                                ));
                            }
                        }
                        let stats_refs: Vec<&str> = stats_lines.iter().map(|s| s.as_str()).collect();
                        render_resources.shader_builder.add_function_lines(
                            FUNCTION_ID_INITIALIZE_METADATA_FS,
                            &stats_refs,
                        );
                    }
                }
            }
        }
    }
}
