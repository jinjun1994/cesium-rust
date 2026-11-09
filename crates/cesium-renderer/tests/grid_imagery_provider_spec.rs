use cesium_math::{
    Color, Ellipsoid, GeographicTilingScheme, WebMercatorTilingScheme,
};
use cesium_renderer::grid_imagery_provider::{
    GridImageryProvider, GridImageryProviderOptions, GridTilingScheme,
};
use glam::DVec3;

#[test]
fn returns_valid_value_for_has_alpha_channel() {
    let provider = GridImageryProvider::new(GridImageryProviderOptions::default());
    assert!(provider.has_alpha_channel());
}

#[test]
fn can_use_a_custom_ellipsoid() {
    let ellipsoid = Ellipsoid::new(DVec3::new(1.0, 2.0, 3.0)).unwrap();
    let provider = GridImageryProvider::new(GridImageryProviderOptions {
        ellipsoid: Some(ellipsoid),
        ..Default::default()
    });

    assert_eq!(provider.tiling_scheme().ellipsoid(), ellipsoid);
}

#[test]
fn can_provide_a_root_tile() {
    pollster::block_on(async {
        let provider = GridImageryProvider::new(GridImageryProviderOptions::default());

        assert_eq!(provider.tile_width(), 256);
        assert_eq!(provider.tile_height(), 256);
        assert!(provider.maximum_level().is_none());
        assert!(provider.minimum_level().is_none());
        assert!(provider.tile_discard_policy().is_none());

        let default_geo = GeographicTilingScheme::default();
        assert_eq!(provider.rectangle(), default_geo.rectangle());
        assert!(matches!(
            provider.tiling_scheme(),
            GridTilingScheme::Geographic(_)
        ));

        let image = provider.request_image(0, 0, 0).await;
        assert_eq!(image.len(), 256 * 256 * 4);
    });
}

#[test]
fn uses_alternate_tiling_scheme_if_provided() {
    let tiling_scheme = WebMercatorTilingScheme::default();
    let provider = GridImageryProvider::new(GridImageryProviderOptions {
        tiling_scheme: Some(GridTilingScheme::WebMercator(tiling_scheme)),
        ..Default::default()
    });

    assert!(matches!(
        provider.tiling_scheme(),
        GridTilingScheme::WebMercator(_)
    ));
    assert_eq!(provider.rectangle(), tiling_scheme.rectangle());
}

#[test]
fn uses_tile_width_and_height_if_provided() {
    let provider = GridImageryProvider::new(GridImageryProviderOptions {
        tile_width: Some(123),
        tile_height: Some(456),
        ..Default::default()
    });

    assert_eq!(provider.tile_width(), 123);
    assert_eq!(provider.tile_height(), 456);
}

#[test]
fn supports_custom_colors_and_rasterization() {
    pollster::block_on(async {
        let provider = GridImageryProvider::new(GridImageryProviderOptions {
            cells: Some(4),
            color: Some(Color::RED),
            glow_color: Some(Color::BLUE),
            glow_width: Some(4.0),
            background_color: Some(Color::BLACK),
            canvas_size: Some(64),
            ..Default::default()
        });

        assert_eq!(provider.cells(), 4);
        assert_eq!(provider.color(), Color::RED);
        assert_eq!(provider.glow_color(), Color::BLUE);
        assert_eq!(provider.glow_width(), 4.0);
        assert_eq!(provider.background_color(), Color::BLACK);
        assert_eq!(provider.canvas_size(), 64);

        let data = provider.request_image(0, 0, 0).await;
        assert_eq!(data.len(), 64 * 64 * 4);
        assert_eq!(provider.canvas_data().len(), 64 * 64 * 4);
    });
}
