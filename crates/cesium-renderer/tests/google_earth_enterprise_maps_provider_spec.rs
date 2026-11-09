use cesium_math::{GeoRectangle, WebMercatorTilingScheme};
use cesium_renderer::google_earth_enterprise_maps_provider::{
    GeeMapsTilingScheme, GoogleEarthEnterpriseMapsProvider,
    GoogleEarthEnterpriseMapsProviderOptions,
};
use serde_json::json;
use std::f64::consts::PI;

fn get_good_metadata() -> String {
    json!({
        "isAuthenticated": true,
        "layers": [
            {
                "icon": "icons/773_l.png",
                "id": 1234,
                "initialState": true,
                "label": "Imagery",
                "lookAt": "none",
                "requestType": "ImageryMaps",
                "version": 1
            },
            {
                "icon": "icons/773_l.png",
                "id": 1007,
                "initialState": true,
                "label": "Labels",
                "lookAt": "none",
                "requestType": "VectorMapsRaster",
                "version": 8
            }
        ],
        "serverUrl": "https://example.invalid",
        "useGoogleLayers": false
    })
    .to_string()
}

#[test]
fn from_url_throws_without_url() {
    pollster::block_on(async {
        let err = GoogleEarthEnterpriseMapsProvider::from_url(
            None,
            Some(1234),
            None,
            |_url| async { Ok("{}".to_string()) },
        )
        .await
        .unwrap_err();

        assert!(err.message.contains("options.url is required"));
    });
}

#[test]
fn from_url_throws_without_channel() {
    pollster::block_on(async {
        let err = GoogleEarthEnterpriseMapsProvider::from_url(
            Some("http://example.com".to_string()),
            None,
            None,
            |_url| async { Ok("{}".to_string()) },
        )
        .await
        .unwrap_err();

        assert!(err.message.contains("options.channel is required"));
    });
}

#[test]
fn from_url_resolves_to_imagery_provider() {
    pollster::block_on(async {
        let provider = GoogleEarthEnterpriseMapsProvider::from_url(
            Some("http://example.invalid".to_string()),
            Some(1234),
            Some(GoogleEarthEnterpriseMapsProviderOptions {
                path: Some("".to_string()),
                ..Default::default()
            }),
            |_url| async { Ok(get_good_metadata()) },
        )
        .await
        .unwrap();

        assert_eq!(provider.url(), "http://example.invalid");
        assert_eq!(provider.channel(), 1234);
        assert_eq!(provider.version(), 1);
    });
}

#[test]
fn from_url_throws_with_invalid_url() {
    pollster::block_on(async {
        let err = GoogleEarthEnterpriseMapsProvider::from_url(
            Some("http://invalid.localhost".to_string()),
            Some(1234),
            None,
            |_url| async { Err("Connection refused".to_string()) },
        )
        .await
        .unwrap_err();

        assert!(err.message.contains("An error occurred while accessing"));
    });
}

#[test]
fn from_url_throws_when_channel_cannot_be_found() {
    pollster::block_on(async {
        let err = GoogleEarthEnterpriseMapsProvider::from_url(
            Some("http://invalid.localhost".to_string()),
            Some(1235),
            None,
            |_url| async { Ok(get_good_metadata()) },
        )
        .await
        .unwrap_err();

        assert!(err
            .message
            .contains("Could not find layer with channel (id) of 1235"));
    });
}

#[test]
fn from_url_throws_when_channel_version_cannot_be_found() {
    pollster::block_on(async {
        let bad_version_json = json!({
            "isAuthenticated": true,
            "layers": [
                {
                    "id": 1234,
                    "label": "Imagery"
                }
            ]
        })
        .to_string();

        let err = GoogleEarthEnterpriseMapsProvider::from_url(
            Some("http://invalid.localhost".to_string()),
            Some(1234),
            None,
            move |_url| {
                let j = bad_version_json.clone();
                async move { Ok(j) }
            },
        )
        .await
        .unwrap_err();

        assert!(err
            .message
            .contains("Could not find a version in channel (id) 1234"));
    });
}

#[test]
fn from_url_throws_when_unsupported_projection_is_specified() {
    pollster::block_on(async {
        let bad_proj_json = json!({
            "isAuthenticated": true,
            "projection": "asdf",
            "layers": [
                {
                    "id": 1234,
                    "version": 1
                }
            ]
        })
        .to_string();

        let err = GoogleEarthEnterpriseMapsProvider::from_url(
            Some("http://invalid.localhost".to_string()),
            Some(1234),
            None,
            move |_url| {
                let j = bad_proj_json.clone();
                async move { Ok(j) }
            },
        )
        .await
        .unwrap_err();

        assert!(err.message.contains("Unsupported projection asdf"));
    });
}

