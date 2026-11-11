use cesium_math::{Cartesian2, Cartesian3, Cartographic, Ray, Rectangle};
use cesium_renderer::picking::{
    FeatureRef, PickVoxelCell, PickingManager, PickingPrimitive, ProjectionType, SceneMode,
};
use glam::DVec3;

#[test]
fn test_feature_ref() {
    let f = FeatureRef {
        mesh: 101,
        feature: 5,
    };
    assert_eq!(f.mesh, 101);
    assert_eq!(f.feature, 5);
}

#[test]
fn test_pick_validation_and_basic() {
    let mut manager = PickingManager::new();

    // 1. throws when window position is undefined
    assert!(manager.pick(None, None, None).is_err());

    // 2. picks a primitive
    let prim = PickingPrimitive::new(1)
        .with_depth(10.0)
        .with_screen_bounds(0.0, 0.0, 10.0, 10.0);
    manager.add_primitive(prim);

    let hit = manager
        .pick(Some(Cartesian2::new(5.0, 5.0)), None, None)
        .expect("pick should succeed");
    assert!(hit.is_some());
    assert_eq!(hit.unwrap().primitive, 1);
}

#[test]
fn test_pick_search_area_and_filtering() {
    let mut manager = PickingManager::new();

    // Primitive centered at (10, 10) with size 2x2: bounds [9..11, 9..11]
    let prim = PickingPrimitive::new(42)
        .with_depth(5.0)
        .with_screen_bounds(9.0, 9.0, 11.0, 11.0);
    manager.add_primitive(prim);

    // Pick at (7, 7) with width=7, height=7 (covers [3.5..10.5, 3.5..10.5]) => hits [9..11]
    let hit_large = manager
        .pick(Some(Cartesian2::new(7.0, 7.0)), Some(7), Some(7))
        .expect("pick succeeded");
    assert!(hit_large.is_some());
    assert_eq!(hit_large.unwrap().primitive, 42);

    // Pick at (7, 7) with width=3, height=3 (covers [5.5..8.5, 5.5..8.5]) => no hit
    let hit_small = manager
        .pick(Some(Cartesian2::new(7.0, 7.0)), Some(3), Some(3))
        .expect("pick succeeded");
    assert!(hit_small.is_none());

    // 3. does not pick primitives when show is false
    manager.clear();
    let hidden = PickingPrimitive::new(43)
        .with_show(false)
        .with_screen_bounds(0.0, 0.0, 10.0, 10.0);
    manager.add_primitive(hidden);
    assert!(manager
        .pick(Some(Cartesian2::new(5.0, 5.0)), None, None)
        .unwrap()
        .is_none());

    // 4. does not pick primitives when alpha is zero
    manager.clear();
    let transparent = PickingPrimitive::new(44)
        .with_alpha(0.0)
        .with_screen_bounds(0.0, 0.0, 10.0, 10.0);
    manager.add_primitive(transparent);
    assert!(manager
        .pick(Some(Cartesian2::new(5.0, 5.0)), None, None)
        .unwrap()
        .is_none());
}

#[test]
fn test_pick_top_primitive_and_modes() {
    let mut manager = PickingManager::new();

    // Bottom primitive (depth 10.0)
    let bottom = PickingPrimitive::new(1)
        .with_depth(10.0)
        .with_screen_bounds(0.0, 0.0, 10.0, 10.0);
    // Top primitive (depth 2.0)
    let top = PickingPrimitive::new(2)
        .with_depth(2.0)
        .with_screen_bounds(0.0, 0.0, 10.0, 10.0);
    manager.add_primitive(bottom);
    manager.add_primitive(top);

    let picked = manager
        .pick(Some(Cartesian2::new(5.0, 5.0)), None, None)
        .unwrap()
        .expect("should pick top");
    assert_eq!(picked.primitive, 2);

    // Pick in 2D
    manager.mode = SceneMode::Scene2D;
    let picked_2d = manager
        .pick(Some(Cartesian2::new(5.0, 5.0)), None, None)
        .unwrap()
        .expect("should pick in 2d");
    assert_eq!(picked_2d.primitive, 2);

    // Pick in 3D with orthographic projection
    manager.mode = SceneMode::Scene3D;
    manager.projection = ProjectionType::Orthographic;
    let picked_ortho = manager
        .pick(Some(Cartesian2::new(5.0, 5.0)), None, None)
        .unwrap()
        .expect("should pick in ortho");
    assert_eq!(picked_ortho.primitive, 2);
}

