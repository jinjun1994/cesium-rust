use cesium_viewer::widgets::viewer::{
    viewer_drag_drop_mixin, Viewer, ViewerContainer, ViewerDragDropOptions,
};

#[test]
fn test_viewer_drag_drop_mixin_sets_default_values() {
    let container = ViewerContainer::new("container", true);
    let mut viewer = Viewer::new(Some(container), None).unwrap();

    viewer_drag_drop_mixin(Some(&mut viewer), None).expect("mixin should succeed");
    assert_eq!(viewer.drop_target.as_deref(), Some("container"));
    assert!(viewer.drop_enabled);
    assert!(viewer.clear_on_drop);
    assert!(viewer.clamp_to_ground);
    assert!(viewer.fly_to_on_drop);

    viewer.destroy();
}

#[test]
fn test_viewer_drag_drop_mixin_sets_option_values() {
    let container = ViewerContainer::new("container", true);
    let mut viewer = Viewer::new(Some(container), None).unwrap();

    let options = ViewerDragDropOptions {
        drop_target: Some("document.body".to_string()),
        clear_on_drop: false,
        clamp_to_ground: false,
        fly_to_on_drop: false,
    };

    viewer_drag_drop_mixin(Some(&mut viewer), Some(options)).unwrap();
    assert_eq!(viewer.drop_target.as_deref(), Some("document.body"));
    assert!(viewer.drop_enabled);
    assert!(!viewer.clear_on_drop);
    assert!(!viewer.clamp_to_ground);
    assert!(!viewer.fly_to_on_drop);

    viewer.destroy();
}

#[test]
fn test_viewer_drag_drop_mixin_throws_if_not_given_a_viewer() {
    let err = viewer_drag_drop_mixin(None, None);
    assert!(err.is_err());
}
