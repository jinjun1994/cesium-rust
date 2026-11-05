use std::collections::BTreeMap;

use cesium_math::{
    IonAttribution, IonEndpoint, IonEndpointOptions, IonResource, IonResourceOptions,
    QueryValue, RequestErrorEvent, DEFAULT_ION_ACCESS_TOKEN, DEFAULT_ION_SERVER,
};

#[test]
fn ion_resource_constructs_with_expected_values() {
    let asset_id = 123890213;
    let endpoint = IonEndpoint {
        endpoint_type: "3DTILES".to_string(),
        url: Some(format!("https://assets.cesium.com/{asset_id}/tileset.json")),
        access_token: Some("not_really_a_refresh_token".to_string()),
        attributions: vec![],
        external_type: None,
        options: None,
    };

    let endpoint_resource = IonResource::create_endpoint_resource(asset_id, None);
    let resource = IonResource::new(endpoint.clone(), endpoint_resource.clone()).unwrap();

    assert_eq!(resource.ion_endpoint, endpoint);
    assert_eq!(resource.retry_attempts, 1);
    assert!(resource.retry_callback.is_some());
    assert_eq!(
        resource.headers.get("Authorization"),
        Some(&"Bearer not_really_a_refresh_token".to_string())
    );
    assert_eq!(
        resource.headers.get("X-Cesium-Client"),
        Some(&"CesiumJS".to_string())
    );
}

#[test]
fn ion_resource_clone_works() {
    let asset_id = 123890213;
    let endpoint = IonEndpoint {
        endpoint_type: "3DTILES".to_string(),
        url: Some(format!("https://assets.cesium.com/{asset_id}/tileset.json")),
        access_token: Some("not_really_a_refresh_token".to_string()),
        attributions: vec![],
        external_type: None,
        options: None,
    };

    let endpoint_resource = IonResource::create_endpoint_resource(asset_id, None);
    let resource = IonResource::new(endpoint, endpoint_resource).unwrap();
    let cloned = resource.clone_resource();

    assert_eq!(cloned.url(), resource.url());
    assert_eq!(
        cloned.headers.get("Authorization"),
        resource.headers.get("Authorization")
    );
    assert_eq!(cloned.retry_attempts, resource.retry_attempts);
    assert_eq!(cloned.ion_endpoint, resource.ion_endpoint);
}

#[test]
fn ion_resource_from_endpoint_non_external() {
    let asset_id = 123890213;
    let endpoint = IonEndpoint {
        endpoint_type: "3DTILES".to_string(),
        url: Some(format!("https://assets.cesium.com/{asset_id}/tileset.json")),
        access_token: Some("token_123".to_string()),
        attributions: vec![],
        external_type: None,
        options: None,
    };

    let resource = IonResource::from_endpoint(asset_id, endpoint.clone(), None).unwrap();
    assert_eq!(resource.url(), endpoint.url.unwrap());
    assert_eq!(
        resource.headers.get("Authorization"),
        Some(&"Bearer token_123".to_string())
    );
    assert_eq!(resource.retry_attempts, 1);
}

#[test]
fn ion_resource_from_endpoint_external_3dtiles() {
    let endpoint = IonEndpoint {
        endpoint_type: "3DTILES".to_string(),
        url: None,
        access_token: None,
        attributions: vec![],
        external_type: Some("3DTILES".to_string()),
        options: Some(IonEndpointOptions {
            url: Some("http://test.invalid/tileset.json".to_string()),
        }),
    };

    let resource = IonResource::from_endpoint(123890213, endpoint, None).unwrap();
    assert_eq!(resource.url(), "http://test.invalid/tileset.json");
    assert!(!resource.headers.contains_key("Authorization"));
    assert_eq!(resource.retry_attempts, 0);
    assert!(resource.retry_callback.is_none());
}

#[test]
fn ion_resource_from_endpoint_external_stk_terrain() {
    let endpoint = IonEndpoint {
        endpoint_type: "TERRAIN".to_string(),
        url: None,
        access_token: None,
        attributions: vec![],
        external_type: Some("STK_TERRAIN_SERVER".to_string()),
        options: Some(IonEndpointOptions {
            url: Some("http://test.invalid/world".to_string()),
        }),
    };

    let resource = IonResource::from_endpoint(123890213, endpoint, None).unwrap();
    assert_eq!(resource.url(), "http://test.invalid/world");
    assert!(!resource.headers.contains_key("Authorization"));
    assert_eq!(resource.retry_attempts, 0);
}