#[test]
fn test_pick_async_and_timeout() {
    let mut manager = PickingManager::new();
    let prim = PickingPrimitive::new(77)
        .with_depth(1.0)
        .with_screen_bounds(0.0, 0.0, 10.0, 10.0);
    manager.add_primitive(prim);

    // Async pick success
    let async_hit = manager
        .pick_async(Some(Cartesian2::new(5.0, 5.0)), None, None, false)
        .expect("async pick should succeed");
    assert_eq!(async_hit.unwrap().primitive, 77);

    // Async pick throws timeout if too slow
    let async_timeout = manager.pick_async(Some(Cartesian2::new(5.0, 5.0)), None, None, true);
    assert!(async_timeout.is_err());
    assert!(async_timeout
        .unwrap_err()
        .contains("Timed out waiting for async pick"));
}

#[test]
fn test_pick_voxel_and_voxel_coordinate() {
    let mut manager = PickingManager::new();

    // Throws when window position is undefined
    assert!(manager.pick_voxel_coordinate(None).is_err());
    assert!(manager.pick_voxel(None).is_err());

    let mut cell = PickVoxelCell::new(0, 0);
    cell = cell.with_property("a", vec![0.0]);

    let voxel_prim = PickingPrimitive::new(999)
        .with_voxel([0, 1, 2, 3], cell)
        .with_screen_bounds(0.0, 0.0, 10.0, 10.0);
    manager.add_primitive(voxel_prim);

    let coord = manager
        .pick_voxel_coordinate(Some(Cartesian2::new(5.0, 5.0)))
        .unwrap();
    assert_eq!(coord, Some([0, 1, 2, 3]));

    let voxel_cell = manager
        .pick_voxel(Some(Cartesian2::new(5.0, 5.0)))
        .unwrap()
        .expect("should pick voxel cell");
    assert_eq!(voxel_cell.tile_index, 0);
    assert_eq!(voxel_cell.sample_index, 0);
    assert!(voxel_cell.has_property("a"));
    assert_eq!(voxel_cell.get_property("a"), Some(&[0.0f32][..]));
    assert!(!voxel_cell.has_property("non_existent"));
}

#[test]
fn test_drill_pick() {
    let mut manager = PickingManager::new();

    // Throws when window position is undefined
    assert!(manager.drill_pick(None, None, None, None).is_err());

    // Multiple primitives at same location with different depths
    let p1 = PickingPrimitive::new(10)
        .with_instance_id(1)
        .with_depth(10.0)
        .with_screen_bounds(0.0, 0.0, 10.0, 10.0);
    let p2 = PickingPrimitive::new(10)
        .with_instance_id(2)
        .with_show(false)
        .with_depth(5.0)
        .with_screen_bounds(0.0, 0.0, 10.0, 10.0);
    let p3 = PickingPrimitive::new(10)
        .with_instance_id(3)
        .with_depth(1.0)
        .with_screen_bounds(0.0, 0.0, 10.0, 10.0);
    let p4 = PickingPrimitive::new(10)
        .with_alpha(0.0)
        .with_depth(0.5)
        .with_screen_bounds(0.0, 0.0, 10.0, 10.0);

    manager.add_primitive(p1);
    manager.add_primitive(p2);
    manager.add_primitive(p3);
    manager.add_primitive(p4);

    let picked = manager
        .drill_pick(Some(Cartesian2::new(5.0, 5.0)), None, None, None)
        .expect("drill pick should succeed");

    // p2 is show=false, p4 is alpha=0.0; only p3 and p1 should be picked, ordered by depth
    assert_eq!(picked.len(), 2);
    assert_eq!(picked[0].primitive, 10);
    assert_eq!(picked[0].id, Some(3));
    assert_eq!(picked[1].primitive, 10);
    assert_eq!(picked[1].id, Some(1));

    // Limit verification
    let limited = manager
        .drill_pick(Some(Cartesian2::new(5.0, 5.0)), Some(1), None, None)
        .expect("limited drill pick");
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0].id, Some(3));

    // Can drill pick without ID
    manager.clear();
    let no_id = PickingPrimitive::new(99)
        .with_depth(3.0)
        .with_screen_bounds(0.0, 0.0, 10.0, 10.0);
    manager.add_primitive(no_id);
    let picked_no_id = manager
        .drill_pick(Some(Cartesian2::new(5.0, 5.0)), None, None, None)
        .unwrap();
    assert_eq!(picked_no_id.len(), 1);
    assert_eq!(picked_no_id[0].primitive, 99);
    assert_eq!(picked_no_id[0].id, None);
}

