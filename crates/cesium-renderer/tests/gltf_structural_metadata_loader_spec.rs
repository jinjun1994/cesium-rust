use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use cesium_renderer::buffer_loader::ResourceLoaderState;
use cesium_renderer::gltf_structural_metadata_loader::{
    GltfStructuralMetadataLoader, GltfStructuralMetadataLoaderOptions,
};
use cesium_renderer::gltf_texture_loader::SupportedImageFormats;
use serde_json::json;

fn decode_b64(data_uri: &str) -> Vec<u8> {
    let b64 = data_uri.split(',').nth(1).unwrap_or(data_uri);
    BASE64.decode(b64.trim()).unwrap()
}

fn make_test_gltf_and_extension() -> (serde_json::Value, serde_json::Value) {
    let gltf = json!({
        "buffers": [
            { "byteLength": 100, "uri": "external.bin" }
        ],
        "bufferViews": [
            { "buffer": 0, "byteOffset": 0, "byteLength": 20 },
            { "buffer": 0, "byteOffset": 20, "byteLength": 20 }
        ],
        "images": [
            { "uri": "map.png" },
            { "uri": "ortho.png" }
        ],
        "textures": [
            { "source": 0 },
            { "source": 1 }
        ]
    });

    let schema = json!({
        "classes": {
            "building": {
                "properties": {
                    "name": { "type": "STRING" },
                    "height": { "type": "SCALAR", "componentType": "FLOAT64" }
                }
            },
            "tree": {
                "properties": {
                    "species": { "type": "STRING", "array": true }
                }
            },
            "map": {
                "properties": {
                    "color": { "type": "SCALAR", "componentType": "UINT8", "array": true, "count": 3 },
                    "intensity": { "type": "SCALAR", "componentType": "UINT8" }
                }
            },
            "ortho": {
                "properties": {
                    "vegetation": { "type": "SCALAR", "componentType": "UINT8", "normalized": true }
                }
            }
        }
    });

    let extension = json!({
        "schema": schema,
        "propertyTables": [
            {
                "name": "Buildings",
                "class": "building",
                "count": 2,
                "properties": {
                    "name": { "values": [ "House", "Hospital" ] },
                    "height": { "values": [ 10.0, 20.0 ] }
                }
            },
            {
                "name": "Trees",
                "class": "tree",
                "count": 2,
                "properties": {
                    "species": { "values": [ ["Sparrow", "Squirrel"], ["Crow"] ] }
                }
            }
        ],
        "propertyTextures": [
            {
                "name": "Map",
                "class": "map",
                "properties": {
                    "color": {
                        "channels": [0, 1, 2],
                        "index": 0,
                        "texCoord": 0
                    },
                    "intensity": {
                        "channels": [3],
                        "index": 0,
                        "texCoord": 0
                    }
                }
            },
            {
                "name": "Ortho",
                "class": "ortho",
                "properties": {
                    "vegetation": {
                        "channels": [0],
                        "index": 1,
                        "texCoord": 1
                    }
                }
            }
        ]
    });

    (gltf, extension)
}

