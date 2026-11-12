use cesium_viewer::widgets::{
    CesiumInspectorViewModel, InspectorPrimitive, InspectorTile,
};

#[test]
fn can_create_and_destroy() {
    let vm = CesiumInspectorViewModel::new(Some("scene_main"), Some("perf_div")).unwrap();
    assert_eq!(vm.scene_id(), "scene_main");
    assert_eq!(vm.performance_container_id(), "perf_div");
    assert!(!vm.is_destroyed());

    vm.destroy();
    assert!(vm.is_destroyed());
}

#[test]
fn throws_if_scene_or_perf_is_undefined() {
    assert!(CesiumInspectorViewModel::new(None::<&str>, Some("perf")).is_err());
    assert!(CesiumInspectorViewModel::new(Some("scene"), None::<&str>).is_err());
}

#[test]
fn frustums_and_performance() {
    let vm = CesiumInspectorViewModel::new(Some("scene"), Some("perf")).unwrap();
    assert!(!vm.frustums());
    vm.set_frustums(true);
    assert!(vm.frustums());
    vm.set_frustums(false);
    assert!(!vm.frustums());

    assert!(!vm.performance());
    vm.set_performance(true);
    assert!(vm.performance());
    vm.set_performance(false);
    assert!(!vm.performance());
}

#[test]
fn primitive_bounding_sphere_filter_and_reference_frame() {
    let vm = CesiumInspectorViewModel::new(Some("scene"), Some("perf")).unwrap();
    let prim_p = InspectorPrimitive::new("prim_p");
    let prim_q = InspectorPrimitive::new("prim_q");

    vm.set_primitive(Some(prim_p.clone()));

    // primitive bounding sphere
    vm.set_primitive_bounding_sphere(true);
    assert!(vm.primitive_bounding_sphere());
    assert!(prim_p.debug_show_bounding_volume());
    vm.set_primitive_bounding_sphere(false);
    assert!(!vm.primitive_bounding_sphere());
    assert!(!prim_p.debug_show_bounding_volume());

    // primitive command filter
    vm.set_filter_primitive(true);
    assert!(vm.filter_primitive());
    assert!(vm.is_command_accepted(&prim_p.id));
    assert!(!vm.is_command_accepted(&prim_q.id));

    vm.set_filter_primitive(false);
    assert!(vm.is_command_accepted(&prim_q.id));

    // primitive reference frame
    assert!(!vm.primitive_reference_frame());
    vm.set_primitive_reference_frame(true);
    assert!(vm.primitive_reference_frame());
    vm.set_primitive_reference_frame(false);
    assert!(!vm.primitive_reference_frame());
}

#[test]
fn wireframe_suspend_updates_and_tile_coords() {
    let vm = CesiumInspectorViewModel::new(Some("scene"), Some("perf")).unwrap();

    vm.set_wireframe(true);
    assert!(vm.wireframe());
    vm.set_wireframe(false);
    assert!(!vm.wireframe());

    vm.set_suspend_updates(true);
    assert!(vm.suspend_updates());
    vm.set_suspend_updates(false);
    assert!(!vm.suspend_updates());

    vm.set_tile_coordinates(true);
    assert!(vm.tile_coordinates());
    vm.set_tile_coordinates(false);
    assert!(!vm.tile_coordinates());
}

#[test]
fn tile_bounding_sphere_and_tile_filter() {
    let vm = CesiumInspectorViewModel::new(Some("scene"), Some("perf")).unwrap();
    let renderable_tile = InspectorTile::new("tile_0_0_0", 0, 0, 0, true);
    let non_renderable_tile = InspectorTile::new("tile_1_0_0", 1, 0, 0, false);

    vm.set_tile(Some(renderable_tile.clone()));

    // tile bounding sphere
    vm.set_tile_bounding_sphere(true);
    assert!(vm.tile_bounding_sphere());
    vm.set_tile_bounding_sphere(false);
    assert!(!vm.tile_bounding_sphere());

    // filter tile (renderable)
    vm.set_filter_tile(true);
    assert!(vm.filter_tile());
    assert!(vm.suspend_updates());
    assert_eq!(vm.tiles_to_render_count(), 1);

    vm.set_filter_tile(false);
    assert!(!vm.filter_tile());
    assert!(!vm.suspend_updates());
    assert_eq!(vm.tiles_to_render_count(), 0);

    // non renderable tile
    vm.set_tile(Some(non_renderable_tile));
    vm.set_filter_tile(true);
    assert_eq!(vm.tiles_to_render_count(), 0);
    assert!(vm.suspend_updates());
    vm.set_filter_tile(false);
    assert!(!vm.suspend_updates());
}
