use cesium_math::{
    AttributeType, ComponentDatatype, IndexDatatype, Matrix4, PrimitiveType,
};
use cesium_renderer::axis::Axis;
use cesium_renderer::buffer_loader::ResourceLoaderState;
use cesium_renderer::gltf_loader::{GltfLoader, GltfLoaderOptions};
use cesium_renderer::model_components::AlphaMode;
use serde_json::json;

type DummyFetchFuture = std::future::Ready<Result<Vec<u8>, String>>;
type DummyFetchFn = fn(String) -> DummyFetchFuture;

fn create_sample_box_gltf() -> serde_json::Value {
    json!({
        "asset": {
            "version": "2.0",
            "generator": "Cesium glTF Loader Spec Generator",
            "copyright": "Cesium Open Source"
        },
        "scene": 0,
        "scenes": [
            {
                "nodes": [0]
            }
        ],
        "nodes": [
            {
                "name": "BoxNode",
                "mesh": 0,
                "matrix": [
                    1.0, 0.0, 0.0, 0.0,
                    0.0, 1.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    10.0, 20.0, 30.0, 1.0
                ]
            }
        ],
        "meshes": [
            {
                "primitives": [
                    {
                        "attributes": {
                            "POSITION": 0,
                            "NORMAL": 1,
                            "TEXCOORD_0": 2
                        },
                        "indices": 3,
                        "material": 0,
                        "mode": 4
                    }
                ]
            }
        ],
        "accessors": [
            {
                "componentType": 5126,
                "count": 24,
                "type": "VEC3",
                "max": [0.5, 0.5, 0.5],
                "min": [-0.5, -0.5, -0.5]
            },
            {
                "componentType": 5126,
                "count": 24,
                "type": "VEC3"
            },
            {
                "componentType": 5126,
                "count": 24,
                "type": "VEC2"
            },
            {
                "componentType": 5123,
                "count": 36,
                "type": "SCALAR"
            }
        ],
        "materials": [
            {
                "pbrMetallicRoughness": {
                    "baseColorFactor": [0.8, 0.8, 0.8, 1.0],
                    "metallicFactor": 0.5,
                    "roughnessFactor": 0.1
                },
                "doubleSided": true,
                "alphaMode": "OPAQUE"
            }
        ]
    })
}

#[test]
fn throws_if_gltf_resource_is_undefined() {
    let err = GltfLoader::new(GltfLoaderOptions {
        gltf_resource: None,
        ..Default::default()
    })
    .unwrap_err();

    assert!(err.message.contains("options.gltfResource is required"));
}

#[test]
fn load_throws_if_gltf_json_fails_to_load() {
    pollster::block_on(async {
        let gltf_loader = GltfLoader::new(GltfLoaderOptions {
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            release_gltf_json: true,
            ..Default::default()
        })
        .unwrap();

        let fetch_err = |_url: String| async move { Err("404 Not Found".to_string()) };
        let err = gltf_loader.load(Some(fetch_err)).await.unwrap_err();

        assert!(err.message.contains("Failed to load glTF"));
        assert!(err.message.contains("https://example.com/model.glb"));
        assert!(err.message.contains("404 Not Found"));
        assert_eq!(gltf_loader.state(), ResourceLoaderState::Failed);
    });
}

#[test]
fn loads_box_and_verifies_components() {
    pollster::block_on(async {
        let box_json = create_sample_box_gltf();
        let gltf_loader = GltfLoader::new(GltfLoaderOptions {
            gltf_resource: Some("https://example.com/box.gltf".to_string()),
            gltf: Some(box_json.clone()),
            up_axis: Some(Axis::Y),
            forward_axis: Some(Axis::Z),
            asynchronous: Some(false),
            incrementally_load_textures: Some(false),
            load_attributes_as_typed_array: true,
            load_indices_as_typed_array: true,
            ..Default::default()
        })
        .unwrap();

        assert!(!gltf_loader.asynchronous());
        assert!(!gltf_loader.incrementally_load_textures());

        let no_fetch: Option<DummyFetchFn> = None;
        gltf_loader.load(no_fetch).await.unwrap();
        assert_eq!(gltf_loader.state(), ResourceLoaderState::Processing);

        let process_ready = gltf_loader.process().unwrap();
        assert!(process_ready);
        assert_eq!(gltf_loader.state(), ResourceLoaderState::Ready);

        let components = gltf_loader.components().expect("components should exist");

        // Verify Asset
        assert_eq!(components.asset.version, "2.0");
        assert_eq!(
            components.asset.generator.as_deref(),
            Some("Cesium glTF Loader Spec Generator")
        );
        assert_eq!(
            components.asset.copyright.as_deref(),
            Some("Cesium Open Source")
        );
        assert_eq!(components.asset.credits, vec!["Cesium Open Source"]);

        // Verify Axis & Transform
        assert_eq!(components.up_axis, Some(Axis::Y));
        assert_eq!(components.forward_axis, Some(Axis::Z));
        assert_eq!(components.transform, Matrix4::IDENTITY);

        // Verify Scene and Nodes
        let scene = components.scene.expect("scene should exist");
        assert_eq!(scene.nodes, vec![0]);
        assert_eq!(components.nodes.len(), 1);

        let node = &components.nodes[0];
        assert_eq!(node.name.as_deref(), Some("BoxNode"));
        assert!(node.matrix.is_some());

        // Verify Primitives
        assert_eq!(node.primitives.len(), 1);
        let prim = &node.primitives[0];
        assert_eq!(prim.primitive_type, Some(PrimitiveType::Triangles));
        assert_eq!(prim.attributes.len(), 3);

        // Check Position attribute
        let pos = prim
            .attributes
            .iter()
            .find(|a| a.semantic.as_deref() == Some("POSITION"))
            .unwrap();
        assert_eq!(pos.component_datatype, Some(ComponentDatatype::Float));
        assert_eq!(pos.attribute_type, Some(AttributeType::Vec3));
        assert_eq!(pos.count, 24);
        assert!(pos.typed_array.is_some());

        // Check Indices
        let indices = prim.indices.as_ref().unwrap();
        assert_eq!(
            indices.index_datatype,
            Some(IndexDatatype::UnsignedShort)
        );
        assert_eq!(indices.count, 36);
        assert!(indices.typed_array.is_some());

        // Check Material
        let mat = prim.material.as_ref().unwrap();
        assert_eq!(mat.alpha_mode, AlphaMode::Opaque);
        assert!(mat.double_sided);

        let mr = mat.metallic_roughness.as_ref().unwrap();
        assert_eq!(mr.metallic_factor, 0.5);
        assert_eq!(mr.roughness_factor, 0.1);
        assert_eq!(mr.base_color_factor.x, 0.8);
        assert_eq!(mr.base_color_factor.y, 0.8);
        assert_eq!(mr.base_color_factor.z, 0.8);
        assert_eq!(mr.base_color_factor.w, 1.0);
    });
}

