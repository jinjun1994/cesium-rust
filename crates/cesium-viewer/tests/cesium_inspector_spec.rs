use cesium_viewer::widgets::{CesiumInspector, CesiumInspectorContainer};

#[test]
fn can_create_and_destroy() {
    let container = CesiumInspectorContainer::new("testContainer", true);
    let widget = CesiumInspector::new(Some(container.clone()), Some("main_scene")).unwrap();

    assert_eq!(widget.container().id, "testContainer");
    assert_eq!(widget.view_model().scene_id(), "main_scene");
    assert!(!widget.is_destroyed());

    widget.destroy();
    assert!(widget.is_destroyed());
}

#[test]
fn constructor_throws_with_no_element() {
    let err = CesiumInspector::new(None, Some("main_scene")).unwrap_err();
    assert!(err.message.contains("container is required."));
}

#[test]
fn constructor_throws_with_string_element_that_does_not_exist() {
    let container = CesiumInspectorContainer::new("does_not_exist", false);
    let err = CesiumInspector::new(Some(container), Some("main_scene")).unwrap_err();
    assert!(err.message.contains("does not exist"));
}

#[test]
fn constructor_throws_with_no_scene() {
    let container = CesiumInspectorContainer::new("testContainer", true);
    let err = CesiumInspector::new(Some(container), None::<&str>).unwrap_err();
    assert!(err.message.contains("scene is required."));
}
