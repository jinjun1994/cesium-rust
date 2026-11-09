use cesium_renderer::buffer_loader::ResourceLoaderState;
use cesium_renderer::gltf_json_loader::{
    is_supported_extension, GltfJsonLoader, GltfJsonLoaderOptions, GLB_CHUNK_TYPE_BIN,
    GLB_CHUNK_TYPE_JSON, GLB_MAGIC,
};
use serde_json::json;

fn create_glb_v2(json_str: &str, bin_data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let json_bytes = json_str.as_bytes();
    let json_padding = (4 - (json_bytes.len() % 4)) % 4;
    let json_chunk_len = json_bytes.len() + json_padding;

    let bin_padding = (4 - (bin_data.len() % 4)) % 4;
    let bin_chunk_len = if !bin_data.is_empty() {
        bin_data.len() + bin_padding
    } else {
        0
    };

    let total_len = 12 + 8 + json_chunk_len + if bin_chunk_len > 0 { 8 + bin_chunk_len } else { 0 };

    // GLB Header
    out.extend_from_slice(&GLB_MAGIC.to_le_bytes()); // magic
    out.extend_from_slice(&2u32.to_le_bytes()); // version
    out.extend_from_slice(&(total_len as u32).to_le_bytes()); // length

    // JSON Chunk
    out.extend_from_slice(&(json_chunk_len as u32).to_le_bytes());
    out.extend_from_slice(&GLB_CHUNK_TYPE_JSON.to_le_bytes());
    out.extend_from_slice(json_bytes);
    out.extend(std::iter::repeat_n(b' ', json_padding));

    // BIN Chunk
    if bin_chunk_len > 0 {
        out.extend_from_slice(&(bin_chunk_len as u32).to_le_bytes());
        out.extend_from_slice(&GLB_CHUNK_TYPE_BIN.to_le_bytes());
        out.extend_from_slice(bin_data);
        out.extend(std::iter::repeat_n(0, bin_padding));
    }

    out
}

fn create_glb_v1(json_str: &str, bin_data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let json_bytes = json_str.as_bytes();
    let total_len = 20 + json_bytes.len() + bin_data.len();

    // GLB v1 Header: magic (4), version (4), length (4), contentLength (4), contentFormat (4)
    out.extend_from_slice(&GLB_MAGIC.to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes());
    out.extend_from_slice(&(total_len as u32).to_le_bytes());
    out.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // JSON format

    out.extend_from_slice(json_bytes);
    out.extend_from_slice(bin_data);
    out
}

