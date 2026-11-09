use cesium_renderer::buffer_loader::ResourceLoaderState;
use cesium_renderer::gltf_vertex_buffer_loader::{
    GltfVertexBufferLoader, GltfVertexBufferLoaderOptions, QuantizationInfo,
};
use serde_json::json;

fn make_test_gltf() -> (serde_json::Value, Vec<u8>) {
    let mut bytes = vec![0u8; 78];
    // Fill sample position data
    bytes[0..36].copy_from_slice(&[1u8; 36]);
    // Fill sample normal data
    bytes[36..72].copy_from_slice(&[2u8; 36]);
    // Fill sample indices
    bytes[72..78].copy_from_slice(&[3u8; 6]);

    let gltf = json!({
        "buffers": [{ "byteLength": 78, "uri": "external.bin" }],
        "bufferViews": [
            { "buffer": 0, "byteOffset": 0, "byteLength": 36 },
            { "buffer": 0, "byteOffset": 36, "byteLength": 36 },
            { "buffer": 0, "byteOffset": 72, "byteLength": 6 }
        ],
        "accessors": [
            { "componentType": 5126, "count": 3, "type": "VEC3", "bufferView": 0, "byteOffset": 0 },
            { "componentType": 5126, "count": 3, "type": "VEC3", "bufferView": 1, "byteOffset": 0 },
            { "componentType": 5123, "count": 3, "type": "SCALAR", "bufferView": 2, "byteOffset": 0 }
        ]
    });

    (gltf, bytes)
}

#[test]
fn throws_if_required_options_are_missing() {
    let (gltf, _) = make_test_gltf();

    // gltf missing
    assert!(GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
        gltf: None,
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        buffer_view_id: Some(0),
        load_buffer: Some(true),
        ..Default::default()
    })
    .is_err());

    // gltf_resource missing
    assert!(GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
        gltf: Some(gltf.clone()),
        gltf_resource: None,
        base_resource: Some("https://example.com/".to_string()),
        buffer_view_id: Some(0),
        load_buffer: Some(true),
        ..Default::default()
    })
    .is_err());

    // base_resource missing
    assert!(GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
        gltf: Some(gltf.clone()),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: None,
        buffer_view_id: Some(0),
        load_buffer: Some(true),
        ..Default::default()
    })
    .is_err());

    // both load_buffer and load_typed_array false
    assert!(GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
        gltf: Some(gltf),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        buffer_view_id: Some(0),
        load_buffer: Some(false),
        load_typed_array: Some(false),
        ..Default::default()
    })
    .is_err());
}

#[test]
fn validates_source_mutual_exclusivity() {
    let (gltf, _) = make_test_gltf();
    let draco_extension = json!({
        "bufferView": 0,
        "attributes": {
            "POSITION": 0,
            "NORMAL": 1
        }
    });
    let primitive = json!({
        "attributes": { "POSITION": 0 },
        "extensions": {
            "KHR_draco_mesh_compression": draco_extension
        }
    });

    // Both bufferViewId and Draco defined for POSITION
    assert!(GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
        gltf: Some(gltf.clone()),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        buffer_view_id: Some(0),
        primitive: Some(primitive.clone()),
        draco: Some(draco_extension.clone()),
        attribute_semantic: Some("POSITION".to_string()),
        accessor_id: Some(0),
        load_buffer: Some(true),
        ..Default::default()
    })
    .is_err());

    // None defined
    assert!(GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
        gltf: Some(gltf.clone()),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        load_buffer: Some(true),
        ..Default::default()
    })
    .is_err());

    // Draco defined but missing attribute_semantic
    assert!(GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
        gltf: Some(gltf.clone()),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        primitive: Some(primitive.clone()),
        draco: Some(draco_extension.clone()),
        attribute_semantic: None,
        accessor_id: Some(0),
        load_buffer: Some(true),
        ..Default::default()
    })
    .is_err());

    // Unrelated draco attribute with SPZ does not conflict
    let spz = json!({ "bufferView": 0 });
    assert!(GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
        gltf: Some(gltf),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        primitive: Some(primitive),
        draco: Some(draco_extension),
        spz: Some(spz),
        attribute_semantic: Some("TANGENT".to_string()), // not in draco attributes
        load_buffer: Some(true),
        ..Default::default()
    })
    .is_ok());
}

#[test]
fn loads_as_buffer() {
    pollster::block_on(async {
        let (gltf, bytes) = make_test_gltf();
        let loader = GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
            gltf: Some(gltf),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            buffer_view_id: Some(0),
            load_buffer: Some(true),
            ..Default::default()
        })
        .unwrap();

        let bytes_clone = bytes.clone();
        let fetch = move |_url: String| {
            let b = bytes_clone.clone();
            async move { Ok(b) }
        };

        loader.load(Some(fetch)).await.unwrap();
        assert_eq!(loader.state(), ResourceLoaderState::Processing);

        loader.process();
        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        let buf = loader.buffer().unwrap();
        assert_eq!(buf.size_in_bytes, 36);
        assert!(loader.typed_array().is_none());
    });
}

