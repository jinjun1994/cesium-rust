//! Spec matching CesiumJS `Specs/Scene/createWorldImageryAsyncSpec.js` (Apache-2.0).

use cesium_math::IonEndpoint;
use cesium_renderer::ion_imagery_provider::{ExternalImageryType, IonImageryProvider};
use cesium_renderer::{
    create_world_imagery_async, create_world_imagery_async_with_fetcher,
    CreateWorldImageryOptions, IonWorldImageryStyle,
};

#[test]
fn test_resolves_to_ion_imagery_provider_instance_with_default_parameters() {
    IonImageryProvider::clear_cache();

    let provider = create_world_imagery_async(None).expect("create_world_imagery_async should succeed");

    assert_eq!(provider.asset_id, 2); // Default style is AERIAL (2)
    assert_eq!(provider.endpoint.endpoint_type, "IMAGERY");
    assert_eq!(provider.external_type, Some(ExternalImageryType::Bing));
}

#[test]
fn test_resolves_with_custom_styles() {
    IonImageryProvider::clear_cache();

    // Aerial with labels (asset id 3)
    let p_labels = create_world_imagery_async(Some(CreateWorldImageryOptions {
        style: Some(IonWorldImageryStyle::AerialWithLabels),
        provider_options: None,
    }))
    .expect("Should succeed with AerialWithLabels");
    assert_eq!(p_labels.asset_id, 3);

    // Road (asset id 4)
    let p_road = create_world_imagery_async(Some(CreateWorldImageryOptions {
        style: Some(IonWorldImageryStyle::Road),
        provider_options: None,
    }))
    .expect("Should succeed with Road");
    assert_eq!(p_road.asset_id, 4);
}

#[test]
fn test_resolves_with_custom_fetcher() {
    IonImageryProvider::clear_cache();

    let provider = create_world_imagery_async_with_fetcher(
        Some(CreateWorldImageryOptions {
            style: Some(IonWorldImageryStyle::Aerial),
            provider_options: None,
        }),
        |asset_id, _opts| {
            assert_eq!(asset_id, 2);
            Ok(IonEndpoint {
                endpoint_type: "IMAGERY".to_string(),
                url: Some("https://mock.ion.test/tiles".to_string()),
                access_token: None,
                external_type: Some("BING".to_string()),
                attributions: vec![],
                options: None,
            })
        },
    )
    .expect("Custom fetcher should resolve");

    assert_eq!(provider.asset_id, 2);
    assert_eq!(provider.endpoint.url.as_deref(), Some("https://mock.ion.test/tiles"));
}

#[test]
fn test_ion_world_imagery_style_from_asset_id() {
    assert_eq!(IonWorldImageryStyle::from_asset_id(2).unwrap(), IonWorldImageryStyle::Aerial);
    assert_eq!(IonWorldImageryStyle::from_asset_id(3).unwrap(), IonWorldImageryStyle::AerialWithLabels);
    assert_eq!(IonWorldImageryStyle::from_asset_id(4).unwrap(), IonWorldImageryStyle::Road);
    assert!(IonWorldImageryStyle::from_asset_id(999).is_err());
}