#[test]
fn test_pick_from_ray() {
    let mut manager = PickingManager::new();

    // 1. Throws if ray is undefined
    assert!(manager.pick_from_ray(None, &[], None).is_err());

    let ray = Ray::new(
        DVec3::new(0.0, 0.0, 100.0),
        DVec3::new(0.0, 0.0, -1.0),
    )
    .unwrap();

    // 2. Throws in 2D and Columbus View
    manager.mode = SceneMode::Scene2D;
    assert!(manager.pick_from_ray(Some(ray), &[], None).is_err());
    manager.mode = SceneMode::ColumbusView;
    assert!(manager.pick_from_ray(Some(ray), &[], None).is_err());

    // 3. 3D mode picking
    manager.mode = SceneMode::Scene3D;
    let p1 = PickingPrimitive::new(1)
        .with_bounding_sphere(Cartesian3::new(0.0, 0.0, 0.0), 10.0);
    let p2 = PickingPrimitive::new(2)
        .with_bounding_sphere(Cartesian3::new(0.0, 0.0, 50.0), 10.0);
    manager.add_primitive(p1);
    manager.add_primitive(p2);

    let res = manager
        .pick_from_ray(Some(ray), &[], None)
        .unwrap()
        .expect("should hit p2 first");
    assert_eq!(res.object.primitive, 2);

    // Excludes objects
    let res_excluded = manager
        .pick_from_ray(Some(ray), &[2], None)
        .unwrap()
        .expect("should hit p1");
    assert_eq!(res_excluded.object.primitive, 1);

    // Does not pick when show is false or alpha is zero
    manager.clear();
    let hidden = PickingPrimitive::new(3)
        .with_show(false)
        .with_bounding_sphere(Cartesian3::new(0.0, 0.0, 0.0), 10.0);
    manager.add_primitive(hidden);
    assert!(manager.pick_from_ray(Some(ray), &[], None).unwrap().is_none());

    // Picks primitive that doesn't write depth
    manager.clear();
    let no_depth = PickingPrimitive::new(4)
        .with_writes_depth(false)
        .with_bounding_sphere(Cartesian3::new(0.0, 0.0, 0.0), 10.0);
    manager.add_primitive(no_depth);
    let hit_no_depth = manager.pick_from_ray(Some(ray), &[], None).unwrap();
    assert!(hit_no_depth.is_some());
    assert_eq!(hit_no_depth.unwrap().object.primitive, 4);

    // Offscreen ray misses
    let miss_ray = Ray::new(
        DVec3::new(1000.0, 1000.0, 100.0),
        DVec3::new(0.0, 0.0, -1.0),
    )
    .unwrap();
    assert!(manager
        .pick_from_ray(Some(miss_ray), &[], None)
        .unwrap()
        .is_none());
}

#[test]
fn test_drill_pick_from_ray() {
    let mut manager = PickingManager::new();
    let ray = Ray::new(
        DVec3::new(0.0, 0.0, 100.0),
        DVec3::new(0.0, 0.0, -1.0),
    )
    .unwrap();

    let p1 = PickingPrimitive::new(1)
        .with_bounding_sphere(Cartesian3::new(0.0, 0.0, 0.0), 10.0);
    let p2 = PickingPrimitive::new(2)
        .with_bounding_sphere(Cartesian3::new(0.0, 0.0, 50.0), 10.0);
    manager.add_primitive(p1);
    manager.add_primitive(p2);

    let hits = manager
        .drill_pick_from_ray(Some(ray), None, &[], None)
        .expect("drill pick hits");
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].object.primitive, 2);
    assert_eq!(hits[1].object.primitive, 1);

    // Test limit
    let limited = manager
        .drill_pick_from_ray(Some(ray), Some(1), &[], None)
        .expect("limited drill pick");
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0].object.primitive, 2);
}

