use cesium_math::{Credit, GeoRectangle};
use cesium_renderer::google_2d_imagery_provider::{
    Google2DIonAssetOptions, Google2DImageryProvider, Google2DImageryProviderOptions,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn requires_the_session_token_to_be_specified() {
    let err = Google2DImageryProvider::new(Google2DImageryProviderOptions {
        session: None,
        key: Some("test-key".to_string()),
        tile_width: Some(256),
        tile_height: Some(256),
        ..Default::default()
    })
    .unwrap_err();

    assert!(err.message.contains("options.session is required"));
}

#[test]
fn requires_the_tile_width_to_be_specified() {
    let err = Google2DImageryProvider::new(Google2DImageryProviderOptions {
        session: Some("a-session-token".to_string()),
        key: Some("test-key".to_string()),
        tile_width: None,
        tile_height: Some(256),
        ..Default::default()
    })
    .unwrap_err();

    assert!(err.message.contains("options.tileWidth is required"));
}

#[test]
fn requires_the_key_to_be_specified() {
    let err = Google2DImageryProvider::new(Google2DImageryProviderOptions {
        session: Some("a-session-token".to_string()),
        key: None,
        tile_width: Some(256),
        tile_height: Some(256),
        ..Default::default()
    })
    .unwrap_err();

    assert!(err.message.contains("options.key is required"));
}

#[test]
fn from_ion_asset_id_throws_if_asset_id_is_not_provided() {
    let err = Google2DImageryProvider::from_ion_asset_id(Google2DIonAssetOptions {
        asset_id: None,
        access_token: None,
    })
    .unwrap_err();

    assert!(err.message.contains("options.assetId is required, actual value was undefined"));
}

#[test]
fn request_image_returns_image_and_formats_url_correctly() {
    pollster::block_on(async {
        let provider = Google2DImageryProvider::new(Google2DImageryProviderOptions {
            session: Some("test-session-token".to_string()),
            key: Some("test-key".to_string()),
            tile_width: Some(256),
            tile_height: Some(256),
            ..Default::default()
        })
        .unwrap();

        assert_eq!(
            provider.url(),
            "https://tile.googleapis.com/v1/2dtiles/{z}/{x}/{y}?session=test-session-token&key=test-key"
        );
        assert_eq!(provider.tile_width(), 256);
        assert_eq!(provider.tile_height(), 256);
        assert_eq!(provider.maximum_level(), 22);
        assert_eq!(provider.minimum_level(), 0);

        let image_data = vec![255, 0, 0, 255];
        let fetch_fn = |url: String| {
            let img = image_data.clone();
            async move {
                assert!(url.contains("/v1/2dtiles/0/0/0"));
                assert!(url.contains("session=test-session-token"));
                assert!(url.contains("key=test-key"));
                Ok(img)
            }
        };

        let result = provider.request_image(0, 0, 0, fetch_fn).await.unwrap();
        assert_eq!(result, vec![255, 0, 0, 255]);
    });
}

#[test]
fn rectangle_passed_to_constructor_does_not_affect_tile_numbering() {
    pollster::block_on(async {
        let rectangle = GeoRectangle::radians(0.1, 0.2, 0.3, 0.4).unwrap();
        let provider = Google2DImageryProvider::new(Google2DImageryProviderOptions {
            session: Some("test-session-token".to_string()),
            key: Some("test-key".to_string()),
            tile_width: Some(256),
            tile_height: Some(256),
            rectangle: Some(rectangle),
            ..Default::default()
        })
        .unwrap();

        assert_eq!(provider.tile_width(), 256);
        assert_eq!(provider.tile_height(), 256);
        assert_eq!(provider.maximum_level(), 22);
        assert_eq!(provider.rectangle(), rectangle);

        let fetch_fn = |url: String| async move {
            assert!(url.contains("/v1/2dtiles/0/0/0"));
            Ok(vec![1, 2, 3, 4])
        };

        let img = provider.request_image(0, 0, 0, fetch_fn).await.unwrap();
        assert_eq!(img, vec![1, 2, 3, 4]);
    });
}

#[test]
fn uses_maximum_level_and_minimum_level_passed_to_constructor() {
    let provider = Google2DImageryProvider::new(Google2DImageryProviderOptions {
        session: Some("test-session-token".to_string()),
        key: Some("test-key".to_string()),
        tile_width: Some(256),
        tile_height: Some(256),
        minimum_level: Some(1),
        maximum_level: Some(5),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(provider.minimum_level(), 1);
    assert_eq!(provider.maximum_level(), 5);
}

#[test]
fn turns_the_supplied_credit_into_attribution() {
    let credit_text = "Thanks to our awesome made up source of this imagery!";
    let credit = Credit::new(credit_text, Some(true));
    let provider = Google2DImageryProvider::new(Google2DImageryProviderOptions {
        session: Some("test-session-token".to_string()),
        key: Some("test-key".to_string()),
        tile_width: Some(256),
        tile_height: Some(256),
        credit: Some(credit.clone()),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(provider.credit().unwrap().html(), credit_text);
    assert!(provider.credit().unwrap().show_on_screen());
}

#[test]
fn handles_tile_request_retries() {
    pollster::block_on(async {
        let provider = Google2DImageryProvider::new(Google2DImageryProviderOptions {
            session: Some("test-session-token".to_string()),
            key: Some("test-key".to_string()),
            tile_width: Some(256),
            tile_height: Some(256),
            ..Default::default()
        })
        .unwrap();

        let tries = Arc::new(AtomicUsize::new(0));
        let tries_clone = tries.clone();

        let fetch_with_retry = move |_url: String| {
            let t = tries_clone.clone();
            async move {
                let current = t.fetch_add(1, Ordering::SeqCst);
                if current < 2 {
                    Err("Network timeout".to_string())
                } else {
                    Ok(vec![10, 20, 30])
                }
            }
        };

        // Simulate 2 failed tries then success on 3rd try
        let mut result = provider.request_image(0, 0, 0, &fetch_with_retry).await;
        assert!(result.is_err());

        result = provider.request_image(0, 0, 0, &fetch_with_retry).await;
        assert!(result.is_err());

        result = provider.request_image(0, 0, 0, &fetch_with_retry).await;
        assert_eq!(result.unwrap(), vec![10, 20, 30]);
        assert_eq!(tries.load(Ordering::SeqCst), 3);
    });
}
