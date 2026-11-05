use cesium_math::{
    CesiumTerrainFormat, CesiumTerrainLayerJson, CesiumTerrainProvider,
    CesiumTerrainProviderOptions, CesiumTerrainTilingScheme, Ellipsoid, HeightmapEncoding,
    TerrainProvider, TileRange,
};

fn create_quantized_mesh_layer_json() -> CesiumTerrainLayerJson {
    CesiumTerrainLayerJson {
        format: Some("quantized-mesh-1.0".to_string()),
        tiles: Some(vec![
            "https://terrain.invalid/tiles/{z}/{x}/{y}.terrain".to_string(),
            "https://terrain-backup.invalid/tiles/{z}/{x}/{y}.terrain".to_string(),
        ]),
        projection: Some("EPSG:4326".to_string()),
        scheme: Some("tms".to_string()),
        maxzoom: Some(19),
        attribution: Some("Sample Terrain Attribution".to_string()),
        extensions: Some(vec![
            "octvertexnormals".to_string(),
            "watermask".to_string(),
            "metadata".to_string(),
        ]),
        available: Some(vec![
            vec![TileRange {
                start_x: 0,
                start_y: 0,
                end_x: 1,
                end_y: 0,
            }],
            vec![TileRange {
                start_x: 0,
                start_y: 0,
                end_x: 3,
                end_y: 1,
            }],
        ]),
        metadata_availability: None,
        parent_url: None,
        version: Some("1.0.0".to_string()),
    }
}

#[test]
fn test_uses_geographic_tiling_scheme_by_default() {
    let layer_json = create_quantized_mesh_layer_json();
    let provider = CesiumTerrainProvider::from_layer_json(&layer_json, None).unwrap();

    assert!(matches!(
        provider.tiling_scheme(),
        CesiumTerrainTilingScheme::Geographic(_)
    ));
}

#[test]
fn test_can_use_custom_ellipsoid() {
    let layer_json = create_quantized_mesh_layer_json();
    let custom_ellipsoid = Ellipsoid::new(glam::DVec3::splat(1000.0)).unwrap();
    let opts = CesiumTerrainProviderOptions {
        ellipsoid: Some(custom_ellipsoid),
        ..Default::default()
    };
    let provider = CesiumTerrainProvider::from_layer_json(&layer_json, Some(opts)).unwrap();

    assert_eq!(
        provider.tiling_scheme().ellipsoid(),
        custom_ellipsoid
    );
}

#[test]
fn test_returns_reasonable_geometric_error_for_various_levels() {
    let layer_json = create_quantized_mesh_layer_json();
    let provider = CesiumTerrainProvider::from_layer_json(&layer_json, None).unwrap();

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
    assert_eq!(
        provider.get_level_maximum_geometric_error(2),
        expected_zero / 4.0
    );
}

#[test]
fn test_credit_handling() {
    let layer_json = create_quantized_mesh_layer_json();
    let provider_with_default_attr =
        CesiumTerrainProvider::from_layer_json(&layer_json, None).unwrap();
    assert_eq!(
        provider_with_default_attr.credit(),
        Some("Sample Terrain Attribution")
    );

    let opts = CesiumTerrainProviderOptions {
        credit: Some("Custom Provider Credit".to_string()),
        ..Default::default()
    };
    let provider_with_override =
        CesiumTerrainProvider::from_layer_json(&layer_json, Some(opts)).unwrap();
    assert_eq!(
        provider_with_override.credit(),
        Some("Custom Provider Credit")
    );

    let mut no_attr_json = layer_json;
    no_attr_json.attribution = None;
    let provider_no_credit =
        CesiumTerrainProvider::from_layer_json(&no_attr_json, None).unwrap();
    assert_eq!(provider_no_credit.credit(), None);
}

#[test]
fn test_has_water_mask_and_vertex_normals() {
    let layer_json = create_quantized_mesh_layer_json();
    let provider = CesiumTerrainProvider::from_layer_json(&layer_json, None).unwrap();

    assert!(provider.has_water_mask());
    assert!(provider.has_vertex_normals());
    assert!(provider.has_metadata());
    assert!(!provider.request_vertex_normals());
    assert!(!provider.request_water_mask());
}

#[test]
fn test_request_vertex_normals_and_water_mask_flags() {
    let layer_json = create_quantized_mesh_layer_json();
    let opts = CesiumTerrainProviderOptions {
        request_vertex_normals: true,
        request_water_mask: true,
        request_metadata: true,
        ..Default::default()
    };
    let provider = CesiumTerrainProvider::from_layer_json(&layer_json, Some(opts)).unwrap();

    assert!(provider.request_vertex_normals());
    assert!(provider.request_water_mask());
    assert!(provider.request_metadata());
}

#[test]
fn test_throws_if_layer_json_does_not_specify_format() {
    let mut layer_json = create_quantized_mesh_layer_json();
    layer_json.format = None;

    assert!(CesiumTerrainProvider::from_layer_json(&layer_json, None).is_err());
}

