use std::collections::HashMap;
use std::sync::Arc;

use cesium_math::PrimitiveType;
use cesium_renderer::buffer::{Buffer, BufferOptions};
use cesium_renderer::classification_pipeline_stage::VertexAttributeSemantic;
use cesium_renderer::gltf_loader_util::ModelTextureReader;
use cesium_renderer::gltf_structural_metadata_loader::{
    PropertyTable, PropertyTexture, PropertyTextureProperty, StructuralMetadata,
};
use cesium_renderer::model_components::{
    Attribute, FeatureId, FeatureIdTexture, Indices, Material, MetallicRoughness, MorphTarget,
    Primitive, SpecularGlossiness,
};
use cesium_renderer::model_runtime_primitive::ModelRuntimePrimitive;
use cesium_renderer::model_statistics::{DynamicBatchTextureStatistics, ModelStatistics};
use cesium_renderer::primitive_statistics_pipeline_stage::PrimitiveStatisticsPipelineStage;
use cesium_renderer::texture::{Texture, TextureOptions};

fn create_buffer(size: usize) -> Buffer {
    Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: size,
        ..Default::default()
    })
    .expect("Buffer creation")
}

fn create_texture(width: u32, height: u32) -> Texture {
    Texture::new(TextureOptions {
        width,
        height,
        ..Default::default()
    })
}

#[test]
fn test_count_geometry_triangle_mesh() {
    let mut statistics = ModelStatistics::new();

    let pos_buf = create_buffer(72);
    let uv_buf = create_buffer(48);
    let idx_buf = create_buffer(24);

    let pos_attr = Attribute {
        semantic: Some(VertexAttributeSemantic::POSITION.to_string()),
        buffer: Some(pos_buf),
        ..Default::default()
    };
    let uv_attr = Attribute {
        semantic: Some(VertexAttributeSemantic::TEXCOORD.to_string()),
        buffer: Some(uv_buf),
        ..Default::default()
    };

    let primitive = Primitive {
        indices: Some(Indices {
            count: 36, // 12 triangles * 3
            buffer: Some(idx_buf),
            typed_array: None,
            ..Default::default()
        }),
        primitive_type: Some(PrimitiveType::Triangles),
        attributes: vec![pos_attr, uv_attr],
        ..Default::default()
    };

    PrimitiveStatisticsPipelineStage::count_geometry(&mut statistics, &primitive);

    assert_eq!(statistics.points_length, 0);
    assert_eq!(statistics.triangles_length, 12);
    assert_eq!(statistics.geometry_byte_length, 72 + 48 + 24);
}

#[test]
fn test_count_geometry_triangle_mesh_without_indices() {
    let mut statistics = ModelStatistics::new();

    let pos_buf = create_buffer(36);
    let pos_attr = Attribute {
        semantic: Some(VertexAttributeSemantic::POSITION.to_string()),
        buffer: Some(pos_buf),
        count: 3,
        ..Default::default()
    };

    let primitive = Primitive {
        indices: None,
        primitive_type: Some(PrimitiveType::Triangles),
        attributes: vec![pos_attr],
        ..Default::default()
    };

    PrimitiveStatisticsPipelineStage::count_geometry(&mut statistics, &primitive);

    assert_eq!(statistics.points_length, 0);
    assert_eq!(statistics.triangles_length, 1);
    assert_eq!(statistics.geometry_byte_length, 36);
}

#[test]
fn test_count_geometry_attributes_with_cpu_copy() {
    let mut statistics = ModelStatistics::new();

    let pos_buf = create_buffer(72);
    let norm_buf = create_buffer(72);
    let idx_buf = create_buffer(24);

    let pos_attr = Attribute {
        semantic: Some(VertexAttributeSemantic::POSITION.to_string()),
        buffer: Some(pos_buf),
        typed_array: Some(vec![0; 72]), // CPU copy
        ..Default::default()
    };
    let norm_attr = Attribute {
        semantic: Some(VertexAttributeSemantic::NORMAL.to_string()),
        buffer: Some(norm_buf),
        typed_array: None,
        ..Default::default()
    };

    let primitive = Primitive {
        indices: Some(Indices {
            count: 36,
            buffer: Some(idx_buf),
            typed_array: Some(vec![0; 24]), // CPU copy
            ..Default::default()
        }),
        primitive_type: Some(PrimitiveType::Triangles),
        attributes: vec![pos_attr, norm_attr],
        ..Default::default()
    };

    PrimitiveStatisticsPipelineStage::count_geometry(&mut statistics, &primitive);

    assert_eq!(statistics.triangles_length, 12);
    // pos has CPU copy (2 * 72), norm has no CPU copy (72), idx has CPU copy (2 * 24)
    assert_eq!(statistics.geometry_byte_length, 2 * 72 + 72 + 2 * 24);
}

