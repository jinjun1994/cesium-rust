//! Parser cases and fixtures adapted from CesiumJS PNTS Specs (Apache-2.0).
use cesium_formats::{decode_point_cloud, parse_pnts, DecodeLimits, PositionEncoding};
use glam::DVec3;
use serde_json::{json, Value};

fn build(table: Value, binary: &[u8]) -> Vec<u8> {
    let json = serde_json::to_vec(&table).unwrap();
    let mut data = b"pnts".to_vec();
    for v in [
        1,
        (28 + json.len() + binary.len()) as u32,
        json.len() as u32,
        binary.len() as u32,
        0,
        0,
    ] {
        data.extend_from_slice(&v.to_le_bytes());
    }
    data.extend(json);
    data.extend_from_slice(binary);
    data
}

#[test]
fn upstream_real_rgb_quantized_and_batched_content() {
    let rgb = include_bytes!("../../../tests/fixtures/cesium-pointcloud-rgb.pnts");
    let p = parse_pnts(rgb, &DecodeLimits::default()).unwrap();
    assert_eq!(p.points_length, 1000);
    assert_eq!(p.position_encoding, PositionEncoding::Float32);
    assert_eq!(
        p.position(0).unwrap(),
        DVec3::new(-2.4975082874298096, -0.3252686858177185, -3.522307872772217)
    );
    assert!(p.position(1000).is_err());
    let quantized = include_bytes!("../../../tests/fixtures/cesium-pointcloud-quantized.pnts");
    let p = parse_pnts(quantized, &DecodeLimits::default()).unwrap();
    assert_eq!(p.position_encoding, PositionEncoding::QuantizedU16);
    assert!((p.normal(0).unwrap().unwrap().length() - 1.0).abs() < 1e-14);
    assert!(p.color(0).unwrap().is_some());
    let expected =
        DVec3::new(478.0, 38231.0, 33885.0) / 65535.0 * 10.0 + p.quantized_volume_offset.unwrap();
    assert!(p.position(0).unwrap().distance(expected) < 1e-9);
    let batched = include_bytes!("../../../tests/fixtures/cesium-pointcloud-batched.pnts");
    let p = parse_pnts(batched, &DecodeLimits::default()).unwrap();
    assert_eq!(p.batch_length, Some(8));
    assert!(p.batch_id(0).unwrap().unwrap() < 8);
    assert_eq!(p.metadata["name"][0], "section0");
    assert_eq!(p.metadata["dimensions"].as_array().unwrap().len(), 8);
    assert_eq!(p.metadata["id"].as_array().unwrap().len(), 8);
}

#[test]
fn decodes_rgb565_oct_normals_batch_ids_and_constant_rgba() {
    let mut binary = vec![0u8; 17];
    binary[12..14].copy_from_slice(&0xf800u16.to_le_bytes());
    binary[14] = 255;
    binary[15] = 255;
    binary[16] = 1;
    let table = json!({"POINTS_LENGTH":1,"POSITION":{"byteOffset":0},"RGB565":{"byteOffset":12},"NORMAL_OCT16P":{"byteOffset":14},"BATCH_ID":{"byteOffset":16,"componentType":"UNSIGNED_BYTE"},"BATCH_LENGTH":2});
    let bytes = build(table, &binary);
    let p = parse_pnts(&bytes, &DecodeLimits::default()).unwrap();
    assert_eq!(p.color(0).unwrap(), Some([1.0, 0.0, 0.0, 1.0]));
    assert_eq!(p.normal(0).unwrap(), Some(-DVec3::Z));
    assert_eq!(p.batch_id(0).unwrap(), Some(1));
    assert!(p.color(1).is_err());
    let bytes = build(
        json!({"POINTS_LENGTH":1,"POSITION":{"byteOffset":0},"CONSTANT_RGBA":[255,128,0,64]}),
        &[0; 12],
    );
    let p = parse_pnts(&bytes, &DecodeLimits::default()).unwrap();
    assert_eq!(
        p.color(0).unwrap(),
        Some([1.0, 128.0 / 255.0, 0.0, 64.0 / 255.0])
    );
    assert_eq!(p.normal(0).unwrap(), None);
    assert_eq!(p.batch_id(0).unwrap(), None);
}

