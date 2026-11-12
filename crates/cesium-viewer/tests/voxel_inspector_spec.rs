use cesium_viewer::widgets::voxel_inspector::{VoxelInspector, VoxelInspectorContainer};
use std::collections::HashMap;

#[test]
fn test_voxel_inspector_can_create_and_destroy() {
    let container = VoxelInspectorContainer::new("testContainer", true);
    let mut lookup_map = HashMap::new();
    lookup_map.insert("testContainer".to_string(), container.clone());

    let widget = VoxelInspector::from_id(
        "testContainer",
        Some("scene_0"),
        |id| lookup_map.get(id).cloned(),
    )
    .expect("Should create VoxelInspector");

    assert_eq!(widget.container().id, "testContainer");
    assert_eq!(widget.view_model().scene(), "scene_0");
    assert!(!widget.is_destroyed());

    widget.destroy();
    assert!(widget.is_destroyed());
}

#[test]
fn test_voxel_inspector_constructor_throws_with_no_element() {
    let err = VoxelInspector::new(None, Some("scene_0"));
    assert!(err.is_err());
}

#[test]
fn test_voxel_inspector_constructor_throws_with_string_element_that_does_not_exist() {
    let err = VoxelInspector::from_id("does not exist", Some("scene_0"), |_| None);
    assert!(err.is_err());
}

#[test]
fn test_voxel_inspector_constructor_throws_with_no_scene() {
    let container = VoxelInspectorContainer::new("testContainer", true);
    let err = VoxelInspector::new(Some(container), None::<&str>);
    assert!(err.is_err());
}

#[test]
fn test_voxel_inspector_view_model_toggle_sections_and_properties() {
    let container = VoxelInspectorContainer::new("testContainer", true);
    let widget = VoxelInspector::new(Some(container), Some("scene_0")).unwrap();
    let vm = widget.view_model();

    assert!(vm.display_visible());
    vm.toggle_display();
    assert!(!vm.display_visible());

    assert!(!vm.transform_visible());
    vm.toggle_transform();
    assert!(vm.transform_visible());

    assert!(!vm.clipping_visible());
    vm.toggle_clipping();
    assert!(vm.clipping_visible());

    assert!(!vm.shader_visible());
    vm.toggle_shader();
    assert!(vm.shader_visible());

    assert_eq!(vm.screen_space_error(), 16.0);
    vm.set_screen_space_error(32.0);
    assert_eq!(vm.screen_space_error(), 32.0);
}
