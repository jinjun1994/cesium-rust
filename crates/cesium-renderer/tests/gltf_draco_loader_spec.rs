use cesium_math::ComponentDatatype;
use cesium_renderer::gltf_draco_loader::{
    get_draco_attribute_type, DracoDecodedAttribute, DracoDecodedData, DracoDecodedIndices,
    DracoDecoderMode, DracoQuantizationInfo, GltfDracoLoader, GltfDracoLoaderOptions,
};
use serde_json::json;
use std::collections::HashMap;

fn create_sample_gltf() -> serde_json::Value {
    json!({
        "buffers": [
            {
                "uri": "external.bin",
                "byteLength": 8
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": 4,
                "byteLength": 4
            }
        ],
        "accessors": [
            {
                "componentType": 5126,
                "count": 3,
                "max": [-1.0, -1.0, -1.0],
                "min": [1.0, 1.0, 1.0],
                "type": "VEC3"
            },
            {
                "componentType": 5126,
                "count": 3,
                "type": "VEC3"
            },
            {
                "componentType": 5123,
                "count": 3,
                "type": "SCALAR"
            }
        ],
        "meshes": [
            {
                "primitives": [
                    {
                        "attributes": {
                            "POSITION": 0,
                            "NORMAL": 1
                        },
                        "indices": 2,
                        "extensions": {
                            "KHR_draco_mesh_compression": {
                                "bufferView": 0,
                                "attributes": {
                                    "POSITION": 0,
                                    "NORMAL": 1
                                }
                            }
                        }
                    }
                ]
            }
        ]
    })
}

fn create_sample_decoded_data() -> DracoDecodedData {
    let mut vertex_attributes = HashMap::new();
    vertex_attributes.insert(
        "POSITION".to_string(),
        DracoDecodedAttribute {
            array: vec![0, 0, 0, 0, 255, 255, 255, 255, 0, 255],
            byte_offset: 0,
            byte_stride: 6,
            component_datatype: ComponentDatatype::UnsignedShort,
            components_per_attribute: 3,
            normalized: false,
            quantization: Some(DracoQuantizationInfo {
                oct_encoded: false,
                quantization_bits: 14,
                min_values: Some(vec![-1.0, -1.0, -1.0]),
                range: Some(2.0),
            }),
        },
    );
    vertex_attributes.insert(
        "NORMAL".to_string(),
        DracoDecodedAttribute {
            array: vec![0, 255, 128, 128, 255, 0],
            byte_offset: 0,
            byte_stride: 2,
            component_datatype: ComponentDatatype::UnsignedByte,
            components_per_attribute: 2,
            normalized: false,
            quantization: Some(DracoQuantizationInfo {
                oct_encoded: true,
                quantization_bits: 10,
                min_values: None,
                range: None,
            }),
        },
    );

    DracoDecodedData {
        indices: Some(DracoDecodedIndices {
            typed_array: vec![0, 0, 1, 0, 2, 0],
            number_of_indices: 3,
        }),
        vertex_attributes,
    }
}

