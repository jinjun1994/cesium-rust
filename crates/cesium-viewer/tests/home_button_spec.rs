use cesium_viewer::widgets::{HomeButton, HomeButtonContainer};

#[test]
fn constructor_sets_default_values() {
    let container = HomeButtonContainer::new("bodyContainer", true);
    let button = HomeButton::new(Some(container.clone()), Some("main_scene"), None).unwrap();

    assert_eq!(button.container().id, "bodyContainer");
    assert_eq!(button.view_model().scene_id(), "main_scene");
    assert!(!button.is_destroyed());

    button.destroy();
    assert!(button.is_destroyed());
}

#[test]
fn constructor_sets_expected_values() {
    let container = HomeButtonContainer::new("bodyContainer", true);
    let button = HomeButton::new(Some(container), Some("main_scene"), Some(2.5)).unwrap();
    assert_eq!(button.view_model().duration(), Some(2.5));
    button.destroy();
}

#[test]
fn constructor_works_with_string_id_container() {
    let container = HomeButtonContainer::new("testElement", true);
    let button = HomeButton::new(Some(container), Some("main_scene"), None).unwrap();
    assert_eq!(button.container().id, "testElement");
    button.destroy();
}

#[test]
fn throws_if_container_is_undefined() {
    assert!(HomeButton::new(None, Some("main_scene"), None).is_err());
}

#[test]
fn throws_if_container_string_is_invalid() {
    let container = HomeButtonContainer::new("does_not_exist", false);
    assert!(HomeButton::new(Some(container), Some("main_scene"), None).is_err());
}
