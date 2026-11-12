use cesium_math::{Cartesian3, Matrix4};
use cesium_renderer::pnts_loader::{
    PntsAttributeType, PntsComponentDatatype, PntsLoader, PntsLoaderOptions,
};
use cesium_renderer::vertex_attribute_semantic::VertexAttributeSemantic;
use serde_json::{json, Value};

fn build_pnts_buffer(
    version: u32,
    ft_json: Value,
    ft_bin: &[u8],
    bt_json: Option<Value>,
) -> Vec<u8> {
    let mut ft_json_bytes = ft_json.to_string().into_bytes();
    while !ft_json_bytes.len().is_multiple_of(8) {
        ft_json_bytes.push(b' ');
    }
    let mut ft_bin_padded = ft_bin.to_vec();
    while !ft_bin_padded.len().is_multiple_of(8) {
        ft_bin_padded.push(0);
    }
    let mut bt_json_bytes = bt_json
        .map(|v| v.to_string().into_bytes())
        .unwrap_or_default();
    if !bt_json_bytes.is_empty() {
        while !bt_json_bytes.len().is_multiple_of(8) {
            bt_json_bytes.push(b' ');
        }
    }

    let total_len = 28 + ft_json_bytes.len() + ft_bin_padded.len() + bt_json_bytes.len();
    let mut buf = Vec::with_capacity(total_len);
    buf.extend_from_slice(b"pnts");
    buf.extend_from_slice(&version.to_le_bytes());
    buf.extend_from_slice(&(total_len as u32).to_le_bytes());
    buf.extend_from_slice(&(ft_json_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(&(ft_bin_padded.len() as u32).to_le_bytes());
    buf.extend_from_slice(&(bt_json_bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&ft_json_bytes);
    buf.extend_from_slice(&ft_bin_padded);
    buf.extend_from_slice(&bt_json_bytes);
    buf
}

#[test]
fn throws_with_invalid_version() {
    let buf = build_pnts_buffer(2, json!({ "POINTS_LENGTH": 0 }), &[], None);
    let mut loader = PntsLoader::new(PntsLoaderOptions {
        array_buffer: buf,
        load_attributes_for_2d: false,
    });
    assert!(loader.load().is_err());
}

#[test]
fn loads_point_cloud_rgb_and_constant_color() {
    // 2 points with float32 positions + uint8 RGB
    let mut ft_bin = Vec::new();
    for v in [0.0f32, 0.0, 0.0, 1.0, 2.0, 3.0] {
        ft_bin.extend_from_slice(&v.to_le_bytes());
    }
    ft_bin.extend_from_slice(&[255u8, 0, 0, 0, 255, 0]);
    let buf = build_pnts_buffer(
        1,
        json!({
            "POINTS_LENGTH": 2,
            "POSITION": { "byteOffset": 0 },
            "RGB": { "byteOffset": 24 }
        }),
        &ft_bin,
        None,
    );

    let mut loader = PntsLoader::new(PntsLoaderOptions {
        array_buffer: buf,
        load_attributes_for_2d: false,
    });
    loader.load().unwrap();
    let components = loader.components().unwrap();
    assert_eq!(components.points_length, 2);
    assert!(components.has_colors);
    assert!(!components.has_normals);

    let attrs = &components.nodes[0].primitives[0].attributes;
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].name, "POSITION");
    assert_eq!(attrs[0].semantic, VertexAttributeSemantic::POSITION);
    assert_eq!(attrs[0].component_datatype, PntsComponentDatatype::Float);
    assert_eq!(attrs[0].attribute_type, PntsAttributeType::Vec3);
    assert_eq!(attrs[0].min, Some(Cartesian3::new(0.0, 0.0, 0.0)));
    assert_eq!(attrs[0].max, Some(Cartesian3::new(1.0, 2.0, 3.0)));
    assert_eq!(attrs[1].name, "COLOR_0");
    assert_eq!(attrs[1].count, 2);
}

#[test]
fn loads_point_cloud_normals_quantized_and_rtc_center() {
    // 2 quantized positions (u16 x 3 = 12 bytes) + 2 normals (f32 x 3 = 24 bytes)
    let mut ft_bin = Vec::new();
    for v in [0u16, 0, 0, 65535, 65535, 65535] {
        ft_bin.extend_from_slice(&v.to_le_bytes());
    }
    for v in [0.0f32, 0.0, 1.0, 0.0, 1.0, 0.0] {
        ft_bin.extend_from_slice(&v.to_le_bytes());
    }
    let buf = build_pnts_buffer(
        1,
        json!({
            "POINTS_LENGTH": 2,
            "POSITION_QUANTIZED": { "byteOffset": 0 },
            "QUANTIZED_VOLUME_OFFSET": [1.0, 2.0, 3.0],
            "QUANTIZED_VOLUME_SCALE": [10.0, 20.0, 30.0],
            "NORMAL": { "byteOffset": 12 },
            "RTC_CENTER": [100.0, 200.0, 300.0]
        }),
        &ft_bin,
        None,
    );

    let mut loader = PntsLoader::new(PntsLoaderOptions {
        array_buffer: buf,
        load_attributes_for_2d: true,
    });
    loader.load().unwrap();
    assert!(loader.load_attributes_for_2d());
    let components = loader.components().unwrap();
    assert!(components.has_normals);
    assert_eq!(
        components.nodes[0].transform,
        Matrix4::from_translation(Cartesian3::new(100.0, 200.0, 300.0))
    );
    let pos_attr = &components.nodes[0].primitives[0].attributes[0];
    let quant = pos_attr.quantization.as_ref().unwrap();
    assert_eq!(
        quant.component_datatype,
        PntsComponentDatatype::UnsignedShort
    );
    assert_eq!(quant.volume_offset, Cartesian3::new(1.0, 2.0, 3.0));
    assert_eq!(quant.volume_dimensions, Cartesian3::new(10.0, 20.0, 30.0));
    assert_eq!(components.nodes[0].primitives[0].attributes.len(), 3);
}

#[test]
fn loads_batched_point_cloud_and_destroys() {
    let mut ft_bin = Vec::new();
    for v in [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0] {
        ft_bin.extend_from_slice(&v.to_le_bytes());
    }
    ft_bin.extend_from_slice(&0u16.to_le_bytes());
    ft_bin.extend_from_slice(&1u16.to_le_bytes());
    let buf = build_pnts_buffer(
        1,
        json!({
            "POINTS_LENGTH": 2,
            "BATCH_LENGTH": 2,
            "POSITION": { "byteOffset": 0 },
            "BATCH_ID": { "byteOffset": 24 }
        }),
        &ft_bin,
        Some(json!({ "name": ["pt0", "pt1"] })),
    );

    let mut loader = PntsLoader::new(PntsLoaderOptions {
        array_buffer: buf,
        load_attributes_for_2d: false,
    });
    loader.load().unwrap();
    let components = loader.components().unwrap();
    assert_eq!(components.batch_length, Some(2));
    assert_eq!(
        components.nodes[0].primitives[0].attributes[2].semantic,
        VertexAttributeSemantic::FEATURE_ID
    );
    assert_eq!(components.metadata["name"][1], json!("pt1"));

    assert!(!loader.is_destroyed());
    loader.destroy();
    assert!(loader.is_destroyed());
    assert!(loader.components().is_none());
    assert!(loader.load().is_err());
}
