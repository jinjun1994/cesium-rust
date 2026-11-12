use cesium_viewer::widgets::InfoBoxViewModel;

#[test]
fn constructor_sets_expected_values() {
    let view_model = InfoBoxViewModel::new();
    assert!(!view_model.enable_camera);
    assert!(!view_model.is_camera_tracking);
    assert!(!view_model.show_info);
    assert_eq!(view_model.max_height_offset(0.0), "500px");
}

#[test]
fn sets_description() {
    let safe_string = "<p>This is a test.</p>";
    let mut view_model = InfoBoxViewModel::new();
    view_model.description = safe_string.to_string();
    assert_eq!(view_model.description, safe_string);
}

#[test]
fn indicates_missing_description() {
    let mut view_model = InfoBoxViewModel::new();
    assert!(view_model._bodyless());
    view_model.description = "Testing".to_string();
    assert!(!view_model._bodyless());
}

#[test]
fn camera_icon_changes_when_tracking_is_not_available_unless_due_to_active_tracking() {
    let mut view_model = InfoBoxViewModel::new();
    view_model.enable_camera = true;
    view_model.is_camera_tracking = false;
    let enabled_tracking_path = view_model.camera_icon_path();

    view_model.enable_camera = false;
    view_model.is_camera_tracking = false;
    assert_ne!(view_model.camera_icon_path(), enabled_tracking_path);

    let disable_tracking_path = view_model.camera_icon_path();

    view_model.enable_camera = true;
    view_model.is_camera_tracking = true;
    assert_eq!(view_model.camera_icon_path(), disable_tracking_path);

    view_model.enable_camera = false;
    view_model.is_camera_tracking = true;
    assert_eq!(view_model.camera_icon_path(), disable_tracking_path);
}
