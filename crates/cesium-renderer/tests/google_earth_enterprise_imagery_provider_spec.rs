use cesium_math::{
    GeoRectangle, GoogleEarthEnterpriseMetadata, GoogleEarthEnterpriseTileInformation,
};
use cesium_renderer::google_earth_enterprise_imagery_provider::GoogleEarthEnterpriseImageryProvider;
use std::f64::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn install_mock_quadtree_packet(metadata: &mut GoogleEarthEnterpriseMetadata) {
    for i in 0..4 {
        metadata.set_tile_information(
            i.to_string(),
            GoogleEarthEnterpriseTileInformation::new(0xff, 1, 1, 1, 0, 0),
        );
    }
}

#[test]
fn from_metadata_throws_without_metadata() {
    let err = GoogleEarthEnterpriseImageryProvider::from_metadata(None, None).unwrap_err();
    assert!(err.message.contains("metadata is required"));
}

#[test]
fn from_metadata_throws_if_there_isnt_imagery() {
    let mut metadata = GoogleEarthEnterpriseMetadata::from_url("made/up/url").unwrap();
    metadata.imagery_present = false;

    let err =
        GoogleEarthEnterpriseImageryProvider::from_metadata(Some(metadata), None).unwrap_err();
    assert!(err
        .message
        .contains("The server made/up/url/ doesn't have imagery"));
}

#[test]
fn from_metadata_resolves_to_created_provider() {
    let mut metadata = GoogleEarthEnterpriseMetadata::from_url("http://fake.fake.invalid").unwrap();
    install_mock_quadtree_packet(&mut metadata);

    let provider =
        GoogleEarthEnterpriseImageryProvider::from_metadata(Some(metadata), None).unwrap();
    assert_eq!(provider.url(), "http://fake.fake.invalid");
}

#[test]
fn returns_false_for_has_alpha_channel() {
    let mut metadata = GoogleEarthEnterpriseMetadata::from_url("http://fake.fake.invalid").unwrap();
    install_mock_quadtree_packet(&mut metadata);

    let provider =
        GoogleEarthEnterpriseImageryProvider::from_metadata(Some(metadata), None).unwrap();
    assert!(!provider.has_alpha_channel());
}

#[test]
fn can_provide_a_root_tile() {
    pollster::block_on(async {
        let url = "http://fake.fake.invalid/";
        let mut metadata = GoogleEarthEnterpriseMetadata::from_url(url).unwrap();
        install_mock_quadtree_packet(&mut metadata);

        let provider =
            GoogleEarthEnterpriseImageryProvider::from_metadata(Some(metadata), None).unwrap();

        assert_eq!(provider.url(), url);
        assert_eq!(provider.tile_width(), 256);
        assert_eq!(provider.tile_height(), 256);
        assert_eq!(provider.maximum_level(), 23);
        assert_eq!(provider.minimum_level(), 0);

        let expected_rect = GeoRectangle {
            west: -PI,
            south: -PI,
            east: PI,
            north: PI,
        };
        assert_eq!(provider.rectangle(), expected_rect);
        assert!(provider.credit().is_none());

        // Test root tile url format: x=0, y=0, level=0 -> flatfile?f1-03-i.1
        let expected_tile_url = "http://fake.fake.invalid/flatfile?f1-03-i.1";
        let tile_url = provider.build_tile_url(0, 0, 0);
        assert_eq!(tile_url, expected_tile_url);

        let fetch_fn = |req_url: String| async move {
            assert_eq!(req_url, "http://fake.fake.invalid/flatfile?f1-03-i.1");
            Ok(vec![255, 0, 0, 255])
        };

        let img = provider.request_image(0, 0, 0, fetch_fn).await.unwrap();
        assert_eq!(img, vec![255, 0, 0, 255]);
    });
}

#[test]
fn raises_error_event_when_image_cannot_be_loaded_and_retries() {
    pollster::block_on(async {
        let mut metadata =
            GoogleEarthEnterpriseMetadata::from_url("http://foo.bar.invalid").unwrap();
        install_mock_quadtree_packet(&mut metadata);

        let provider =
            GoogleEarthEnterpriseImageryProvider::from_metadata(Some(metadata), None).unwrap();

        let tries = Arc::new(AtomicUsize::new(0));
        let tries_clone = tries.clone();

        let fetch_with_retry = move |_url: String| {
            let t = tries_clone.clone();
            async move {
                let current = t.fetch_add(1, Ordering::SeqCst);
                if current < 2 {
                    Err("Network timeout".to_string())
                } else {
                    Ok(vec![1, 2, 3, 4])
                }
            }
        };

        // Try 1 and 2 fail
        assert!(provider.request_image(0, 0, 0, &fetch_with_retry).await.is_err());
        assert!(provider.request_image(0, 0, 0, &fetch_with_retry).await.is_err());

        // Try 3 succeeds
        let res = provider.request_image(0, 0, 0, &fetch_with_retry).await.unwrap();
        assert_eq!(res, vec![1, 2, 3, 4]);
        assert_eq!(tries.load(Ordering::SeqCst), 3);
    });
}
