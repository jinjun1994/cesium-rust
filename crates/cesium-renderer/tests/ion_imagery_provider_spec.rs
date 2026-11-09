use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use cesium_math::{GeoRectangle, IonAttribution, IonEndpoint, IonEndpointOptions};
use cesium_renderer::imagery_layer::ImageryProvider;
use cesium_renderer::ion_imagery_provider::{
    ExternalImageryType, IonImageryProvider, IonImageryProviderOptions,
};

fn default_endpoint() -> IonEndpoint {
    IonEndpoint {
        endpoint_type: "IMAGERY".to_string(),
        url: Some("http://test.invalid/layer".to_string()),
        access_token: Some("not_really_a_refresh_token".to_string()),
        attributions: vec![],
        external_type: None,
        options: None,
    }
}

#[test]
fn from_asset_id_throws_with_invalid_asset_id() {
    assert!(IonImageryProvider::from_endpoint(0, default_endpoint(), None).is_err());
    assert!(IonImageryProvider::from_endpoint(-1, default_endpoint(), None).is_err());
}

#[test]
fn from_asset_id_throws_with_non_imagery_asset() {
    let mut ep = default_endpoint();
    ep.endpoint_type = "3DTILES".to_string();

    let err = IonImageryProvider::from_endpoint(12335, ep, None).unwrap_err();
    assert_eq!(
        err.message,
        "Cesium ion asset 12335 is not an imagery asset."
    );
}

#[test]
fn from_asset_id_rejects_with_unknown_external_asset_type() {
    let mut ep = default_endpoint();
    ep.external_type = Some("TUBELCANE".to_string());

    let err = IonImageryProvider::from_endpoint(12335, ep, None).unwrap_err();
    assert_eq!(
        err.message,
        "Unrecognized Cesium ion imagery type: TUBELCANE"
    );
}

#[test]
fn from_asset_id_resolves_to_created_provider_and_propagates_properties() {
    let ep = default_endpoint();
    let provider = IonImageryProvider::from_endpoint(12335, ep, None).unwrap();

    assert_eq!(provider.asset_id, 12335);
    assert_eq!(provider.rectangle(), GeoRectangle::MAXIMUM);
    assert_eq!(provider.tile_width(), 256);
    assert_eq!(provider.tile_height(), 256);
    assert_eq!(provider.minimum_level(), 0);
    assert!(provider.maximum_level().is_none());
    assert!(provider.has_alpha_channel());
    assert!(provider.is_ready());
    assert!(provider.external_type.is_none());
}

#[test]
fn supports_all_external_imagery_types() {
    let types = [
        ("ARCGIS_MAPSERVER", ExternalImageryType::ArcGisMapServer),
        ("BING", ExternalImageryType::Bing),
        ("GOOGLE_EARTH", ExternalImageryType::GoogleEarth),
        ("MAPBOX", ExternalImageryType::Mapbox),
        ("SINGLE_TILE", ExternalImageryType::SingleTile),
        ("TMS", ExternalImageryType::Tms),
        ("URL_TEMPLATE", ExternalImageryType::UrlTemplate),
        ("WMS", ExternalImageryType::Wms),
        ("WMTS", ExternalImageryType::Wmts),
        ("GOOGLE_2D_MAPS", ExternalImageryType::Google2DMaps),
        ("AZURE_MAPS", ExternalImageryType::AzureMaps),
    ];

    for (type_name, expected_type) in types {
        let mut ep = default_endpoint();
        ep.external_type = Some(type_name.to_string());
        ep.options = Some(IonEndpointOptions {
            url: Some("http://test.invalid".to_string()),
        });

        let provider = IonImageryProvider::from_endpoint(12335, ep, None).unwrap();
        assert_eq!(provider.external_type, Some(expected_type));
    }
}

#[test]
fn caches_endpoint_across_repeated_requests() {
    IonImageryProvider::clear_cache();

    let fetch_count = Arc::new(AtomicUsize::new(0));

    let fc_clone1 = fetch_count.clone();
    let fetcher = move |_asset_id: i64, _options: Option<&IonImageryProviderOptions>| {
        fc_clone1.fetch_add(1, Ordering::SeqCst);
        let mut ep = default_endpoint();
        ep.external_type = Some("TMS".to_string());
        Ok(ep)
    };

    let opts = IonImageryProviderOptions {
        access_token: Some("my_token".to_string()),
        server: Some("https://api.cesium.com".to_string()),
    };

    let p1 = IonImageryProvider::from_asset_id_with_fetcher(12335, Some(&opts), fetcher).unwrap();
    assert_eq!(p1.external_type, Some(ExternalImageryType::Tms));
    assert_eq!(fetch_count.load(Ordering::SeqCst), 1);

    // Second call with same parameters should hit cache
    let fc_clone2 = fetch_count.clone();
    let fetcher2 = move |_asset_id: i64, _options: Option<&IonImageryProviderOptions>| {
        fc_clone2.fetch_add(1, Ordering::SeqCst);
        Ok(default_endpoint())
    };

    let p2 = IonImageryProvider::from_asset_id_with_fetcher(12335, Some(&opts), fetcher2).unwrap();
    assert_eq!(p2.external_type, Some(ExternalImageryType::Tms));
    assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
}

#[test]
fn processes_server_sent_credits_and_collapsible_flag() {
    let mut ep = default_endpoint();
    ep.attributions = vec![
        IonAttribution {
            html: Some("<a href=\"http://test.invalid/\">Text</a>".to_string()),
            text: None,
            collapsible: false,
        },
        IonAttribution {
            html: None,
            text: Some("Collapsible Attribution".to_string()),
            collapsible: true,
        },
    ];

    let provider = IonImageryProvider::from_endpoint(12335, ep, None).unwrap();
    let credits = provider.get_tile_credits(0, 0, 0);

    assert_eq!(credits.len(), 2);
    assert_eq!(credits[0].html(), "<a href=\"http://test.invalid/\">Text</a>");
    assert!(credits[0].show_on_screen()); // !collapsible

    assert_eq!(credits[1].html(), "Collapsible Attribution");
    assert!(!credits[1].show_on_screen()); // collapsible
}