#[test]
fn returns_valid_value_for_has_alpha_channel() {
    pollster::block_on(async {
        let provider = GoogleEarthEnterpriseMapsProvider::from_url(
            Some("http://example.invalid".to_string()),
            Some(1234),
            None,
            |_url| async { Ok(get_good_metadata()) },
        )
        .await
        .unwrap();

        assert!(provider.has_alpha_channel());
    });
}

#[test]
fn can_provide_a_root_tile() {
    pollster::block_on(async {
        let provider = GoogleEarthEnterpriseMapsProvider::from_url(
            Some("http://example.invalid".to_string()),
            Some(1234),
            Some(GoogleEarthEnterpriseMapsProviderOptions {
                path: Some("".to_string()),
                ..Default::default()
            }),
            |_url| async { Ok(get_good_metadata()) },
        )
        .await
        .unwrap();

        assert_eq!(provider.url(), "http://example.invalid");
        assert_eq!(provider.path(), "");
        assert_eq!(provider.channel(), 1234);
        assert_eq!(provider.tile_width(), 256);
        assert_eq!(provider.tile_height(), 256);
        assert_eq!(provider.minimum_level(), 0);
        assert_eq!(provider.maximum_level(), None);
        assert_eq!(provider.version(), 1);
        assert!(matches!(
            provider.tiling_scheme(),
            GeeMapsTilingScheme::Mercator(_)
        ));

        let default_wm_rect = WebMercatorTilingScheme::default().rectangle();
        assert_eq!(provider.rectangle(), default_wm_rect);

        // Verify tile request url: x=0, y=0, z=1 (level 0 + 1)
        let expected_tile_url =
            "http://example.invalid/query?request=ImageryMaps&channel=1234&version=1&x=0&y=0&z=1";
        assert_eq!(provider.build_tile_url(0, 0, 0), expected_tile_url);

        let fetch_fn = |req_url: String| async move {
            assert_eq!(
                req_url,
                "http://example.invalid/query?request=ImageryMaps&channel=1234&version=1&x=0&y=0&z=1"
            );
            Ok(vec![1, 2, 3, 4])
        };

        let img = provider.request_image(0, 0, 0, fetch_fn).await.unwrap();
        assert_eq!(img, vec![1, 2, 3, 4]);
    });
}

#[test]
fn handles_malformed_json_data_returned_by_the_server() {
    pollster::block_on(async {
        let loose_json = r#"{
isAuthenticated: true,
layers: [
   {
        icon: "icons/773_l.png",
        id: 1234,
        initialState: true,
        label: "Imagery",
        lookAt: "none",
        requestType: "ImageryMaps",
        version: 1
    },{
        icon: "icons/773_l.png",
        id: 1007,
        initialState: true,
        label: "Labels",
        lookAt: "none",
        requestType: "VectorMapsRaster",
        version: 8
    }
],
serverUrl: "https://example.invalid",
useGoogleLayers: false
}"#;

        let provider = GoogleEarthEnterpriseMapsProvider::from_url(
            Some("http://example.invalid".to_string()),
            Some(1234),
            Some(GoogleEarthEnterpriseMapsProviderOptions {
                path: Some("/default_map".to_string()),
                ..Default::default()
            }),
            move |_url| async move { Ok(loose_json.to_string()) },
        )
        .await
        .unwrap();

        assert_eq!(provider.url(), "http://example.invalid");
        assert_eq!(provider.path(), "/default_map");
        assert_eq!(provider.channel(), 1234);
        assert_eq!(provider.version(), 1);
    });
}

#[test]
fn projection_is_geographic_when_server_projection_is_flat() {
    pollster::block_on(async {
        let flat_json = json!({
            "isAuthenticated": true,
            "projection": "flat",
            "layers": [
                {
                    "id": 1234,
                    "version": 1
                }
            ]
        })
        .to_string();

        let provider = GoogleEarthEnterpriseMapsProvider::from_url(
            Some("http://example.invalid".to_string()),
            Some(1234),
            None,
            move |_url| async move { Ok(flat_json.clone()) },
        )
        .await
        .unwrap();

        assert!(matches!(
            provider.tiling_scheme(),
            GeeMapsTilingScheme::Geographic(_)
        ));

        let expected_rect = GeoRectangle {
            west: -PI,
            south: -PI,
            east: PI,
            north: PI,
        };
        assert_eq!(provider.rectangle(), expected_rect);
    });
}
