use cesium_math::{BingMapsGeocoderService, BingMapsGeocoderServiceOptions};

#[test]
fn test_returns_geocoder_results() {
    let service = BingMapsGeocoderService::new(BingMapsGeocoderServiceOptions {
        key: "not_the_real_key;".to_string(),
        culture: None,
    })
    .unwrap();

    let url = service.build_url("some query");
    assert!(url.contains("query=some query"));
    assert!(url.contains("key=not_the_real_key;"));
    assert!(!url.contains("culture="));

    let json_data = r#"{
        "resourceSets": [
            {
                "resources": [
                    {
                        "name": "a",
                        "bbox": [3.0, 3.0, 32.0, 4.0]
                    }
                ]
            }
        ]
    }"#;

    let results = service.parse_response(json_data).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].display_name, "a");
    // bbox: [south=3.0, west=3.0, north=32.0, east=4.0]
    let dest = &results[0].destination;
    assert!((dest.south.to_degrees() - 3.0).abs() < 1e-6);
    assert!((dest.west.to_degrees() - 3.0).abs() < 1e-6);
    assert!((dest.north.to_degrees() - 32.0).abs() < 1e-6);
    assert!((dest.east.to_degrees() - 4.0).abs() < 1e-6);
}

#[test]
fn test_uses_supplied_culture() {
    let service = BingMapsGeocoderService::new(BingMapsGeocoderServiceOptions {
        key: "not_the_real_key;".to_string(),
        culture: Some("ja".to_string()),
    })
    .unwrap();

    let url = service.build_url("some query");
    assert!(url.contains("query=some query"));
    assert!(url.contains("key=not_the_real_key;"));
    assert!(url.contains("culture=ja"));
}

#[test]
fn test_returns_no_results_when_empty() {
    let service = BingMapsGeocoderService::new(BingMapsGeocoderServiceOptions {
        key: "".to_string(),
        culture: None,
    })
    .unwrap();

    // No resource sets
    let json_empty_sets = r#"{"resourceSets": []}"#;
    let res = service.parse_response(json_empty_sets).unwrap();
    assert!(res.is_empty());

    // Empty resources
    let json_empty_res = r#"{"resourceSets": [{"resources": []}]}"#;
    let res = service.parse_response(json_empty_res).unwrap();
    assert!(res.is_empty());
}

#[test]
fn test_credit_returns_expected_value() {
    let service = BingMapsGeocoderService::new(BingMapsGeocoderServiceOptions {
        key: "".to_string(),
        culture: None,
    })
    .unwrap();

    let credit = service.credit();
    assert_eq!(
        credit.html(),
        r#"<img src="http:\/\/dev.virtualearth.net\/Branding\/logo_powered_by.png"/>"#
    );
    assert!(!credit.show_on_screen());
}
