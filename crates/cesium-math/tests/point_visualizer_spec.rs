//! Direct port and verification against CesiumJS `PointVisualizerSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    BoundingSphere, BoundingSphereState, Cartesian3, Color, ConstantPositionProperty,
    ConstantProperty, DistanceDisplayCondition, Entity, EntityCluster,
    EntityCollection, HeightReference, JulianDate, NearFarScalar, PointGraphics,
    PointGraphicsOptions, PointVisualizer, SplitDirection,
};

#[test]
fn test_constructor_throws_if_no_cluster() {
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let res = PointVisualizer::new(None, Some(collection));
    assert!(res.is_err());
}

#[test]
fn test_constructor_throws_if_no_entity_collection() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let res = PointVisualizer::new(Some(cluster), None);
    assert!(res.is_err());
}

#[test]
fn test_update_throws_if_no_time_specified() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer = PointVisualizer::new(Some(cluster), Some(collection)).unwrap();
    assert!(visualizer.update(None).is_err());
}

#[test]
fn test_is_destroy_returns_false_until_destroyed() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer = PointVisualizer::new(Some(cluster), Some(collection)).unwrap();
    assert!(!visualizer.is_destroyed());
    visualizer.destroy();
    assert!(visualizer.is_destroyed());
}

#[test]
fn test_removes_listener_when_destroyed() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        PointVisualizer::new(Some(cluster), Some(Rc::clone(&collection))).unwrap();
    assert_eq!(
        collection.borrow().collection_changed.number_of_listeners(),
        1
    );
    visualizer.destroy();
    assert_eq!(
        collection.borrow().collection_changed.number_of_listeners(),
        0
    );
}

#[test]
fn test_object_with_no_point_does_not_create_point_primitive() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        PointVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&JulianDate::now())).unwrap();
    assert!(cluster.borrow().point_collection.is_none());
}

#[test]
fn test_object_with_no_position_does_not_create_point_primitive() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        PointVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    let mut pg = PointGraphics::default();
    pg.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object.point = Some(pg);
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&JulianDate::now())).unwrap();
    assert!(cluster.borrow().point_collection.is_none());
}