#[test]
fn test_count_geometry_point_cloud() {
    let mut statistics = ModelStatistics::new();

    let pos_buf = create_buffer(2500 * 12);
    let pos_attr = Attribute {
        semantic: Some(VertexAttributeSemantic::POSITION.to_string()),
        buffer: Some(pos_buf),
        count: 2500,
        ..Default::default()
    };

    let primitive = Primitive {
        indices: None,
        primitive_type: Some(PrimitiveType::Points),
        attributes: vec![pos_attr],
        ..Default::default()
    };

    PrimitiveStatisticsPipelineStage::count_geometry(&mut statistics, &primitive);

    assert_eq!(statistics.points_length, 2500);
    assert_eq!(statistics.triangles_length, 0);
    assert_eq!(statistics.geometry_byte_length, 2500 * 12);
}

#[test]
fn test_count_geometry_triangle_strip_and_fan() {
    let mut statistics = ModelStatistics::new();

    let pos_buf = create_buffer(48);
    let pos_attr = Attribute {
        semantic: Some(VertexAttributeSemantic::POSITION.to_string()),
        buffer: Some(pos_buf),
        count: 4,
        ..Default::default()
    };

    let primitive_strip = Primitive {
        indices: None,
        primitive_type: Some(PrimitiveType::TriangleStrip),
        attributes: vec![pos_attr.clone()],
        ..Default::default()
    };

    PrimitiveStatisticsPipelineStage::count_geometry(&mut statistics, &primitive_strip);
    assert_eq!(statistics.triangles_length, 2); // 4 - 2 = 2

    let mut statistics_fan = ModelStatistics::new();
    let primitive_fan = Primitive {
        indices: None,
        primitive_type: Some(PrimitiveType::TriangleFan),
        attributes: vec![pos_attr],
        ..Default::default()
    };

    PrimitiveStatisticsPipelineStage::count_geometry(&mut statistics_fan, &primitive_fan);
    assert_eq!(statistics_fan.triangles_length, 2); // 4 - 2 = 2
}

#[test]
fn test_count_geometry_outline_coordinates() {
    let mut statistics = ModelStatistics::new();

    let pos_buf = create_buffer(72);
    let outline_buf = create_buffer(36);

    let primitive = Primitive {
        primitive_type: Some(PrimitiveType::Triangles),
        attributes: vec![Attribute {
            buffer: Some(pos_buf),
            count: 6,
            ..Default::default()
        }],
        outline_coordinates: Some(Attribute {
            buffer: Some(outline_buf),
            count: 6,
            ..Default::default()
        }),
        ..Default::default()
    };

    PrimitiveStatisticsPipelineStage::count_geometry(&mut statistics, &primitive);
    assert_eq!(statistics.geometry_byte_length, 72 + 36);
}

#[test]
fn test_count_2d_positions() {
    let mut statistics = ModelStatistics::new();

    // No 2D positions
    PrimitiveStatisticsPipelineStage::count_2d_positions(&mut statistics, None);
    assert_eq!(statistics.geometry_byte_length, 0);

    // With 2D positions: counts 2x (GPU + CPU copies)
    let buffer_2d = create_buffer(48);
    PrimitiveStatisticsPipelineStage::count_2d_positions(&mut statistics, Some(&buffer_2d));
    assert_eq!(statistics.geometry_byte_length, 2 * 48);
}

#[test]
fn test_count_morph_target_attributes() {
    let mut statistics = ModelStatistics::new();

    let buf1 = create_buffer(48);
    let buf2 = create_buffer(48);

    let primitive = Primitive {
        morph_targets: vec![
            MorphTarget {
                attributes: vec![Attribute {
                    buffer: Some(buf1),
                    ..Default::default()
                }],
            },
            MorphTarget {
                attributes: vec![Attribute {
                    buffer: Some(buf2),
                    ..Default::default()
                }],
            },
        ],
        ..Default::default()
    };

    PrimitiveStatisticsPipelineStage::count_morph_target_attributes(&mut statistics, &primitive);
    assert_eq!(statistics.geometry_byte_length, 48 + 48);
}

