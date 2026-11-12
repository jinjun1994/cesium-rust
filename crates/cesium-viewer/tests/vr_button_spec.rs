use cesium_viewer::widgets::vr_button::{VRButton, VRButtonContainer};
use cesium_viewer::widgets::vr_button_view_model::VRScene;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_vr_button_constructor_sets_default_values() {
    let scene = Rc::new(RefCell::new(VRScene::new()));
    let container = VRButtonContainer::new("body", true);

    let vr_button = VRButton::new(Some(container.clone()), Some(scene), None::<&str>)
        .expect("Constructor should succeed");

    assert_eq!(vr_button.container().id, "body");
    assert_eq!(vr_button.view_model().vr_element(), "document.body");
    assert!(!vr_button.is_destroyed());

    vr_button.destroy();
    assert!(vr_button.is_destroyed());
}

#[test]
fn test_vr_button_constructor_sets_expected_values() {
    let scene = Rc::new(RefCell::new(VRScene::new()));
    let container = VRButtonContainer::new("body", true);

    let vr_button = VRButton::new(
        Some(container),
        Some(scene),
        Some("testElement"),
    )
    .expect("Constructor should succeed");

    assert_eq!(vr_button.view_model().vr_element(), "testElement");
    vr_button.destroy();
}

#[test]
fn test_vr_button_constructor_works_with_string_id_container() {
    let scene = Rc::new(RefCell::new(VRScene::new()));
    let mut lookup_map = HashMap::new();
    lookup_map.insert(
        "testElement".to_string(),
        VRButtonContainer::new("testElement", true),
    );

    let vr_button = VRButton::from_id(
        "testElement",
        Some(scene),
        None::<&str>,
        |id| lookup_map.get(id).cloned(),
    )
    .expect("from_id should succeed");

    assert_eq!(vr_button.container().id, "testElement");
    vr_button.destroy();
}

#[test]
fn test_vr_button_throws_if_container_is_undefined() {
    let scene = Rc::new(RefCell::new(VRScene::new()));
    let err = VRButton::new(None, Some(scene), None::<&str>);
    assert!(err.is_err());
}

#[test]
fn test_vr_button_throws_if_container_string_is_undefined() {
    let scene = Rc::new(RefCell::new(VRScene::new()));
    let err = VRButton::from_id("testElement", Some(scene), None::<&str>, |_| None);
    assert!(err.is_err());
}

#[test]
fn test_vr_button_throws_if_scene_is_undefined() {
    let container = VRButtonContainer::new("body", true);
    let err = VRButton::new(Some(container), None, None::<&str>);
    assert!(err.is_err());
}