#[test]
fn test_sample_height_and_clamping() {
    let mut manager = PickingManager::new();

    // 1. Position undefined throws
    assert!(manager.sample_height(None, &[], None).is_err());
    assert!(manager.clamp_to_height(None, &[], None).is_err());

    // 2. 2D / CV throws
    let carto = Cartographic::from_radians(0.0, 0.0);
    let cartesian = Cartesian3::from_radians(0.0, 0.0, Some(0.0), None).unwrap();

    manager.mode = SceneMode::Scene2D;
    assert!(manager.sample_height(Some(carto), &[], None).is_err());
    assert!(manager.clamp_to_height(Some(cartesian), &[], None).is_err());

    manager.mode = SceneMode::ColumbusView;
    assert!(manager.sample_height(Some(carto), &[], None).is_err());
    assert!(manager.clamp_to_height(Some(cartesian), &[], None).is_err());

    // 3. 3D mode height sampling
    manager.mode = SceneMode::Scene3D;
    let rect = Rectangle::from_degrees(-1.0, -1.0, 1.0, 1.0);
    let prim1 = PickingPrimitive::new(10).with_surface_rectangle(rect, 100.0);
    let prim2 = PickingPrimitive::new(20).with_surface_rectangle(rect, 250.0);
    let no_depth_prim = PickingPrimitive::new(30)
        .with_writes_depth(false)
        .with_surface_rectangle(rect, 500.0);

    manager.add_primitive(prim1);
    manager.add_primitive(prim2);
    manager.add_primitive(no_depth_prim);

    // Should sample the top primitive that writes depth (prim2 at 250.0, not no_depth at 500.0)
    let sampled = manager
        .sample_height(Some(carto), &[], None)
        .unwrap()
        .expect("should sample height");
    assert!((sampled - 250.0).abs() < 1e-4);

    // Exclude prim2 => should sample prim1 (100.0)
    let sampled_excluded = manager
        .sample_height(Some(carto), &[20], None)
        .unwrap()
        .expect("should sample height excluding 20");
    assert!((sampled_excluded - 100.0).abs() < 1e-4);

    // Clamp to height
    let clamped = manager
        .clamp_to_height(Some(cartesian), &[], None)
        .unwrap()
        .expect("should clamp to height");
    let clamped_carto = cesium_math::Ellipsoid::WGS84
        .cartesian_to_cartographic(glam::DVec3::new(clamped.x, clamped.y, clamped.z))
        .expect("cartographic");
    assert!((clamped_carto.height - 250.0).abs() < 1e-2);

    // Unsupported flags throw
    manager.sample_height_supported = false;
    assert!(manager.sample_height(Some(carto), &[], None).is_err());
    manager.sample_height_supported = true;

    manager.clamp_to_height_supported = false;
    assert!(manager.clamp_to_height(Some(cartesian), &[], None).is_err());
    manager.clamp_to_height_supported = true;

    // Batch sample_height_most_detailed & clamp_to_height_most_detailed
    let empty_cartos: Vec<Cartographic> = vec![];
    assert_eq!(
        manager
            .sample_height_most_detailed(&empty_cartos, &[], None)
            .unwrap(),
        vec![]
    );

    let empty_cartesians: Vec<Cartesian3> = vec![];
    assert_eq!(
        manager
            .clamp_to_height_most_detailed(&empty_cartesians, &[], None)
            .unwrap(),
        vec![]
    );

    let batch_sampled = manager
        .sample_height_most_detailed(&[carto], &[], None)
        .unwrap();
    assert_eq!(batch_sampled.len(), 1);
    assert_eq!(batch_sampled[0], Some(250.0));
}
