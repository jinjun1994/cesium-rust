use cesium_renderer::i3s_decoder::{
    set_decoder_initialized_for_test, I3SDecoder, I3SFeatureDataItem, I3SGeometryData,
    I3SGeometryParent, I3SGeometrySchema, I3SParentBounding,
};
use serde_json::json;

fn make_default_schema() -> I3SGeometrySchema {
    I3SGeometrySchema {
        ordering: Vec::new(),
        feature_attribute_order: Vec::new(),
    }
}

fn make_geometry_data_obb() -> I3SGeometryData {
    I3SGeometryData {
        binary_data: vec![0u8; 8],
        parent: Some(I3SGeometryParent {
            data: I3SParentBounding {
                obb: Some(json!({ "center": [0.0, 0.0, 0.0] })),
                mbs: None,
            },
            inverse_rotation_matrix: Some([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]),
        }),
        geometry_buffer_info: None,
    }
}

fn make_geometry_data_mbs() -> I3SGeometryData {
    I3SGeometryData {
        binary_data: vec![0u8; 8],
        parent: Some(I3SGeometryParent {
            data: I3SParentBounding {
                obb: None,
                mbs: Some(vec![0.0, 0.0, 0.0]),
            },
            inverse_rotation_matrix: Some([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]),
        }),
        geometry_buffer_info: None,
    }
}

#[test]
fn constructor() {
    let decoder = I3SDecoder::new();
    let _ = decoder;
}

#[test]
fn throws_with_no_url() {
    pollster::block_on(async {
        let schema = make_default_schema();
        let geom = make_geometry_data_obb();

        let err = I3SDecoder::decode(None, Some(&schema), Some(&geom), None, None)
            .await
            .unwrap_err();

        assert!(err.message.contains("url is required"));
    });
}

#[test]
fn throws_with_no_default_geometry_schema() {
    pollster::block_on(async {
        let geom = make_geometry_data_obb();

        let err = I3SDecoder::decode(Some("mockUrl"), None, Some(&geom), None, None)
            .await
            .unwrap_err();

        assert!(err.message.contains("defaultGeometrySchema is required"));
    });
}

#[test]
fn throws_with_no_geometry_data() {
    pollster::block_on(async {
        let schema = make_default_schema();

        let err = I3SDecoder::decode(Some("mockUrl"), Some(&schema), None, None, None)
            .await
            .unwrap_err();

        assert!(err.message.contains("geometryData is required"));
    });
}

#[test]
fn throws_if_not_initialized() {
    pollster::block_on(async {
        set_decoder_initialized_for_test(false);

        let schema = make_default_schema();
        let geom = make_geometry_data_obb();

        let err = I3SDecoder::decode(
            Some("mockUrl"),
            Some(&schema),
            Some(&geom),
            None,
            None,
        )
        .await
        .unwrap_err();

        assert!(err.message.contains("I3S decoder could not be initialized"));

        // Reset state
        set_decoder_initialized_for_test(true);
    });
}

#[test]
fn empty_geometry_with_obb() {
    pollster::block_on(async {
        set_decoder_initialized_for_test(true);

        let schema = make_default_schema();
        let geom = make_geometry_data_obb();

        let result = I3SDecoder::decode(
            Some("mockUrl"),
            Some(&schema),
            Some(&geom),
            None,
            None,
        )
        .await
        .unwrap();

        assert!(result.mesh_data.buffers.is_empty());
        assert!(result.mesh_data.buffer_views.is_empty());
        assert!(result.mesh_data.accessors.is_empty());
        assert!(result.mesh_data.meshes.is_empty());
        assert!(result.mesh_data.nodes.is_empty());
        assert!(result.mesh_data.nodes_in_scene.is_empty());
    });
}

#[test]
fn empty_geometry_with_mbs() {
    pollster::block_on(async {
        set_decoder_initialized_for_test(true);

        let schema = make_default_schema();
        let geom = make_geometry_data_mbs();

        let result = I3SDecoder::decode(
            Some("mockUrl"),
            Some(&schema),
            Some(&geom),
            None,
            None,
        )
        .await
        .unwrap();

        assert!(result.mesh_data.buffers.is_empty());
        assert!(result.mesh_data.buffer_views.is_empty());
        assert!(result.mesh_data.accessors.is_empty());
        assert!(result.mesh_data.meshes.is_empty());
        assert!(result.mesh_data.nodes.is_empty());
        assert!(result.mesh_data.nodes_in_scene.is_empty());
    });
}

#[test]
fn empty_geometry_with_empty_feature_data() {
    pollster::block_on(async {
        set_decoder_initialized_for_test(true);

        let schema = make_default_schema();
        let geom = make_geometry_data_mbs();
        let features = [I3SFeatureDataItem { data: json!({}) }];

        let result = I3SDecoder::decode(
            Some("mockUrl"),
            Some(&schema),
            Some(&geom),
            Some(&features),
            None,
        )
        .await
        .unwrap();

        assert!(result.mesh_data.buffers.is_empty());
        assert!(result.mesh_data.buffer_views.is_empty());
        assert!(result.mesh_data.accessors.is_empty());
        assert!(result.mesh_data.meshes.is_empty());
        assert!(result.mesh_data.nodes.is_empty());
        assert!(result.mesh_data.nodes_in_scene.is_empty());
    });
}
