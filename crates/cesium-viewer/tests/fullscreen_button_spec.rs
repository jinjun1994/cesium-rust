use cesium_viewer::widgets::{FullscreenButton, FullscreenButtonContainer};

#[test]
fn constructor_sets_default_values() {
    let container = FullscreenButtonContainer::new("bodyContainer", true);
    let button = FullscreenButton::new(Some(container.clone()), None::<&str>).unwrap();

    assert_eq!(button.container().id, "bodyContainer");
    assert_eq!(button.view_model().fullscreen_element(), "document.body");
    assert!(!button.is_destroyed());

    button.destroy();
    assert!(button.is_destroyed());
}

#[test]
fn constructor_sets_expected_values() {
    let container = FullscreenButtonContainer::new("bodyContainer", true);
    let button = FullscreenButton::new(Some(container), Some("canvasElement")).unwrap();
    assert_eq!(button.view_model().fullscreen_element(), "canvasElement");
    button.destroy();
}

#[test]
fn constructor_works_with_string_id_container() {
    let container = FullscreenButtonContainer::new("testElement", true);
    let button = FullscreenButton::new(Some(container), None::<&str>).unwrap();
    assert_eq!(button.container().id, "testElement");
    button.destroy();
}

#[test]
fn throws_if_container_is_undefined() {
    assert!(FullscreenButton::new(None, None::<&str>).is_err());
}

#[test]
fn throws_if_container_string_is_invalid() {
    let container = FullscreenButtonContainer::new("does_not_exist", false);
    assert!(FullscreenButton::new(Some(container), None::<&str>).is_err());
}