#[test]
fn throws_if_required_options_are_missing() {
    let gltf = create_sample_gltf();
    let primitive = gltf["meshes"][0]["primitives"][0].clone();
    let draco = primitive["extensions"]["KHR_draco_mesh_compression"].clone();
    let gltf_resource = "https://example.com/model.glb".to_string();
    let base_resource = "https://example.com/model.glb".to_string();

    // throws if resourceCache is undefined
    let err = GltfDracoLoader::new(GltfDracoLoaderOptions {
        resource_cache: false,
        gltf: Some(gltf.clone()),
        primitive: Some(primitive.clone()),
        draco: Some(draco.clone()),
        gltf_resource: Some(gltf_resource.clone()),
        base_resource: Some(base_resource.clone()),
        ..Default::default()
    })
    .unwrap_err();
    assert!(err.message.contains("resourceCache is required"));

    // throws if gltf is undefined
    let err = GltfDracoLoader::new(GltfDracoLoaderOptions {
        resource_cache: true,
        gltf: None,
        primitive: Some(primitive.clone()),
        draco: Some(draco.clone()),
        gltf_resource: Some(gltf_resource.clone()),
        base_resource: Some(base_resource.clone()),
        ..Default::default()
    })
    .unwrap_err();
    assert!(err.message.contains("options.gltf is required"));

    // throws if primitive is undefined
    let err = GltfDracoLoader::new(GltfDracoLoaderOptions {
        resource_cache: true,
        gltf: Some(gltf.clone()),
        primitive: None,
        draco: Some(draco.clone()),
        gltf_resource: Some(gltf_resource.clone()),
        base_resource: Some(base_resource.clone()),
        ..Default::default()
    })
    .unwrap_err();
    assert!(err.message.contains("options.primitive is required"));

    // throws if draco is undefined
    let err = GltfDracoLoader::new(GltfDracoLoaderOptions {
        resource_cache: true,
        gltf: Some(gltf.clone()),
        primitive: Some(primitive.clone()),
        draco: None,
        gltf_resource: Some(gltf_resource.clone()),
        base_resource: Some(base_resource.clone()),
        ..Default::default()
    })
    .unwrap_err();
    assert!(err.message.contains("options.draco is required"));

    // throws if gltfResource is undefined
    let err = GltfDracoLoader::new(GltfDracoLoaderOptions {
        resource_cache: true,
        gltf: Some(gltf.clone()),
        primitive: Some(primitive.clone()),
        draco: Some(draco.clone()),
        gltf_resource: None,
        base_resource: Some(base_resource.clone()),
        ..Default::default()
    })
    .unwrap_err();
    assert!(err.message.contains("options.gltfResource is required"));

    // throws if baseResource is undefined
    let err = GltfDracoLoader::new(GltfDracoLoaderOptions {
        resource_cache: true,
        gltf: Some(gltf.clone()),
        primitive: Some(primitive.clone()),
        draco: Some(draco.clone()),
        gltf_resource: Some(gltf_resource.clone()),
        base_resource: None,
        ..Default::default()
    })
    .unwrap_err();
    assert!(err.message.contains("options.baseResource is required"));
}

#[test]
fn load_throws_if_buffer_view_fails_to_load() {
    pollster::block_on(async {
        let gltf = create_sample_gltf();
        let primitive = gltf["meshes"][0]["primitives"][0].clone();
        let draco = primitive["extensions"]["KHR_draco_mesh_compression"].clone();

        let loader = GltfDracoLoader::new(GltfDracoLoaderOptions {
            resource_cache: true,
            gltf: Some(gltf),
            primitive: Some(primitive),
            draco: Some(draco),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/model.glb".to_string()),
            ..Default::default()
        })
        .unwrap();

        let fetch_err = |_url: String| async move { Err("404 Not Found".to_string()) };
        let err = loader.load(Some(fetch_err)).await.unwrap_err();

        assert!(err.message.contains("Failed to load Draco"));
        assert!(err.message.contains("Failed to load buffer view"));
        assert!(err.message.contains("404 Not Found"));
    });
}

#[test]
fn process_throws_if_draco_decoding_fails() {
    pollster::block_on(async {
        let gltf = create_sample_gltf();
        let primitive = gltf["meshes"][0]["primitives"][0].clone();
        let draco = primitive["extensions"]["KHR_draco_mesh_compression"].clone();

        let loader = GltfDracoLoader::new(GltfDracoLoaderOptions {
            resource_cache: true,
            gltf: Some(gltf),
            primitive: Some(primitive),
            draco: Some(draco),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/model.glb".to_string()),
            decoder_mode: Some(DracoDecoderMode::Direct(Err(
                "Draco decode failed".to_string()
            ))),
            ..Default::default()
        })
        .unwrap();

        let buffer = vec![1u8, 3, 7, 15, 31, 63, 127, 255];
        let fetch_ok = move |_url: String| {
            let b = buffer.clone();
            async move { Ok(b) }
        };

        loader.load(Some(fetch_ok)).await.unwrap();

        // First process triggers the decode failure
        let err = loader.process().unwrap_err();
        assert!(err.message.contains("Failed to load Draco"));
        assert!(err.message.contains("Draco decode failed"));

        // Subsequent process calls do not throw error
        let res = loader.process();
        assert!(res.is_ok());
        assert!(!res.unwrap());
    });
}