#[test]
fn throws_if_required_options_are_missing() {
    let (gltf, extension) = make_test_gltf_and_extension();
    let frame_state = json!({ "context": { "id": "01234" } });

    assert!(GltfStructuralMetadataLoader::new(GltfStructuralMetadataLoaderOptions {
        gltf: None,
        extension: Some(extension.clone()),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        supported_image_formats: Some(SupportedImageFormats::default()),
        frame_state: Some(frame_state.clone()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfStructuralMetadataLoader::new(GltfStructuralMetadataLoaderOptions {
        gltf: Some(gltf.clone()),
        extension: None,
        extension_legacy: None,
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        supported_image_formats: Some(SupportedImageFormats::default()),
        frame_state: Some(frame_state.clone()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfStructuralMetadataLoader::new(GltfStructuralMetadataLoaderOptions {
        gltf: Some(gltf.clone()),
        extension: Some(extension.clone()),
        gltf_resource: None,
        base_resource: Some("https://example.com/".to_string()),
        supported_image_formats: Some(SupportedImageFormats::default()),
        frame_state: Some(frame_state.clone()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfStructuralMetadataLoader::new(GltfStructuralMetadataLoaderOptions {
        gltf: Some(gltf.clone()),
        extension: Some(extension.clone()),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: None,
        supported_image_formats: Some(SupportedImageFormats::default()),
        frame_state: Some(frame_state.clone()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfStructuralMetadataLoader::new(GltfStructuralMetadataLoaderOptions {
        gltf: Some(gltf.clone()),
        extension: Some(extension.clone()),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        supported_image_formats: None,
        frame_state: Some(frame_state.clone()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfStructuralMetadataLoader::new(GltfStructuralMetadataLoaderOptions {
        gltf: Some(gltf),
        extension: Some(extension),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        supported_image_formats: Some(SupportedImageFormats::default()),
        frame_state: None,
        ..Default::default()
    })
    .is_err());
}

#[test]
fn load_throws_if_texture_fails_to_load() {
    pollster::block_on(async {
        let (gltf, extension) = make_test_gltf_and_extension();
        let loader = GltfStructuralMetadataLoader::new(GltfStructuralMetadataLoaderOptions {
            gltf: Some(gltf),
            extension: Some(extension),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            supported_image_formats: Some(SupportedImageFormats::default()),
            frame_state: Some(json!({})),
            ..Default::default()
        })
        .unwrap();

        let png_bytes = decode_b64("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+ip1sAAAAASUVORK5CYII=");

        let fetch = move |url: String| {
            let p = png_bytes.clone();
            async move {
                if url.contains("map.png") {
                    Err("404 Not Found".to_string())
                } else if url.contains(".png") {
                    Ok(p)
                } else {
                    Ok(vec![0u8; 10])
                }
            }
        };

        let res = loader.load(Some(fetch)).await;
        assert!(res.is_err());
        let err = res.unwrap_err().to_string();
        assert!(err.contains("Failed to load structural metadata"));
        assert!(err.contains("Failed to load texture"));
        assert!(err.contains("Failed to load image: map.png"));
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn load_throws_if_external_schema_fails_to_load() {
    pollster::block_on(async {
        let (gltf, mut extension) = make_test_gltf_and_extension();
        extension.as_object_mut().unwrap().remove("schema");
        extension["schemaUri"] = json!("https://example.com/schema.json");

        let loader = GltfStructuralMetadataLoader::new(GltfStructuralMetadataLoaderOptions {
            gltf: Some(gltf),
            extension: Some(extension),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            supported_image_formats: Some(SupportedImageFormats::default()),
            frame_state: Some(json!({})),
            ..Default::default()
        })
        .unwrap();

        let png_bytes = decode_b64("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+ip1sAAAAASUVORK5CYII=");

        let fetch = move |url: String| {
            let p = png_bytes.clone();
            async move {
                if url.contains("schema.json") {
                    Err("404 Not Found".to_string())
                } else if url.contains(".png") {
                    Ok(p)
                } else {
                    Ok(vec![0u8; 20])
                }
            }
        };

        let res = loader.load(Some(fetch)).await;
        assert!(res.is_err());
        let err = res.unwrap_err().to_string();
        assert!(err.contains("Failed to load structural metadata"));
        assert!(err.contains("Failed to load schema: https://example.com/schema.json"));
        assert_eq!(loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn loads_structural_metadata_full() {
    pollster::block_on(async {
        let (gltf, extension) = make_test_gltf_and_extension();
        let loader = GltfStructuralMetadataLoader::new(GltfStructuralMetadataLoaderOptions {
            gltf: Some(gltf),
            extension: Some(extension),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            supported_image_formats: Some(SupportedImageFormats::default()),
            frame_state: Some(json!({})),
            ..Default::default()
        })
        .unwrap();

        let png_bytes = decode_b64("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+ip1sAAAAASUVORK5CYII=");

        let fetch = move |url: String| {
            let p = png_bytes.clone();
            async move {
                if url.contains(".png") {
                    Ok(p)
                } else {
                    Ok(vec![0u8; 40])
                }
            }
        };

        loader.load(Some(fetch)).await.unwrap();
        assert_eq!(loader.state(), ResourceLoaderState::Loaded);

        let process_ok = loader.process();
        assert!(process_ok);
        assert_eq!(loader.state(), ResourceLoaderState::Ready);

        let sm = loader.structural_metadata().unwrap();

        // 1. Property tables
        let buildings = sm.get_property_table(0).unwrap();
        assert_eq!(buildings.id, 0);
        assert_eq!(buildings.get_property(0, "name"), Some(json!("House")));
        assert_eq!(buildings.get_property(1, "name"), Some(json!("Hospital")));

        let trees = sm.get_property_table(1).unwrap();
        assert_eq!(trees.id, 1);
        assert_eq!(
            trees.get_property(0, "species"),
            Some(json!(["Sparrow", "Squirrel"]))
        );
        assert_eq!(trees.get_property(1, "species"), Some(json!(["Crow"])));

        // 2. Property textures
        let map_tex = sm.get_property_texture(0).unwrap();
        assert_eq!(map_tex.id, 0);
        let ortho_tex = sm.get_property_texture(1).unwrap();
        assert_eq!(ortho_tex.id, 1);

        let color_prop = map_tex.get_property("color").unwrap();
        let intensity_prop = map_tex.get_property("intensity").unwrap();
        let vegetation_prop = ortho_tex.get_property("vegetation").unwrap();

        let color_tex = color_prop.texture_reader.texture.as_ref().unwrap();
        let intensity_tex = intensity_prop.texture_reader.texture.as_ref().unwrap();
        let veg_tex = vegetation_prop.texture_reader.texture.as_ref().unwrap();

        assert_eq!(color_tex.width, 1);
        assert_eq!(color_tex.height, 1);
        assert_eq!(color_tex.id, intensity_tex.id); // shared texture 0
        assert_ne!(color_tex.id, veg_tex.id); // different texture 1

        // 3. Schema classes
        let classes: Vec<String> = sm.schema.classes.keys().cloned().collect();
        assert_eq!(classes, vec!["building", "map", "ortho", "tree"]);

        // 4. Destroy
        assert!(!loader.is_destroyed());
        loader.destroy();
        assert!(loader.is_destroyed());
        assert!(loader.structural_metadata().is_none());
    });
}
