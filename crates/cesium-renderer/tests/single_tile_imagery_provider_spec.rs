use cesium_math::{Cartesian3, Ellipsoid, GeoRectangle};
use cesium_renderer::imagery_layer::ImageryProvider;
use cesium_renderer::single_tile_imagery_provider::{
    SingleTileImageryProvider, SingleTileImageryProviderFromUrlOptions,
    SingleTileImageryProviderOptions,
};

#[test]
fn test_conforms_to_imagery_provider_interface() {
    let provider = SingleTileImageryProvider::new(SingleTileImageryProviderOptions {
        url: Some("Data/Images/Red16x16.png".to_string()),
        tile_width: Some(16),
        tile_height: Some(16),
        ..Default::default()
    })
    .expect("should construct provider");

    let trait_obj: &dyn ImageryProvider = &provider;
    assert_eq!(trait_obj.tile_width(), 16);
    assert_eq!(trait_obj.tile_height(), 16);
    assert_eq!(trait_obj.minimum_level(), 0);
    assert_eq!(trait_obj.maximum_level(), Some(0));
    assert!(trait_obj.has_alpha_channel());
}

#[test]
fn test_constructor_throws_without_url() {
    let result = SingleTileImageryProvider::new(SingleTileImageryProviderOptions {
        url: None,
        tile_width: Some(16),
        tile_height: Some(16),
        ..Default::default()
    });
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("options.url is required."));
}

#[test]
fn test_constructor_throws_without_tile_dimensions() {
    let res1 = SingleTileImageryProvider::new(SingleTileImageryProviderOptions {
        url: Some("Data/Images/Red16x16.png".to_string()),
        tile_width: None,
        tile_height: Some(16),
        ..Default::default()
    });
    assert!(res1.is_err());
    assert!(res1.unwrap_err().message.contains("options.tileWidth is required."));

    let res2 = SingleTileImageryProvider::new(SingleTileImageryProviderOptions {
        url: Some("Data/Images/Red16x16.png".to_string()),
        tile_width: Some(16),
        tile_height: None,
        ..Default::default()
    });
    assert!(res2.is_err());
    assert!(res2.unwrap_err().message.contains("options.tileHeight is required."));
}

#[test]
fn test_from_url_throws_without_url() {
    let result = SingleTileImageryProvider::from_url(None, None);
    assert!(result.is_err());
}

#[test]
fn test_from_url_resolves_to_created_provider() {
    let provider = SingleTileImageryProvider::from_url(Some("Data/Images/Red16x16.png"), None)
        .expect("should resolve from url");
    assert_eq!(provider.url, "Data/Images/Red16x16.png");
    assert_eq!(provider.tile_width, 16);
    assert_eq!(provider.tile_height, 16);
    assert!(provider.has_alpha_channel);
}

#[test]
fn test_from_url_throws_on_failed_request() {
    let result = SingleTileImageryProvider::from_url(Some("invalid.image.url"), None);
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("Failed to load image invalid.image.url"));
}

#[test]
fn test_properties_are_gettable() {
    let url = "Data/Images/Red16x16.png";
    let rect = GeoRectangle::radians(0.1, 0.2, 0.3, 0.4).unwrap();
    let credit = "hi".to_string();

    let provider = SingleTileImageryProvider::from_url(
        Some(url),
        Some(SingleTileImageryProviderFromUrlOptions {
            rectangle: Some(rect),
            credit: Some(credit.clone()),
            ellipsoid: None,
        }),
    )
    .expect("should construct from url");

    assert_eq!(provider.url, url);
    assert_eq!(provider.rectangle, rect);
    assert!(provider.has_alpha_channel);
    assert_eq!(provider.tiling_scheme.rectangle(), rect);
    assert_eq!(provider.tile_width, 16);
    assert_eq!(provider.tile_height, 16);
    assert_eq!(provider.maximum_level(), Some(0));
    assert_eq!(provider.credit.as_deref(), Some("hi"));
}

#[test]
fn test_can_use_custom_ellipsoid() {
    let custom_ellipsoid =
        Ellipsoid::new(Cartesian3::new(10.0, 20.0, 30.0).into()).unwrap();
    let provider = SingleTileImageryProvider::from_url(
        Some("Data/Images/Red16x16.png"),
        Some(SingleTileImageryProviderFromUrlOptions {
            ellipsoid: Some(custom_ellipsoid),
            ..Default::default()
        }),
    )
    .expect("should construct with custom ellipsoid");

    assert_eq!(provider.tiling_scheme.ellipsoid(), custom_ellipsoid);
}

#[test]
fn test_requests_single_image() {
    let provider = SingleTileImageryProvider::from_url(Some("Data/Images/Red16x16.png"), None)
        .expect("should construct");
    let image = provider.request_image(0, 0, 0);
    assert!(image.is_some());
    assert_eq!(image.unwrap().len(), 16 * 16 * 4);
}
