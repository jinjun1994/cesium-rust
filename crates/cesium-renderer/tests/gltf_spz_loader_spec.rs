use cesium_renderer::buffer_loader::ResourceLoaderState;
use cesium_renderer::gltf_spz_loader::{
    estimate_spz_memory_bytes, get_spz_info_from_gltf, GltfSpzLoader, GltfSpzLoaderOptions,
    SpzInfo,
};
use serde_json::json;

#[test]
fn derives_point_count_and_spherical_harmonics_degree_from_gltf_json() {
    let gltf = json!({
        "accessors": [
            { "count": 4913000 }
        ]
    });

    let primitive = json!({
        "attributes": {
            "POSITION": 0,
            "_SH_DEGREE_1_COEF_0": 1,
            "_SH_DEGREE_3_COEF_0": 2
        }
    });

    assert_eq!(
        get_spz_info_from_gltf(&gltf, &primitive),
        Some(SpzInfo {
            num_points: 4913000,
            sh_degree: 3,
        })
    );
}

#[test]
fn returns_none_when_gltf_metadata_needed_for_estimate_is_missing() {
    let gltf = json!({ "accessors": [] });
    let primitive = json!({ "attributes": {} });
    assert_eq!(get_spz_info_from_gltf(&gltf, &primitive), None);
}

#[test]
fn estimates_spz_memory_usage() {
    assert_eq!(estimate_spz_memory_bytes(4913000, 3), 2318936000);
}

#[test]
fn throws_if_required_options_are_missing() {
    let gltf = json!({});
    let primitive = json!({});
    let spz = json!({ "bufferView": 1 });

    assert!(GltfSpzLoader::new(GltfSpzLoaderOptions {
        gltf: None,
        primitive: Some(primitive.clone()),
        spz: Some(spz.clone()),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfSpzLoader::new(GltfSpzLoaderOptions {
        gltf: Some(gltf.clone()),
        primitive: None,
        spz: Some(spz.clone()),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        ..Default::default()
    })
    .is_err());

    assert!(GltfSpzLoader::new(GltfSpzLoaderOptions {
        gltf: Some(gltf.clone()),
        primitive: Some(primitive),
        spz: None,
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/".to_string()),
        ..Default::default()
    })
    .is_err());
}

#[test]
fn loads_buffer_view_specified_by_spz_extension() {
    pollster::block_on(async {
        let binary_bytes = vec![1u8, 2, 3, 4];
        let gltf = json!({
            "buffers": [{ "byteLength": binary_bytes.len(), "uri": "external.bin" }],
            "bufferViews": [{ "buffer": 0, "byteOffset": 0, "byteLength": binary_bytes.len() }]
        });
        let primitive = json!({});
        let spz = json!({ "bufferView": 0 });

        let loader = GltfSpzLoader::new(GltfSpzLoaderOptions {
            gltf: Some(gltf),
            primitive: Some(primitive),
            spz: Some(spz),
            gltf_resource: Some("https://example.com/model.glb".to_string()),
            base_resource: Some("https://example.com/".to_string()),
            ..Default::default()
        })
        .unwrap();

        let b = binary_bytes.clone();
        let fetch = move |_url: String| {
            let data = b.clone();
            async move { Ok(data) }
        };

        loader.load(Some(fetch)).await.unwrap();
        assert_eq!(loader.state(), ResourceLoaderState::Processing);

        let process_res = loader.process().unwrap();
        assert!(process_res);
        assert_eq!(loader.state(), ResourceLoaderState::Ready);
        assert!(loader.decoded_data().is_some());

        assert!(!loader.is_destroyed());
        loader.destroy();
        assert!(loader.is_destroyed());
        assert!(loader.decoded_data().is_none());
    });
}
