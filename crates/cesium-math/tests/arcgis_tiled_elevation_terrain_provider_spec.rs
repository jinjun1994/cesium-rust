use cesium_math::{
    ArcGISMetadata, ArcGISMetadataExtent, ArcGISMetadataLod, ArcGISMetadataSpatialReference,
    ArcGISMetadataTileInfo, ArcGISTiledElevationTerrainProvider, ArcGISTilingScheme, Ellipsoid,
    HeightmapEncoding, TerrainProvider,
};

fn create_test_metadata() -> ArcGISMetadata {
    ArcGISMetadata {
        copyright_text: Some(
            "Source: USGS, NGA, NASA, CGIAR, GEBCO, N Robinson, NCEAS, NLS, OS, NMA, Geodatastyrelsen and the GIS User Community".to_string(),
        ),
        spatial_reference: Some(ArcGISMetadataSpatialReference {
            wkid: Some(102100),
            latest_wkid: Some(3857),
        }),
        extent: Some(ArcGISMetadataExtent {
            xmin: -2.0037507842788246e7,
            ymin: -2.0037508659999996e7,
            xmax: 2.0037509157211754e7,
            ymax: 2.0037508340000004e7,
        }),
        tile_info: Some(ArcGISMetadataTileInfo {
            rows: 64,
            cols: 64,
            format: Some("LERC".to_string()),
            lods: vec![
                ArcGISMetadataLod {
                    level: 0,
                    resolution: Some(156543.033928),
                    scale: Some(591657527.591555),
                },
                ArcGISMetadataLod {
                    level: 1,
                    resolution: Some(78271.5169639999),
                    scale: Some(295828763.795777),
                },
            ],
        }),
        capabilities: Some("Image,Tilemap,Mensuration".to_string()),
    }
}

#[test]
fn test_returns_reasonable_geometric_error_for_various_levels() {
    let metadata = create_test_metadata();
    let provider =
        ArcGISTiledElevationTerrainProvider::from_metadata(&metadata, None).unwrap();

    let expected_zero = TerrainProvider::get_estimated_level_zero_geometric_error_for_a_heightmap(
        &provider.tiling_scheme().ellipsoid(),
        65,
        provider.tiling_scheme().number_of_x_tiles_at_level(0),
    );

    assert_eq!(provider.get_level_maximum_geometric_error(0), expected_zero);
    assert_eq!(
        provider.get_level_maximum_geometric_error(1),
        expected_zero / 2.0
    );
}

#[test]
fn test_logo_defined_when_credit_provided() {
    let metadata = create_test_metadata();
    let provider =
        ArcGISTiledElevationTerrainProvider::from_metadata(&metadata, None).unwrap();
    assert!(provider.credit().is_some());
}

#[test]
fn test_logo_undefined_when_credit_not_provided() {
    let mut metadata = create_test_metadata();
    metadata.copyright_text = None;
    let provider =
        ArcGISTiledElevationTerrainProvider::from_metadata(&metadata, None).unwrap();
    assert!(provider.credit().is_none());
}

#[test]
fn test_does_not_have_water_mask() {
    let metadata = create_test_metadata();
    let provider =
        ArcGISTiledElevationTerrainProvider::from_metadata(&metadata, None).unwrap();
    assert!(!provider.has_water_mask());
}

#[test]
fn test_detects_web_mercator_tiling_scheme() {
    let metadata = create_test_metadata();
    let provider =
        ArcGISTiledElevationTerrainProvider::from_metadata(&metadata, None).unwrap();
    assert!(matches!(
        provider.tiling_scheme(),
        ArcGISTilingScheme::WebMercator(_)
    ));
}

#[test]
fn test_detects_geographic_tiling_scheme() {
    let mut metadata = create_test_metadata();
    metadata.spatial_reference = Some(ArcGISMetadataSpatialReference {
        wkid: Some(4326),
        latest_wkid: None,
    });
    metadata.extent = Some(ArcGISMetadataExtent {
        xmin: -180.0,
        ymin: -90.0,
        xmax: 180.0,
        ymax: 90.0,
    });

    let provider =
        ArcGISTiledElevationTerrainProvider::from_metadata(&metadata, None).unwrap();
    assert!(matches!(
        provider.tiling_scheme(),
        ArcGISTilingScheme::Geographic(_)
    ));
}

#[test]
fn test_throws_if_srs_not_supported() {
    let mut metadata = create_test_metadata();
    metadata.spatial_reference = Some(ArcGISMetadataSpatialReference {
        wkid: Some(1234),
        latest_wkid: None,
    });

    assert!(ArcGISTiledElevationTerrainProvider::from_metadata(&metadata, None).is_err());
}

#[test]
fn test_throws_if_tile_info_missing() {
    let mut metadata = create_test_metadata();
    metadata.tile_info = None;

    assert!(ArcGISTiledElevationTerrainProvider::from_metadata(&metadata, None).is_err());
}

#[test]
fn test_checks_availability_if_tilemap_capability_exists() {
    let metadata = create_test_metadata();
    let provider =
        ArcGISTiledElevationTerrainProvider::from_metadata(&metadata, None).unwrap();
    assert!(provider.has_availability());
    assert_eq!(provider.is_tile_available(0, 0, 0), Some(true));
}

#[test]
fn test_does_not_check_availability_if_tilemap_capability_missing() {
    let mut metadata = create_test_metadata();
    metadata.capabilities = Some("Image,Mensuration".to_string());
    let provider =
        ArcGISTiledElevationTerrainProvider::from_metadata(&metadata, None).unwrap();
    assert!(!provider.has_availability());
    assert_eq!(provider.is_tile_available(0, 0, 0), Some(true));
    assert_eq!(provider.is_tile_available(1, 0, 0), Some(true));
    // Beyond lod_count (lod_count is 1):
    assert_eq!(provider.is_tile_available(2, 0, 0), Some(false));
}

#[test]
fn test_provides_heightmap_terrain_data() {
    let metadata = create_test_metadata();
    let provider =
        ArcGISTiledElevationTerrainProvider::from_metadata(&metadata, Some(Ellipsoid::WGS84))
            .unwrap();

    assert_eq!(provider.width(), 65);
    assert_eq!(provider.height(), 65);
    assert_eq!(provider.encoding(), HeightmapEncoding::Lerc);

    let terrain_data = provider.request_tile_geometry(0, 0, 0, None).unwrap();
    assert_eq!(terrain_data.encoding(), HeightmapEncoding::Lerc);
}