#[test]
fn loads_as_typed_array() {
    pollster::block_on(async {
        let (gltf, bytes) = make_test_gltf();
        let loader = GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
            gltf: Some(gltf),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            buffer_view_id: Some(1),
            load_typed_array: Some(true),
            ..Default::default()
        })
        .unwrap();

        let bytes_clone = bytes.clone();
        let fetch = move |_url: String| {
            let b = bytes_clone.clone();
            async move { Ok(b) }
        };

        loader.load(Some(fetch)).await.unwrap();
        loader.process();

        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        assert!(loader.buffer().is_none());
        assert_eq!(loader.typed_array().unwrap().len(), 36);
    });
}

#[test]
fn loads_as_buffer_and_typed_array() {
    pollster::block_on(async {
        let (gltf, bytes) = make_test_gltf();
        let loader = GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
            gltf: Some(gltf),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            buffer_view_id: Some(2),
            load_buffer: Some(true),
            load_typed_array: Some(true),
            ..Default::default()
        })
        .unwrap();

        let bytes_clone = bytes.clone();
        let fetch = move |_url: String| {
            let b = bytes_clone.clone();
            async move { Ok(b) }
        };

        loader.load(Some(fetch)).await.unwrap();
        loader.process();

        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        assert_eq!(loader.buffer().unwrap().size_in_bytes, 6);
        assert_eq!(loader.typed_array().unwrap().len(), 6);
    });
}

#[test]
fn load_throws_if_buffer_view_fails_to_load() {
    pollster::block_on(async {
        let (gltf, _) = make_test_gltf();
        let loader = GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
            gltf: Some(gltf),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            buffer_view_id: Some(0),
            load_buffer: Some(true),
            ..Default::default()
        })
        .unwrap();

        let fetch = |_url: String| async move { Err("404 Not Found".to_string()) };

        let result = loader.load(Some(fetch)).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to load buffer view"));
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn loads_from_draco_with_quantization() {
    pollster::block_on(async {
        let gltf = json!({
            "accessors": [{ "componentType": 5126, "type": "VEC3" }]
        });
        let draco = json!({
            "attributes": { "POSITION": 0 }
        });
        let primitive = json!({
            "attributes": { "POSITION": 0 },
            "extensions": { "KHR_draco_mesh_compression": draco.clone() }
        });

        let decoded_data = vec![0u8; 18];
        let quant = QuantizationInfo {
            component_datatype: 5123,
            oct_encoded: false,
            oct_encoded_zxy: true,
            attribute_type: "VEC3".to_string(),
            quantized_volume_offset: vec![-1.0, -1.0, -1.0],
            quantized_volume_dimensions: vec![2.0, 2.0, 2.0],
            quantized_volume_step_size: vec![0.001, 0.001, 0.001],
            normalization_range: 65535.0,
        };

        let loader = GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
            gltf: Some(gltf),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            primitive: Some(primitive),
            draco: Some(draco),
            attribute_semantic: Some("POSITION".to_string()),
            accessor_id: Some(0),
            decoded_draco_attribute: Some((decoded_data, Some(quant.clone()))),
            load_buffer: Some(true),
            ..Default::default()
        })
        .unwrap();

        loader
            .load(None::<fn(String) -> std::future::Ready<Result<Vec<u8>, String>>>)
            .await
            .unwrap();
        assert_eq!(loader.state(), ResourceLoaderState::Loaded);

        loader.process();
        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        assert_eq!(loader.buffer().unwrap().size_in_bytes, 18);
        assert_eq!(loader.quantization().unwrap(), quant);
    });
}

#[test]
fn process_throws_if_draco_fails() {
    pollster::block_on(async {
        let gltf = json!({
            "accessors": [{ "componentType": 5126, "type": "VEC3" }]
        });
        let draco = json!({
            "attributes": { "POSITION": 0 }
        });
        let primitive = json!({
            "attributes": { "POSITION": 0 },
            "extensions": { "KHR_draco_mesh_compression": draco.clone() }
        });

        let loader = GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
            gltf: Some(gltf),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            primitive: Some(primitive),
            draco: Some(draco),
            attribute_semantic: Some("POSITION".to_string()),
            accessor_id: Some(0),
            draco_should_fail: Some(true),
            load_buffer: Some(true),
            ..Default::default()
        })
        .unwrap();

        let res = loader
            .load(None::<fn(String) -> std::future::Ready<Result<Vec<u8>, String>>>)
            .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Failed to load Draco"));
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn destroys_vertex_buffer() {
    pollster::block_on(async {
        let (gltf, bytes) = make_test_gltf();
        let loader = GltfVertexBufferLoader::new(GltfVertexBufferLoaderOptions {
            gltf: Some(gltf),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            buffer_view_id: Some(0),
            load_buffer: Some(true),
            ..Default::default()
        })
        .unwrap();

        let fetch = move |_url: String| {
            let b = bytes.clone();
            async move { Ok(b) }
        };

        loader.load(Some(fetch)).await.unwrap();
        loader.process();

        assert!(loader.buffer().is_some());
        assert!(!loader.is_destroyed());

        loader.destroy();
        assert!(loader.buffer().is_none());
        assert!(loader.is_destroyed());
    });
}
