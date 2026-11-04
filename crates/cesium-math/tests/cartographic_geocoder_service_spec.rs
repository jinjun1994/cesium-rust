use cesium_math::{Cartesian3, CartographicGeocoderService};

#[test]
fn test_geocoder_ns_ew_input() {
    let service = CartographicGeocoderService::new();
    let query = "35N 75W";
    let results = service.geocode(query);
    assert_eq!(results.len(), 1);
    let expected = Cartesian3::from_degrees(-75.0, 35.0, Some(300.0), None).unwrap();
    assert_eq!(results[0].destination, expected);
}

#[test]
fn test_geocoder_ew_ns_input() {
    let service = CartographicGeocoderService::new();
    let query = "75W 35N";
    let results = service.geocode(query);
    assert_eq!(results.len(), 1);
    let expected = Cartesian3::from_degrees(-75.0, 35.0, Some(300.0), None).unwrap();
    assert_eq!(results[0].destination, expected);
}

#[test]
fn test_geocoder_long_lat_height_input() {
    let service = CartographicGeocoderService::new();
    let query = " 1.0, 2.0, 3.0 ";
    let results = service.geocode(query);
    assert_eq!(results.len(), 1);
    let expected = Cartesian3::from_degrees(1.0, 2.0, Some(3.0), None).unwrap();
    assert_eq!(results[0].destination, expected);
}

#[test]
fn test_geocoder_long_lat_input() {
    let service = CartographicGeocoderService::new();
    let query = " 1.0, 2.0 ";
    let results = service.geocode(query);
    assert_eq!(results.len(), 1);
    let expected = Cartesian3::from_degrees(1.0, 2.0, Some(300.0), None).unwrap();
    assert_eq!(results[0].destination, expected);
}

#[test]
fn test_geocoder_invalid_inputs_return_empty() {
    let service = CartographicGeocoderService::new();

    // Only longitudinal coordinates
    assert_eq!(service.geocode(" 1e 1e ").len(), 0);

    // Only one NSEW coordinate
    assert_eq!(service.geocode(" 1e 1 ").len(), 0);

    // Only one number
    assert_eq!(service.geocode(" 2.0 ").len(), 0);

    // Non-numeric string
    assert_eq!(service.geocode(" aoeu ").len(), 0);
}
