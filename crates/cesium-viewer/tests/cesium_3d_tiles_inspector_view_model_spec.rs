use cesium_viewer::widgets::{
    Cesium3DTilesInspectorViewModel, InspectorTileset,
};

#[test]
fn can_create_and_destroy() {
    let vm = Cesium3DTilesInspectorViewModel::new(
        Some("scene_01"),
        Some("perf_div"),
    )
    .unwrap();

    assert_eq!(vm.scene_id(), "scene_01");
    assert_eq!(vm.performance_container_id(), "perf_div");
    assert!(!vm.is_destroyed());

    vm.destroy();
    assert!(vm.is_destroyed());
}

#[test]
fn throws_if_scene_or_perf_is_undefined() {
    assert!(Cesium3DTilesInspectorViewModel::new(None::<&str>, Some("perf")).is_err());
    assert!(Cesium3DTilesInspectorViewModel::new(Some("scene"), None::<&str>).is_err());
}

#[test]
fn tileset_properties_and_display_options() {
    let vm = Cesium3DTilesInspectorViewModel::new(Some("scene"), Some("perf")).unwrap();
    let tileset = InspectorTileset::new();
    vm.set_tileset(Some(tileset));

    let props = vm.properties();
    assert!(props.contains(&"id".to_string()));
    assert!(props.contains(&"Longitude".to_string()));
    assert!(props.contains(&"Latitude".to_string()));
    assert!(props.contains(&"Height".to_string()));

    // colorize
    vm.set_colorize(true);
    assert!(vm.colorize());
    vm.set_colorize(false);
    assert!(!vm.colorize());

    // wireframe
    vm.set_wireframe(true);
    assert!(vm.wireframe());
    vm.set_wireframe(false);
    assert!(!vm.wireframe());

    // bounding volumes
    vm.set_show_bounding_volumes(true);
    assert!(vm.show_bounding_volumes());
    vm.set_show_content_bounding_volumes(true);
    assert!(vm.show_content_bounding_volumes());
    vm.set_show_request_volumes(true);
    assert!(vm.show_request_volumes());

    // label / geom error / stats / memory / url
    vm.set_show_only_picked_tile_debug_label(true);
    assert!(vm.show_only_picked_tile_debug_label());
    vm.set_show_geometric_error(true);
    assert!(vm.show_geometric_error());
    vm.set_show_rendering_statistics(true);
    assert!(vm.show_rendering_statistics());
    vm.set_show_memory_usage(true);
    assert!(vm.show_memory_usage());
    vm.set_show_url(true);
    assert!(vm.show_url());

    // point cloud
    vm.set_point_cloud_shading(true);
    assert!(vm.point_cloud_shading());
    vm.set_geometric_error_scale(2.5);
    assert_eq!(vm.geometric_error_scale(), 2.5);
    vm.set_maximum_attenuation(16.0);
    assert_eq!(vm.maximum_attenuation(), Some(16.0));
    vm.set_maximum_attenuation(0.0);
    assert_eq!(vm.maximum_attenuation(), None);
    vm.set_base_resolution(0.5);
    assert_eq!(vm.base_resolution(), Some(0.5));
    vm.set_base_resolution(0.0);
    assert_eq!(vm.base_resolution(), None);
    vm.set_eye_dome_lighting(true);
    assert!(vm.eye_dome_lighting());
    vm.set_eye_dome_lighting_strength(1.5);
    assert_eq!(vm.eye_dome_lighting_strength(), 1.5);
    vm.set_eye_dome_lighting_radius(2.0);
    assert_eq!(vm.eye_dome_lighting_radius(), 2.0);
}

#[test]
fn update_options_and_exponential_density_slider() {
    let vm = Cesium3DTilesInspectorViewModel::new(Some("scene"), Some("perf")).unwrap();
    let tileset = InspectorTileset::new();
    vm.set_tileset(Some(tileset));

    // freeze frame
    vm.set_freeze_frame(true);
    assert!(vm.freeze_frame());

    // sse
    vm.set_maximum_screen_space_error(10.0);
    assert_eq!(vm.maximum_screen_space_error(), 10.0);
    vm.set_dynamic_screen_space_error(true);
    assert!(vm.dynamic_screen_space_error());
    vm.set_dynamic_screen_space_error_factor(2.0);
    assert_eq!(vm.dynamic_screen_space_error_factor(), 2.0);

    // Slider uses exponential mapping: density = slider^6
    let slider_val = 0.2;
    vm.set_dynamic_screen_space_error_density_slider_value(slider_val);
    let expected_density = slider_val.powi(6);
    assert!((vm.dynamic_screen_space_error_density() - expected_density).abs() < 1e-8);
    assert!((vm.dynamic_screen_space_error_density_slider_value() - slider_val).abs() < 1e-6);
}

#[test]
fn style_loading_and_recompilation() {
    let vm = Cesium3DTilesInspectorViewModel::new(Some("scene"), Some("perf")).unwrap();
    let tileset = InspectorTileset::new();
    let initial_style = serde_json::json!({
        "color": "color('purple', 0.5)",
        "meta": { "description": "test" }
    });
    tileset.options.borrow_mut().style = Some(initial_style);
    vm.set_tileset(Some(tileset));

    vm.update();
    assert!(vm.style_string().contains("purple"));

    // Modify style string and compile
    let new_style = serde_json::json!({
        "color": "color('red')",
        "meta": { "description": "updated" }
    });
    vm.set_style_string(new_style.to_string());
    vm.compile_style();

    let compiled = vm.tileset().unwrap().options.borrow().style.clone().unwrap();
    assert_eq!(compiled["color"], "color('red')");

    // Invalid json should not throw panic
    vm.set_style_string("invalid json syntax");
    vm.compile_style();
}