#[test]
fn inline_attributes_match_binary_point_values() {
    let table = json!({
        "POINTS_LENGTH":2,
        "POSITION":[0.0,1.0,2.0,3.0,4.0,5.0],
        "RGB":[255,0,128,0,255,64],
        "NORMAL":[0.0,0.0,1.0,0.0,1.0,0.0],
        "BATCH_ID":[0,1], "BATCH_LENGTH":2
    });
    let bytes = build(table, &[]);
    let p = parse_pnts(&bytes, &DecodeLimits::default()).unwrap();
    assert!(matches!(p.feature_binary, std::borrow::Cow::Owned(_)));
    assert_eq!(p.position(1).unwrap(), DVec3::new(3.0, 4.0, 5.0));
    assert_eq!(p.color(0).unwrap(), Some([1.0, 0.0, 128.0 / 255.0, 1.0]));
    assert_eq!(p.normal(1).unwrap(), Some(DVec3::Y));
    assert_eq!(p.batch_id(1).unwrap(), Some(1));
    let malformed = build(json!({"POINTS_LENGTH":1,"POSITION":[0.0,1.0]}), &[]);
    assert!(parse_pnts(&malformed, &DecodeLimits::default()).is_err());
    let oversized = build(
        json!({"POINTS_LENGTH":1,"POSITION":[0.0,0.0,0.0],"RGB":[256,0,0]}),
        &[],
    );
    assert!(parse_pnts(&oversized, &DecodeLimits::default()).is_err());
    let constrained = DecodeLimits {
        max_decoded_bytes: 10,
        ..Default::default()
    };
    assert!(parse_pnts(&bytes, &constrained).is_err());
}

#[test]
fn materialized_points_preserve_precision_and_metadata() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/cesium-pointcloud-rgb.pnts").as_slice(),
        include_bytes!("../../../tests/fixtures/cesium-pointcloud-quantized.pnts").as_slice(),
        include_bytes!("../../../tests/fixtures/cesium-pointcloud-batched.pnts").as_slice(),
    ] {
        let limits = DecodeLimits::default();
        let parsed = parse_pnts(bytes, &limits).unwrap();
        let cloud = decode_point_cloud(bytes, &limits).unwrap();
        assert_eq!(cloud.points.len(), parsed.points_length);
        assert_eq!(cloud.metadata, parsed.metadata);
        for index in [0, 123, 999] {
            let expected = parsed.position(index).unwrap() + parsed.rtc_center;
            let actual =
                cloud.origin + DVec3::from_array(cloud.points[index].position.map(f64::from));
            assert!(actual.distance(expected) < 1e-6);
            assert_eq!(
                cloud.points[index].feature_id,
                parsed.batch_id(index).unwrap().unwrap_or(index as u32)
            );
        }
        let limit = DecodeLimits {
            max_decoded_bytes: parsed.decoded_bytes
                + std::mem::size_of_val(cloud.points.as_slice())
                - 1,
            ..limits
        };
        assert!(parse_pnts(bytes, &limit).is_ok());
        assert!(decode_point_cloud(bytes, &limit).is_err());
    }
}

#[test]
fn inline_and_metadata_combined_budget_is_enforced() {
    let table = json!({"POINTS_LENGTH":2,"POSITION":[0.0,1.0,2.0,3.0,4.0,5.0]});
    let data = build(table, &[]);
    let parsed = parse_pnts(&data, &DecodeLimits::default()).unwrap();
    let limit = DecodeLimits {
        max_decoded_bytes: parsed.decoded_bytes - 1,
        ..Default::default()
    };
    assert!(parse_pnts(&data, &limit).is_err());
}

