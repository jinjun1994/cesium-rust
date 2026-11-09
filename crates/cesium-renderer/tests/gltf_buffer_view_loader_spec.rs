use cesium_math::RuntimeError;
use cesium_renderer::{
    BufferLoader, BufferLoaderOptions, GltfBufferViewLoader, GltfBufferViewLoaderOptions,
    ResourceLoaderState,
};
use serde_json::json;

type EmptyFetcher = fn(String) -> std::future::Ready<Result<Vec<u8>, String>>;

async fn load_embedded(loader: &GltfBufferViewLoader) -> Result<(), RuntimeError> {
    loader.load::<EmptyFetcher, _>(None).await
}

#[test]
fn test_throws_if_gltf_invalid() {
    let empty_gltf = json!({});
    let res = GltfBufferViewLoader::new(GltfBufferViewLoaderOptions {
        gltf: empty_gltf,
        buffer_view_id: 0,
        gltf_resource: None,
        base_resource: None,
        buffer_loader: None,
        cache_key: None,
    });
    assert!(res.is_err());
}

#[test]
fn test_loads_from_embedded_buffer() {
    pollster::block_on(async {
        let gltf = json!({
            "buffers": [{ "byteLength": 8 }],
            "bufferViews": [{ "buffer": 0, "byteOffset": 2, "byteLength": 3 }]
        });

        let buffer_bytes = vec![1, 3, 7, 15, 31, 63, 127, 255];
        let buffer_loader = BufferLoader::new(BufferLoaderOptions {
            typed_array: Some(buffer_bytes),
            ..Default::default()
        })
        .unwrap();

        let loader = GltfBufferViewLoader::new(GltfBufferViewLoaderOptions {
            gltf,
            buffer_view_id: 0,
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: None,
            buffer_loader: Some(buffer_loader),
            cache_key: Some("bufferView_0".to_string()),
        })
        .unwrap();

        assert_eq!(loader.state(), ResourceLoaderState::Unloaded);
        assert_eq!(loader.cache_key(), Some("bufferView_0".to_string()));

        load_embedded(&loader).await.unwrap();

        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        assert_eq!(loader.typed_array(), Some(vec![7, 15, 31]));
    });
}

#[test]
fn test_loads_from_external_buffer() {
    pollster::block_on(async {
        let gltf = json!({
            "buffers": [{ "uri": "external.bin", "byteLength": 8 }],
            "bufferViews": [{ "buffer": 0, "byteOffset": 2, "byteLength": 3 }]
        });

        let loader = GltfBufferViewLoader::new(GltfBufferViewLoaderOptions {
            gltf,
            buffer_view_id: 0,
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            base_resource: Some("https://example.com".to_string()),
            buffer_loader: None,
            cache_key: None,
        })
        .unwrap();

        let fetcher = |url: String| async move {
            assert_eq!(url, "https://example.com/external.bin");
            Ok(vec![1, 3, 7, 15, 31, 63, 127, 255])
        };

        loader.load(Some(fetcher)).await.unwrap();

        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        assert_eq!(loader.typed_array(), Some(vec![7, 15, 31]));
    });
}

#[test]
fn test_handles_external_buffer_failure() {
    pollster::block_on(async {
        let gltf = json!({
            "buffers": [{ "uri": "missing.bin", "byteLength": 8 }],
            "bufferViews": [{ "buffer": 0, "byteOffset": 2, "byteLength": 3 }]
        });

        let loader = GltfBufferViewLoader::new(GltfBufferViewLoaderOptions {
            gltf,
            buffer_view_id: 0,
            gltf_resource: None,
            base_resource: None,
            buffer_loader: None,
            cache_key: None,
        })
        .unwrap();

        let fetcher = |_url: String| async move { Err("404 Not Found".to_string()) };

        let res = loader.load(Some(fetcher)).await;
        assert!(res.is_err());
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn test_handles_slice_exceeding_buffer() {
    pollster::block_on(async {
        let gltf = json!({
            "buffers": [{ "byteLength": 4 }],
            "bufferViews": [{ "buffer": 0, "byteOffset": 2, "byteLength": 10 }]
        });

        let buffer_loader = BufferLoader::new(BufferLoaderOptions {
            typed_array: Some(vec![1, 2, 3, 4]),
            ..Default::default()
        })
        .unwrap();

        let loader = GltfBufferViewLoader::new(GltfBufferViewLoaderOptions {
            gltf,
            buffer_view_id: 0,
            gltf_resource: None,
            base_resource: None,
            buffer_loader: Some(buffer_loader),
            cache_key: None,
        })
        .unwrap();

        let res = load_embedded(&loader).await;
        assert!(res.is_err());
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn test_meshopt_extension_metadata() {
    pollster::block_on(async {
        let gltf = json!({
            "buffers": [{ "byteLength": 29 }, { "byteLength": 360 }],
            "bufferViews": [
                {
                    "buffer": 1,
                    "byteOffset": 96,
                    "byteLength": 192,
                    "byteStride": 8,
                    "extensions": {
                        "KHR_meshopt_compression": {
                            "buffer": 0,
                            "byteOffset": 10,
                            "byteLength": 124,
                            "byteStride": 8,
                            "mode": "ATTRIBUTES",
                            "count": 24,
                            "filter": "COLOR"
                        }
                    }
                }
            ]
        });

        let buffer_0_bytes = vec![0u8; 150];
        let buffer_loader = BufferLoader::new(BufferLoaderOptions {
            typed_array: Some(buffer_0_bytes),
            ..Default::default()
        })
        .unwrap();

        let loader = GltfBufferViewLoader::new(GltfBufferViewLoaderOptions {
            gltf,
            buffer_view_id: 0,
            gltf_resource: None,
            base_resource: None,
            buffer_loader: Some(buffer_loader),
            cache_key: None,
        })
        .unwrap();

        assert!(loader.meshopt().is_some());
        let info = loader.meshopt().unwrap();
        assert_eq!(info.buffer, 0);
        assert_eq!(info.byte_offset, 10);
        assert_eq!(info.byte_length, 124);
        assert_eq!(info.byte_stride, 8);
        assert_eq!(info.count, 24);
        assert_eq!(info.mode, "ATTRIBUTES");
        assert_eq!(info.filter, "COLOR");

        load_embedded(&loader).await.unwrap();

        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        assert_eq!(loader.typed_array().unwrap().len(), 124);

        // Destroy
        assert!(!loader.is_destroyed());
        loader.destroy();
        assert!(loader.is_destroyed());
        assert!(loader.typed_array().is_none());
    });
}