#[test]
fn test_point_graphics_causes_point_primitive_created_and_updated() {
    let time = JulianDate::now();
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        PointVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut entity = Entity::with_id("test");
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));

    let options = PointGraphicsOptions {
        show: Some(Rc::new(ConstantProperty::from_value(true))),
        color: Some(Rc::new(ConstantProperty::from_value(Color::new(0.1, 0.2, 0.3, 0.4)))),
        outline_color: Some(Rc::new(ConstantProperty::from_value(Color::new(0.5, 0.6, 0.7, 0.8)))),
        outline_width: Some(Rc::new(ConstantProperty::from_value(9.0))),
        pixel_size: Some(Rc::new(ConstantProperty::from_value(10.0))),
        scale_by_distance: Some(Rc::new(ConstantProperty::from_value(NearFarScalar::new(11.0, 12.0, 13.0, 14.0)))),
        distance_display_condition: Some(Rc::new(ConstantProperty::from_value(DistanceDisplayCondition::new(10.0, 100.0)))),
        disable_depth_test_distance: Some(Rc::new(ConstantProperty::from_value(10.0))),
        split_direction: Some(Rc::new(ConstantProperty::from_value(SplitDirection::Left))),
        ..Default::default()
    };
    entity.point = Some(PointGraphics::from_options(options));
    collection.borrow_mut().add(entity.clone()).unwrap();

    visualizer.update(Some(&time)).unwrap();

    let cl = cluster.borrow();
    let col = cl.point_collection.as_ref().unwrap();
    assert_eq!(col.len(), 1);

    let pt = col.get(0).unwrap();
    assert!(pt.show);
    assert_eq!(pt.position, Cartesian3::new(1234.0, 5678.0, 9101112.0));
    assert_eq!(pt.color, Color::new(0.1, 0.2, 0.3, 0.4));
    assert_eq!(pt.outline_color, Color::new(0.5, 0.6, 0.7, 0.8));
    assert_eq!(pt.outline_width, 9.0);
    assert_eq!(pt.pixel_size, 10.0);
    assert_eq!(pt.scale_by_distance, Some(NearFarScalar::new(11.0, 12.0, 13.0, 14.0)));
    assert_eq!(pt.distance_display_condition, Some(DistanceDisplayCondition::new(10.0, 100.0)));
    assert_eq!(pt.disable_depth_test_distance, 10.0);
    assert_eq!(pt.split_direction, SplitDirection::Left);
    drop(cl);

    // Update properties and check re-synchronization
    if let Some(ent) = collection.borrow_mut().get_by_id_mut("test") {
        if let Some(p) = ent.point.as_mut() {
            p.color = Some(Rc::new(ConstantProperty::from_value(Color::new(0.15, 0.16, 0.17, 0.18))));
            p.outline_color = Some(Rc::new(ConstantProperty::from_value(Color::new(0.19, 0.2, 0.21, 0.22))));
            p.pixel_size = Some(Rc::new(ConstantProperty::from_value(23.0)));
            p.outline_width = Some(Rc::new(ConstantProperty::from_value(24.0)));
            p.scale_by_distance = Some(Rc::new(ConstantProperty::from_value(NearFarScalar::new(25.0, 26.0, 27.0, 28.0))));
            p.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(DistanceDisplayCondition::new(1000.0, 1000000.0))));
            p.disable_depth_test_distance = Some(Rc::new(ConstantProperty::from_value(20.0)));
            p.split_direction = Some(Rc::new(ConstantProperty::from_value(SplitDirection::Right)));
        }
    }

    visualizer.update(Some(&time)).unwrap();

    let cl = cluster.borrow();
    let col = cl.point_collection.as_ref().unwrap();
    let pt = col.get(0).unwrap();
    assert_eq!(pt.color, Color::new(0.15, 0.16, 0.17, 0.18));
    assert_eq!(pt.outline_color, Color::new(0.19, 0.2, 0.21, 0.22));
    assert_eq!(pt.pixel_size, 23.0);
    assert_eq!(pt.outline_width, 24.0);
    assert_eq!(pt.disable_depth_test_distance, 20.0);
    assert_eq!(pt.split_direction, SplitDirection::Right);
    drop(cl);

    // Toggle show
    if let Some(ent) = collection.borrow_mut().get_by_id_mut("test") {
        ent.point.as_mut().unwrap().show = Some(Rc::new(ConstantProperty::from_value(false)));
    }
    visualizer.update(Some(&time)).unwrap();
    let cl = cluster.borrow();
    assert!(!cl.point_collection.as_ref().unwrap().get(0).unwrap().show);
}

#[test]
fn test_point_graphics_on_terrain_causes_billboard_created_and_updated() {
    let time = JulianDate::now();
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        PointVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut entity = Entity::with_id("test");
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));

    let options = PointGraphicsOptions {
        show: Some(Rc::new(ConstantProperty::from_value(true))),
        color: Some(Rc::new(ConstantProperty::from_value(Color::new(0.1, 0.2, 0.3, 0.4)))),
        outline_color: Some(Rc::new(ConstantProperty::from_value(Color::new(0.5, 0.6, 0.7, 0.8)))),
        outline_width: Some(Rc::new(ConstantProperty::from_value(9.0))),
        pixel_size: Some(Rc::new(ConstantProperty::from_value(10.0))),
        scale_by_distance: Some(Rc::new(ConstantProperty::from_value(NearFarScalar::new(11.0, 12.0, 13.0, 14.0)))),
        distance_display_condition: Some(Rc::new(ConstantProperty::from_value(DistanceDisplayCondition::new(10.0, 100.0)))),
        disable_depth_test_distance: Some(Rc::new(ConstantProperty::from_value(10.0))),
        height_reference: Some(Rc::new(ConstantProperty::from_value(HeightReference::ClampToGround))),
        split_direction: Some(Rc::new(ConstantProperty::from_value(SplitDirection::Left))),
        ..Default::default()
    };
    entity.point = Some(PointGraphics::from_options(options));
    collection.borrow_mut().add(entity).unwrap();

    visualizer.update(Some(&time)).unwrap();

    let cl = cluster.borrow();
    let col = cl.billboard_collection.as_ref().unwrap();
    assert_eq!(col.len(), 1);
    let bb = col.get(0).unwrap();
    assert!(bb.show);
    assert_eq!(bb.position, Cartesian3::new(1234.0, 5678.0, 9101112.0));
    assert_eq!(bb.height_reference, HeightReference::ClampToGround);
    assert_eq!(bb.disable_depth_test_distance, Some(10.0));
    assert_eq!(bb.split_direction, SplitDirection::Left);
    drop(cl);

    // Toggle show to false
    if let Some(ent) = collection.borrow_mut().get_by_id_mut("test") {
        ent.point.as_mut().unwrap().show = Some(Rc::new(ConstantProperty::from_value(false)));
    }
    visualizer.update(Some(&time)).unwrap();
    let cl = cluster.borrow();
    assert!(!cl.billboard_collection.as_ref().unwrap().get(0).unwrap().show);
}

