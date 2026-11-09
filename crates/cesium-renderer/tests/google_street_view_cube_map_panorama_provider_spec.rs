use cesium_math::Cartographic;
use cesium_renderer::google_street_view_cube_map_panorama_provider::{
    set_default_street_view_static_api_key, FaceUrlOptions,
    GoogleStreetViewCubeMapPanoramaProvider, GoogleStreetViewCubeMapPanoramaProviderOptions,
    LoadPanoramaOptions,
};
use serde_json::json;

#[test]
fn constructs_with_key() {
    let provider =
        GoogleStreetViewCubeMapPanoramaProvider::new(GoogleStreetViewCubeMapPanoramaProviderOptions {
            key: Some("test-key".to_string()),
            url: Some("https://example.com/".to_string()),
            tile_size: Some(512),
            ..Default::default()
        });

    assert_eq!(provider.key(), "test-key");
    assert_eq!(provider.tile_size(), 512);
}

#[test]
fn defaults_tile_size_to_600() {
    let provider =
        GoogleStreetViewCubeMapPanoramaProvider::new(GoogleStreetViewCubeMapPanoramaProviderOptions {
            key: Some("test-key".to_string()),
            ..Default::default()
        });

    assert_eq!(provider.tile_size(), 600);
}

#[test]
fn from_url_uses_default_key_when_none_is_provided() {
    pollster::block_on(async {
        set_default_street_view_static_api_key(Some("global-test-key".to_string()));

        let provider = GoogleStreetViewCubeMapPanoramaProvider::from_url(
            GoogleStreetViewCubeMapPanoramaProviderOptions::default(),
        )
        .await
        .unwrap();

        assert_eq!(provider.key(), "global-test-key");

        // Clean up global key
        set_default_street_view_static_api_key(None);
    });
}

