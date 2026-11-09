use cesium_renderer::gltf_gpm_loader::{
    GltfMeshPrimitiveGpmLoader, GltfMeshPrimitiveGpmLoaderOptions,
};
use cesium_renderer::gltf_texture_loader::SupportedImageFormats;
use serde_json::json;

fn create_embedded_gltf(gpm_extension: Option<serde_json::Value>) -> serde_json::Value {
    let mut mesh_primitive_extensions = serde_json::Map::new();
    let mut extensions_used = Vec::new();

    if let Some(ext) = gpm_extension {
        mesh_primitive_extensions.insert("NGA_gpm_local".to_string(), ext);
        extensions_used.push("NGA_gpm_local");
    }

    json!({
        "asset": {
            "version": "2.0"
        },
        "meshes": [
            {
                "primitives": [
                    {
                        "attributes": {
                            "POSITION": 1,
                            "NORMAL": 2,
                            "TEXCOORD_0": 3
                        },
                        "indices": 0,
                        "mode": 4,
                        "extensions": mesh_primitive_extensions
                    }
                ]
            }
        ],
        "extensionsUsed": extensions_used
    })
}

fn create_sample_gpm_extension() -> serde_json::Value {
    json!({
        "ppeTextures": [
            {
                "traits": {
                    "source": "SIGZ",
                    "min": 0.0,
                    "max": 16.0
                },
                "index": 0,
                "noData": 255,
                "offset": 0.0,
                "scale": 0.06274509803921569,
                "texCoord": 0
            }
        ]
    })
}

#[test]
fn test_throws_if_gltf_is_undefined() {
    let opts = GltfMeshPrimitiveGpmLoaderOptions {
        gltf: None,
        extension: None,
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/model.glb".to_string()),
        supported_image_formats: Some(SupportedImageFormats::default()),
        frame_state: Some(json!({"context": {"id": "01234"}})),
    };
    assert!(GltfMeshPrimitiveGpmLoader::new(opts).is_err());
}

#[test]
fn test_throws_if_gltf_resource_is_undefined() {
    let gltf = create_embedded_gltf(None);
    let opts = GltfMeshPrimitiveGpmLoaderOptions {
        gltf: Some(gltf),
        extension: None,
        gltf_resource: None,
        base_resource: Some("https://example.com/model.glb".to_string()),
        supported_image_formats: Some(SupportedImageFormats::default()),
        frame_state: Some(json!({"context": {"id": "01234"}})),
    };
    assert!(GltfMeshPrimitiveGpmLoader::new(opts).is_err());
}

#[test]
fn test_throws_if_base_resource_is_undefined() {
    let gltf = create_embedded_gltf(None);
    let opts = GltfMeshPrimitiveGpmLoaderOptions {
        gltf: Some(gltf),
        extension: None,
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: None,
        supported_image_formats: Some(SupportedImageFormats::default()),
        frame_state: Some(json!({"context": {"id": "01234"}})),
    };
    assert!(GltfMeshPrimitiveGpmLoader::new(opts).is_err());
}

#[test]
fn test_throws_if_supported_image_formats_is_undefined() {
    let gltf = create_embedded_gltf(None);
    let opts = GltfMeshPrimitiveGpmLoaderOptions {
        gltf: Some(gltf),
        extension: None,
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/model.glb".to_string()),
        supported_image_formats: None,
        frame_state: Some(json!({"context": {"id": "01234"}})),
    };
    assert!(GltfMeshPrimitiveGpmLoader::new(opts).is_err());
}

#[test]
fn test_throws_if_frame_state_is_undefined() {
    let gltf = create_embedded_gltf(None);
    let opts = GltfMeshPrimitiveGpmLoaderOptions {
        gltf: Some(gltf),
        extension: None,
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/model.glb".to_string()),
        supported_image_formats: Some(SupportedImageFormats::default()),
        frame_state: None,
    };
    assert!(GltfMeshPrimitiveGpmLoader::new(opts).is_err());
}

