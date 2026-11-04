use cesium_math::{EllipsoidTerrainProvider, GeographicTilingScheme, TerrainProvider};

#[test]
fn test_ellipsoid_terrain_provider_basic() {
    let provider = EllipsoidTerrainProvider::default();

    // requestTileGeometry creates terrain data
    let terrain_data = provider.request_tile_geometry(0, 0, 0);
    assert!(terrain_data.is_some());
    let data = terrain_data.unwrap();
    assert_eq!(data.width(), 16);
    assert_eq!(data.height(), 16);

    // getTileDataAvailable returns None
    assert!(provider.get_tile_data_available(0, 0, 0).is_none());

    // has_water_mask and has_vertex_normals are false
    assert!(!provider.has_water_mask());
    assert!(!provider.has_vertex_normals());

    // level maximum geometric error decreases by half per level
    let err0 = provider.get_level_maximum_geometric_error(0);
    let err1 = provider.get_level_maximum_geometric_error(1);
    assert!((err1 - err0 * 0.5).abs() < 1e-10);

    let tiling_scheme = GeographicTilingScheme::default();
    let expected_err0 = TerrainProvider::get_estimated_level_zero_geometric_error_for_a_heightmap(
        &tiling_scheme.ellipsoid(),
        64,
        tiling_scheme.number_of_x_tiles_at_level(0),
    );
    assert!((err0 - expected_err0).abs() < 1e-10);
}
