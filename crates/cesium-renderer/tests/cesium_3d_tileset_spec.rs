use cesium_renderer::cesium_3d_tileset::{
    Cesium3DTileset, Cesium3DTilesetOptions, TilesetHeightReference,
};
use cesium_renderer::create_vector_tile_buffers::BlendOption;
use cesium_renderer::scene_primitive::PrimitiveRenderFrameState;
use serde_json::json;

#[test]
fn loads_json_with_static_load_json_method() {
    let tileset_json = json!({
        "asset": {
            "version": 1.1
        }
    });

    // Raw JSON string
    let parsed1 = Cesium3DTileset::load_json(&tileset_json.to_string()).unwrap();
    assert_eq!(parsed1["asset"]["version"], 1.1);

    // Base64 data URI
    // "eyJhc3NldCI6eyJ2ZXJzaW9uIjoxLjF9fQ==" is base64 for '{"asset":{"version":1.1}}'
    let data_uri = "data:text/plain;base64,eyJhc3NldCI6eyJ2ZXJzaW9uIjoxLjF9fQ==";
    let parsed2 = Cesium3DTileset::load_json(data_uri).unwrap();
    assert_eq!(parsed2["asset"]["version"], 1.1);
}

#[test]
fn from_url_throws_without_url() {
    let res = Cesium3DTileset::from_url(None, None);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(err.is_ok()); // DeveloperError
    assert_eq!(err.unwrap().message, "url is required, actual value was undefined");
}

#[test]
fn throws_with_clamping_height_reference_and_no_scene() {
    let res = Cesium3DTileset::new(Cesium3DTilesetOptions {
        height_reference: Some(TilesetHeightReference::ClampToGround),
        has_scene: false,
        ..Default::default()
    });
    assert!(res.is_err());
    assert_eq!(
        res.err().unwrap().message,
        "HeightReference requires scene to be defined."
    );
}

#[test]
fn vector_blend_option_defaults_and_configures() {
    let default_ts = Cesium3DTileset::default();
    assert_eq!(default_ts.vector_blend_option, BlendOption::Translucent);

    let opaque_ts = Cesium3DTileset::new(Cesium3DTilesetOptions {
        vector_blend_option: Some(BlendOption::Opaque),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(opaque_ts.vector_blend_option, BlendOption::Opaque);
}

#[test]
fn from_url_throws_with_unsupported_version_or_unsupported_extension() {
    // Version 2.0 -> error
    let unsupported_ver = json!({
        "asset": {
            "version": "2.0"
        }
    });
    let res1 = Cesium3DTileset::from_url(Some(&unsupported_ver.to_string()), None);
    assert!(res1.is_err());
    let err1 = res1.err().unwrap();
    assert!(err1.is_err()); // RuntimeError
    assert_eq!(
        err1.unwrap_err().message,
        "The tileset must be 3D Tiles version 0.0, 1.0, or 1.1"
    );

    // Unsupported extension
    let unsupported_ext = json!({
        "asset": {
            "version": "1.0"
        },
        "extensionsUsed": ["unsupported_extension"],
        "extensionsRequired": ["unsupported_extension"]
    });
    let res2 = Cesium3DTileset::from_url(Some(&unsupported_ext.to_string()), None);
    assert!(res2.is_err());
    let err2 = res2.err().unwrap();
    assert!(err2.is_err());
    assert_eq!(
        err2.unwrap_err().message,
        "Unsupported 3D Tiles Extension: unsupported_extension"
    );
}

#[test]
fn from_ion_asset_id_throws_without_asset_id() {
    let res = Cesium3DTileset::from_ion_asset_id(None, None);
    assert!(res.is_err());
    assert_eq!(
        res.err().unwrap().message,
        "assetId is required, actual value was undefined"
    );
}

#[test]
fn loads_tileset_json_configures_root_and_traversal_emits_commands() {
    let valid_tileset = json!({
        "asset": {
            "version": "1.0",
            "tilesetVersion": "1.2.3"
        },
        "root": {
            "geometricError": 10.0,
            "refine": "REPLACE",
            "boundingVolume": {
                "sphere": [0.0, 0.0, 0.0, 50.0]
            }
        }
    });

    let mut tileset = Cesium3DTileset::from_url(Some(&valid_tileset.to_string()), None)
        .expect("valid tileset loading");

    assert!(tileset.asset.is_some());
    let asset = tileset.asset.as_ref().unwrap();
    assert_eq!(asset.version, "1.0");
    assert_eq!(asset.tileset_version.as_deref(), Some("1.2.3"));
    assert!(tileset.root.is_some());

    // Update emits draw commands
    let mut frame_state = PrimitiveRenderFrameState::default();
    tileset.update(&mut frame_state);
    assert!(!frame_state.command_list.is_empty());

    // show = false suppresses commands
    tileset.show = false;
    let mut frame_state_hidden = PrimitiveRenderFrameState::default();
    tileset.update(&mut frame_state_hidden);
    assert!(frame_state_hidden.command_list.is_empty());

    // destroy
    tileset.destroy();
    assert!(tileset.is_destroyed());
    assert!(tileset.root.is_none());
}