#[test]
fn test_loads_mesh_primitive_gpm_extension_data() {
    let ext = create_sample_gpm_extension();
    let gltf = create_embedded_gltf(Some(ext.clone()));

    let opts = GltfMeshPrimitiveGpmLoaderOptions {
        gltf: Some(gltf),
        extension: Some(ext),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/model.glb".to_string()),
        supported_image_formats: Some(SupportedImageFormats::default()),
        frame_state: Some(json!({"context": {"id": "01234"}})),
    };

    let mut loader = GltfMeshPrimitiveGpmLoader::new(opts).expect("Loader creation should succeed");
    loader.load().expect("load should succeed");
    assert!(loader.process().expect("process should succeed"));

    let gpm_data = loader
        .mesh_primitive_gpm_local
        .as_ref()
        .expect("GPM data should exist");
    assert_eq!(gpm_data.ppe_textures.len(), 1);

    let ppe_tex = &gpm_data.ppe_textures[0];
    assert_eq!(ppe_tex.index, 0);
    assert_eq!(ppe_tex.tex_coord, 0);
    assert_eq!(ppe_tex.no_data, 255);
    assert_eq!(ppe_tex.offset, 0.0);
    assert!((ppe_tex.scale - 0.06274509803921569).abs() < 1e-6);

    assert_eq!(ppe_tex.traits.min, 0.0);
    assert_eq!(ppe_tex.traits.max, 16.0);
    assert_eq!(ppe_tex.traits.source, "SIGZ");
}

#[test]
fn test_converts_mesh_primitive_gpm_extension_data_into_structural_metadata() {
    let ext = create_sample_gpm_extension();
    let gltf = create_embedded_gltf(Some(ext.clone()));

    let opts = GltfMeshPrimitiveGpmLoaderOptions {
        gltf: Some(gltf),
        extension: Some(ext),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/model.glb".to_string()),
        supported_image_formats: Some(SupportedImageFormats::default()),
        frame_state: Some(json!({"context": {"id": "01234"}})),
    };

    let mut loader = GltfMeshPrimitiveGpmLoader::new(opts).expect("Loader creation should succeed");
    loader.load().expect("load should succeed");
    loader.process().expect("process should succeed");

    let structural_metadata = loader
        .structural_metadata
        .as_ref()
        .expect("Structural metadata should exist");
    let ppe_prop_tex = structural_metadata
        .get_property_texture(0)
        .expect("Property texture at index 0 should exist");
    assert_eq!(ppe_prop_tex.id, 0);

    let sigz_prop = ppe_prop_tex
        .get_property("SIGZ")
        .expect("SIGZ property should exist");
    assert_eq!(sigz_prop.texture_reader.tex_coord, 0);
    assert_eq!(sigz_prop.channels, vec![0]);
}

#[test]
fn test_destroys_structural_metadata() {
    let ext = create_sample_gpm_extension();
    let gltf = create_embedded_gltf(Some(ext.clone()));

    let opts = GltfMeshPrimitiveGpmLoaderOptions {
        gltf: Some(gltf),
        extension: Some(ext),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/model.glb".to_string()),
        supported_image_formats: Some(SupportedImageFormats::default()),
        frame_state: Some(json!({"context": {"id": "01234"}})),
    };

    let mut loader = GltfMeshPrimitiveGpmLoader::new(opts).expect("Loader creation should succeed");
    loader.load().expect("load should succeed");
    assert!(loader.structural_metadata.is_some());
    assert!(!loader.is_destroyed());

    loader.destroy();

    assert!(loader.structural_metadata.is_none());
    assert!(loader.is_destroyed());
    assert_eq!(loader.texture_destroy_count, 1);
}

#[test]
fn test_handles_resolving_resources_after_destroy() {
    let ext = create_sample_gpm_extension();
    let gltf = create_embedded_gltf(Some(ext.clone()));

    let opts = GltfMeshPrimitiveGpmLoaderOptions {
        gltf: Some(gltf),
        extension: Some(ext),
        gltf_resource: Some("https://example.com/model.glb".to_string()),
        base_resource: Some("https://example.com/model.glb".to_string()),
        supported_image_formats: Some(SupportedImageFormats::default()),
        frame_state: Some(json!({"context": {"id": "01234"}})),
    };

    let mut loader = GltfMeshPrimitiveGpmLoader::new(opts).expect("Loader creation should succeed");
    loader.destroy();
    assert!(loader.is_destroyed());

    // Calling load after destroy should succeed gracefully
    assert!(loader.load().is_ok());
    assert!(loader.structural_metadata.is_none());
}