#[test]
fn loads_draco() {
    pollster::block_on(async {
        let gltf = create_sample_gltf();
        let primitive = gltf["meshes"][0]["primitives"][0].clone();
        let draco = primitive["extensions"]["KHR_draco_mesh_compression"].clone();
        let decoded_expected = create_sample_decoded_data();

        let loader = GltfDracoLoader::new(GltfDracoLoaderOptions {
            resource_cache: true,
            gltf: Some(gltf),
            primitive: Some(primitive),
            draco: Some(draco),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/model.glb".to_string()),
            decoder_mode: Some(DracoDecoderMode::Stepped {
                calls_remaining: 3,
                result: Ok(decoded_expected.clone()),
            }),
            ..Default::default()
        })
        .unwrap();

        let buffer = vec![1u8, 3, 7, 15, 31, 63, 127, 255];
        let fetch_ok = move |_url: String| {
            let b = buffer.clone();
            async move { Ok(b) }
        };

        loader.load(Some(fetch_ok)).await.unwrap();

        // Frame 1, 2, 3 return false (simulating Web Worker decode in progress)
        assert!(!loader.process().unwrap());
        assert!(!loader.process().unwrap());
        assert!(!loader.process().unwrap());

        // Frame 4 completes and returns true
        assert!(loader.process().unwrap());

        // Decoded data matches
        let data = loader.decoded_data().expect("decodedData defined");
        assert_eq!(data, decoded_expected);
    });
}

#[test]
fn destroys_draco_loader() {
    pollster::block_on(async {
        let gltf = create_sample_gltf();
        let primitive = gltf["meshes"][0]["primitives"][0].clone();
        let draco = primitive["extensions"]["KHR_draco_mesh_compression"].clone();
        let decoded_expected = create_sample_decoded_data();

        let loader = GltfDracoLoader::new(GltfDracoLoaderOptions {
            resource_cache: true,
            gltf: Some(gltf),
            primitive: Some(primitive),
            draco: Some(draco),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/model.glb".to_string()),
            decoder_mode: Some(DracoDecoderMode::Direct(Ok(decoded_expected))),
            ..Default::default()
        })
        .unwrap();

        let buffer = vec![1u8, 3, 7, 15, 31, 63, 127, 255];
        let fetch_ok = move |_url: String| {
            let b = buffer.clone();
            async move { Ok(b) }
        };

        loader.load(Some(fetch_ok)).await.unwrap();
        assert!(loader.process().unwrap());

        assert!(loader.decoded_data().is_some());
        assert!(!loader.is_destroyed());

        loader.destroy();

        assert!(loader.decoded_data().is_none());
        assert!(loader.is_destroyed());
    });
}

#[test]
fn handles_resolving_buffer_view_after_destroy() {
    pollster::block_on(async {
        let gltf = create_sample_gltf();
        let primitive = gltf["meshes"][0]["primitives"][0].clone();
        let draco = primitive["extensions"]["KHR_draco_mesh_compression"].clone();

        let loader = GltfDracoLoader::new(GltfDracoLoaderOptions {
            resource_cache: true,
            gltf: Some(gltf),
            primitive: Some(primitive),
            draco: Some(draco),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/model.glb".to_string()),
            ..Default::default()
        })
        .unwrap();

        let loader_clone = loader.clone();
        let fetch_and_destroy = move |_url: String| {
            let l = loader_clone.clone();
            async move {
                l.destroy();
                Ok(vec![1, 2, 3, 4, 5, 6, 7, 8])
            }
        };

        let res = loader.load(Some(fetch_and_destroy)).await;
        assert!(res.is_ok());
        assert!(loader.decoded_data().is_none());
        assert!(loader.is_destroyed());
    });
}

