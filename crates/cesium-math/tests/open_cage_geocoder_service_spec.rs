use cesium_math::{Cartesian3, GeocoderDestination, OpenCageGeocoderService};

#[test]
fn test_constructor_throws_without_url_or_key() {
    assert!(OpenCageGeocoderService::new("", "key", None).is_err());
    assert!(OpenCageGeocoderService::new("http://endpoint/", "", None).is_err());
}

#[test]
fn test_returns_geocoder_results_with_bounds() {
    let endpoint = "https://api.opencagedata.com/geocode/v1/";
    let api_key = "c2a490d593b14612aefa6ec2e6b77c47";
    let service = OpenCageGeocoderService::new(endpoint, api_key, None).unwrap();

    let json_data = r#"{
        "results": [
            {
                "bounds": {
                    "northeast": {
                        "lat": -22.6790826,
                        "lng": 14.5269016
                    },
                    "southwest": {
                        "lat": -22.6792826,
                        "lng": 14.5267016
                    }
                },
                "formatted": "Beryl's Restaurant, Woermann St, Swakopmund, Namibia",
                "geometry": {
                    "lat": -22.6795394,
                    "lng": 14.5276006
                }
            }
        ]
    }"#;

    let results = service.parse_response(json_data).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].display_name,
        "Beryl's Restaurant, Woermann St, Swakopmund, Namibia"
    );

    match &results[0].destination {
        GeocoderDestination::Rectangle(rect) => {
            assert!((rect.west.to_degrees() - 14.5267016).abs() < 1e-6);
            assert!((rect.south.to_degrees() - (-22.6792826)).abs() < 1e-6);
            assert!((rect.east.to_degrees() - 14.5269016).abs() < 1e-6);
            assert!((rect.north.to_degrees() - (-22.6790826)).abs() < 1e-6);
        }
        _ => panic!("Expected rectangle destination"),
    }
}

#[test]
fn test_returns_geocoder_result_as_cartesian3_if_no_bounds() {
    let endpoint = "https://api.opencagedata.com/geocode/v1/";
    let api_key = "c2a490d593b14612aefa6ec2e6b77c47";
    let service = OpenCageGeocoderService::new(endpoint, api_key, None).unwrap();

    let json_data = r#"{
        "results": [
            {
                "formatted": "Beryl's Restaurant, Woermann St, Swakopmund, Namibia",
                "geometry": {
                    "lat": -22.6795394,
                    "lng": 14.5276006
                }
            }
        ]
    }"#;

    let results = service.parse_response(json_data).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(
        results[0].display_name,
        "Beryl's Restaurant, Woermann St, Swakopmund, Namibia"
    );

    let expected = Cartesian3::from_degrees(14.5276006, -22.6795394, Some(0.0), None).unwrap();
    match &results[0].destination {
        GeocoderDestination::Cartesian(cart) => {
            assert!((cart.x - expected.x).abs() < 1e-4);
            assert!((cart.y - expected.y).abs() < 1e-4);
            assert!((cart.z - expected.z).abs() < 1e-4);
        }
        _ => panic!("Expected Cartesian3 destination"),
    }
}

#[test]
fn test_returns_no_geocoder_results_if_empty() {
    let endpoint = "https://api.opencagedata.com/geocode/v1/";
    let api_key = "c2a490d593b14612aefa6ec2e6b77c47";
    let service = OpenCageGeocoderService::new(endpoint, api_key, None).unwrap();

    let json_data = r#"{"results": []}"#;
    let results = service.parse_response(json_data).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_credit_returns_expected_value() {
    let endpoint = "https://api.opencagedata.com/geocode/v1/";
    let api_key = "c2a490d593b14612aefa6ec2e6b77c47";
    let service = OpenCageGeocoderService::new(endpoint, api_key, None).unwrap();

    let credit = service.credit();
    assert_eq!(
        credit.html(),
        r#"Geodata copyright <a href="https://www.openstreetmap.org/">OpenStreetMap</a> contributors"#
    );
    assert!(!credit.show_on_screen());
}
