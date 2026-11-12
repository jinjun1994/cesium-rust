use cesium_viewer::widgets::{
    Geocoder, GeocoderContainer, GeocoderViewModelOptions,
};

#[test]
fn constructor_sets_expected_properties() {
    let container = GeocoderContainer::new("geocoder_container", true);
    let geocoder = Geocoder::new(
        Some(container.clone()),
        Some(GeocoderViewModelOptions {
            scene_id: Some("scene_main".to_string()),
            flight_duration: Some(1234.0),
            ..Default::default()
        }),
    )
    .unwrap();

    assert_eq!(geocoder.container().id, "geocoder_container");
    assert_eq!(geocoder.view_model().scene_id(), "scene_main");
    assert_eq!(geocoder.view_model().flight_duration(), 1234.0);
    assert!(!geocoder.is_destroyed());

    geocoder.destroy();
    assert!(geocoder.is_destroyed());
}

#[test]
fn can_create_and_destroy() {
    let container = GeocoderContainer::new("testContainer", true);
    let widget = Geocoder::new(
        Some(container),
        Some(GeocoderViewModelOptions {
            scene_id: Some("scene_main".to_string()),
            ..Default::default()
        }),
    )
    .unwrap();

    assert!(!widget.is_destroyed());
    widget.destroy();
    assert!(widget.is_destroyed());
}

#[test]
fn constructor_throws_with_no_scene() {
    let container = GeocoderContainer::new("testContainer", true);
    let err = Geocoder::new(Some(container), Some(GeocoderViewModelOptions::default())).unwrap_err();
    assert!(err.message.contains("scene is required."));
}

#[test]
fn constructor_throws_with_no_element() {
    let options = GeocoderViewModelOptions {
        scene_id: Some("scene_main".to_string()),
        ..Default::default()
    };
    let err = Geocoder::new(None, Some(options)).unwrap_err();
    assert!(err.message.contains("container is required."));
}

#[test]
fn constructor_throws_with_string_element_that_does_not_exist() {
    let container = GeocoderContainer::new("does_not_exist", false);
    let options = GeocoderViewModelOptions {
        scene_id: Some("scene_main".to_string()),
        ..Default::default()
    };
    let err = Geocoder::new(Some(container), Some(options)).unwrap_err();
    assert!(err.message.contains("does not exist"));
}
