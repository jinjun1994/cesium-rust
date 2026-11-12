//! Direct port and verification against CesiumJS `StaticGroundGeometryColorBatchSpec.js` (Apache-2.0).

use std::rc::Rc;

use cesium_math::{
    Cartesian3, ClassificationType, Color, ColorGeometryInstanceAttribute,
    ColorMaterialProperty, ConstantPositionProperty, ConstantProperty,
    DistanceDisplayCondition, EllipseGeometryUpdater, EllipseGraphics, EllipseGraphicsOptions,
    Entity, GeometryVisualizerScene, JulianDate, StaticGroundGeometryColorBatch, TimeInterval,
    TimeIntervalCollection, TimeIntervalCollectionProperty,
};

#[test]
fn updates_color_attribute_after_rebuilding_primitive() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundGeometryColorBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
    );

    let mut ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        show: Some(Rc::new(ConstantProperty::from_value(true))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(Color::RED))),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(ellipse.clone());

    let updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
    batch.add(time, &updater);

    assert!(batch.update(time));

    {
        let prims = scene.ground_primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(
            attributes.color,
            Some(ColorGeometryInstanceAttribute::to_value(&Color::RED))
        );
        assert_eq!(batch.batches.len(), 1);
    }

    // Change color to GREEN
    ellipse.material = Some(Rc::new(ColorMaterialProperty::from_color(Color::GREEN)));
    entity.ellipse = Some(ellipse);
    let mut updater2 = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
    updater2.on_entity_property_changed(&entity, "ellipse");
    batch.remove(&updater);
    batch.add(time, &updater2);

    assert!(batch.update(time));

    {
        let prims = scene.ground_primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(
            attributes.color,
            Some(ColorGeometryInstanceAttribute::to_value(&Color::GREEN))
        );
        assert_eq!(batch.batches.len(), 1);
    }

    batch.remove_all_primitives();
    assert_eq!(scene.ground_primitives.borrow().len(), 0);
}

#[test]
fn batches_overlapping_geometry_separately() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundGeometryColorBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
    );

    let entity1 = {
        let mut e = Entity::new();
        e.position = Some(ConstantPositionProperty::new(
            Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
            None,
        ));
        e.ellipse = Some(EllipseGraphics::from_options(EllipseGraphicsOptions {
            semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
            semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
            show: Some(Rc::new(ConstantProperty::from_value(true))),
            material: Some(Rc::new(ColorMaterialProperty::from_color(Color::RED))),
            ..Default::default()
        }));
        e
    };

    let entity2 = {
        let mut e = Entity::new();
        e.position = Some(ConstantPositionProperty::new(
            Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
            None,
        ));
        e.ellipse = Some(EllipseGraphics::from_options(EllipseGraphicsOptions {
            semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
            semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
            show: Some(Rc::new(ConstantProperty::from_value(true))),
            material: Some(Rc::new(ColorMaterialProperty::from_color(Color::RED))),
            ..Default::default()
        }));
        e
    };

    let updater1 = EllipseGeometryUpdater::new(entity1, None).unwrap();
    let updater2 = EllipseGeometryUpdater::new(entity2, None).unwrap();

    batch.add(time, &updater1);
    batch.add(time, &updater2);

    assert_eq!(batch.batches.len(), 2);
}

#[test]
fn updates_with_sampled_distance_display_condition_out_of_range() {
    let valid_time = JulianDate::from_iso8601("2018-02-14T04:10:00Z").unwrap();
    let out_of_range_time = JulianDate::from_iso8601("2018-02-14T04:20:00Z").unwrap();

    let mut ddc_prop = TimeIntervalCollectionProperty::new();
    ddc_prop.intervals.add_interval(TimeInterval::with_data(
        JulianDate::from_iso8601("2018-02-14T04:00:00Z").unwrap(),
        JulianDate::from_iso8601("2018-02-14T04:15:00Z").unwrap(),
        true,
        true,
        DistanceDisplayCondition::new(1.0, 2.0),
    ));

    let mut entity_availability = TimeIntervalCollection::new();
    entity_availability.add_interval(TimeInterval::new(
        JulianDate::from_iso8601("2018-02-14T04:00:00Z").unwrap(),
        JulianDate::from_iso8601("2018-02-14T04:30:00Z").unwrap(),
        true,
        true,
    ));

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(Color::RED))),
        distance_display_condition: Some(Rc::new(ddc_prop)),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.availability = Some(entity_availability);
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(ellipse);

    let scene = GeometryVisualizerScene::new();
    let mut batch = StaticGroundGeometryColorBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
    );

    let updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
    batch.add(valid_time, &updater);

    assert!(batch.update(valid_time));

    {
        let prims = scene.ground_primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attributes.distance_display_condition, Some([1.0, 2.0]));
    }

    assert!(batch.update(out_of_range_time));

    {
        let prims = scene.ground_primitives.borrow();
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(
            attributes.distance_display_condition,
            Some([0.0, f32::INFINITY])
        );
    }

    batch.remove_all_primitives();
}

