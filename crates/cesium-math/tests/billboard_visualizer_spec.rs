//! Direct port and verification against CesiumJS `BillboardVisualizerSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    BillboardGraphics, BillboardGraphicsOptions, BillboardVisualizer, BoundingRectangle,
    BoundingSphere, BoundingSphereState, Cartesian2, Cartesian3, Color, ConstantPositionProperty,
    ConstantProperty, DistanceDisplayCondition, DynProperty, Entity, EntityCluster,
    EntityCollection, HeightReference, HorizontalOrigin, JulianDate, NearFarScalar, SplitDirection,
    VerticalOrigin,
};

#[test]
fn test_constructor_throws_if_no_cluster() {
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let res = BillboardVisualizer::new(None, Some(collection));
    assert!(res.is_err());
}

#[test]
fn test_update_throws_if_no_time_specified() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer = BillboardVisualizer::new(Some(cluster), Some(collection)).unwrap();
    assert!(visualizer.update(None).is_err());
}

#[test]
fn test_is_destroy_returns_false_until_destroyed() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer = BillboardVisualizer::new(Some(cluster), Some(collection)).unwrap();
    assert!(!visualizer.is_destroyed());
    visualizer.destroy();
    assert!(visualizer.is_destroyed());
}

#[test]
fn test_removes_listener_when_destroyed() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        BillboardVisualizer::new(Some(cluster), Some(Rc::clone(&collection))).unwrap();
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
fn test_object_with_no_billboard_does_not_create_billboard() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        BillboardVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&JulianDate::now())).unwrap();
    assert!(cluster.borrow().billboard_collection.is_none());
}

#[test]
fn test_object_with_no_position_does_not_create_billboard() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        BillboardVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    let mut bg = BillboardGraphics::default();
    bg.show = Some(Rc::new(ConstantProperty::from_value(true)));
    bg.image = Some(Rc::new(ConstantProperty::from_value("Data/Images/Blue.png".to_string())));
    test_object.billboard = Some(bg);
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&JulianDate::now())).unwrap();
    assert!(cluster.borrow().billboard_collection.is_none());
}

#[test]
fn test_object_with_no_image_does_not_create_billboard() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        BillboardVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    let mut bg = BillboardGraphics::default();
    bg.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object.billboard = Some(bg);
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&JulianDate::now())).unwrap();
    assert!(cluster.borrow().billboard_collection.is_none());
}