#[test]
fn ion_resource_from_endpoint_rejects_external_imagery() {
    let endpoint = IonEndpoint {
        endpoint_type: "IMAGERY".to_string(),
        url: Some("http://test.invalid/world".to_string()),
        access_token: None,
        attributions: vec![],
        external_type: Some("URL_TEMPLATE".to_string()),
        options: None,
    };

    let result = IonResource::from_endpoint(123890213, endpoint, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("use IonImageryProvider instead"));
}

#[test]
fn ion_resource_create_endpoint_resource_default_options() {
    let asset_id = 123890213;
    let endpoint_resource = IonResource::create_endpoint_resource(asset_id, None);

    let expected_url = format!(
        "{DEFAULT_ION_SERVER}v1/assets/{asset_id}/endpoint?access_token={DEFAULT_ION_ACCESS_TOKEN}"
    );
    assert_eq!(endpoint_resource.url(), expected_url);
    assert_eq!(
        endpoint_resource.headers.get("X-Cesium-Client"),
        Some(&"CesiumJS".to_string())
    );
}

#[test]
fn ion_resource_create_endpoint_resource_custom_options() {
    let asset_id = 123890213;
    let mut custom_params = BTreeMap::new();
    custom_params.insert("custom_param".to_string(), QueryValue::Single("custom_value".to_string()));

    let options = IonResourceOptions {
        server: Some("https://custom.api.cesium.com/".to_string()),
        access_token: Some("custom_token_abc".to_string()),
        query_parameters: Some(custom_params),
    };

    let endpoint_resource = IonResource::create_endpoint_resource(asset_id, Some(&options));

    let expected_url = format!(
        "https://custom.api.cesium.com/v1/assets/{asset_id}/endpoint?access_token=custom_token_abc&custom_param=custom_value"
    );
    assert_eq!(endpoint_resource.url(), expected_url);
}

#[test]
fn ion_resource_credits_with_default_and_custom_token() {
    let asset_id = 123890213;
    let endpoint = IonEndpoint {
        endpoint_type: "3DTILES".to_string(),
        url: Some("https://assets.cesium.com/123890213/tileset.json".to_string()),
        access_token: Some("some_token".to_string()),
        attributions: vec![IonAttribution {
            html: Some("<a href=\"http://test.com\">Attribution</a>".to_string()),
            text: None,
            collapsible: true,
        }],
        external_type: None,
        options: None,
    };

    // Default token endpoint resource
    let default_endpoint_resource = IonResource::create_endpoint_resource(asset_id, None);
    let default_resource =
        IonResource::new(endpoint.clone(), default_endpoint_resource).unwrap();
    let credits = default_resource.credits();
    // Should contain the attribution + defaultTokenCredit
    assert_eq!(credits.len(), 2);
    assert_eq!(credits[0].html(), "<a href=\"http://test.com\">Attribution</a>");
    assert!(credits[1].html().contains("Cesium's default ion access token"));

    // Custom token endpoint resource
    let custom_options = IonResourceOptions {
        access_token: Some("my_own_private_token".to_string()),
        ..Default::default()
    };
    let custom_endpoint_resource =
        IonResource::create_endpoint_resource(asset_id, Some(&custom_options));
    let custom_resource = IonResource::new(endpoint, custom_endpoint_resource).unwrap();
    let custom_credits = custom_resource.credits();
    // Only the asset attribution, no default token warning
    assert_eq!(custom_credits.len(), 1);
    assert_eq!(custom_credits[0].html(), "<a href=\"http://test.com\">Attribution</a>");
}

#[test]
fn ion_resource_retry_on_error_behavior() {
    let asset_id = 123890213;
    let endpoint = IonEndpoint {
        endpoint_type: "3DTILES".to_string(),
        url: Some("https://assets.cesium.com/123890213/tileset.json".to_string()),
        access_token: Some("tok".to_string()),
        attributions: vec![],
        external_type: None,
        options: None,
    };
    let endpoint_resource = IonResource::create_endpoint_resource(asset_id, None);
    let mut resource = IonResource::new(endpoint, endpoint_resource).unwrap();

    let err_404 = RequestErrorEvent::new(Some(404), None, None);
    let err_401 = RequestErrorEvent::new(Some(401), None, None);

    // Non-401 returns false
    assert!(!resource.retry_on_error(Some(&err_404)));
    assert_eq!(resource.retry_count(), 1);

    // Reset resource to test 401
    let mut resource2 = resource.clone_resource();
    assert!(resource2.retry_on_error(Some(&err_401)));
    assert_eq!(resource2.retry_count(), 1);

    // Exceeding retry_attempts (retry_attempts = 1)
    assert!(!resource2.retry_on_error(Some(&err_401)));
    assert_eq!(resource2.retry_count(), 1);
}
