use cesium_math::{
    Cesium3DTilesTerrainProvider, Cesium3DTilesTerrainProviderOptions,
};

#[test]
fn test_from_url_throws_if_url_is_not_provided() {
    assert!(Cesium3DTilesTerrainProvider::from_url(None, None).is_err());
    assert!(Cesium3DTilesTerrainProvider::from_url(Some(""), None).is_err());
}

#[test]
fn test_from_url_rejects_when_url_is_invalid() {
    assert!(Cesium3DTilesTerrainProvider::from_url(Some("https://invalid.com/tiles"), None).is_err());
}

#[test]
fn test_from_url_works_with_valid_path() {
    let path = "Data/Cesium3DTiles/Terrain/Test/tileset.json";
    let provider = Cesium3DTilesTerrainProvider::from_url(Some(path), None).unwrap();
    assert_eq!(provider.url(), path);
    assert!(!provider.has_water_mask());
    assert!(!provider.has_vertex_normals());
}

#[test]
fn test_credit_handling() {
    let path = "Data/Cesium3DTiles/Terrain/Test/tileset.json";
    let provider_no_credit = Cesium3DTilesTerrainProvider::from_url(Some(path), None).unwrap();
    assert!(provider_no_credit.credit().is_none());

    let opts = Cesium3DTilesTerrainProviderOptions {
        credit: Some("Cesium Ion Terrain Credit".to_string()),
        ..Default::default()
    };
    let provider_with_credit =
        Cesium3DTilesTerrainProvider::from_url(Some(path), Some(opts)).unwrap();
    assert_eq!(provider_with_credit.credit(), Some("Cesium Ion Terrain Credit"));
}

#[test]
fn test_has_water_mask_when_requested() {
    let path = "Data/Cesium3DTiles/Terrain/Test/tileset.json";
    let opts = Cesium3DTilesTerrainProviderOptions {
        request_water_mask: true,
        request_vertex_normals: true,
        ..Default::default()
    };
    let provider = Cesium3DTilesTerrainProvider::from_url(Some(path), Some(opts)).unwrap();
    assert!(provider.has_water_mask());
    assert!(provider.has_vertex_normals());
}

#[test]
fn test_request_tile_geometry() {
    let path = "Data/Cesium3DTiles/Terrain/Test/tileset.json";
    let provider = Cesium3DTilesTerrainProvider::from_url(Some(path), None).unwrap();
    let data = provider.request_tile_geometry(0, 0, 0).unwrap();
    assert_eq!(data.minimum_height(), 0.0);
    assert_eq!(data.maximum_height(), 100.0);
}
