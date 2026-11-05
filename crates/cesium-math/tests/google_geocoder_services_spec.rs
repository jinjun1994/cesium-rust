use cesium_math::{GoogleGeocoderService, GoogleGeocoderServiceOptions};

#[test]
fn test_constructor_throws_without_key() {
    let result = GoogleGeocoderService::new(GoogleGeocoderServiceOptions {
        key: "".to_string(),
    });
    assert!(result.is_err());
}

#[test]
fn test_constructor_sets_key_on_resource() {
    let key = "test-api-key-12345";
    let service = GoogleGeocoderService::new(GoogleGeocoderServiceOptions {
        key: key.to_string(),
    })
    .unwrap();

    assert_eq!(
        service.resource_url(),
        "https://maps.googleapis.com/maps/api/geocode/json?key=test-api-key-12345"
    );
    assert_eq!(service.key(), key);
}

#[test]
fn test_geocode_returns_results_for_status_ok() {
    let key = "test-key";
    let service = GoogleGeocoderService::new(GoogleGeocoderServiceOptions {
        key: key.to_string(),
    })
    .unwrap();

    let json_data = r#"{
        "results": [
            {
                "formatted_address": "1600 Amphitheatre Pkwy, Mountain View, CA 94043, USA",
                "geometry": {
                    "viewport": {
                        "northeast": {
                            "lat": 37.4237349802915,
                            "lng": -122.083183169709
                        },
                        "southwest": {
                            "lat": 37.4210370197085,
                            "lng": -122.085881130292
                        }
                    }
                }
            }
        ],
        "status": "OK"
    }"#;

    let results = service.parse_response(json_data).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].display_name,
        "1600 Amphitheatre Pkwy, Mountain View, CA 94043, USA"
    );

    let dest = &results[0].destination;
    assert!((dest.west.to_degrees() - (-122.085881130292)).abs() < 1e-6);
    assert!((dest.south.to_degrees() - 37.4210370197085).abs() < 1e-6);
    assert!((dest.east.to_degrees() - (-122.083183169709)).abs() < 1e-6);
    assert!((dest.north.to_degrees() - 37.4237349802915).abs() < 1e-6);

    assert_eq!(
        results[0].attribution.html,
        r#"<img alt="Google" src="https://assets.ion.cesium.com/google-credit.png" style="vertical-align:-5px">"#
    );
    assert!(!results[0].attribution.collapsible);
}

#[test]
fn test_returns_empty_array_for_zero_results() {
    let service = GoogleGeocoderService::new(GoogleGeocoderServiceOptions {
        key: "key".to_string(),
    })
    .unwrap();

    let json_data = r#"{"status": "ZERO_RESULTS"}"#;
    let results = service.parse_response(json_data).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_throws_on_error_status() {
    let service = GoogleGeocoderService::new(GoogleGeocoderServiceOptions {
        key: "key".to_string(),
    })
    .unwrap();

    let json_data = r#"{"status": "REQUEST_DENIED", "error_message": "Invalid API key"}"#;
    let results = service.parse_response(json_data);
    assert!(results.is_err());
}