#[test]
fn from_url_creates_provider() {
    pollster::block_on(async {
        let provider = GoogleStreetViewCubeMapPanoramaProvider::from_url(
            GoogleStreetViewCubeMapPanoramaProviderOptions {
                key: Some("abc".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(provider.key(), "abc");
    });
}

#[test]
fn from_url_throws_without_key() {
    pollster::block_on(async {
        set_default_street_view_static_api_key(None);

        let err = GoogleStreetViewCubeMapPanoramaProvider::from_url(
            GoogleStreetViewCubeMapPanoramaProviderOptions::default(),
        )
        .await
        .unwrap_err();

        assert!(err.message.contains("required"));
    });
}

#[test]
fn get_nearest_pano_id_throws_without_position() {
    pollster::block_on(async {
        let provider =
            GoogleStreetViewCubeMapPanoramaProvider::new(GoogleStreetViewCubeMapPanoramaProviderOptions {
                key: Some("test-key".to_string()),
                ..Default::default()
            });

        let dummy_fetch = |_url: String| async { Ok(json!({})) };
        let err = provider
            .get_nearest_pano_id(None, None, dummy_fetch)
            .await
            .unwrap_err();

        assert!(err.message.contains("position is required"));
    });
}

#[test]
fn get_nearest_pano_id_returns_parsed_pano_data() {
    pollster::block_on(async {
        let provider =
            GoogleStreetViewCubeMapPanoramaProvider::new(GoogleStreetViewCubeMapPanoramaProviderOptions {
                key: Some("test-key".to_string()),
                url: Some("https://example.com/".to_string()),
                ..Default::default()
            });

        let mock_response = json!({
            "status": "OK",
            "pano_id": "pano123",
            "location": {
                "lat": 10.0,
                "lng": 20.0
            }
        });

        let fetch_fn = move |url: String| {
            let resp = mock_response.clone();
            async move {
                assert!(url.contains("location=10,20"));
                assert!(url.contains("key=test-key"));
                assert!(url.contains("radius=50"));
                Ok(resp)
            }
        };

        let position = Cartographic::from_degrees(20.0, 10.0, Some(0.0));
        let result = provider
            .get_nearest_pano_id(Some(position), None, fetch_fn)
            .await
            .unwrap();

        assert_eq!(result.pano_id, "pano123");
        assert!((result.latitude - 10.0).abs() < 1e-6);
        assert!((result.longitude - 20.0).abs() < 1e-6);
    });
}

#[test]
fn get_nearest_pano_id_throws_on_non_ok_status() {
    pollster::block_on(async {
        let provider =
            GoogleStreetViewCubeMapPanoramaProvider::new(GoogleStreetViewCubeMapPanoramaProviderOptions {
                key: Some("test-key".to_string()),
                ..Default::default()
            });

        let fetch_fn = |_url: String| async {
            Ok(json!({
                "status": "ZERO_RESULTS"
            }))
        };

        let position = Cartographic::from_degrees(0.0, 0.0, Some(0.0));
        let err = provider
            .get_nearest_pano_id(Some(position), None, fetch_fn)
            .await
            .unwrap_err();

        assert!(err.message.contains("ZERO_RESULTS"));
    });
}

#[test]
fn get_pano_id_metadata_returns_metadata() {
    pollster::block_on(async {
        let provider =
            GoogleStreetViewCubeMapPanoramaProvider::new(GoogleStreetViewCubeMapPanoramaProviderOptions {
                key: Some("test-key".to_string()),
                url: Some("https://example.com/".to_string()),
                ..Default::default()
            });

        let mock_response = json!({
            "status": "OK",
            "location": { "lat": 1.0, "lng": 2.0 }
        });

        let fetch_fn = move |url: String| {
            let resp = mock_response.clone();
            async move {
                assert!(url.contains("pano=abc"));
                assert!(url.contains("key=test-key"));
                Ok(resp)
            }
        };

        let result = provider
            .get_pano_id_metadata("abc", fetch_fn)
            .await
            .unwrap();
        assert_eq!(result["location"]["lat"], 1.0);
    });
}

#[test]
fn get_pano_id_metadata_throws_on_error_status() {
    pollster::block_on(async {
        let provider =
            GoogleStreetViewCubeMapPanoramaProvider::new(GoogleStreetViewCubeMapPanoramaProviderOptions {
                key: Some("test-key".to_string()),
                ..Default::default()
            });

        let fetch_fn = |_url: String| async {
            Ok(json!({
                "status": "INVALID_REQUEST"
            }))
        };

        let err = provider
            .get_pano_id_metadata("abc", fetch_fn)
            .await
            .unwrap_err();

        assert!(err.message.contains("INVALID_REQUEST"));
    });
}

#[test]
fn build_face_url_builds_url_with_query_params() {
    let provider =
        GoogleStreetViewCubeMapPanoramaProvider::new(GoogleStreetViewCubeMapPanoramaProviderOptions {
            key: Some("test-key".to_string()),
            url: Some("https://example.com/".to_string()),
            tile_size: Some(512),
            ..Default::default()
        });

    let url = provider.build_face_url(&FaceUrlOptions {
        heading: 0,
        pitch: 0,
        tile_size_string: "512x512".to_string(),
        pano_id: "test".to_string(),
        signature: None,
    });

    assert!(url.starts_with("https://example.com/?"));
    assert!(url.contains("size=512x512"));
    assert!(url.contains("pano=test"));
    assert!(url.contains("heading=0"));
    assert!(url.contains("pitch=0"));
    assert!(url.contains("key=test-key"));
}

#[test]
fn build_face_url_includes_signature_when_defined() {
    let provider =
        GoogleStreetViewCubeMapPanoramaProvider::new(GoogleStreetViewCubeMapPanoramaProviderOptions {
            key: Some("test-key".to_string()),
            url: Some("https://example.com/".to_string()),
            tile_size: Some(512),
            ..Default::default()
        });

    let url = provider.build_face_url(&FaceUrlOptions {
        heading: 90,
        pitch: 10,
        tile_size_string: "512x512".to_string(),
        pano_id: "abc".to_string(),
        signature: Some("mysignature".to_string()),
    });

    assert!(url.contains("heading=90"));
    assert!(url.contains("pitch=10"));
    assert!(url.contains("signature=mysignature"));
}

#[test]
fn load_panorama_throws_without_cartographic() {
    pollster::block_on(async {
        let provider =
            GoogleStreetViewCubeMapPanoramaProvider::new(GoogleStreetViewCubeMapPanoramaProviderOptions {
                key: Some("test-key".to_string()),
                ..Default::default()
            });

        let dummy_fetch = |_url: String| async { Ok(vec![0u8]) };
        let err = provider
            .load_panorama(LoadPanoramaOptions::default(), dummy_fetch)
            .await
            .unwrap_err();

        assert!(err.message.contains("cartographic is required"));
    });
}

#[test]
fn load_panorama_builds_cube_map_panorama() {
    pollster::block_on(async {
        let provider =
            GoogleStreetViewCubeMapPanoramaProvider::new(GoogleStreetViewCubeMapPanoramaProviderOptions {
                key: Some("test-key".to_string()),
                url: Some("https://example.com/".to_string()),
                tile_size: Some(512),
                ..Default::default()
            });

        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count_clone = call_count.clone();

        let fetch_fn = move |_url: String| {
            let c = count_clone.clone();
            async move {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(vec![1, 2, 3])
            }
        };

        let cartographic = Cartographic::from_degrees(20.0, 10.0, Some(0.0));
        let panorama = provider
            .load_panorama(
                LoadPanoramaOptions {
                    cartographic: Some(cartographic),
                    pano_id: Some("abc".to_string()),
                    ..Default::default()
                },
                fetch_fn,
            )
            .await
            .unwrap();

        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 6);
        assert!(panorama.show);
        assert!(panorama.sources.is_some());
    });
}

#[test]
fn parse_metadata_converts_to_cartographic() {
    let metadata = json!({
        "location": {
            "lat": 10.0,
            "lng": 20.0
        }
    });

    let carto =
        GoogleStreetViewCubeMapPanoramaProvider::parse_metadata(&metadata).unwrap();
    assert!((carto.latitude.to_degrees() - 10.0).abs() < 1e-6);
    assert!((carto.longitude.to_degrees() - 20.0).abs() < 1e-6);
}