#[test]
fn test_count_material_textures() {
    let mut statistics = ModelStatistics::new();

    // No material
    PrimitiveStatisticsPipelineStage::count_material_textures(&mut statistics, None);
    assert_eq!(statistics.textures_byte_length, 0);

    // Metallic roughness with base color and emissive
    let tex1 = create_texture(2, 2); // 2*2*4 = 16 bytes
    let tex2 = create_texture(4, 4); // 4*4*4 = 64 bytes

    let material = Material {
        emissive_texture: Some(ModelTextureReader {
            texture: Some(tex1.clone()),
            ..Default::default()
        }),
        metallic_roughness: Some(MetallicRoughness {
            base_color_texture: Some(ModelTextureReader {
                texture: Some(tex2.clone()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    PrimitiveStatisticsPipelineStage::count_material_textures(&mut statistics, Some(&material));
    assert_eq!(statistics.textures_byte_length, 16 + 64);

    // Specular glossiness
    let mut sg_statistics = ModelStatistics::new();
    let sg_mat = Material {
        specular_glossiness: Some(SpecularGlossiness {
            diffuse_texture: Some(ModelTextureReader {
                texture: Some(tex1),
                ..Default::default()
            }),
            specular_glossiness_texture: Some(ModelTextureReader {
                texture: Some(tex2),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    PrimitiveStatisticsPipelineStage::count_material_textures(&mut sg_statistics, Some(&sg_mat));
    assert_eq!(sg_statistics.textures_byte_length, 16 + 64);
}

#[test]
fn test_count_feature_id_textures() {
    let mut statistics = ModelStatistics::new();

    let tex = create_texture(2, 2); // 16 bytes
    let feature_ids = vec![FeatureId::Texture(FeatureIdTexture {
        feature_count: 4,
        texture_reader: Some(ModelTextureReader {
            texture: Some(tex),
            ..Default::default()
        }),
        ..Default::default()
    })];

    PrimitiveStatisticsPipelineStage::count_feature_id_textures(&mut statistics, &feature_ids);
    assert_eq!(statistics.textures_byte_length, 16);
}

#[test]
fn test_count_binary_metadata() {
    let mut statistics = ModelStatistics::new();

    let tex = create_texture(2, 2); // 16 bytes
    let mut properties = HashMap::new();
    properties.insert(
        "insideTemperature".to_string(),
        PropertyTextureProperty {
            texture_reader: ModelTextureReader {
                texture: Some(tex),
                ..Default::default()
            },
            channels: vec![0],
        },
    );

    let prop_texture = PropertyTexture {
        id: 0,
        name: None,
        class_name: "Building".to_string(),
        properties,
    };

    let prop_table = PropertyTable {
        id: 0,
        name: None,
        class_name: "Building".to_string(),
        count: 10,
        properties: HashMap::new(),
    };

    let structural_metadata = StructuralMetadata {
        property_tables: vec![prop_table],
        property_textures: vec![prop_texture],
        ..Default::default()
    };

    let batch_texture = Arc::new(DynamicBatchTextureStatistics::new("batch_uuid"));
    batch_texture.set_batch_texture_size(32);

    PrimitiveStatisticsPipelineStage::count_binary_metadata(
        &mut statistics,
        Some(&structural_metadata),
        120, // property_tables_byte_length
        &[batch_texture],
    );

    assert_eq!(statistics.property_tables_byte_length, 120);
    assert_eq!(statistics.textures_byte_length, 16);
    assert_eq!(statistics.batch_textures_byte_length(), 32);
}

#[test]
fn test_process_counts_all_for_model() {
    let mut statistics = ModelStatistics::new();

    let pos_buf = create_buffer(72);
    let uv_buf = create_buffer(48);
    let idx_buf = create_buffer(24);
    let tex = create_texture(2, 2); // 16 bytes

    let primitive = Primitive {
        indices: Some(Indices {
            count: 36,
            buffer: Some(idx_buf),
            ..Default::default()
        }),
        primitive_type: Some(PrimitiveType::Triangles),
        attributes: vec![
            Attribute {
                semantic: Some(VertexAttributeSemantic::POSITION.to_string()),
                buffer: Some(pos_buf),
                ..Default::default()
            },
            Attribute {
                semantic: Some(VertexAttributeSemantic::TEXCOORD.to_string()),
                buffer: Some(uv_buf),
                ..Default::default()
            },
        ],
        material: Some(Material {
            metallic_roughness: Some(MetallicRoughness {
                base_color_texture: Some(ModelTextureReader {
                    texture: Some(tex),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let runtime_primitive = ModelRuntimePrimitive::default();

    PrimitiveStatisticsPipelineStage::process(
        &mut statistics,
        &primitive,
        &runtime_primitive,
        None,
        0,
        &[],
    );

    assert_eq!(statistics.points_length, 0);
    assert_eq!(statistics.triangles_length, 12);
    assert_eq!(statistics.geometry_byte_length, 72 + 48 + 24);
    assert_eq!(statistics.textures_byte_length, 16);
    assert_eq!(statistics.property_tables_byte_length, 0);
}