#[test]
fn test_billboard_graphics_causes_billboard_created_and_updated() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        BillboardVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let time = JulianDate::now();
    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));

    let options = BillboardGraphicsOptions {
        show: Some(Rc::new(ConstantProperty::from_value(true))),
        color: Some(Rc::new(ConstantProperty::from_value(Color::new(0.5, 0.5, 0.5, 0.5)))),
        image: Some(Rc::new(ConstantProperty::from_value("Data/Images/Blue.png".to_string()))),
        image_sub_region: Some(Rc::new(ConstantProperty::from_value(BoundingRectangle::new(0.0, 0.0, 1.0, 1.0)))),
        eye_offset: Some(Rc::new(ConstantProperty::from_value(Cartesian3::new(1.0, 2.0, 3.0)))),
        scale: Some(Rc::new(ConstantProperty::from_value(12.5))),
        rotation: Some(Rc::new(ConstantProperty::from_value(1.5))),
        aligned_axis: Some(Rc::new(ConstantProperty::from_value(Cartesian3::UNIT_Z))),
        height_reference: Some(Rc::new(ConstantProperty::from_value(HeightReference::ClampToGround))),
        horizontal_origin: Some(Rc::new(ConstantProperty::from_value(HorizontalOrigin::Right))),
        vertical_origin: Some(Rc::new(ConstantProperty::from_value(VerticalOrigin::Top))),
        pixel_offset: Some(Rc::new(ConstantProperty::from_value(Cartesian2::new(3.0, 2.0)))),
        width: Some(Rc::new(ConstantProperty::from_value(15.0))),
        height: Some(Rc::new(ConstantProperty::from_value(5.0))),
        scale_by_distance: Some(Rc::new(ConstantProperty::from_value(NearFarScalar::default()))),
        translucency_by_distance: Some(Rc::new(ConstantProperty::from_value(NearFarScalar::default()))),
        pixel_offset_scale_by_distance: Some(Rc::new(ConstantProperty::from_value(NearFarScalar::new(1.0, 0.0, 3.0e9, 0.0)))),
        size_in_meters: Some(Rc::new(ConstantProperty::from_value(true))),
        distance_display_condition: Some(Rc::new(ConstantProperty::from_value(DistanceDisplayCondition::new(10.0, 100.0)))),
        disable_depth_test_distance: Some(Rc::new(ConstantProperty::from_value(10.0))),
        split_direction: Some(Rc::new(ConstantProperty::from_value(SplitDirection::Left))),
    };
    test_object.billboard = Some(BillboardGraphics::from_options(options));
    collection.borrow_mut().add(test_object.clone()).unwrap();

    visualizer.update(Some(&time)).unwrap();

    let cl = cluster.borrow();
    let col = cl.billboard_collection.as_ref().unwrap();
    assert_eq!(col.len(), 1);

    let bb = col.get(0).unwrap();
    assert!(bb.ready);
    assert!(bb.show);
    assert_eq!(bb.position, Cartesian3::new(1234.0, 5678.0, 9101112.0));
    assert_eq!(bb.color, Color::new(0.5, 0.5, 0.5, 0.5));
    assert_eq!(bb.eye_offset, Cartesian3::new(1.0, 2.0, 3.0));
    assert_eq!(bb.scale, 12.5);
    assert_eq!(bb.rotation, 1.5);
    assert_eq!(bb.aligned_axis, Cartesian3::UNIT_Z);
    assert_eq!(bb.height_reference, HeightReference::ClampToGround);
    assert_eq!(bb.horizontal_origin, HorizontalOrigin::Right);
    assert_eq!(bb.vertical_origin, VerticalOrigin::Top);
    assert_eq!(bb.pixel_offset, Cartesian2::new(3.0, 2.0));
    assert_eq!(bb.width, Some(15.0));
    assert_eq!(bb.height, Some(5.0));
    assert!(bb.size_in_meters);
    assert_eq!(bb.disable_depth_test_distance, Some(10.0));
    assert_eq!(bb.split_direction, SplitDirection::Left);
    assert_eq!(bb.image.as_deref(), Some("Data/Images/Blue.png"));
}

#[test]
fn test_display_billboard_after_toggling_show() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        BillboardVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let time = JulianDate::now();
    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));

    let mut bg = BillboardGraphics::default();
    let show_prop: Rc<dyn DynProperty<bool>> = Rc::new(ConstantProperty::from_value(true));
    bg.show = Some(Rc::clone(&show_prop));
    bg.image = Some(Rc::new(ConstantProperty::from_value("Data/Images/Blue.png".to_string())));
    test_object.billboard = Some(bg);
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&time)).unwrap();
    {
        let cl = cluster.borrow();
        assert!(cl.billboard_collection.as_ref().unwrap().get(0).unwrap().show);
    }

    // Toggle show to false
    if let Some(ent) = collection.borrow_mut().get_by_id_mut("test") {
        ent.billboard.as_mut().unwrap().show = Some(Rc::new(ConstantProperty::from_value(false)));
    }
    visualizer.update(Some(&time)).unwrap();
    {
        let cl = cluster.borrow();
        assert!(!cl.billboard_collection.as_ref().unwrap().get(0).unwrap().show);
    }

    // Toggle show to true
    if let Some(ent) = collection.borrow_mut().get_by_id_mut("test") {
        ent.billboard.as_mut().unwrap().show = Some(Rc::new(ConstantProperty::from_value(true)));
    }
    visualizer.update(Some(&time)).unwrap();
    {
        let cl = cluster.borrow();
        assert!(cl.billboard_collection.as_ref().unwrap().get(0).unwrap().show);
    }
}