#[test]
fn test_reuses_primitives_when_hiding_one_and_showing_another() {
    let time = JulianDate::now();
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        PointVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    let mut pg = PointGraphics::default();
    pg.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object.point = Some(pg);
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&time)).unwrap();
    assert_eq!(cluster.borrow().point_collection.as_ref().unwrap().len(), 1);

    // Hide test
    if let Some(ent) = collection.borrow_mut().get_by_id_mut("test") {
        ent.point.as_mut().unwrap().show = Some(Rc::new(ConstantProperty::from_value(false)));
    }
    visualizer.update(Some(&time)).unwrap();
    assert_eq!(cluster.borrow().point_collection.as_ref().unwrap().len(), 1);

    // Add and show test2
    let mut test_object2 = Entity::with_id("test2");
    test_object2.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    let mut pg2 = PointGraphics::default();
    pg2.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object2.point = Some(pg2);
    collection.borrow_mut().add(test_object2).unwrap();

    visualizer.update(Some(&time)).unwrap();
    assert_eq!(cluster.borrow().point_collection.as_ref().unwrap().len(), 1);
}

#[test]
fn test_clear_hides_point_primitives() {
    let time = JulianDate::now();
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        PointVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    let mut pg = PointGraphics::default();
    pg.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object.point = Some(pg);
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&time)).unwrap();
    assert_eq!(cluster.borrow().point_collection.as_ref().unwrap().len(), 1);

    collection.borrow_mut().remove_all();
    visualizer.update(Some(&time)).unwrap();
    {
        let cl = cluster.borrow();
        let pt = cl.point_collection.as_ref().unwrap().get(0).unwrap();
        assert!(!pt.show);
        assert!(pt.id.is_none());
    }
}

#[test]
fn test_visualizer_sets_entity_property() {
    let time = JulianDate::now();
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        PointVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    let mut pg = PointGraphics::default();
    pg.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object.point = Some(pg);
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&time)).unwrap();
    let cl = cluster.borrow();
    let pt = cl.point_collection.as_ref().unwrap().get(0).unwrap();
    assert_eq!(pt.id.as_deref(), Some("test"));
}

#[test]
fn test_computes_bounding_sphere() {
    let time = JulianDate::now();
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        PointVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    let pos = Cartesian3::new(1234.0, 5678.0, 9101112.0);
    test_object.position = Some(ConstantPositionProperty::new(Some(pos), None));
    let mut pg = PointGraphics::default();
    pg.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object.point = Some(pg);
    collection.borrow_mut().add(test_object.clone()).unwrap();

    visualizer.update(Some(&time)).unwrap();

    let mut result = BoundingSphere::default();
    let state = visualizer
        .get_bounding_sphere(Some(&test_object), Some(&mut result))
        .unwrap();

    assert_eq!(state, BoundingSphereState::Done);
    assert_eq!(result.center, pos.into());
    assert_eq!(result.radius, 0.0);
}

#[test]
fn test_fails_bounding_sphere_for_entity_without_point() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let test_object = Entity::with_id("test");
    collection.borrow_mut().add(test_object.clone()).unwrap();

    let mut visualizer = PointVisualizer::new(Some(cluster), Some(collection)).unwrap();
    visualizer.update(Some(&JulianDate::now())).unwrap();

    let mut result = BoundingSphere::default();
    let state = visualizer
        .get_bounding_sphere(Some(&test_object), Some(&mut result))
        .unwrap();
    assert_eq!(state, BoundingSphereState::Failed);
}

#[test]
fn test_compute_bounding_sphere_validation_errors() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let test_object = Entity::with_id("test");
    let visualizer = PointVisualizer::new(Some(cluster), Some(collection)).unwrap();

    let mut result = BoundingSphere::default();
    assert!(visualizer.get_bounding_sphere(None, Some(&mut result)).is_err());
    assert!(visualizer.get_bounding_sphere(Some(&test_object), None).is_err());
}