#[test]
fn updates_with_sampled_show_out_of_range() {
    let valid_time = JulianDate::from_iso8601("2018-02-14T04:10:00Z").unwrap();
    let out_of_range_time = JulianDate::from_iso8601("2018-02-14T04:20:00Z").unwrap();

    let mut show_prop = TimeIntervalCollectionProperty::new();
    show_prop.intervals.add_interval(TimeInterval::with_data(
        JulianDate::from_iso8601("2018-02-14T04:00:00Z").unwrap(),
        JulianDate::from_iso8601("2018-02-14T04:15:00Z").unwrap(),
        true,
        true,
        true,
    ));

    let mut entity_availability = TimeIntervalCollection::new();
    entity_availability.add_interval(TimeInterval::new(
        JulianDate::from_iso8601("2018-02-14T04:00:00Z").unwrap(),
        JulianDate::from_iso8601("2018-02-14T04:30:00Z").unwrap(),
        true,
        true,
    ));

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(Color::RED))),
        show: Some(Rc::new(show_prop)),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.availability = Some(entity_availability);
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(ellipse);

    let scene = GeometryVisualizerScene::new();
    let mut batch = StaticGroundGeometryColorBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
    );

    let updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
    batch.add(valid_time, &updater);

    assert!(batch.update(valid_time));

    {
        let prims = scene.ground_primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attributes.show, Some([1]));
    }

    assert!(batch.update(out_of_range_time));

    {
        let prims = scene.ground_primitives.borrow();
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attributes.show, Some([0]));
    }

    batch.remove_all_primitives();
}

#[test]
fn shows_only_one_primitive_while_rebuilding_primitive() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundGeometryColorBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
    );

    let mut entity1 = Entity::new();
    entity1.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity1.ellipse = Some(EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(0.2))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(0.1))),
        outline: Some(Rc::new(ConstantProperty::from_value(true))),
        outline_color: Some(Rc::new(ConstantProperty::from_value(Color::new(
            1.0, 0.0, 0.0, 0.5,
        )))),
        ..Default::default()
    }));

    let mut entity2 = Entity::new();
    entity2.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 4678.0, 9101112.0)),
        None,
    ));
    entity2.ellipse = Some(EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(0.2))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(0.1))),
        outline: Some(Rc::new(ConstantProperty::from_value(true))),
        outline_color: Some(Rc::new(ConstantProperty::from_value(Color::new(
            1.0, 0.0, 0.0, 0.5,
        )))),
        ..Default::default()
    }));

    let updater1 = EllipseGeometryUpdater::new(entity1, None).unwrap();
    let updater2 = EllipseGeometryUpdater::new(entity2, None).unwrap();

    batch.add(time, &updater1);
    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 1);
    assert!(scene.ground_primitives.borrow()[0].show);

    batch.add(time, &updater2);
    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 1);
    assert!(scene.ground_primitives.borrow()[0].show);

    batch.remove_all_primitives();
}

#[test]
fn has_correct_show_attribute_after_rebuilding_primitive() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundGeometryColorBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
    );

    let mut entity1 = Entity::new();
    entity1.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity1.ellipse = Some(EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(0.2))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(0.1))),
        outline: Some(Rc::new(ConstantProperty::from_value(true))),
        outline_color: Some(Rc::new(ConstantProperty::from_value(Color::new(
            1.0, 0.0, 0.0, 0.5,
        )))),
        ..Default::default()
    }));

    let mut entity2 = Entity::new();
    entity2.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 4678.0, 9101112.0)),
        None,
    ));
    entity2.ellipse = Some(EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(0.2))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(0.1))),
        outline: Some(Rc::new(ConstantProperty::from_value(true))),
        outline_color: Some(Rc::new(ConstantProperty::from_value(Color::new(
            1.0, 0.0, 0.0, 0.5,
        )))),
        ..Default::default()
    }));

    let mut updater1 = EllipseGeometryUpdater::new(entity1.clone(), None).unwrap();
    batch.add(time, &updater1);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 1);
    {
        let prims = scene.ground_primitives.borrow();
        let attrs = prims[0].get_geometry_instance_attributes(&entity1).unwrap();
        assert_eq!(attrs.show, Some([1]));
    }

    entity1.show = false;
    updater1.on_entity_property_changed(&entity1, "isShowing");
    batch.add(time, &updater1);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 1);
    {
        let prims = scene.ground_primitives.borrow();
        let attrs = prims[0].get_geometry_instance_attributes(&entity1).unwrap();
        assert_eq!(attrs.show, Some([0]));
    }

    let updater2 = EllipseGeometryUpdater::new(entity2.clone(), None).unwrap();
    batch.add(time, &updater2);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 1);
    {
        let prims = scene.ground_primitives.borrow();
        let attrs1 = prims[0].get_geometry_instance_attributes(&entity1).unwrap();
        assert_eq!(attrs1.show, Some([0]));

        let attrs2 = prims[0].get_geometry_instance_attributes(&entity2).unwrap();
        assert_eq!(attrs2.show, Some([1]));
    }

    batch.remove_all_primitives();
}
