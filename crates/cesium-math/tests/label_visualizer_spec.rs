//! Direct port and verification against CesiumJS `LabelVisualizerSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    BoundingSphere, BoundingSphereState, Cartesian2, Cartesian3, Color, ConstantPositionProperty,
    ConstantProperty, DistanceDisplayCondition, Entity, EntityCluster, EntityCollection,
    HorizontalOrigin, JulianDate, LabelGraphics, LabelGraphicsOptions, LabelStyle, LabelVisualizer,
    NearFarScalar, VerticalOrigin,
};
use glam::DVec3;

#[test]
fn test_constructor_throws_if_no_cluster() {
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let res = LabelVisualizer::new(None, Some(collection));
    assert!(res.is_err());
}

#[test]
fn test_constructor_throws_if_no_entity_collection() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let res = LabelVisualizer::new(Some(cluster), None);
    assert!(res.is_err());
}

#[test]
fn test_update_throws_if_no_time_specified() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer = LabelVisualizer::new(Some(cluster), Some(collection)).unwrap();
    assert!(visualizer.update(None).is_err());
}

#[test]
fn test_is_destroy_returns_false_until_destroyed() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer = LabelVisualizer::new(Some(cluster), Some(collection)).unwrap();

    assert!(!visualizer.is_destroyed());
    visualizer.destroy();
    assert!(visualizer.is_destroyed());
}

#[test]
fn test_removes_listener_when_destroyed() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        LabelVisualizer::new(Some(cluster), Some(Rc::clone(&collection))).unwrap();

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
fn test_object_with_no_label_does_not_create_label() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        LabelVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    collection.borrow_mut().add(test_object).unwrap();

    let time = JulianDate::now();
    visualizer.update(Some(&time)).unwrap();

    assert!(cluster.borrow().label_collection.is_none());
}

#[test]
fn test_object_with_no_position_does_not_create_label() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        LabelVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    let mut label = LabelGraphics::default();
    label.show = Some(Rc::new(ConstantProperty::from_value(true)));
    label.text = Some(Rc::new(ConstantProperty::from_value("lorum ipsum".to_string())));
    test_object.label = Some(label);
    collection.borrow_mut().add(test_object).unwrap();

    let time = JulianDate::now();
    visualizer.update(Some(&time)).unwrap();

    assert!(cluster.borrow().label_collection.is_none());
}

#[test]
fn test_object_with_no_text_does_not_create_label() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        LabelVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    let mut label = LabelGraphics::default();
    label.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object.label = Some(label);
    collection.borrow_mut().add(test_object).unwrap();

    let time = JulianDate::now();
    visualizer.update(Some(&time)).unwrap();

    assert!(cluster.borrow().label_collection.is_none());
}

