use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use cesium_renderer::buffer_loader::ResourceLoaderState;
use cesium_renderer::gltf_texture_loader::{
    GltfTextureLoader, GltfTextureLoaderOptions, SupportedImageFormats,
};
use cesium_renderer::sampler::TextureMinificationFilter;
use serde_json::json;

fn decode_b64(data_uri: &str) -> Vec<u8> {
    let b64 = data_uri.split(',').nth(1).unwrap_or(data_uri);
    BASE64.decode(b64.trim()).unwrap()
}

fn create_ktx2_buffer(width: u32, height: u32, level_count: u32) -> Vec<u8> {
    let mut out = vec![0u8; 64];
    out[0..12].copy_from_slice(&[
        0xab, 0x4b, 0x54, 0x58, 0x20, 0x32, 0x30, 0xbb, 0x0d, 0x0a, 0x1a, 0x0a,
    ]);
    out[20..24].copy_from_slice(&width.to_le_bytes());
    out[24..28].copy_from_slice(&height.to_le_bytes());
    out[40..44].copy_from_slice(&level_count.to_le_bytes());
    out
}

fn make_test_gltf() -> serde_json::Value {
    json!({
        "images": [
            { "uri": "image.png" },
            { "uri": "Green4x4_ETC1S.ktx2" },
            { "uri": "Green4x4Mipmap_ETC1S.ktx2" }
        ],
        "textures": [
            { "source": 0 },
            { "source": 0, "sampler": 0 },
            { "source": 0, "sampler": 1 },
            {
                "source": 0,
                "sampler": 0,
                "extensions": {
                    "KHR_texture_basisu": { "source": 1 }
                }
            },
            {
                "source": 0,
                "sampler": 1,
                "extensions": {
                    "KHR_texture_basisu": { "source": 2 }
                }
            }
        ],
        "samplers": [
            {
                "magFilter": 9728, // NEAREST
                "minFilter": 9984, // NEAREST_MIPMAP_NEAREST
                "wrapS": 10497,    // REPEAT
                "wrapT": 10497     // REPEAT
            },
            {
                "magFilter": 9728, // NEAREST
                "minFilter": 9728, // NEAREST
                "wrapS": 33071,    // CLAMP_TO_EDGE
                "wrapT": 33071     // CLAMP_TO_EDGE
            }
        ],
        "materials": [
            {
                "emissiveTexture": { "index": 0 },
                "occlusionTexture": { "index": 1 },
                "normalTexture": { "index": 2 }
            }
        ]
    })
}

