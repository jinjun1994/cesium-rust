use cesium_math::IndexDatatype;
use cesium_renderer::buffer_loader::ResourceLoaderState;
use cesium_renderer::gltf_index_buffer_loader::{
    GltfIndexBufferLoader, GltfIndexBufferLoaderOptions,
};
use serde_json::json;

fn make_test_gltf() -> (serde_json::Value, Vec<u8>) {
    let mut bytes = vec![0u8; 72]; // pos + norm
    // buffer 0:
    // offset 0: positions (Float32Array[-1, -1, -1, 1, 1, 1, 0, 1, 0] = 36 bytes)
    // offset 36: normals (Float32Array[-1, 0, 0, 1, 0, 0, 0, 1, 0] = 36 bytes)
    // offset 72: uint32 indices [0, 1, 2] = 12 bytes
    // offset 84: uint16 indices [0, 1, 2] = 6 bytes
    // offset 90: uint8 indices [0, 1, 2] = 3 bytes
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&2u32.to_le_bytes()); // 72..84

    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes()); // 84..90

    bytes.extend_from_slice(&[0u8, 1u8, 2u8]); // 90..93

    let gltf = json!({
        "buffers": [{ "byteLength": bytes.len(), "uri": "external.bin" }],
        "bufferViews": [
            { "buffer": 0, "byteOffset": 0, "byteLength": 36 },
            { "buffer": 0, "byteOffset": 36, "byteLength": 36 },
            { "buffer": 0, "byteOffset": 72, "byteLength": 12 },
            { "buffer": 0, "byteOffset": 84, "byteLength": 6 },
            { "buffer": 0, "byteOffset": 90, "byteLength": 3 }
        ],
        "accessors": [
            { "componentType": 5126, "count": 3, "type": "VEC3", "bufferView": 0, "byteOffset": 0 },
            { "componentType": 5126, "count": 3, "type": "VEC3", "bufferView": 1, "byteOffset": 0 },
            { "componentType": 5125, "count": 3, "type": "SCALAR", "bufferView": 2, "byteOffset": 0 },
            { "componentType": 5123, "count": 3, "type": "SCALAR", "bufferView": 3, "byteOffset": 0 },
            { "componentType": 5121, "count": 3, "type": "SCALAR", "bufferView": 4, "byteOffset": 0 }
        ]
    });

    (gltf, bytes)
}

