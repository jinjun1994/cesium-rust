use cesium_formats::gltf_pipeline::add_extensions_required;
use serde_json::json;

#[test]
fn test_adds_an_extension_to_extensions_required() {
    let mut gltf = json!({});
    add_extensions_required(&mut gltf, "KHR_materials_pbrSpecularGlossiness");
    add_extensions_required(&mut gltf, "KHR_draco_mesh_compression");
    add_extensions_required(&mut gltf, "KHR_draco_mesh_compression"); // Test adding duplicate

    assert_eq!(
        gltf["extensionsRequired"],
        json!([
            "KHR_materials_pbrSpecularGlossiness",
            "KHR_draco_mesh_compression"
        ])
    );
    assert_eq!(
        gltf["extensionsUsed"],
        json!([
            "KHR_materials_pbrSpecularGlossiness",
            "KHR_draco_mesh_compression"
        ])
    );
}