#[test]
fn test_label_graphics_causes_label_created_and_updated() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        LabelVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));

    let label_opts = LabelGraphicsOptions {
        text: Some(Rc::new(ConstantProperty::from_value("a".to_string()))),
        font: Some(Rc::new(ConstantProperty::from_value("sans serif".to_string()))),
        style: Some(Rc::new(ConstantProperty::from_value(LabelStyle::Fill))),
        fill_color: Some(Rc::new(ConstantProperty::from_value(Color::new(
            0.5, 0.8, 0.6, 0.7,
        )))),
        outline_color: Some(Rc::new(ConstantProperty::from_value(Color::new(
            0.4, 0.3, 0.2, 0.1,
        )))),
        outline_width: Some(Rc::new(ConstantProperty::from_value(4.5))),
        horizontal_origin: Some(Rc::new(ConstantProperty::from_value(HorizontalOrigin::Right))),
        vertical_origin: Some(Rc::new(ConstantProperty::from_value(VerticalOrigin::Top))),
        eye_offset: Some(Rc::new(ConstantProperty::from_value(Cartesian3::new(
            1.0, 2.0, 3.0,
        )))),
        pixel_offset: Some(Rc::new(ConstantProperty::from_value(Cartesian2::new(3.0, 2.0)))),
        scale: Some(Rc::new(ConstantProperty::from_value(12.5))),
        show: Some(Rc::new(ConstantProperty::from_value(true))),
        translucency_by_distance: Some(Rc::new(ConstantProperty::from_value(NearFarScalar::new(
            10.0, 1.0, 100.0, 0.0,
        )))),
        pixel_offset_scale_by_distance: Some(Rc::new(ConstantProperty::from_value(NearFarScalar::new(
            20.0, 2.0, 200.0, 0.0,
        )))),
        scale_by_distance: Some(Rc::new(ConstantProperty::from_value(NearFarScalar::new(
            30.0, 3.0, 300.0, 0.0,
        )))),
        distance_display_condition: Some(Rc::new(ConstantProperty::from_value(
            DistanceDisplayCondition::new(10.0, 100.0),
        ))),
        disable_depth_test_distance: Some(Rc::new(ConstantProperty::from_value(10.0))),
        ..Default::default()
    };
    test_object.label = Some(LabelGraphics::from_options(label_opts));
    collection.borrow_mut().add(test_object).unwrap();

    let time = JulianDate::now();
    visualizer.update(Some(&time)).unwrap();

    {
        let cluster_ref = cluster.borrow();
        let label_col = cluster_ref.label_collection.as_ref().unwrap();
        assert_eq!(label_col.len(), 1);

        let l = label_col.get(0).unwrap();
        assert_eq!(l.position, Cartesian3::new(1234.0, 5678.0, 9101112.0));
        assert_eq!(l.text, "a");
        assert_eq!(l.font, "sans serif");
        assert_eq!(l.style, LabelStyle::Fill);
        assert_eq!(l.fill_color, Color::new(0.5, 0.8, 0.6, 0.7));
        assert_eq!(l.outline_color, Color::new(0.4, 0.3, 0.2, 0.1));
        assert_eq!(l.outline_width, 4.5);
        assert_eq!(l.horizontal_origin, HorizontalOrigin::Right);
        assert_eq!(l.vertical_origin, VerticalOrigin::Top);
        assert_eq!(l.eye_offset, Cartesian3::new(1.0, 2.0, 3.0));
        assert_eq!(l.pixel_offset, Cartesian2::new(3.0, 2.0));
        assert_eq!(l.scale, 12.5);
        assert!(l.show);
        assert_eq!(
            l.translucency_by_distance,
            Some(NearFarScalar::new(10.0, 1.0, 100.0, 0.0))
        );
        assert_eq!(
            l.pixel_offset_scale_by_distance,
            Some(NearFarScalar::new(20.0, 2.0, 200.0, 0.0))
        );
        assert_eq!(
            l.scale_by_distance,
            Some(NearFarScalar::new(30.0, 3.0, 300.0, 0.0))
        );
        assert_eq!(
            l.distance_display_condition,
            Some(DistanceDisplayCondition::new(10.0, 100.0))
        );
        assert_eq!(l.disable_depth_test_distance, Some(10.0));
    }

    // Now mutate entity and update
    if let Some(ent) = collection.borrow_mut().get_by_id_mut("test") {
        ent.position = Some(ConstantPositionProperty::new(
            Some(Cartesian3::new(5678.0, 1234.0, 1293434.0)),
            None,
        ));
        let label_opts2 = LabelGraphicsOptions {
            text: Some(Rc::new(ConstantProperty::from_value("b".to_string()))),
            font: Some(Rc::new(ConstantProperty::from_value("serif".to_string()))),
            style: Some(Rc::new(ConstantProperty::from_value(
                LabelStyle::FillAndOutline,
            ))),
            fill_color: Some(Rc::new(ConstantProperty::from_value(Color::new(
                0.1, 0.2, 0.3, 0.4,
            )))),
            outline_color: Some(Rc::new(ConstantProperty::from_value(Color::new(
                0.8, 0.7, 0.6, 0.5,
            )))),
            outline_width: Some(Rc::new(ConstantProperty::from_value(0.5))),
            horizontal_origin: Some(Rc::new(ConstantProperty::from_value(HorizontalOrigin::Center))),
            vertical_origin: Some(Rc::new(ConstantProperty::from_value(VerticalOrigin::Bottom))),
            eye_offset: Some(Rc::new(ConstantProperty::from_value(Cartesian3::new(
                3.0, 1.0, 2.0,
            )))),
            pixel_offset: Some(Rc::new(ConstantProperty::from_value(Cartesian2::new(2.0, 3.0)))),
            scale: Some(Rc::new(ConstantProperty::from_value(2.5))),
            show: Some(Rc::new(ConstantProperty::from_value(true))),
            translucency_by_distance: Some(Rc::new(ConstantProperty::from_value(NearFarScalar::new(
                1.0, 0.0, 2.0, 1.0,
            )))),
            pixel_offset_scale_by_distance: Some(Rc::new(ConstantProperty::from_value(NearFarScalar::new(
                3.0, 0.0, 4.0, 1.0,
            )))),
            scale_by_distance: Some(Rc::new(ConstantProperty::from_value(NearFarScalar::new(
                5.0, 0.0, 6.0, 1.0,
            )))),
            distance_display_condition: Some(Rc::new(ConstantProperty::from_value(
                DistanceDisplayCondition::new(5.0, 50.0),
            ))),
            disable_depth_test_distance: Some(Rc::new(ConstantProperty::from_value(20.0))),
            ..Default::default()
        };
        ent.label = Some(LabelGraphics::from_options(label_opts2));
    }

    visualizer.update(Some(&time)).unwrap();

    {
        let cluster_ref = cluster.borrow();
        let label_col = cluster_ref.label_collection.as_ref().unwrap();
        let l = label_col.get(0).unwrap();
        assert_eq!(l.position, Cartesian3::new(5678.0, 1234.0, 1293434.0));
        assert_eq!(l.text, "b");
        assert_eq!(l.font, "serif");
        assert_eq!(l.style, LabelStyle::FillAndOutline);
        assert_eq!(l.fill_color, Color::new(0.1, 0.2, 0.3, 0.4));
        assert_eq!(l.outline_color, Color::new(0.8, 0.7, 0.6, 0.5));
        assert_eq!(l.outline_width, 0.5);
        assert_eq!(l.horizontal_origin, HorizontalOrigin::Center);
        assert_eq!(l.vertical_origin, VerticalOrigin::Bottom);
        assert_eq!(l.eye_offset, Cartesian3::new(3.0, 1.0, 2.0));
        assert_eq!(l.pixel_offset, Cartesian2::new(2.0, 3.0));
        assert_eq!(l.scale, 2.5);
        assert!(l.show);
        assert_eq!(
            l.translucency_by_distance,
            Some(NearFarScalar::new(1.0, 0.0, 2.0, 1.0))
        );
        assert_eq!(
            l.pixel_offset_scale_by_distance,
            Some(NearFarScalar::new(3.0, 0.0, 4.0, 1.0))
        );
        assert_eq!(
            l.scale_by_distance,
            Some(NearFarScalar::new(5.0, 0.0, 6.0, 1.0))
        );
        assert_eq!(
            l.distance_display_condition,
            Some(DistanceDisplayCondition::new(5.0, 50.0))
        );
        assert_eq!(l.disable_depth_test_distance, Some(20.0));
    }

    // Hide label
    if let Some(ent) = collection.borrow_mut().get_by_id_mut("test") {
        ent.label.as_mut().unwrap().show = Some(Rc::new(ConstantProperty::from_value(false)));
    }
    visualizer.update(Some(&time)).unwrap();

    let cluster_ref = cluster.borrow();
    let label_col = cluster_ref.label_collection.as_ref().unwrap();
    let l = label_col.get(0).unwrap();
    assert!(!l.show);
}

