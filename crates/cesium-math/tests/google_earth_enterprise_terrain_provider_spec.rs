use cesium_math::{
    GoogleEarthEnterpriseMetadata, GoogleEarthEnterpriseTerrainProvider,
    GoogleEarthEnterpriseTerrainProviderOptions, GoogleEarthEnterpriseTileInformation,
};

#[test]
fn test_from_metadata_requires_terrain_present() {
    let mut metadata = GoogleEarthEnterpriseMetadata::from_url("http://test.invalid").unwrap();
    metadata.terrain_present = false;

    let res = GoogleEarthEnterpriseTerrainProvider::from_metadata(metadata, None);
    assert!(res.is_err());
}

#[test]
fn test_provider_properties_and_tile_availability() {
    let mut metadata = GoogleEarthEnterpriseMetadata::from_url("http://test.invalid").unwrap();

    let tile_info = GoogleEarthEnterpriseTileInformation::new(0xff, 1, 1, 1, 0, 0);
    metadata.set_tile_information("0", tile_info);

    let provider = GoogleEarthEnterpriseTerrainProvider::from_metadata(
        metadata,
        Some(GoogleEarthEnterpriseTerrainProviderOptions {
            credit: Some("GEE Data Credit".to_string()),
        }),
    )
    .unwrap();

    assert_eq!(provider.credit(), Some("GEE Data Credit"));
    assert!(!provider.has_water_mask());
    assert!(!provider.has_vertex_normals());
    assert!((provider.get_level_zero_maximum_geometric_error() - 40075.16).abs() < 1e-4);

    // QuadKey "0" corresponds to tile (0, 1, 0) in GEE/tms quad scheme
    assert!(provider.get_tile_data_available(0, 1, 0));
}

#[test]
fn test_request_tile_geometry() {
    let mut metadata = GoogleEarthEnterpriseMetadata::from_url("http://test.invalid").unwrap();
    let mut tile_info = GoogleEarthEnterpriseTileInformation::new(0xff, 1, 1, 1, 0, 0);
    tile_info.ancestor_has_terrain = true;
    metadata.set_tile_information("2", tile_info);

    let provider =
        GoogleEarthEnterpriseTerrainProvider::from_metadata(metadata, None).unwrap();

    // QuadKey "2" corresponds to (1, 0, 0)
    let terrain_data = provider.request_tile_geometry(1, 0, 0).unwrap();
    assert_eq!(terrain_data.child_tile_mask(), 15);
}
