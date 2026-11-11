//! Updates the shader with defines, structs, and functions to handle voxel properties and statistics.
//! Adapted from CesiumJS `Scene/processVoxelProperties.js` (Apache-2.0).

use crate::metadata_type::MetadataType;
use crate::shader_builder::ShaderDestination;
use crate::voxel_primitive::VoxelPrimitive;
use crate::voxel_render_resources::VoxelRenderResources;

/// Updates the shader in `render_resources` with defines, structs, and functions
/// to handle voxel properties and statistics from `primitive`.
pub fn process_voxel_properties(
    render_resources: &mut VoxelRenderResources,
    primitive: &VoxelPrimitive,
) {
    let provider = match &primitive.provider {
        Some(p) => p,
        None => return,
    };

    let names = provider.names();
    let types = provider.types();
    let _component_types = provider.component_types();
    let minimum_values = provider.min_values_per_component();
    let maximum_values = provider.max_values_per_component();

    let attribute_length = types.len();
    let has_statistics = minimum_values.is_some() && maximum_values.is_some();

    let shader_builder = &mut render_resources.shader_builder;

    shader_builder.add_define(
        "METADATA_COUNT",
        Some(&attribute_length.to_string()),
        Some(ShaderDestination::FRAGMENT),
    );

    if has_statistics {
        shader_builder.add_define("STATISTICS", None, Some(ShaderDestination::FRAGMENT));
    }

    // PropertyStatistics structs
    for i in 0..attribute_length {
        let name = &names[i];
        let glsl_type = get_glsl_type(types[i]);
        let property_statistics_struct_id = format!("PropertyStatistics_{name}");
        shader_builder.add_struct(
            &property_statistics_struct_id,
            &property_statistics_struct_id,
            ShaderDestination::FRAGMENT,
        );
        shader_builder.add_struct_field(&property_statistics_struct_id, glsl_type, "min");
        shader_builder.add_struct_field(&property_statistics_struct_id, glsl_type, "max");
    }

    // MetadataStatistics struct
    let metadata_statistics_struct_id = "MetadataStatistics";
    let metadata_statistics_struct_name = "MetadataStatistics";
    shader_builder.add_struct(
        metadata_statistics_struct_id,
        metadata_statistics_struct_name,
        ShaderDestination::FRAGMENT,
    );
    for name in &names {
        let property_struct_name = format!("PropertyStatistics_{name}");
        shader_builder.add_struct_field(metadata_statistics_struct_id, &property_struct_name, name);
    }

    // Metadata struct
    let metadata_struct_id = "Metadata";
    let metadata_struct_name = "Metadata";
    shader_builder.add_struct(
        metadata_struct_id,
        metadata_struct_name,
        ShaderDestination::FRAGMENT,
    );
    for i in 0..attribute_length {
        let glsl_type = get_glsl_type(types[i]);
        shader_builder.add_struct_field(metadata_struct_id, glsl_type, &names[i]);
    }

    // Attributes struct
    let attributes_struct_id = "Attributes";
    let attributes_struct_name = "Attributes";
    shader_builder.add_struct(
        attributes_struct_id,
        attributes_struct_name,
        ShaderDestination::FRAGMENT,
    );
    shader_builder.add_struct_field(attributes_struct_id, "vec3", "positionEC");
    shader_builder.add_struct_field(attributes_struct_id, "vec3", "normalEC");

    // Voxel struct
    let voxel_struct_id = "Voxel";
    let voxel_struct_name = "Voxel";
    shader_builder.add_struct(
        voxel_struct_id,
        voxel_struct_name,
        ShaderDestination::FRAGMENT,
    );
    shader_builder.add_struct_field(voxel_struct_id, "vec3", "viewDirUv");
    shader_builder.add_struct_field(voxel_struct_id, "float", "travelDistance");
    shader_builder.add_struct_field(voxel_struct_id, "int", "stepCount");
    shader_builder.add_struct_field(voxel_struct_id, "int", "tileIndex");
    shader_builder.add_struct_field(voxel_struct_id, "int", "sampleIndex");
    shader_builder.add_struct_field(voxel_struct_id, "float", "distanceToDepthBuffer");

    // FragmentInput struct
    let fragment_input_struct_id = "FragmentInput";
    let fragment_input_struct_name = "FragmentInput";
    shader_builder.add_struct(
        fragment_input_struct_id,
        fragment_input_struct_name,
        ShaderDestination::FRAGMENT,
    );
    shader_builder.add_struct_field(
        fragment_input_struct_id,
        metadata_statistics_struct_name,
        "metadataStatistics",
    );
    shader_builder.add_struct_field(fragment_input_struct_id, metadata_struct_name, "metadata");
    shader_builder.add_struct_field(
        fragment_input_struct_id,
        attributes_struct_name,
        "attributes",
    );
    shader_builder.add_struct_field(fragment_input_struct_id, voxel_struct_name, "voxel");

    // Properties struct
    let properties_struct_id = "Properties";
    let properties_struct_name = "Properties";
    shader_builder.add_struct(
        properties_struct_id,
        properties_struct_name,
        ShaderDestination::FRAGMENT,
    );
    for i in 0..attribute_length {
        let glsl_type = get_glsl_type(types[i]);
        shader_builder.add_struct_field(properties_struct_id, glsl_type, &names[i]);
    }

    // clearProperties function
    {
        let function_id = "clearProperties";
        shader_builder.add_function(
            function_id,
            "Properties clearProperties()",
            ShaderDestination::FRAGMENT,
        );
        let mut lines = vec!["Properties properties;".to_string()];
        for i in 0..attribute_length {
            let glsl_type = get_glsl_type(types[i]);
            lines.push(format!("properties.{} = {glsl_type}(0.0);", names[i]));
        }
        lines.push("return properties;".to_string());
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        shader_builder.add_function_lines(function_id, &line_refs);
    }

    // sumProperties function
    {
        let function_id = "sumProperties";
        shader_builder.add_function(
            function_id,
            "Properties sumProperties(Properties propertiesA, Properties propertiesB)",
            ShaderDestination::FRAGMENT,
        );
        let mut lines = vec!["Properties properties;".to_string()];
        for name in &names {
            lines.push(format!(
                "properties.{name} = propertiesA.{name} + propertiesB.{name};"
            ));
        }
        lines.push("return properties;".to_string());
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        shader_builder.add_function_lines(function_id, &line_refs);
    }

    // scaleProperties function
    {
        let function_id = "scaleProperties";
        shader_builder.add_function(
            function_id,
            "Properties scaleProperties(Properties properties, float scale)",
            ShaderDestination::FRAGMENT,
        );
        let mut lines = vec!["Properties scaledProperties = properties;".to_string()];
        for name in &names {
            lines.push(format!("scaledProperties.{name} *= scale;"));
        }
        lines.push("return scaledProperties;".to_string());
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        shader_builder.add_function_lines(function_id, &line_refs);
    }

    // mixProperties function
    {
        let function_id = "mixProperties";
        shader_builder.add_function(
            function_id,
            "Properties mixProperties(Properties propertiesA, Properties propertiesB, float mixFactor)",
            ShaderDestination::FRAGMENT,
        );
        let mut lines = vec!["Properties properties;".to_string()];
        for name in &names {
            lines.push(format!(
                "properties.{name} = mix(propertiesA.{name}, propertiesB.{name}, mixFactor);"
            ));
        }
        lines.push("return properties;".to_string());
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        shader_builder.add_function_lines(function_id, &line_refs);
    }

    // copyPropertiesToMetadata function
    {
        let function_id = "copyPropertiesToMetadata";
        shader_builder.add_function(
            function_id,
            "void copyPropertiesToMetadata(in Properties properties, inout Metadata metadata)",
            ShaderDestination::FRAGMENT,
        );
        let mut lines = Vec::new();
        for name in &names {
            lines.push(format!("metadata.{name} = properties.{name};"));
        }
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        shader_builder.add_function_lines(function_id, &line_refs);
    }

    // setStatistics function
    if has_statistics {
        let function_id = "setStatistics";
        shader_builder.add_function(
            function_id,
            "void setStatistics(inout MetadataStatistics metadataStatistics)",
            ShaderDestination::FRAGMENT,
        );
        let mut lines = Vec::new();
        let min_vec = minimum_values.as_ref().unwrap();
        let max_vec = maximum_values.as_ref().unwrap();
        for i in 0..attribute_length {
            let name = &names[i];
            let r#type = types[i];
            let component_count = r#type.get_component_count();
            for j in 0..component_count {
                let glsl_field = get_glsl_field(r#type, j);
                if let (Some(row_min), Some(row_max)) = (min_vec.get(i), max_vec.get(i)) {
                    if let (Some(&min_val), Some(&max_val)) = (row_min.get(j), row_max.get(j)) {
                        lines.push(format!(
                            "metadataStatistics.{name}.min{glsl_field} = {};",
                            get_glsl_number_as_float(min_val)
                        ));
                        lines.push(format!(
                            "metadataStatistics.{name}.max{glsl_field} = {};",
                            get_glsl_number_as_float(max_val)
                        ));
                    }
                }
            }
        }
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        shader_builder.add_function_lines(function_id, &line_refs);
    }

    // getPropertiesFromMegatextureAtUv function
    {
        let function_id = "getPropertiesFromMegatextureAtUv";
        shader_builder.add_function(
            function_id,
            "Properties getPropertiesFromMegatextureAtUv(vec3 texcoord)",
            ShaderDestination::FRAGMENT,
        );
        let mut lines = vec!["Properties properties;".to_string()];
        for i in 0..attribute_length {
            let swizzle = get_glsl_texture_swizzle(types[i]);
            lines.push(format!(
                "properties.{} = texture(u_megatextureTextures[{i}], texcoord){swizzle};",
                names[i]
            ));
        }
        lines.push("return properties;".to_string());
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        shader_builder.add_function_lines(function_id, &line_refs);
    }
}

fn get_glsl_type(r#type: MetadataType) -> &'static str {
    match r#type {
        MetadataType::Scalar => "float",
        MetadataType::Vec2 => "vec2",
        MetadataType::Vec3 => "vec3",
        MetadataType::Vec4 => "vec4",
        _ => "float",
    }
}

fn get_glsl_texture_swizzle(r#type: MetadataType) -> &'static str {
    match r#type {
        MetadataType::Scalar => ".r",
        MetadataType::Vec2 => ".ra",
        MetadataType::Vec3 => ".rgb",
        MetadataType::Vec4 => "",
        _ => "",
    }
}

fn get_glsl_number_as_float(number: f64) -> String {
    let s = number.to_string();
    if !s.contains('.') {
        format!("{s}.0")
    } else {
        s
    }
}

fn get_glsl_field(r#type: MetadataType, index: usize) -> String {
    if r#type == MetadataType::Scalar {
        String::new()
    } else {
        format!("[{index}]")
    }
}
