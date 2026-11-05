use cesium_math::{
    GeocodeType, IonGeocodeProviderType, IonGeocoderService, IonGeocoderServiceOptions,
    DEFAULT_ION_ACCESS_TOKEN, DEFAULT_ION_SERVER,
};

#[test]
fn test_creates_with_default_parameters() {
    let service = IonGeocoderService::new(None).unwrap();
    assert_eq!(service.access_token(), DEFAULT_ION_ACCESS_TOKEN);
    assert_eq!(service.server(), DEFAULT_ION_SERVER);
    assert_eq!(
        service.geocode_provider_type(),
        IonGeocodeProviderType::Default
    );
}

#[test]
fn test_creates_with_specified_parameters() {
    let access_token = "123456".to_string();
    let server = "http://not.ion.invalid/".to_string();
    let geocode_provider_type = IonGeocodeProviderType::Google;

    let service = IonGeocoderService::new(Some(IonGeocoderServiceOptions {
        access_token: Some(access_token.clone()),
        server: Some(server.clone()),
        geocode_provider_type: Some(geocode_provider_type),
    }))
    .unwrap();

    assert_eq!(service.access_token(), access_token);
    assert_eq!(service.server(), server);
    assert_eq!(service.geocode_provider_type(), geocode_provider_type);
}

#[test]
fn test_credit_returns_expected_value() {
    let service = IonGeocoderService::new(None).unwrap();
    assert!(service.credit().is_none());
}

#[test]
fn test_setting_geocode_provider_type_updates_query_parameters() {
    let mut service = IonGeocoderService::new(Some(IonGeocoderServiceOptions {
        geocode_provider_type: Some(IonGeocodeProviderType::Default),
        ..Default::default()
    }))
    .unwrap();

    // GOOGLE
    service
        .set_geocode_provider_type(IonGeocodeProviderType::Google)
        .unwrap();
    assert_eq!(
        service
            .pelias()
            .query_parameters()
            .get("geocoder")
            .map(|s| s.as_str()),
        Some("google")
    );

    // BING
    service
        .set_geocode_provider_type(IonGeocodeProviderType::Bing)
        .unwrap();
    assert_eq!(
        service
            .pelias()
            .query_parameters()
            .get("geocoder")
            .map(|s| s.as_str()),
        Some("bing")
    );

    // DEFAULT removes the query parameter
    service
        .set_geocode_provider_type(IonGeocodeProviderType::Default)
        .unwrap();
    assert!(service
        .pelias()
        .query_parameters()
        .get("geocoder")
        .is_none());
}

#[test]
fn test_throws_if_setting_invalid_geocode_provider_type() {
    assert!("junk".parse::<IonGeocodeProviderType>().is_err());
    assert!("GOOGLE".parse::<IonGeocodeProviderType>().is_ok());
    assert!("bing".parse::<IonGeocodeProviderType>().is_ok());
}

#[test]
fn test_calls_inner_geocoder_and_returns_result() {
    let service = IonGeocoderService::new(None).unwrap();
    let url = service.build_url("some query", GeocodeType::Search);
    assert!(url.contains("v1/geocode/search?text=some query"));
    assert!(url.contains("access_token="));

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
}