#[test]
fn test_reuses_primitives_when_hiding_one_and_showing_another() {
    let time = JulianDate::now();
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        BillboardVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    let mut bg = BillboardGraphics::default();
    bg.image = Some(Rc::new(ConstantProperty::from_value("Data/Images/Blue.png".to_string())));
    bg.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object.billboard = Some(bg);
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&time)).unwrap();
    assert_eq!(cluster.borrow().billboard_collection.as_ref().unwrap().len(), 1);

    // Hide testObject
    if let Some(ent) = collection.borrow_mut().get_by_id_mut("test") {
        ent.billboard.as_mut().unwrap().show = Some(Rc::new(ConstantProperty::from_value(false)));
    }
    visualizer.update(Some(&time)).unwrap();
    assert_eq!(cluster.borrow().billboard_collection.as_ref().unwrap().len(), 1);

    // Add and show test2
    let mut test_object2 = Entity::with_id("test2");
    test_object2.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    let mut bg2 = BillboardGraphics::default();
    bg2.image = Some(Rc::new(ConstantProperty::from_value("Data/Images/Blue.png".to_string())));
    bg2.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object2.billboard = Some(bg2);
    collection.borrow_mut().add(test_object2).unwrap();

    visualizer.update(Some(&time)).unwrap();
    // Primitive is reused; length stays 1
    assert_eq!(cluster.borrow().billboard_collection.as_ref().unwrap().len(), 1);
}

#[test]
fn test_clear_hides_billboards() {
    let time = JulianDate::now();
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        BillboardVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    let mut bg = BillboardGraphics::default();
    bg.image = Some(Rc::new(ConstantProperty::from_value("Data/Images/Blue.png".to_string())));
    bg.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object.billboard = Some(bg);
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&time)).unwrap();
    assert_eq!(cluster.borrow().billboard_collection.as_ref().unwrap().len(), 1);
    assert!(cluster.borrow().billboard_collection.as_ref().unwrap().get(0).unwrap().show);

    // Remove all hides the billboard and clears id for reuse
    collection.borrow_mut().remove_all();
    visualizer.update(Some(&time)).unwrap();
    {
        let cl = cluster.borrow();
        let bb = cl.billboard_collection.as_ref().unwrap().get(0).unwrap();
        assert!(!bb.show);
        assert!(bb.id.is_none());
    }
}

#[test]
fn test_visualizer_sets_entity_property() {
    let time = JulianDate::now();
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        BillboardVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    let mut bg = BillboardGraphics::default();
    bg.image = Some(Rc::new(ConstantProperty::from_value("Data/Images/Blue.png".to_string())));
    bg.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object.billboard = Some(bg);
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&time)).unwrap();

    let cl = cluster.borrow();
    let bb = cl.billboard_collection.as_ref().unwrap().get(0).unwrap();
    assert_eq!(bb.id.as_deref(), Some("test"));
}

#[test]
fn test_computes_bounding_sphere() {
    let time = JulianDate::now();
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        BillboardVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    let pos = Cartesian3::new(1234.0, 5678.0, 9101112.0);
    test_object.position = Some(ConstantPositionProperty::new(Some(pos), None));
    let mut bg = BillboardGraphics::default();
    bg.image = Some(Rc::new(ConstantProperty::from_value("Data/Images/Blue.png".to_string())));
    bg.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object.billboard = Some(bg);
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
fn test_fails_bounding_sphere_for_entity_without_billboard() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let test_object = Entity::with_id("test");
    collection.borrow_mut().add(test_object.clone()).unwrap();

    let mut visualizer =
        BillboardVisualizer::new(Some(cluster), Some(collection)).unwrap();
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
    let visualizer = BillboardVisualizer::new(Some(cluster), Some(collection)).unwrap();

    let mut result = BoundingSphere::default();
    assert!(visualizer.get_bounding_sphere(None, Some(&mut result)).is_err());
    assert!(visualizer.get_bounding_sphere(Some(&test_object), None).is_err());
}
