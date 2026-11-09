use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use cesium_renderer::buffer_loader::ResourceLoaderState;
use cesium_renderer::gltf_image_loader::{
    get_mime_type_from_bytes, is_ktx2_uri, GltfImageLoader, GltfImageLoaderOptions,
};
use serde_json::json;

fn decode_b64(data_uri: &str) -> Vec<u8> {
    let b64 = data_uri.split(',').nth(1).unwrap_or(data_uri);
    BASE64.decode(b64.trim()).unwrap()
}

fn create_ktx2_buffer(width: u32, height: u32, level_count: u32) -> Vec<u8> {
    let mut out = vec![0u8; 64];
    // Identifier: [0xab, 0x4b, 0x54, 0x58, 0x20, 0x32, 0x30, 0xbb, 0x0d, 0x0a, 0x1a, 0x0a]
    out[0] = 0xab;
    out[1] = 0x4b;
    out[2] = 0x54;
    out[3] = 0x58;
    out[4] = 0x20;
    out[5] = 0x32;
    out[6] = 0x30;
    out[7] = 0xbb;
    out[8] = 0x0d;
    out[9] = 0x0a;
    out[10] = 0x1a;
    out[11] = 0x0a;

    // pixelWidth at 20..24
    out[20..24].copy_from_slice(&width.to_le_bytes());
    // pixelHeight at 24..28
    out[24..28].copy_from_slice(&height.to_le_bytes());
    // levelCount at 40..44
    out[40..44].copy_from_slice(&level_count.to_le_bytes());
    out
}

