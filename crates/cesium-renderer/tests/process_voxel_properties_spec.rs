use std::sync::Arc;

use cesium_renderer::process_voxel_properties::process_voxel_properties;
use cesium_renderer::voxel_primitive::{MockVoxelProvider, VoxelPrimitive, VoxelPrimitiveOptions};
use cesium_renderer::voxel_render_resources::VoxelRenderResources;

fn assert_equal_unordered(actual: &[String], expected: &[&str]) {
    let mut actual_sorted = actual.to_vec();
    actual_sorted.sort();
    let mut expected_sorted: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
    expected_sorted.sort();
    assert_eq!(actual_sorted, expected_sorted);
}

#[test]
fn adds_shader_defines_and_structs() {
    let provider = Arc::new(MockVoxelProvider {
        minimum_values: vec![0.0],
        maximum_values: vec![1.0],
        ..Default::default()
    });

    let primitive = VoxelPrimitive::new(Some(VoxelPrimitiveOptions {
        provider: Some(provider),
        ..Default::default()
    }));

    let mut render_resources = VoxelRenderResources::new(&primitive);
    process_voxel_properties(&mut render_resources, &primitive);

    let shader_builder = &render_resources.shader_builder;

    // Check fragment shader defines
    let define_lines = shader_builder.fragment_define_lines();
    assert!(define_lines.iter().any(|line| line == "METADATA_COUNT 1"));
    assert!(define_lines.iter().any(|line| line == "STATISTICS"));

    // Check for PropertyStatistics structs
    let prop_stats = shader_builder.get_struct("PropertyStatistics_a").unwrap();
    assert_eq!(prop_stats.name, "PropertyStatistics_a");
    assert_equal_unordered(&prop_stats.fields, &["    float min;", "    float max;"]);

    // Check for MetadataStatistics struct
    let meta_stats = shader_builder.get_struct("MetadataStatistics").unwrap();
    assert_eq!(meta_stats.name, "MetadataStatistics");
    assert_equal_unordered(&meta_stats.fields, &["    PropertyStatistics_a a;"]);

    // Check for Metadata struct
    let metadata = shader_builder.get_struct("Metadata").unwrap();
    assert_eq!(metadata.name, "Metadata");
    assert_equal_unordered(&metadata.fields, &["    float a;"]);

    // Check for Attributes struct
    let attributes = shader_builder.get_struct("Attributes").unwrap();
    assert_eq!(attributes.name, "Attributes");
    assert_equal_unordered(
        &attributes.fields,
        &["    vec3 positionEC;", "    vec3 normalEC;"],
    );

    // Check for Voxel struct
    let voxel = shader_builder.get_struct("Voxel").unwrap();
    assert_eq!(voxel.name, "Voxel");
    assert_equal_unordered(
        &voxel.fields,
        &[
            "    vec3 viewDirUv;",
            "    float travelDistance;",
            "    int stepCount;",
            "    int sampleIndex;",
            "    int tileIndex;",
            "    float distanceToDepthBuffer;",
        ],
    );

    // Check for FragmentInput struct
    let frag_input = shader_builder.get_struct("FragmentInput").unwrap();
    assert_eq!(frag_input.name, "FragmentInput");
    assert_equal_unordered(
        &frag_input.fields,
        &[
            "    MetadataStatistics metadataStatistics;",
            "    Metadata metadata;",
            "    Attributes attributes;",
            "    Voxel voxel;",
        ],
    );

    // Check for Properties struct
    let properties = shader_builder.get_struct("Properties").unwrap();
    assert_eq!(properties.name, "Properties");
    assert_equal_unordered(&properties.fields, &["    float a;"]);

    // Check clearProperties function
    let clear_fn = shader_builder.get_function("clearProperties").unwrap();
    assert_eq!(clear_fn.signature, "Properties clearProperties()");
    assert_equal_unordered(
        &clear_fn.body,
        &[
            "    Properties properties;",
            "    properties.a = float(0.0);",
            "    return properties;",
        ],
    );

    // Check sumProperties function
    let sum_fn = shader_builder.get_function("sumProperties").unwrap();
    assert_eq!(
        sum_fn.signature,
        "Properties sumProperties(Properties propertiesA, Properties propertiesB)"
    );
    assert_equal_unordered(
        &sum_fn.body,
        &[
            "    Properties properties;",
            "    properties.a = propertiesA.a + propertiesB.a;",
            "    return properties;",
        ],
    );

    // Check scaleProperties function
    let scale_fn = shader_builder.get_function("scaleProperties").unwrap();
    assert_eq!(
        scale_fn.signature,
        "Properties scaleProperties(Properties properties, float scale)"
    );
    assert_equal_unordered(
        &scale_fn.body,
        &[
            "    Properties scaledProperties = properties;",
            "    scaledProperties.a *= scale;",
            "    return scaledProperties;",
        ],
    );

    // Check mixProperties function
    let mix_fn = shader_builder.get_function("mixProperties").unwrap();
    assert_eq!(
        mix_fn.signature,
        "Properties mixProperties(Properties propertiesA, Properties propertiesB, float mixFactor)"
    );
    assert_equal_unordered(
        &mix_fn.body,
        &[
            "    Properties properties;",
            "    properties.a = mix(propertiesA.a, propertiesB.a, mixFactor);",
            "    return properties;",
        ],
    );

    // Check copyPropertiesToMetadata function
    let copy_fn = shader_builder.get_function("copyPropertiesToMetadata").unwrap();
    assert_eq!(
        copy_fn.signature,
        "void copyPropertiesToMetadata(in Properties properties, inout Metadata metadata)"
    );
    assert_equal_unordered(&copy_fn.body, &["    metadata.a = properties.a;"]);

    // Check setStatistics function
    let stats_fn = shader_builder.get_function("setStatistics").unwrap();
    assert_eq!(
        stats_fn.signature,
        "void setStatistics(inout MetadataStatistics metadataStatistics)"
    );
    assert_equal_unordered(
        &stats_fn.body,
        &[
            "    metadataStatistics.a.min = 0.0;",
            "    metadataStatistics.a.max = 1.0;",
        ],
    );

    // Check getPropertiesFromMegatextureAtUv function
    let mega_fn = shader_builder
        .get_function("getPropertiesFromMegatextureAtUv")
        .unwrap();
    assert_eq!(
        mega_fn.signature,
        "Properties getPropertiesFromMegatextureAtUv(vec3 texcoord)"
    );
    assert_equal_unordered(
        &mega_fn.body,
        &[
            "    Properties properties;",
            "    properties.a = texture(u_megatextureTextures[0], texcoord).r;",
            "    return properties;",
        ],
    );
}