#[test]
fn releases_gltf_json_after_parse_when_configured() {
    pollster::block_on(async {
        let box_json = create_sample_box_gltf();
        let gltf_loader = GltfLoader::new(GltfLoaderOptions {
            gltf_resource: Some("https://example.com/box.gltf".to_string()),
            gltf: Some(box_json),
            release_gltf_json: true,
            ..Default::default()
        })
        .unwrap();

        let no_fetch: Option<DummyFetchFn> = None;
        gltf_loader.load(no_fetch).await.unwrap();
        assert!(gltf_loader.gltf().is_some());

        gltf_loader.process().unwrap();
        assert_eq!(gltf_loader.state(), ResourceLoaderState::Ready);

        // gltf JSON released after parse
        assert!(gltf_loader.gltf().is_none());
    });
}

#[test]
fn sets_custom_transform() {
    pollster::block_on(async {
        let box_json = create_sample_box_gltf();
        let custom_matrix = Matrix4::from_translation(cesium_math::Cartesian3::new(1.0, 2.0, 3.0));

        let gltf_loader = GltfLoader::new(GltfLoaderOptions {
            gltf_resource: Some("https://example.com/box.gltf".to_string()),
            gltf: Some(box_json),
            custom_transform: Some(custom_matrix),
            ..Default::default()
        })
        .unwrap();

        let no_fetch: Option<DummyFetchFn> = None;
        gltf_loader.load(no_fetch).await.unwrap();
        gltf_loader.process().unwrap();

        let components = gltf_loader.components().unwrap();
        assert_eq!(components.transform, custom_matrix);
    });
}

#[test]
fn destroys_gltf_loader() {
    pollster::block_on(async {
        let box_json = create_sample_box_gltf();
        let gltf_loader = GltfLoader::new(GltfLoaderOptions {
            gltf_resource: Some("https://example.com/box.gltf".to_string()),
            gltf: Some(box_json),
            ..Default::default()
        })
        .unwrap();

        let no_fetch: Option<DummyFetchFn> = None;
        gltf_loader.load(no_fetch).await.unwrap();
        gltf_loader.process().unwrap();

        assert!(!gltf_loader.is_destroyed());
        assert!(gltf_loader.components().is_some());

        gltf_loader.destroy();

        assert!(gltf_loader.is_destroyed());
        assert!(gltf_loader.components().is_none());
        assert!(gltf_loader.gltf().is_none());
    });
}

#[test]
fn handles_resolving_gltf_json_after_destroy() {
    pollster::block_on(async {
        let gltf_loader = GltfLoader::new(GltfLoaderOptions {
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            ..Default::default()
        })
        .unwrap();

        let loader_clone = gltf_loader.clone();
        let fetch_and_destroy = move |_url: String| {
            let l = loader_clone.clone();
            async move {
                l.destroy();
                let json_bytes = serde_json::to_vec(&create_sample_box_gltf()).unwrap();
                Ok(json_bytes)
            }
        };

        let res = gltf_loader.load(Some(fetch_and_destroy)).await;
        assert!(res.is_ok());
        assert!(gltf_loader.is_destroyed());
        assert!(gltf_loader.components().is_none());
    });
}

#[test]
fn handles_rejecting_gltf_json_after_destroy() {
    pollster::block_on(async {
        let gltf_loader = GltfLoader::new(GltfLoaderOptions {
            gltf_resource: Some("https://example.com/model.gltf".to_string()),
            ..Default::default()
        })
        .unwrap();

        let loader_clone = gltf_loader.clone();
        let fetch_fail_and_destroy = move |_url: String| {
            let l = loader_clone.clone();
            async move {
                l.destroy();
                Err("Load Failed".to_string())
            }
        };

        let res = gltf_loader.load(Some(fetch_fail_and_destroy)).await;
        assert!(res.is_ok());
        assert!(gltf_loader.is_destroyed());
        assert!(gltf_loader.components().is_none());
    });
}
