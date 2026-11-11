#![allow(clippy::excessive_precision)]

use cesium_renderer::cesium_3d_tile_content_type::Cesium3DTileContentType;
use cesium_renderer::preprocess_3d_tile_content::preprocess_3d_tile_content;
use serde_json::json;

fn make_binary_file(magic: &str) -> Vec<u8> {
    let mut data = Vec::with_capacity(magic.len() + 8);
    data.extend_from_slice(magic.as_bytes());
    data.resize(magic.len() + 8, 0);
    data
}

fn make_json_file(val: &serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(val).unwrap()
}

fn make_glb(json_val: &serde_json::Value, binary_data: Option<&[u8]>) -> Vec<u8> {
    let mut json_bytes = serde_json::to_vec(json_val).unwrap();
    // pad to 4-byte alignment with space (0x20)
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(0x20);
    }

    let mut bin_bytes = binary_data.unwrap_or(&[]).to_vec();
    while !bin_bytes.len().is_multiple_of(4) {
        bin_bytes.push(0);
    }

    let mut total_len = 12 + 8 + json_bytes.len();
    if !bin_bytes.is_empty() {
        total_len += 8 + bin_bytes.len();
    }

    let mut glb = Vec::with_capacity(total_len);
    // Header
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&(total_len as u32).to_le_bytes());

    // JSON chunk
    glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    glb.extend_from_slice(&0x4e4f534au32.to_le_bytes()); // "JSON"
    glb.extend_from_slice(&json_bytes);

    // BIN chunk
    if !bin_bytes.is_empty() {
        glb.extend_from_slice(&(bin_bytes.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x004e4942u32.to_le_bytes()); // "BIN\0"
        glb.extend_from_slice(&bin_bytes);
    }

    glb
}

#[test]
fn detects_binary_contents_by_magic_number() {
    let magics = ["b3dm", "i3dm", "pnts", "cmpt", "vctr", "geom", "subt"];
    for magic in magics {
        let typed_array = make_binary_file(magic);
        let results = preprocess_3d_tile_content(&typed_array).unwrap();
        assert_eq!(results.content_type, magic);
        assert_eq!(results.binary_payload, Some(typed_array));
        assert!(results.json_payload.is_none());
    }
}

#[test]
fn detects_external_tilesets() {
    let external_tileset = json!({
        "asset": {
            "version": "1.0",
        },
        "geometricError": 100,
        "root": {}
    });
    let payload = make_json_file(&external_tileset);
    let results = preprocess_3d_tile_content(&payload).unwrap();
    assert_eq!(
        results.content_type,
        Cesium3DTileContentType::EXTERNAL_TILESET
    );
    assert_eq!(results.json_payload, Some(external_tileset));
    assert!(results.binary_payload.is_none());
}

#[test]
fn labels_binary_gltf_as_glb() {
    let typed_array = make_binary_file("glTF");
    let results = preprocess_3d_tile_content(&typed_array).unwrap();
    assert_eq!(results.content_type, Cesium3DTileContentType::GLTF_BINARY);
    assert_eq!(results.binary_payload, Some(typed_array));
    assert!(results.json_payload.is_none());
}

#[test]
fn detects_gltf_json_content() {
    let gltf = json!({
        "asset": {
            "version": "1.0",
        },
        "meshes": []
    });
    let payload = make_json_file(&gltf);
    let results = preprocess_3d_tile_content(&payload).unwrap();
    assert_eq!(results.content_type, Cesium3DTileContentType::GLTF);
    assert_eq!(results.json_payload, Some(gltf));
    assert!(results.binary_payload.is_none());
}

#[test]
fn detects_gltf_json_subtrees() {
    let gltf = json!({
        "asset": {
            "version": "2.1",
        },
        "extensions": {
            "3DTILES_subtree": {
                "tileAvailability": { "constant": 1 },
                "childSubtreeAvailability": { "constant": 0 }
            }
        }
    });
    let payload = make_json_file(&gltf);
    let results = preprocess_3d_tile_content(&payload).unwrap();
    assert_eq!(
        results.content_type,
        Cesium3DTileContentType::IMPLICIT_SUBTREE_GLTF
    );
    assert_eq!(results.json_payload, Some(gltf));
    assert!(results.binary_payload.is_none());
}

#[test]
fn detects_glb_subtrees() {
    let gltf = json!({
        "asset": {
            "version": "2.1",
        },
        "extensions": {
            "3DTILES_subtree": {
                "tileAvailability": { "constant": 1 },
                "childSubtreeAvailability": { "constant": 0 }
            }
        }
    });
    let glb = make_glb(&gltf, Some(&[0u8; 8]));
    let results = preprocess_3d_tile_content(&glb).unwrap();
    assert_eq!(
        results.content_type,
        Cesium3DTileContentType::IMPLICIT_SUBTREE_GLB
    );
    assert_eq!(results.binary_payload, Some(glb));
    assert!(results.json_payload.is_none());
}

#[test]
fn labels_a_glb_with_a_json_chunk_and_no_subtree_extension_as_glb() {
    let gltf = json!({
        "asset": {
            "version": "2.0",
        },
        "meshes": []
    });
    let glb = make_glb(&gltf, None);
    let results = preprocess_3d_tile_content(&glb).unwrap();
    assert_eq!(results.content_type, Cesium3DTileContentType::GLTF_BINARY);
    assert_eq!(results.binary_payload, Some(glb));
    assert!(results.json_payload.is_none());
}

#[test]
fn detects_geojson_content() {
    let geojson = json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [
                        [
                            [-45.0, -34.98261613683529, -6.162892770953476],
                            [-45.0, -35.264389681864863, -6.091842861846089],
                            [-44.700613777413345, -35.404907941900547, -6.194363857619464],
                            [-44.700613777413345, -35.122642334455918, -6.294969339855015],
                            [-45.0, -34.98261613683529, -6.162892770953476]
                        ]
                    ]
                }
            }
        ]
    });
    let payload = make_json_file(&geojson);
    let results = preprocess_3d_tile_content(&payload).unwrap();
    assert_eq!(results.content_type, Cesium3DTileContentType::GEOJSON);
    assert_eq!(results.json_payload, Some(geojson));
    assert!(results.binary_payload.is_none());
}

#[test]
fn throws_for_invalid_content() {
    let payload = make_binary_file("fake");
    let result = preprocess_3d_tile_content(&payload);
    assert!(result.is_err());
}