#[test]
fn test_throws_if_layer_json_specifies_unknown_format() {
    let mut layer_json = create_quantized_mesh_layer_json();
    layer_json.format = Some("unknown-format-2.0".to_string());

    assert!(CesiumTerrainProvider::from_layer_json(&layer_json, None).is_err());
}

#[test]
fn test_supports_quantized_mesh_minor_versions() {
    let mut layer_json = create_quantized_mesh_layer_json();
    layer_json.format = Some("quantized-mesh-1.1".to_string());

    let provider = CesiumTerrainProvider::from_layer_json(&layer_json, None).unwrap();
    assert_eq!(provider.format(), CesiumTerrainFormat::QuantizedMesh);
    assert_eq!(provider.format_version(), "1.1");

    layer_json.format = Some("quantized-mesh-1.2".to_string());
    let provider2 = CesiumTerrainProvider::from_layer_json(&layer_json, None).unwrap();
    assert_eq!(provider2.format_version(), "1.2");
}

#[test]
fn test_throws_if_layer_json_does_not_specify_tiles_property_or_is_empty() {
    let mut layer_json = create_quantized_mesh_layer_json();
    layer_json.tiles = None;
    assert!(CesiumTerrainProvider::from_layer_json(&layer_json, None).is_err());

    layer_json.tiles = Some(vec![]);
    assert!(CesiumTerrainProvider::from_layer_json(&layer_json, None).is_err());
}

#[test]
fn test_uses_multiple_urls_specified_in_layer_json() {
    let layer_json = create_quantized_mesh_layer_json();
    let provider = CesiumTerrainProvider::from_layer_json(&layer_json, None).unwrap();

    assert_eq!(provider.tile_url_templates().len(), 2);
    assert_eq!(
        provider.tile_url_templates()[0],
        "https://terrain.invalid/tiles/{z}/{x}/{y}.terrain"
    );
    assert_eq!(
        provider.tile_url_templates()[1],
        "https://terrain-backup.invalid/tiles/{z}/{x}/{y}.terrain"
    );
}

#[test]
fn test_provides_heightmap_terrain_data() {
    let mut layer_json = create_quantized_mesh_layer_json();
    layer_json.format = Some("heightmap-1.0".to_string());

    let provider = CesiumTerrainProvider::from_layer_json(&layer_json, None).unwrap();
    assert_eq!(provider.format(), CesiumTerrainFormat::Heightmap);
    assert!(provider.has_water_mask());

    let tile = provider.request_heightmap_tile(0, 0, 0, None).unwrap();
    assert_eq!(tile.width(), 65);
    assert_eq!(tile.height(), 65);
    assert_eq!(tile.encoding(), HeightmapEncoding::None);
}

#[test]
fn test_provides_quantized_mesh_terrain_data() {
    let layer_json = create_quantized_mesh_layer_json();
    let provider = CesiumTerrainProvider::from_layer_json(&layer_json, None).unwrap();

    let tile = provider.request_quantized_mesh_tile(0, 0, 0, None).unwrap();
    assert!(tile.has_vertex_normals());
    assert!(tile.has_water_mask());
}

#[test]
fn test_availability_and_tms_coordinate_conversion() {
    let layer_json = create_quantized_mesh_layer_json();
    let provider = CesiumTerrainProvider::from_layer_json(&layer_json, None).unwrap();

    // At level 0: GeographicTilingScheme has 2 x 1 tiles (x in 0..2, y in 0..1).
    // TMS range: start_y = 0, end_y = 0 => Cesium y = 1 - 0 - 1 = 0.
    assert_eq!(provider.is_tile_available(0, 0, 0), Some(true));
    assert_eq!(provider.is_tile_available(0, 1, 0), Some(true));
    assert_eq!(provider.is_tile_available(0, 2, 0), Some(false));

    // get_tile_data_available takes TMS coords:
    assert_eq!(provider.get_tile_data_available(0, 0, 0), Some(true));
    assert_eq!(provider.get_tile_data_available(1, 0, 0), Some(true));
    assert_eq!(provider.get_tile_data_available(2, 0, 0), Some(false));

    // At level 1: GeographicTilingScheme has 4 x 2 tiles (y in 0..2).
    // TMS range: start_x=0..3, start_y=0..1 => Cesium y = 2 - 1 - 1 = 0 and 2 - 0 - 1 = 1.
    assert_eq!(provider.is_tile_available(1, 0, 0), Some(true));
    assert_eq!(provider.is_tile_available(1, 3, 1), Some(true));
    assert_eq!(provider.is_tile_available(1, 4, 0), Some(false));
}

#[test]
fn test_detects_web_mercator_tiling_scheme() {
    let mut layer_json = create_quantized_mesh_layer_json();
    layer_json.projection = Some("EPSG:3857".to_string());

    let provider = CesiumTerrainProvider::from_layer_json(&layer_json, None).unwrap();
    assert!(matches!(
        provider.tiling_scheme(),
        CesiumTerrainTilingScheme::WebMercator(_)
    ));
}