#[test]
fn handles_rejecting_buffer_view_after_destroy() {
    pollster::block_on(async {
        let gltf = create_sample_gltf();
        let primitive = gltf["meshes"][0]["primitives"][0].clone();
        let draco = primitive["extensions"]["KHR_draco_mesh_compression"].clone();

        let loader = GltfDracoLoader::new(GltfDracoLoaderOptions {
            resource_cache: true,
            gltf: Some(gltf),
            primitive: Some(primitive),
            draco: Some(draco),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/model.glb".to_string()),
            ..Default::default()
        })
        .unwrap();

        let loader_clone = loader.clone();
        let fetch_fail_and_destroy = move |_url: String| {
            let l = loader_clone.clone();
            async move {
                l.destroy();
                Err("Fetch Failed".to_string())
            }
        };

        let res = loader.load(Some(fetch_fail_and_destroy)).await;
        assert!(res.is_ok());
        assert!(loader.decoded_data().is_none());
        assert!(loader.is_destroyed());
    });
}

#[test]
fn handles_resolving_draco_after_destroy() {
    pollster::block_on(async {
        let gltf = create_sample_gltf();
        let primitive = gltf["meshes"][0]["primitives"][0].clone();
        let draco = primitive["extensions"]["KHR_draco_mesh_compression"].clone();

        let loader = GltfDracoLoader::new(GltfDracoLoaderOptions {
            resource_cache: true,
            gltf: Some(gltf),
            primitive: Some(primitive),
            draco: Some(draco),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/model.glb".to_string()),
            decoder_mode: Some(DracoDecoderMode::Direct(Ok(create_sample_decoded_data()))),
            ..Default::default()
        })
        .unwrap();

        let buffer = vec![1u8, 3, 7, 15, 31, 63, 127, 255];
        loader
            .load(Some(move |_url| {
                let b = buffer.clone();
                async move { Ok(b) }
            }))
            .await
            .unwrap();

        // Destroy before processing/decode completion
        loader.destroy();

        assert!(!loader.process().unwrap());
        assert!(loader.decoded_data().is_none());
        assert!(loader.is_destroyed());
    });
}

#[test]
fn handles_rejecting_draco_after_destroy() {
    pollster::block_on(async {
        let gltf = create_sample_gltf();
        let primitive = gltf["meshes"][0]["primitives"][0].clone();
        let draco = primitive["extensions"]["KHR_draco_mesh_compression"].clone();

        let loader = GltfDracoLoader::new(GltfDracoLoaderOptions {
            resource_cache: true,
            gltf: Some(gltf),
            primitive: Some(primitive),
            draco: Some(draco),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/model.glb".to_string()),
            decoder_mode: Some(DracoDecoderMode::Direct(Err(
                "Draco decode failed".to_string()
            ))),
            ..Default::default()
        })
        .unwrap();

        let buffer = vec![1u8, 3, 7, 15, 31, 63, 127, 255];
        loader
            .load(Some(move |_url| {
                let b = buffer.clone();
                async move { Ok(b) }
            }))
            .await
            .unwrap();

        // Destroy before process
        loader.destroy();

        // Process shouldn't throw error since loader is already destroyed
        let res = loader.process();
        assert!(res.is_ok());
        assert!(!res.unwrap());
        assert!(loader.decoded_data().is_none());
        assert!(loader.is_destroyed());
    });
}

#[test]
fn tests_attribute_type_semantic_mapping() {
    assert_eq!(get_draco_attribute_type("POSITION"), Some("POSITION"));
    assert_eq!(get_draco_attribute_type("POSITION_0"), Some("POSITION"));
    assert_eq!(get_draco_attribute_type("NORMAL"), Some("NORMAL"));
    assert_eq!(get_draco_attribute_type("NORMAL_1"), Some("NORMAL"));
    assert_eq!(get_draco_attribute_type("COLOR_0"), Some("COLOR"));
    assert_eq!(get_draco_attribute_type("TEXCOORD_0"), Some("TEX_COORD"));
    assert_eq!(get_draco_attribute_type("CUSTOM_SEMANTIC"), None);
}