#[test]
fn test_reuses_primitives_when_hiding_one_and_showing_another() {
    let time = JulianDate::now();
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        LabelVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    let mut label = LabelGraphics::default();
    label.text = Some(Rc::new(ConstantProperty::from_value("a".to_string())));
    label.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object.label = Some(label);
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&time)).unwrap();
    assert_eq!(cluster.borrow().label_collection.as_ref().unwrap().len(), 1);

    if let Some(ent) = collection.borrow_mut().get_by_id_mut("test") {
        ent.label.as_mut().unwrap().show = Some(Rc::new(ConstantProperty::from_value(false)));
    }

    visualizer.update(Some(&time)).unwrap();
    assert_eq!(cluster.borrow().label_collection.as_ref().unwrap().len(), 1);

    let mut test_object2 = Entity::with_id("test2");
    test_object2.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    let mut label2 = LabelGraphics::default();
    label2.text = Some(Rc::new(ConstantProperty::from_value("b".to_string())));
    label2.show = Some(Rc::new(ConstantProperty::from_value(true)));
    test_object2.label = Some(label2);
    collection.borrow_mut().add(test_object2).unwrap();

    visualizer.update(Some(&time)).unwrap();
    assert_eq!(cluster.borrow().label_collection.as_ref().unwrap().len(), 1);
}

