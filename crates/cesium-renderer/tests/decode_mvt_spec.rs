//! Spec matching CesiumJS `Specs/Scene/decodeMVTSpec.js` (Apache-2.0).

use cesium_renderer::decode_mvt;
use serde_json::json;

fn encode_varint(mut value: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    while value >= 0x80 {
        bytes.push(((value & 0x7f) | 0x80) as u8);
        value >>= 7;
    }
    bytes.push(value as u8);
    bytes
}

fn encode_tag(field_number: u32, wire_type: u32) -> Vec<u8> {
    encode_varint((field_number << 3) | wire_type)
}

fn encode_uint(field_number: u32, value: u32) -> Vec<u8> {
    let mut res = encode_tag(field_number, 0);
    res.extend(encode_varint(value));
    res
}

fn encode_bytes(field_number: u32, value_bytes: &[u8]) -> Vec<u8> {
    let mut res = encode_tag(field_number, 2);
    res.extend(encode_varint(value_bytes.len() as u32));
    res.extend_from_slice(value_bytes);
    res
}

fn encode_string(field_number: u32, value: &str) -> Vec<u8> {
    encode_bytes(field_number, value.as_bytes())
}

fn encode_packed_varints(field_number: u32, values: &[u32]) -> Vec<u8> {
    let mut payload = Vec::new();
    for &v in values {
        payload.extend(encode_varint(v));
    }
    encode_bytes(field_number, &payload)
}

fn encode_value_string(value: &str) -> Vec<u8> {
    encode_string(1, value)
}

struct FeatureOptions<'a> {
    geom_type: u32,
    tags: &'a [u32],
    geometry_commands: &'a [u32],
}

fn encode_feature(opts: FeatureOptions) -> Vec<u8> {
    let mut bytes = Vec::new();
    if !opts.tags.is_empty() {
        bytes.extend(encode_packed_varints(2, opts.tags));
    }
    bytes.extend(encode_uint(3, opts.geom_type));
    bytes.extend(encode_packed_varints(4, opts.geometry_commands));
    bytes
}

struct LayerOptions<'a> {
    name: &'a str,
    features: &'a [Vec<u8>],
    keys: &'a [&'a str],
    values: &'a [Vec<u8>],
    extent: Option<u32>,
}

fn encode_layer(opts: LayerOptions) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(encode_string(1, opts.name));
    for feat in opts.features {
        bytes.extend(encode_bytes(2, feat));
    }
    for &k in opts.keys {
        bytes.extend(encode_string(3, k));
    }
    for v in opts.values {
        bytes.extend(encode_bytes(4, v));
    }
    bytes.extend(encode_uint(5, opts.extent.unwrap_or(4096)));
    bytes.extend(encode_uint(15, 2)); // layer version 2
    bytes
}

fn encode_tile(layers: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for l in layers {
        bytes.extend(encode_bytes(3, l));
    }
    bytes
}

#[test]
fn test_decodes_point_geometry_and_feature_properties() {
    let point_feature = encode_feature(FeatureOptions {
        geom_type: 1, // POINT
        tags: &[0, 0], // key[0] -> value[0]
        geometry_commands: &[9, 50, 34], // MoveTo(1), dx=25, dy=17
    });

    let layer = encode_layer(LayerOptions {
        name: "roads",
        features: &[point_feature],
        keys: &["id"],
        values: &[encode_value_string("p1")],
        extent: None,
    });

    let tile = encode_tile(&[layer]);

    let decoded = decode_mvt(&tile).expect("MVT decoding should succeed");
    assert_eq!(decoded.layers.len(), 1);
    assert_eq!(decoded.layers[0].name, "roads");
    assert_eq!(decoded.layers[0].features.len(), 1);

    let feature = &decoded.layers[0].features[0];
    assert_eq!(feature.feature_type, "Point");
    assert_eq!(feature.geometry, json!([{ "x": 25, "y": 17 }]));
    assert_eq!(feature.properties.get("id").unwrap(), "p1");
}

#[test]
fn test_decodes_linestring_and_polygon_command_streams() {
    let line_feature = encode_feature(FeatureOptions {
        geom_type: 2, // LINESTRING
        tags: &[],
        geometry_commands: &[9, 0, 0, 18, 20, 0, 0, 10],
    });

    let polygon_feature = encode_feature(FeatureOptions {
        geom_type: 3, // POLYGON
        tags: &[],
        geometry_commands: &[9, 0, 0, 26, 20, 0, 0, 20, 19, 0, 15],
    });

    let layer = encode_layer(LayerOptions {
        name: "shapes",
        features: &[line_feature, polygon_feature],
        keys: &[],
        values: &[],
        extent: None,
    });

    let tile = encode_tile(&[layer]);

    let decoded = decode_mvt(&tile).expect("MVT decoding should succeed");
    assert_eq!(decoded.layers.len(), 1);
    assert_eq!(decoded.layers[0].features.len(), 2);

    let line = &decoded.layers[0].features[0];
    let polygon = &decoded.layers[0].features[1];

    assert_eq!(line.feature_type, "LineString");
    assert_eq!(
        line.geometry[0],
        json!([
            { "x": 0, "y": 0 },
            { "x": 10, "y": 0 },
            { "x": 10, "y": 5 }
        ])
    );

    assert_eq!(polygon.feature_type, "Polygon");
    assert_eq!(
        polygon.geometry[0],
        json!([
            { "x": 0, "y": 0 },
            { "x": 10, "y": 0 },
            { "x": 10, "y": 10 },
            { "x": 0, "y": 10 },
            { "x": 0, "y": 0 }
        ])
    );
}
