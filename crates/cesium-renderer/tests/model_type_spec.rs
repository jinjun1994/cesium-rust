//! Port of CesiumJS `Scene/Model/ModelTypeSpec.js` (Apache-2.0).

use cesium_renderer::model_type::ModelType;

#[test]
fn is_3d_tiles_works() {
    assert!(!ModelType::is_3d_tiles(ModelType::Gltf));
    assert!(ModelType::is_3d_tiles(ModelType::TileGltf));
    assert!(ModelType::is_3d_tiles(ModelType::TileB3dm));
    assert!(ModelType::is_3d_tiles(ModelType::TileI3dm));
    assert!(ModelType::is_3d_tiles(ModelType::TilePnts));
    assert!(ModelType::is_3d_tiles(ModelType::TileGeoJson));
}

#[test]
fn is_3d_tiles_throws_for_invalid_value() {
    assert!(ModelType::is_3d_tiles_str(Some("4DTiles")).is_err());
}

#[test]
fn is_3d_tiles_throws_without_model_type() {
    assert!(ModelType::is_3d_tiles_str(None).is_err());
}