#[test]
fn test_clear_hides_labels() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        LabelVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    let time = JulianDate::now();
    let mut label = LabelGraphics::default();
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    label.show = Some(Rc::new(ConstantProperty::from_value(true)));
    label.text = Some(Rc::new(ConstantProperty::from_value("lorum ipsum".to_string())));
    test_object.label = Some(label);
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&time)).unwrap();

    {
        let cluster_ref = cluster.borrow();
        let label_col = cluster_ref.label_collection.as_ref().unwrap();
        assert_eq!(label_col.len(), 1);
        let l = label_col.get(0).unwrap();
        assert!(l.show);
    }

    collection.borrow_mut().remove_all();
    visualizer.update(Some(&time)).unwrap();

    let cluster_ref = cluster.borrow();
    let label_col = cluster_ref.label_collection.as_ref().unwrap();
    let l = label_col.get(0).unwrap();
    assert!(!l.show);
    assert!(l.id.is_none());
}

#[test]
fn test_visualizer_sets_entity_property() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        LabelVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    let time = JulianDate::now();
    let mut label = LabelGraphics::default();
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    label.show = Some(Rc::new(ConstantProperty::from_value(true)));
    label.text = Some(Rc::new(ConstantProperty::from_value("lorum ipsum".to_string())));
    test_object.label = Some(label);
    collection.borrow_mut().add(test_object).unwrap();

    visualizer.update(Some(&time)).unwrap();

    let cluster_ref = cluster.borrow();
    let label_col = cluster_ref.label_collection.as_ref().unwrap();
    assert_eq!(label_col.len(), 1);
    let l = label_col.get(0).unwrap();
    assert_eq!(l.id.as_deref(), Some("test"));
}

#[test]
fn test_computes_bounding_sphere() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        LabelVisualizer::new(Some(Rc::clone(&cluster)), Some(Rc::clone(&collection))).unwrap();

    let mut test_object = Entity::with_id("test");
    let time = JulianDate::now();
    let mut label = LabelGraphics::default();
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    label.show = Some(Rc::new(ConstantProperty::from_value(true)));
    label.text = Some(Rc::new(ConstantProperty::from_value("lorum ipsum".to_string())));
    test_object.label = Some(label);
    collection.borrow_mut().add(test_object.clone()).unwrap();

    visualizer.update(Some(&time)).unwrap();

    let mut result = BoundingSphere::default();
    let state = visualizer
        .get_bounding_sphere(Some(&test_object), Some(&mut result))
        .unwrap();

    assert_eq!(state, BoundingSphereState::Done);
    assert_eq!(
        result.center,
        DVec3::from(
            test_object
                .position
                .as_ref()
                .unwrap()
                .get_value(Some(&time))
                .unwrap()
        )
    );
    assert_eq!(result.radius, 0.0);
}

#[test]
fn test_fails_bounding_sphere_for_entity_without_label() {
    let cluster = Rc::new(RefCell::new(EntityCluster::default()));
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        LabelVisualizer::new(Some(cluster), Some(Rc::clone(&collection))).unwrap();

    let test_object = Entity::with_id("test");
    collection.borrow_mut().add(test_object.clone()).unwrap();

    let time = JulianDate::now();
    visualizer.update(Some(&time)).unwrap();

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
    let visualizer = LabelVisualizer::new(Some(cluster), Some(collection)).unwrap();

    let mut result = BoundingSphere::default();
    assert!(visualizer
        .get_bounding_sphere(None, Some(&mut result))
        .is_err());

    let test_object = Entity::new();
    assert!(visualizer
        .get_bounding_sphere(Some(&test_object), None)
        .is_err());
}