#[test]
fn throws_if_required_options_are_missing() {
    let (gltf, _) = make_test_gltf();

    // gltf missing
    assert!(GltfIndexBufferLoader::new(GltfIndexBufferLoaderOptions {
        gltf: None,
        accessor_id: Some(3),
        gltf_resource: Some("https://example.com/model.gltf".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        load_buffer: Some(true),
        ..Default::default()
    })
    .is_err());

    // accessor_id missing
    assert!(GltfIndexBufferLoader::new(GltfIndexBufferLoaderOptions {
        gltf: Some(gltf.clone()),
        accessor_id: None,
        gltf_resource: Some("https://example.com/model.gltf".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        load_buffer: Some(true),
        ..Default::default()
    })
    .is_err());

    // gltf_resource missing
    assert!(GltfIndexBufferLoader::new(GltfIndexBufferLoaderOptions {
        gltf: Some(gltf.clone()),
        accessor_id: Some(3),
        gltf_resource: None,
        base_resource: Some("https://example.com/".to_string()),
        load_buffer: Some(true),
        ..Default::default()
    })
    .is_err());

    // base_resource missing
    assert!(GltfIndexBufferLoader::new(GltfIndexBufferLoaderOptions {
        gltf: Some(gltf.clone()),
        accessor_id: Some(3),
        gltf_resource: Some("https://example.com/model.gltf".to_string()),
        base_resource: None,
        load_buffer: Some(true),
        ..Default::default()
    })
    .is_err());

    // both load_buffer and load_typed_array are false
    assert!(GltfIndexBufferLoader::new(GltfIndexBufferLoaderOptions {
        gltf: Some(gltf),
        accessor_id: Some(3),
        gltf_resource: Some("https://example.com/model.gltf".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        load_buffer: Some(false),
        load_typed_array: Some(false),
        ..Default::default()
    })
    .is_err());
}

#[test]
fn loads_from_accessor_into_buffer() {
    pollster::block_on(async {
        let (gltf, bytes) = make_test_gltf();
        let loader = GltfIndexBufferLoader::new(GltfIndexBufferLoaderOptions {
            gltf: Some(gltf),
            accessor_id: Some(3),
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            load_buffer: Some(true),
            load_typed_array: Some(false),
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

        let processed = loader.process();
        assert!(processed);
        assert_eq!(loader.state(), ResourceLoaderState::Ready);

        let buf = loader.buffer().unwrap();
        assert_eq!(buf.size_in_bytes, 6);
        assert_eq!(buf.index_datatype, Some(IndexDatatype::UNSIGNED_SHORT));
        assert!(loader.typed_array().is_none());
    });
}

#[test]
fn loads_from_accessor_as_typed_array() {
    pollster::block_on(async {
        let (gltf, bytes) = make_test_gltf();
        let loader = GltfIndexBufferLoader::new(GltfIndexBufferLoaderOptions {
            gltf: Some(gltf),
            accessor_id: Some(3),
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            load_buffer: Some(false),
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
        let typed_array = loader.typed_array().unwrap();
        assert_eq!(typed_array.len(), 6);
    });
}

#[test]
fn loads_from_accessor_as_buffer_and_typed_array() {
    pollster::block_on(async {
        let (gltf, bytes) = make_test_gltf();
        let loader = GltfIndexBufferLoader::new(GltfIndexBufferLoaderOptions {
            gltf: Some(gltf),
            accessor_id: Some(3),
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            base_resource: Some("https://example.com/".to_string()),
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
fn loads_uint32_and_uint8_indices() {
    pollster::block_on(async {
        let (gltf, bytes) = make_test_gltf();

        // uint32 (accessor 2)
        let loader_u32 = GltfIndexBufferLoader::new(GltfIndexBufferLoaderOptions {
            gltf: Some(gltf.clone()),
            accessor_id: Some(2),
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            load_buffer: Some(true),
            ..Default::default()
        })
        .unwrap();

        let b1 = bytes.clone();
        loader_u32
            .load(Some(move |_| {
                let b = b1.clone();
                async move { Ok(b) }
            }))
            .await
            .unwrap();
        loader_u32.process();
        assert_eq!(loader_u32.buffer().unwrap().size_in_bytes, 12);
        assert_eq!(
            loader_u32.index_datatype(),
            IndexDatatype::UNSIGNED_INT
        );

        // uint8 (accessor 4)
        let loader_u8 = GltfIndexBufferLoader::new(GltfIndexBufferLoaderOptions {
            gltf: Some(gltf),
            accessor_id: Some(4),
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            load_buffer: Some(true),
            ..Default::default()
        })
        .unwrap();

        let b2 = bytes.clone();
        loader_u8
            .load(Some(move |_| {
                let b = b2.clone();
                async move { Ok(b) }
            }))
            .await
            .unwrap();
        loader_u8.process();
        assert_eq!(loader_u8.buffer().unwrap().size_in_bytes, 3);
        assert_eq!(
            loader_u8.index_datatype(),
            IndexDatatype::UNSIGNED_BYTE
        );
    });
}

#[test]
fn load_throws_if_buffer_view_fails_to_load() {
    pollster::block_on(async {
        let (gltf, _) = make_test_gltf();
        let loader = GltfIndexBufferLoader::new(GltfIndexBufferLoaderOptions {
            gltf: Some(gltf),
            accessor_id: Some(3),
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            base_resource: Some("https://example.com/".to_string()),
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
fn loads_from_draco_and_overrides_index_datatype() {
    pollster::block_on(async {
        let gltf = json!({
            "accessors": [
                { "componentType": 5125, "count": 3, "type": "SCALAR" }
            ]
        });

        let decoded_indices = vec![0u8, 0u8, 1u8, 0u8, 2u8, 0u8]; // uint16 3 indices = 6 bytes

        let loader = GltfIndexBufferLoader::new(GltfIndexBufferLoaderOptions {
            gltf: Some(gltf),
            accessor_id: Some(0),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            draco: Some(json!({ "bufferView": 0 })),
            decoded_draco_indices: Some((decoded_indices, IndexDatatype::UNSIGNED_SHORT)),
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
        assert_eq!(loader.index_datatype(), IndexDatatype::UNSIGNED_SHORT);
        assert_eq!(loader.buffer().unwrap().size_in_bytes, 6);
    });
}

#[test]
fn process_throws_if_draco_fails_to_load() {
    pollster::block_on(async {
        let gltf = json!({
            "accessors": [
                { "componentType": 5123, "count": 3, "type": "SCALAR" }
            ]
        });

        let loader = GltfIndexBufferLoader::new(GltfIndexBufferLoaderOptions {
            gltf: Some(gltf),
            accessor_id: Some(0),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            draco: Some(json!({ "bufferView": 0 })),
            draco_should_fail: Some(true),
            load_buffer: Some(true),
            ..Default::default()
        })
        .unwrap();

        let result = loader
            .load(None::<fn(String) -> std::future::Ready<Result<Vec<u8>, String>>>)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to load Draco"));
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn destroys_index_buffer() {
    pollster::block_on(async {
        let (gltf, bytes) = make_test_gltf();
        let loader = GltfIndexBufferLoader::new(GltfIndexBufferLoaderOptions {
            gltf: Some(gltf),
            accessor_id: Some(3),
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            base_resource: Some("https://example.com/".to_string()),
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