#[test]
fn throws_if_required_options_are_missing() {
    let gltf = json!({
        "images": [{ "uri": "image.png" }]
    });

    assert!(GltfImageLoader::new(GltfImageLoaderOptions {
        gltf: None,
        image_id: Some(0),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfImageLoader::new(GltfImageLoaderOptions {
        gltf: Some(gltf.clone()),
        image_id: None,
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfImageLoader::new(GltfImageLoaderOptions {
        gltf: Some(gltf.clone()),
        image_id: Some(0),
        gltf_resource: None,
        base_resource: Some("https://example.com/".to_string()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfImageLoader::new(GltfImageLoaderOptions {
        gltf: Some(gltf),
        image_id: Some(0),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: None,
        ..Default::default()
    })
    .is_err());
}

#[test]
fn test_ktx2_regex_and_mime_recognition() {
    assert!(is_ktx2_uri("data:image/ktx2;base64,..."));
    assert!(is_ktx2_uri("texture.ktx2"));
    assert!(is_ktx2_uri("../../textures/test.ktx2?v=123"));
    assert!(!is_ktx2_uri("texture.png"));

    let png_header = [0x89, 0x50, 0x4e, 0x47];
    assert_eq!(get_mime_type_from_bytes(&png_header).unwrap(), "image/png");

    let jpg_header = [0xff, 0xd8, 0xff, 0xe0];
    assert_eq!(get_mime_type_from_bytes(&jpg_header).unwrap(), "image/jpeg");

    let ktx2_header = [0xab, 0x4b, 0x54, 0x58];
    assert_eq!(get_mime_type_from_bytes(&ktx2_header).unwrap(), "image/ktx2");

    let riff_webp = b"RIFF\x00\x00\x00\x00WEBP";
    assert_eq!(get_mime_type_from_bytes(riff_webp).unwrap(), "image/webp");

    let gif_header = b"GIF89a";
    assert!(get_mime_type_from_bytes(gif_header).is_err());
}

#[test]
fn load_throws_if_buffer_view_fails_to_load() {
    pollster::block_on(async {
        let gltf = json!({
            "buffers": [{ "byteLength": 10, "uri": "external.bin" }],
            "bufferViews": [{ "buffer": 0, "byteOffset": 0, "byteLength": 10 }],
            "images": [{ "bufferView": 0 }]
        });

        let loader = GltfImageLoader::new(GltfImageLoaderOptions {
            gltf: Some(gltf),
            image_id: Some(0),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            ..Default::default()
        })
        .unwrap();

        let fetch = |_url: String| async move { Err("404 Not Found".to_string()) };

        let res = loader.load(Some(fetch)).await;
        assert!(res.is_err());
        let err = res.unwrap_err().to_string();
        assert!(err.contains("Failed to load embedded image"));
        assert!(err.contains("Failed to load buffer view"));
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn load_throws_if_image_format_is_not_recognized() {
    pollster::block_on(async {
        let gif_bytes = decode_b64("R0lGODdhBAAEAIAAAP///////ywAAAAABAAEAAACBISPCQUAOw==");
        let gltf = json!({
            "buffers": [{ "byteLength": gif_bytes.len(), "uri": "external.bin" }],
            "bufferViews": [{ "buffer": 0, "byteOffset": 0, "byteLength": gif_bytes.len() }],
            "images": [{ "bufferView": 0 }]
        });

        let loader = GltfImageLoader::new(GltfImageLoaderOptions {
            gltf: Some(gltf),
            image_id: Some(0),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            ..Default::default()
        })
        .unwrap();

        let b = gif_bytes.clone();
        let fetch = move |_url: String| {
            let data = b.clone();
            async move { Ok(data) }
        };

        let res = loader.load(Some(fetch)).await;
        assert!(res.is_err());
        let err = res.unwrap_err().to_string();
        assert!(err.contains("Failed to load embedded image"));
        assert!(err.contains("Image format is not recognized"));
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn load_throws_if_uri_fails_to_load() {
    pollster::block_on(async {
        let gltf = json!({
            "images": [{ "uri": "image.png" }]
        });

        let loader = GltfImageLoader::new(GltfImageLoaderOptions {
            gltf: Some(gltf),
            image_id: Some(0),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            ..Default::default()
        })
        .unwrap();

        let fetch = |_url: String| async move { Err("404 Not Found".to_string()) };

        let res = loader.load(Some(fetch)).await;
        assert!(res.is_err());
        let err = res.unwrap_err().to_string();
        assert!(err.contains("Failed to load image: image.png"));
        assert!(err.contains("404 Not Found"));
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn loads_png_from_buffer_view() {
    pollster::block_on(async {
        let png_bytes = decode_b64("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+ip1sAAAAASUVORK5CYII=");
        let gltf = json!({
            "buffers": [{ "byteLength": png_bytes.len(), "uri": "external.bin" }],
            "bufferViews": [{ "buffer": 0, "byteOffset": 0, "byteLength": png_bytes.len() }],
            "images": [{ "bufferView": 0, "mimeType": "image/png" }]
        });

        let loader = GltfImageLoader::new(GltfImageLoaderOptions {
            gltf: Some(gltf),
            image_id: Some(0),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            ..Default::default()
        })
        .unwrap();

        let b = png_bytes.clone();
        let fetch = move |_url: String| {
            let data = b.clone();
            async move { Ok(data) }
        };

        loader.load(Some(fetch)).await.unwrap();
        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        let img = loader.image().unwrap();
        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 1);
    });
}

#[test]
fn loads_jpeg_from_buffer_view() {
    pollster::block_on(async {
        let jpg_bytes = decode_b64("/9j/4AAQSkZJRgABAQEAYABgAAD/2wBDAP//////////////////////////////////////////////////////////////////////////////////////wAALCAABAAEBAREA/8QAJgABAAAAAAAAAAAAAAAAAAAAAxABAAAAAAAAAAAAAAAAAAAAAP/aAAgBAQAAPwBH/9k=");
        let gltf = json!({
            "buffers": [{ "byteLength": jpg_bytes.len(), "uri": "external.bin" }],
            "bufferViews": [{ "buffer": 0, "byteOffset": 0, "byteLength": jpg_bytes.len() }],
            "images": [{ "bufferView": 0, "mimeType": "image/jpeg" }]
        });

        let loader = GltfImageLoader::new(GltfImageLoaderOptions {
            gltf: Some(gltf),
            image_id: Some(0),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            ..Default::default()
        })
        .unwrap();

        let b = jpg_bytes.clone();
        let fetch = move |_url: String| {
            let data = b.clone();
            async move { Ok(data) }
        };

        loader.load(Some(fetch)).await.unwrap();
        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        let img = loader.image().unwrap();
        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 1);
    });
}

#[test]
fn loads_ktx2_basis_from_buffer_view_and_mipmap() {
    pollster::block_on(async {
        // Without mipmap
        let ktx2_bytes = create_ktx2_buffer(4, 4, 1);
        let gltf = json!({
            "buffers": [{ "byteLength": ktx2_bytes.len(), "uri": "external.bin" }],
            "bufferViews": [{ "buffer": 0, "byteOffset": 0, "byteLength": ktx2_bytes.len() }],
            "images": [{ "bufferView": 0, "mimeType": "image/ktx2" }]
        });

        let loader = GltfImageLoader::new(GltfImageLoaderOptions {
            gltf: Some(gltf),
            image_id: Some(0),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            ..Default::default()
        })
        .unwrap();

        let b = ktx2_bytes.clone();
        let fetch = move |_url: String| {
            let data = b.clone();
            async move { Ok(data) }
        };

        loader.load(Some(fetch)).await.unwrap();
        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        let img = loader.image().unwrap();
        assert!(img.is_compressed());
        assert_eq!(img.width(), 4);
        assert_eq!(img.height(), 4);
        assert!(loader.mip_levels().is_none());

        // With mipmap (levelCount = 2)
        let ktx2_mip_bytes = create_ktx2_buffer(4, 4, 2);
        let gltf_mip = json!({
            "buffers": [{ "byteLength": ktx2_mip_bytes.len(), "uri": "external.bin" }],
            "bufferViews": [{ "buffer": 0, "byteOffset": 0, "byteLength": ktx2_mip_bytes.len() }],
            "images": [{ "bufferView": 0, "mimeType": "image/ktx2" }]
        });

        let loader_mip = GltfImageLoader::new(GltfImageLoaderOptions {
            gltf: Some(gltf_mip),
            image_id: Some(0),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            ..Default::default()
        })
        .unwrap();

        let b_mip = ktx2_mip_bytes.clone();
        let fetch_mip = move |_url: String| {
            let data = b_mip.clone();
            async move { Ok(data) }
        };

        loader_mip.load(Some(fetch_mip)).await.unwrap();
        assert_eq!(loader_mip.state(), ResourceLoaderState::Ready);
        let mips = loader_mip.mip_levels().unwrap();
        assert_eq!(mips.len(), 1);
    });
}

#[test]
fn loads_from_uri_and_destroys() {
    pollster::block_on(async {
        let png_bytes = decode_b64("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+ip1sAAAAASUVORK5CYII=");
        let gltf = json!({
            "images": [
                { "uri": "image.png" },
                { "uri": "texture.ktx2" }
            ]
        });

        // 1. Load PNG from uri
        let loader_png = GltfImageLoader::new(GltfImageLoaderOptions {
            gltf: Some(gltf.clone()),
            image_id: Some(0),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/textures/".to_string()),
            ..Default::default()
        })
        .unwrap();

        let b = png_bytes.clone();
        let fetch = move |_url: String| {
            let data = b.clone();
            async move { Ok(data) }
        };

        loader_png.load(Some(fetch)).await.unwrap();
        assert_eq!(loader_png.state(), ResourceLoaderState::Ready);
        let img = loader_png.image().unwrap();
        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 1);

        assert!(!loader_png.is_destroyed());
        loader_png.destroy();
        assert!(loader_png.is_destroyed());
        assert!(loader_png.image().is_none());

        // 2. Load KTX2 from uri
        let ktx2_bytes = create_ktx2_buffer(4, 4, 1);
        let loader_ktx2 = GltfImageLoader::new(GltfImageLoaderOptions {
            gltf: Some(gltf),
            image_id: Some(1),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/textures/".to_string()),
            ..Default::default()
        })
        .unwrap();

        let b_ktx = ktx2_bytes.clone();
        let fetch_ktx = move |_url: String| {
            let data = b_ktx.clone();
            async move { Ok(data) }
        };

        loader_ktx2.load(Some(fetch_ktx)).await.unwrap();
        assert_eq!(loader_ktx2.state(), ResourceLoaderState::Ready);
        assert_eq!(loader_ktx2.image().unwrap().width(), 4);
    });
}