#[test]
fn throws_if_required_options_are_missing() {
    let gltf = make_test_gltf();
    let tex_info = json!({ "index": 0 });

    assert!(GltfTextureLoader::new(GltfTextureLoaderOptions {
        gltf: None,
        texture_info: Some(tex_info.clone()),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        supported_image_formats: Some(SupportedImageFormats::default()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfTextureLoader::new(GltfTextureLoaderOptions {
        gltf: Some(gltf.clone()),
        texture_info: None,
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        supported_image_formats: Some(SupportedImageFormats::default()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfTextureLoader::new(GltfTextureLoaderOptions {
        gltf: Some(gltf.clone()),
        texture_info: Some(tex_info.clone()),
        gltf_resource: None,
        base_resource: Some("https://example.com/".to_string()),
        supported_image_formats: Some(SupportedImageFormats::default()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfTextureLoader::new(GltfTextureLoaderOptions {
        gltf: Some(gltf.clone()),
        texture_info: Some(tex_info.clone()),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: None,
        supported_image_formats: Some(SupportedImageFormats::default()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfTextureLoader::new(GltfTextureLoaderOptions {
        gltf: Some(gltf),
        texture_info: Some(tex_info),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        supported_image_formats: None,
        ..Default::default()
    })
    .is_err());
}

#[test]
fn load_throws_if_image_fails_to_load() {
    pollster::block_on(async {
        let gltf = make_test_gltf();
        let loader = GltfTextureLoader::new(GltfTextureLoaderOptions {
            gltf: Some(gltf.clone()),
            texture_info: Some(gltf["materials"][0]["emissiveTexture"].clone()),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            supported_image_formats: Some(SupportedImageFormats::default()),
            ..Default::default()
        })
        .unwrap();

        let fetch = |_url: String| async move { Err("404 Not Found".to_string()) };

        let res = loader.load(Some(fetch)).await;
        assert!(res.is_err());
        let err = res.unwrap_err().to_string();
        assert!(err.contains("Failed to load texture"));
        assert!(err.contains("Failed to load image: image.png"));
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn loads_texture_and_creates_gpu_texture() {
    pollster::block_on(async {
        let png_bytes = decode_b64("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+ip1sAAAAASUVORK5CYII=");
        let gltf = make_test_gltf();

        let loader = GltfTextureLoader::new(GltfTextureLoaderOptions {
            gltf: Some(gltf.clone()),
            texture_info: Some(gltf["materials"][0]["emissiveTexture"].clone()),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            supported_image_formats: Some(SupportedImageFormats::default()),
            asynchronous: Some(false),
            ..Default::default()
        })
        .unwrap();

        let b = png_bytes.clone();
        let fetch = move |_url: String| {
            let data = b.clone();
            async move { Ok(data) }
        };

        loader.load(Some(fetch)).await.unwrap();
        assert_eq!(loader.state(), ResourceLoaderState::Processing);

        let processed = loader.process();
        assert!(processed);
        assert_eq!(loader.state(), ResourceLoaderState::Ready);

        let texture = loader.texture().unwrap();
        assert_eq!(texture.width, 1);
        assert_eq!(texture.height, 1);
        assert_eq!(texture.size_in_bytes(), 4);
    });
}

#[test]
fn loads_ktx2_texture_and_handles_incompatible_mipmap_sampler() {
    pollster::block_on(async {
        // Texture index 3: uses sampler 0 (minFilter: NEAREST_MIPMAP_NEAREST), but KTX2 has levelCount = 1 (no mipmap)
        let ktx2_bytes = create_ktx2_buffer(4, 4, 1);
        let gltf = make_test_gltf();

        let loader = GltfTextureLoader::new(GltfTextureLoaderOptions {
            gltf: Some(gltf),
            texture_info: Some(json!({ "index": 3 })),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            supported_image_formats: Some(SupportedImageFormats::with_flags(true, false)),
            ..Default::default()
        })
        .unwrap();

        let b = ktx2_bytes.clone();
        let fetch = move |_url: String| {
            let data = b.clone();
            async move { Ok(data) }
        };

        loader.load(Some(fetch)).await.unwrap();
        loader.process();

        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        let texture = loader.texture().unwrap();
        assert_eq!(texture.width, 4);
        assert_eq!(texture.height, 4);
        // Sampler was downgraded from NEAREST_MIPMAP_NEAREST to NEAREST because there's no mipmap
        assert_eq!(
            texture.sampler.minification_filter,
            TextureMinificationFilter::Nearest
        );
    });
}

#[test]
fn handles_npot_texture_resizing() {
    pollster::block_on(async {
        // 3x2 PNG image (non-power-of-two)
        let npot_png = decode_b64("iVBORw0KGgoAAAANSUhEUgAAAAMAAAACAQMAAACnuvRZAAAAA3NCSVQICAjb4U/gAAAABlBMVEUAAAD///+l2Z/dAAAAAnRSTlP/AOW3MEoAAAAJcEhZcwAACxIAAAsSAdLdfvwAAAAcdEVYdFNvZnR3YXJlAEFkb2JlIEZpcmV3b3JrcyBDUzQGstOgAAAAFnRFWHRDcmVhdGlvbiBUaW1lADAxLzA0LzE0Kb6O2wAAAAxJREFUCJljeMDwAAADhAHBgGgjpQAAAABJRU5ErkJggg==");
        let gltf = make_test_gltf();

        // 1. Occlusion texture uses sampler 0 (REPEAT + MIPMAP) -> requires POT resize (3x2 -> 4x2)
        let loader_pot = GltfTextureLoader::new(GltfTextureLoaderOptions {
            gltf: Some(gltf.clone()),
            texture_info: Some(gltf["materials"][0]["occlusionTexture"].clone()),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            supported_image_formats: Some(SupportedImageFormats::default()),
            ..Default::default()
        })
        .unwrap();

        let b1 = npot_png.clone();
        loader_pot
            .load(Some(move |_| {
                let data = b1.clone();
                async move { Ok(data) }
            }))
            .await
            .unwrap();
        loader_pot.process();
        let tex_pot = loader_pot.texture().unwrap();
        assert_eq!(tex_pot.width, 4);
        assert_eq!(tex_pot.height, 2);

        // 2. Normal texture uses sampler 1 (CLAMP_TO_EDGE, NEAREST, no mipmap) -> does not resize (3x2 -> 3x2)
        let loader_npot = GltfTextureLoader::new(GltfTextureLoaderOptions {
            gltf: Some(gltf.clone()),
            texture_info: Some(gltf["materials"][0]["normalTexture"].clone()),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            supported_image_formats: Some(SupportedImageFormats::default()),
            ..Default::default()
        })
        .unwrap();

        let b2 = npot_png.clone();
        loader_npot
            .load(Some(move |_| {
                let data = b2.clone();
                async move { Ok(data) }
            }))
            .await
            .unwrap();
        loader_npot.process();
        let tex_npot = loader_npot.texture().unwrap();
        assert_eq!(tex_npot.width, 3);
        assert_eq!(tex_npot.height, 2);
    });
}

#[test]
fn destroys_texture_loader() {
    pollster::block_on(async {
        let png_bytes = decode_b64("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+ip1sAAAAASUVORK5CYII=");
        let gltf = make_test_gltf();

        let loader = GltfTextureLoader::new(GltfTextureLoaderOptions {
            gltf: Some(gltf.clone()),
            texture_info: Some(gltf["materials"][0]["emissiveTexture"].clone()),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            supported_image_formats: Some(SupportedImageFormats::default()),
            ..Default::default()
        })
        .unwrap();

        let b = png_bytes.clone();
        let fetch = move |_url: String| {
            let data = b.clone();
            async move { Ok(data) }
        };

        loader.load(Some(fetch)).await.unwrap();
        loader.process();

        assert!(loader.texture().is_some());
        assert!(!loader.is_destroyed());

        loader.destroy();
        assert!(loader.texture().is_none());
        assert!(loader.is_destroyed());
    });
}