#[test]
fn upstream_required_fields_and_truncated_input() {
    let l = DecodeLimits::default();
    for table in [
        json!({"POSITION":{"byteOffset":0}}),
        json!({"POINTS_LENGTH":1}),
        json!({"POINTS_LENGTH":1,"POSITION_QUANTIZED":{"byteOffset":0},"QUANTIZED_VOLUME_OFFSET":[0,0,0]}),
        json!({"POINTS_LENGTH":1,"POSITION_QUANTIZED":{"byteOffset":0},"QUANTIZED_VOLUME_SCALE":[1,1,1]}),
        json!({"POINTS_LENGTH":1,"POSITION":{"byteOffset":0},"BATCH_ID":{"byteOffset":12}}),
    ] {
        assert!(parse_pnts(&build(table, &[0; 20]), &l).is_err());
    }
    let bytes = include_bytes!("../../../tests/fixtures/cesium-pointcloud-rgb.pnts");
    let mut bad = bytes.to_vec();
    bad[4..8].copy_from_slice(&2u32.to_le_bytes());
    assert!(parse_pnts(&bad, &l).is_err());
    for n in 0..bytes.len() {
        assert!(parse_pnts(&bytes[..n], &l).is_err());
    }
    let mut bad = bytes.to_vec();
    bad[12..16].copy_from_slice(&0u32.to_le_bytes());
    assert!(parse_pnts(&bad, &l).is_err());
    let invalid_ids = json!({"POINTS_LENGTH":1,"POSITION":{"byteOffset":0},"BATCH_ID":{"byteOffset":12,"componentType":"UNSIGNED_BYTE"},"BATCH_LENGTH":1});
    let mut binary = [0u8; 13];
    binary[12] = 1;
    assert!(parse_pnts(&build(invalid_ids, &binary), &l).is_err());
}

#[test]
fn quantized_local_offsets_survive_large_world_origins() {
    let data = build(
        json!({
            "POINTS_LENGTH": 1, "POSITION_QUANTIZED": [32768, 16384, 65535],
            "QUANTIZED_VOLUME_SCALE": [1, 1, 1],
            "QUANTIZED_VOLUME_OFFSET": [1e16, 1e16, 1e16]
        }),
        &[],
    );
    let cloud = decode_point_cloud(&data, &DecodeLimits::default()).unwrap();
    assert_eq!(cloud.origin, DVec3::splat(1e16));
    let position = cloud.points[0].position;
    assert!((position[0] - 32768.0 / 65535.0).abs() < 1e-7);
    assert!((position[1] - 16384.0 / 65535.0).abs() < 1e-7);
    assert_eq!(position[2], 1.0);
}

#[test]
fn upstream_draco_full_partial_and_batched_point_clouds() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/cesium-pointcloud-draco.pnts").as_slice(),
        include_bytes!("../../../tests/fixtures/cesium-pointcloud-draco-partial.pnts").as_slice(),
        include_bytes!("../../../tests/fixtures/cesium-pointcloud-draco-batched.pnts").as_slice(),
    ] {
        let cloud = decode_point_cloud(bytes, &DecodeLimits::default()).unwrap();
        assert_eq!(cloud.points.len(), 1000);
        assert!(cloud.has_colors && cloud.has_normals);
        assert!(cloud
            .points
            .iter()
            .all(|p| p.position.iter().all(|v| v.is_finite())));
        if cloud.metadata.get("name").is_some() {
            assert_eq!(cloud.metadata["name"][0], "section0");
            assert!(cloud.points.iter().all(|p| p.feature_id < 8));
        } else {
            assert_eq!(
                cloud.metadata["temperature"].as_array().unwrap().len(),
                1000
            );
            assert_eq!(
                cloud.metadata["secondaryColor"][0]
                    .as_array()
                    .unwrap()
                    .len(),
                3
            );
            assert_eq!(cloud.metadata["id"].as_array().unwrap().len(), 1000);
        }
    }
    let invalid = include_bytes!("../../../tests/fixtures/cesium-pointcloud-draco-invalid.pnts");
    assert!(decode_point_cloud(invalid, &DecodeLimits::default()).is_err());
}