#[test]
fn throws_if_gltf_resource_or_base_resource_is_undefined() {
    assert!(GltfJsonLoader::new(GltfJsonLoaderOptions {
        gltf_resource: None,
        base_resource: Some("https://example.com/".to_string()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfJsonLoader::new(GltfJsonLoaderOptions {
        gltf_resource: Some("https://example.com/model.gltf".to_string()),
        base_resource: None,
        ..Default::default()
    })
    .is_err());
}

#[test]
fn loads_from_json_object() {
    pollster::block_on(async {
        let gltf_json = json!({
            "asset": { "version": "2.0" },
            "buffers": [{ "byteLength": 100 }]
        });

        let loader = GltfJsonLoader::new(GltfJsonLoaderOptions {
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            gltf_json: Some(gltf_json.clone()),
            ..Default::default()
        })
        .unwrap();

        assert_eq!(loader.state(), ResourceLoaderState::Unloaded);
        loader
            .load(None::<fn(String) -> std::future::Ready<Result<Vec<u8>, String>>>)
            .await
            .unwrap();

        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        let loaded = loader.gltf().unwrap();
        assert_eq!(loaded["asset"]["version"], "2.0");
    });
}

#[test]
fn loads_from_typed_array_json() {
    pollster::block_on(async {
        let json_text = r#"{"asset": {"version": "2.0"}, "scenes": []}"#;
        let bytes = json_text.as_bytes().to_vec();

        let loader = GltfJsonLoader::new(GltfJsonLoaderOptions {
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            typed_array: Some(bytes),
            ..Default::default()
        })
        .unwrap();

        loader
            .load(None::<fn(String) -> std::future::Ready<Result<Vec<u8>, String>>>)
            .await
            .unwrap();

        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        let loaded = loader.gltf().unwrap();
        assert_eq!(loaded["asset"]["version"], "2.0");
    });
}

#[test]
fn loads_from_glb_v2_typed_array() {
    pollster::block_on(async {
        let json_str = r#"{"asset":{"version":"2.0"},"buffers":[{"byteLength":4}]}"#;
        let bin_data = vec![1, 2, 3, 4];
        let glb = create_glb_v2(json_str, &bin_data);

        let loader = GltfJsonLoader::new(GltfJsonLoaderOptions {
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            typed_array: Some(glb),
            ..Default::default()
        })
        .unwrap();

        loader
            .load(None::<fn(String) -> std::future::Ready<Result<Vec<u8>, String>>>)
            .await
            .unwrap();

        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        let loaded = loader.gltf().unwrap();
        assert_eq!(loaded["asset"]["version"], "2.0");
        assert_eq!(
            loaded["buffers"][0]["extras"]["_pipeline"]["source_len"],
            4
        );
    });
}

#[test]
fn loads_from_glb_v1_typed_array() {
    pollster::block_on(async {
        let json_str = r#"{"asset":{"version":"1.0"},"buffers":[{"byteLength":8}]}"#;
        let bin_data = vec![10, 20, 30, 40, 50, 60, 70, 80];
        let glb = create_glb_v1(json_str, &bin_data);

        let loader = GltfJsonLoader::new(GltfJsonLoaderOptions {
            gltf_resource: Some("https://example.com/model1.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            typed_array: Some(glb),
            ..Default::default()
        })
        .unwrap();

        loader
            .load(None::<fn(String) -> std::future::Ready<Result<Vec<u8>, String>>>)
            .await
            .unwrap();

        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        let loaded = loader.gltf().unwrap();
        assert_eq!(loaded["asset"]["version"], "1.0");
        assert_eq!(
            loaded["buffers"][0]["extras"]["_pipeline"]["source_len"],
            8
        );
    });
}

#[test]
fn loads_via_fetch_function() {
    pollster::block_on(async {
        let loader = GltfJsonLoader::new(GltfJsonLoaderOptions {
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            ..Default::default()
        })
        .unwrap();

        let fetch = |url: String| async move {
            if url == "https://example.com/model.gltf" {
                Ok(r#"{"asset":{"version":"2.0"}}"#.as_bytes().to_vec())
            } else {
                Err("404 Not Found".to_string())
            }
        };

        loader.load(Some(fetch)).await.unwrap();
        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        assert_eq!(loader.gltf().unwrap()["asset"]["version"], "2.0");
    });
}

#[test]
fn rejects_on_fetch_error() {
    pollster::block_on(async {
        let loader = GltfJsonLoader::new(GltfJsonLoaderOptions {
            gltf_resource: Some("https://example.com/nonexistent.gltf".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            ..Default::default()
        })
        .unwrap();

        let fetch = |_url: String| async move { Err("Network error".to_string()) };

        let result = loader.load(Some(fetch)).await;
        assert!(result.is_err());
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn rejects_on_unsupported_version() {
    pollster::block_on(async {
        let gltf_json = json!({
            "asset": { "version": "0.9" }
        });

        let loader = GltfJsonLoader::new(GltfJsonLoaderOptions {
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            gltf_json: Some(gltf_json),
            ..Default::default()
        })
        .unwrap();

        let res = loader
            .load(None::<fn(String) -> std::future::Ready<Result<Vec<u8>, String>>>)
            .await;
        assert!(res.is_err());
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn rejects_on_unsupported_extension() {
    pollster::block_on(async {
        let gltf_json = json!({
            "asset": { "version": "2.0" },
            "extensionsRequired": ["KHR_some_unknown_ext"]
        });

        let loader = GltfJsonLoader::new(GltfJsonLoaderOptions {
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            gltf_json: Some(gltf_json),
            ..Default::default()
        })
        .unwrap();

        let res = loader
            .load(None::<fn(String) -> std::future::Ready<Result<Vec<u8>, String>>>)
            .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("KHR_some_unknown_ext"));
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn test_unload_and_destroy() {
    pollster::block_on(async {
        let gltf_json = json!({
            "asset": { "version": "2.0" }
        });

        let loader = GltfJsonLoader::new(GltfJsonLoaderOptions {
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            gltf_json: Some(gltf_json),
            ..Default::default()
        })
        .unwrap();

        loader
            .load(None::<fn(String) -> std::future::Ready<Result<Vec<u8>, String>>>)
            .await
            .unwrap();
        assert!(loader.gltf().is_some());

        loader.unload();
        assert!(loader.gltf().is_none());

        assert!(!loader.is_destroyed());
        loader.destroy();
        assert!(loader.is_destroyed());
    });
}

#[test]
fn test_supported_extensions_helper() {
    assert!(is_supported_extension("KHR_meshopt_compression"));
    assert!(is_supported_extension("EXT_meshopt_compression"));
    assert!(is_supported_extension("KHR_draco_mesh_compression"));
    assert!(!is_supported_extension("UNKNOWN_EXTENSION"));
}
