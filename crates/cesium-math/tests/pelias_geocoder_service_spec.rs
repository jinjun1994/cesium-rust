use cesium_math::{GeocodeType, GeocoderDestination, PeliasGeocoderService};

#[test]
fn test_constructor_throws_without_url() {
    assert!(PeliasGeocoderService::new("").is_err());
}

#[test]
fn test_returns_geocoder_results_point() {
    let service = PeliasGeocoderService::new("http://test.invalid/v1/").unwrap();

    let json_data = r#"{
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [-75.172489, 39.927828]
                },
                "properties": {
                    "label": "1826 S 16th St, Philadelphia, PA, USA"
                }
            }
        ]
    }"#;

    let results = service.parse_response(json_data).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].display_name,
        "1826 S 16th St, Philadelphia, PA, USA"
    );

    match &results[0].destination {
        GeocoderDestination::Cartesian(cart) => {
            assert!(cart.x.abs() > 100.0);
        }
        _ => panic!("Expected Cartesian destination"),
    }
}

#[test]
fn test_returns_geocoder_results_with_attributions() {
    let service = PeliasGeocoderService::new("http://test.invalid/v1/").unwrap();

    let json_data = r#"{
        "attributions": [
            {
                "html": "Credit",
                "collapsible": true
            }
        ],
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [-75.172489, 39.927828]
                },
                "properties": {
                    "label": "1826 S 16th St, Philadelphia, PA, USA"
                }
            }
        ]
    }"#;

    let results = service.parse_response(json_data).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].attributions.len(), 1);
    assert_eq!(results[0].attributions[0].html, "Credit");
    assert!(results[0].attributions[0].collapsible);
}

#[test]
fn test_returns_no_geocoder_results_if_empty() {
    let service = PeliasGeocoderService::new("http://test.invalid/v1/").unwrap();

    let json_data = r#"{"features": []}"#;
    let results = service.parse_response(json_data).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_calls_search_endpoint_if_specified() {
    let service = PeliasGeocoderService::new("http://test.invalid/v1/").unwrap();
    let url = service.build_url("some query", GeocodeType::Search);
    assert_eq!(url, "http://test.invalid/v1/search?text=some query");
}

#[test]
fn test_calls_autocomplete_endpoint_if_specified() {
    let service = PeliasGeocoderService::new("http://test.invalid/v1/").unwrap();
    let url = service.build_url("some query", GeocodeType::Autocomplete);
    assert_eq!(url, "http://test.invalid/v1/autocomplete?text=some query");
}
