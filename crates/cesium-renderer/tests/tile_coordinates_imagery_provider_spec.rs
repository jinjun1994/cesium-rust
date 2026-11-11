use cesium_math::{
    Ellipsoid, GeographicTilingScheme, WebMercatorTilingScheme,
};
use cesium_renderer::imagery_layer::ImageryProvider;
use cesium_renderer::quadtree_tile::AnyTilingScheme;
use cesium_renderer::{
    TileCoordinatesImageryProvider, TileCoordinatesImageryProviderOptions,
};

#[test]
fn test_conforms_to_imagery_provider_interface() {
    let provider = TileCoordinatesImageryProvider::new(None);
    let trait_obj: &dyn ImageryProvider = &provider;
    assert_eq!(trait_obj.tile_width(), 256);
    assert_eq!(trait_obj.tile_height(), 256);
    assert!(trait_obj.has_alpha_channel());
}

#[test]
fn test_returns_valid_value_for_has_alpha_channel() {
    let provider = TileCoordinatesImageryProvider::new(None);
    assert!(provider.has_alpha_channel());
}

#[test]
fn test_can_use_a_custom_ellipsoid() {
    let ellipsoid = Ellipsoid::new(glam::DVec3::new(1.0, 2.0, 3.0)).unwrap();
    let provider = TileCoordinatesImageryProvider::new(Some(
        TileCoordinatesImageryProviderOptions {
            ellipsoid: Some(ellipsoid),
            ..Default::default()
        },
    ));

    assert_eq!(provider.tiling_scheme().ellipsoid(), ellipsoid);
}

#[test]
fn test_can_provide_a_root_tile() {
    let provider = TileCoordinatesImageryProvider::new(None);

    assert_eq!(provider.tile_width(), 256);
    assert_eq!(provider.tile_height(), 256);
    assert!(provider.maximum_level().is_none());
    assert!(matches!(provider.tiling_scheme(), AnyTilingScheme::Geographic(_)));
    assert_eq!(provider.rectangle(), GeographicTilingScheme::default().rectangle());

    let image = provider.request_image(0, 0, 0);
    assert!(image.is_some());
    assert_eq!(image.unwrap().len(), 256 * 256 * 4);
}

#[test]
fn test_uses_alternate_tiling_scheme_if_provided() {
    let tiling_scheme = WebMercatorTilingScheme::new(
        Ellipsoid::WGS84,
        None,
        1,
        1,
    )
    .unwrap();
    let provider = TileCoordinatesImageryProvider::new(Some(
        TileCoordinatesImageryProviderOptions {
            tiling_scheme: Some(tiling_scheme.into()),
            ..Default::default()
        },
    ));

    assert!(matches!(provider.tiling_scheme(), AnyTilingScheme::WebMercator(_)));
}

#[test]
fn test_uses_tile_width_and_height_if_provided() {
    let provider = TileCoordinatesImageryProvider::new(Some(
        TileCoordinatesImageryProviderOptions {
            tile_width: Some(123),
            tile_height: Some(456),
            ..Default::default()
        },
    ));

    assert_eq!(provider.tile_width(), 123);
    assert_eq!(provider.tile_height(), 456);
    let img = provider.request_image(1, 2, 3);
    assert_eq!(img.unwrap().len(), 123 * 456 * 4);
}
