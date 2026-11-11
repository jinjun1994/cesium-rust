use cesium_renderer::{
    ImageryProvider, WebMapTileServiceImageryProvider, WebMapTileServiceImageryProviderOptions,
};

#[test]
fn test_wmts_generates_expected_tile_urls() {
    let options = WebMapTileServiceImageryProviderOptions {
        url: Some("http://wmts.invalid".to_string()),
        format: Some("image/png".to_string()),
        layer: Some("someLayer".to_string()),
        style: Some("someStyle".to_string()),
        tile_matrix_set_id: Some("someTMS".to_string()),
        tile_matrix_labels: Some(vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ]),
        ..Default::default()
    };

    let provider = WebMapTileServiceImageryProvider::new(options).unwrap();

    let tilecol = 12;
    let tilerow = 5;
    let level = 1;
    let url1 = provider.build_image_url(tilecol, tilerow, level);

    assert!(url1.contains("request=GetTile"));
    assert!(url1.contains("service=WMTS"));
    assert!(url1.contains("version=1.0.0"));
    assert!(url1.contains("format=image/png"));
    assert!(url1.contains("layer=someLayer"));
    assert!(url1.contains("style=someStyle"));
    assert!(url1.contains("tilecol=12"));
    assert!(url1.contains("tilerow=5"));
    assert!(url1.contains("tilematrixset=someTMS"));
    assert!(url1.contains("tilematrix=second"));

    let url2 = provider.build_image_url(1, 3, 2);
    assert!(url2.contains("tilecol=1"));
    assert!(url2.contains("tilerow=3"));
    assert!(url2.contains("tilematrix=third"));
}

#[test]
fn test_wmts_generates_expected_tile_urls_for_subdomains() {
    let options = WebMapTileServiceImageryProviderOptions {
        url: Some("http://wmts{s}.invalid".to_string()),
        format: Some("image/png".to_string()),
        layer: Some("someLayer".to_string()),
        style: Some("someStyle".to_string()),
        tile_matrix_set_id: Some("someTMS".to_string()),
        tile_matrix_labels: Some(vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ]),
        subdomains: Some(vec!["a".to_string(), "b".to_string()]),
        ..Default::default()
    };

    let provider = WebMapTileServiceImageryProvider::new(options).unwrap();

    let url1 = provider.build_image_url(12, 5, 1);
    assert!(url1.contains("wmtsa.invalid"));

    let url2 = provider.build_image_url(2, 3, 2);
    assert!(url2.contains("wmtsb.invalid"));
}

#[test]
fn test_wmts_supports_subdomains_string_urls() {
    let subdomains: Vec<String> = "123".chars().map(|c| c.to_string()).collect();
    let options = WebMapTileServiceImageryProviderOptions {
        url: Some("{s}".to_string()),
        layer: Some("".to_string()),
        style: Some("".to_string()),
        subdomains: Some(subdomains),
        tile_matrix_set_id: Some("".to_string()),
        ..Default::default()
    };

    let provider = WebMapTileServiceImageryProvider::new(options).unwrap();
    let url = provider.build_image_url(1, 1, 1);
    assert!(["1", "2", "3"].iter().any(|&s| url.starts_with(s)));
}

#[test]
fn test_wmts_supports_subdomains_array_urls() {
    let options = WebMapTileServiceImageryProviderOptions {
        url: Some("{s}".to_string()),
        layer: Some("".to_string()),
        style: Some("".to_string()),
        subdomains: Some(vec!["foo".to_string(), "bar".to_string()]),
        tile_matrix_set_id: Some("".to_string()),
        ..Default::default()
    };

    let provider = WebMapTileServiceImageryProvider::new(options).unwrap();
    let url = provider.build_image_url(1, 1, 1);
    assert!(["foo", "bar"].iter().any(|&s| url.starts_with(s)));
}

#[test]
fn test_wmts_generates_expected_tile_urls_from_template() {
    let options = WebMapTileServiceImageryProviderOptions {
        url: Some(
            "http://wmts.invalid/{style}/{TileMatrixSet}/{TileMatrix}/{TileRow}/{TileCol}.png"
                .to_string(),
        ),
        format: Some("image/png".to_string()),
        layer: Some("someLayer".to_string()),
        style: Some("someStyle".to_string()),
        tile_matrix_set_id: Some("someTMS".to_string()),
        tile_matrix_labels: Some(vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ]),
        ..Default::default()
    };

    let provider = WebMapTileServiceImageryProvider::new(options).unwrap();
    let url = provider.build_image_url(12, 5, 1);
    assert_eq!(
        url,
        "http://wmts.invalid/someStyle/someTMS/second/5/12.png"
    );
}

#[test]
fn test_wmts_requires_url() {
    let res = WebMapTileServiceImageryProvider::new(WebMapTileServiceImageryProviderOptions {
        layer: Some("someLayer".to_string()),
        style: Some("someStyle".to_string()),
        tile_matrix_set_id: Some("someTMS".to_string()),
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_wmts_requires_layer() {
    let res = WebMapTileServiceImageryProvider::new(WebMapTileServiceImageryProviderOptions {
        url: Some("http://wmts.invalid".to_string()),
        style: Some("someStyle".to_string()),
        tile_matrix_set_id: Some("someTMS".to_string()),
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_wmts_requires_style() {
    let res = WebMapTileServiceImageryProvider::new(WebMapTileServiceImageryProviderOptions {
        url: Some("http://wmts.invalid".to_string()),
        layer: Some("someLayer".to_string()),
        tile_matrix_set_id: Some("someTMS".to_string()),
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_wmts_requires_tile_matrix_set_id() {
    let res = WebMapTileServiceImageryProvider::new(WebMapTileServiceImageryProviderOptions {
        url: Some("http://wmts.invalid".to_string()),
        layer: Some("someLayer".to_string()),
        style: Some("someStyle".to_string()),
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_wmts_imagery_provider_trait() {
    let options = WebMapTileServiceImageryProviderOptions {
        url: Some("http://wmts.invalid".to_string()),
        layer: Some("someLayer".to_string()),
        style: Some("someStyle".to_string()),
        tile_matrix_set_id: Some("someTMS".to_string()),
        ..Default::default()
    };

    let provider = WebMapTileServiceImageryProvider::new(options).unwrap();
    assert_eq!(provider.tile_width(), 256);
    assert_eq!(provider.tile_height(), 256);
    assert!(provider.has_alpha_channel());
}
