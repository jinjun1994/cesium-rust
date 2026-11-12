//! Direct port and verification against CesiumJS `StaticGeometryColorBatchSpec.js` (Apache-2.0).

use std::rc::Rc;

use cesium_math::{
    BoundingSphere, BoundingSphereState, Cartesian3, Color, ColorGeometryInstanceAttribute,
    ColorMaterialProperty, ConstantPositionProperty, ConstantProperty,
    DistanceDisplayCondition, EllipseGeometryUpdater, EllipseGraphics, EllipseGraphicsOptions,
    Entity, GeometryVisualizerScene, JulianDate, ShadowMode, ShowGeometryInstanceAttribute,
    StaticGeometryColorBatch, TimeInterval, TimeIntervalCollection,
    TimeIntervalCollectionProperty,
};

#[test]
fn updates_color_attribute_after_rebuilding_geometry_primitive() {
    let scene = GeometryVisualizerScene::new();
    let time = JulianDate::now();

    let mut batch = StaticGeometryColorBatch::new(
        Rc::clone(&scene.primitives),
        None,
        None,
        false,
        ShadowMode::Disabled,
    );

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(Color::RED))),
        height: Some(Rc::new(ConstantProperty::from_value(0.0))),
        show: Some(Rc::new(ConstantProperty::from_value(true))),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(ellipse);

    let mut updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
    batch.add(time, &updater);

    assert!(batch.update(time));

    {
        let prims = scene.primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(
            attributes.color,
            Some(ColorGeometryInstanceAttribute::to_value(&Color::RED))
        );
    }

    if let Some(ref mut el) = entity.ellipse {
        el.material = Some(Rc::new(ColorMaterialProperty::from_color(Color::GREEN)));
    }
    updater.on_entity_property_changed(&entity, "ellipse");
    batch.remove(&updater);
    batch.add(time, &updater);

    assert!(batch.update(time));

    {
        let prims = scene.primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(
            attributes.color,
            Some(ColorGeometryInstanceAttribute::to_value(&Color::GREEN))
        );
    }

    batch.remove_all_primitives();
    assert_eq!(scene.primitives.borrow().len(), 0);
}

#[test]
fn updates_with_sampled_color_out_of_range() {
    let valid_time = JulianDate::from_iso8601("2018-02-14T04:10:00Z").unwrap();
    let out_of_range_time = JulianDate::from_iso8601("2018-02-14T04:20:00Z").unwrap();

    let mut color_prop = TimeIntervalCollectionProperty::new();
    color_prop.intervals.add_interval(TimeInterval::with_data(
        JulianDate::from_iso8601("2018-02-14T04:00:00Z").unwrap(),
        JulianDate::from_iso8601("2018-02-14T04:15:00Z").unwrap(),
        true,
        true,
        Color::RED,
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
        extruded_height: Some(Rc::new(ConstantProperty::from_value(20.0))),
        material: Some(Rc::new(ColorMaterialProperty::from_dynamic_color(Rc::new(
            color_prop,
        )))),
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
    let mut batch = StaticGeometryColorBatch::new(
        Rc::clone(&scene.primitives),
        None,
        None,
        false,
        ShadowMode::Disabled,
    );

    let updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
    batch.add(valid_time, &updater);

    assert!(batch.update(valid_time));

    {
        let prims = scene.primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(
            attributes.color,
            Some(ColorGeometryInstanceAttribute::to_value(&Color::RED))
        );
    }

    assert!(batch.update(out_of_range_time));

    {
        let prims = scene.primitives.borrow();
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(
            attributes.color,
            Some(ColorGeometryInstanceAttribute::to_value(&Color::WHITE))
        );
    }

    batch.remove_all_primitives();
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

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        extruded_height: Some(Rc::new(ConstantProperty::from_value(20.0))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(Color::RED))),
        distance_display_condition: Some(Rc::new(ddc_prop)),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(ellipse);

    let scene = GeometryVisualizerScene::new();
    let mut batch = StaticGeometryColorBatch::new(
        Rc::clone(&scene.primitives),
        None,
        None,
        false,
        ShadowMode::Disabled,
    );

    let updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
    batch.add(valid_time, &updater);

    assert!(batch.update(valid_time));

    {
        let prims = scene.primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attributes.distance_display_condition, Some([1.0, 2.0]));
    }

    assert!(batch.update(out_of_range_time));

    {
        let prims = scene.primitives.borrow();
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

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        extruded_height: Some(Rc::new(ConstantProperty::from_value(20.0))),
        show: Some(Rc::new(show_prop)),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(ellipse);

    let scene = GeometryVisualizerScene::new();
    let mut batch = StaticGeometryColorBatch::new(
        Rc::clone(&scene.primitives),
        None,
        None,
        false,
        ShadowMode::Disabled,
    );

    let updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
    batch.add(valid_time, &updater);

    assert!(batch.update(valid_time));

    {
        let prims = scene.primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attributes.show, Some(ShowGeometryInstanceAttribute::to_value(true)));
    }

    assert!(batch.update(out_of_range_time));

    {
        let prims = scene.primitives.borrow();
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attributes.show, Some(ShowGeometryInstanceAttribute::to_value(false)));
    }

    batch.remove_all_primitives();
}

#[test]
fn computes_bounding_sphere() {
    let scene = GeometryVisualizerScene::new();
    let time = JulianDate::now();

    let mut batch = StaticGeometryColorBatch::new(
        Rc::clone(&scene.primitives),
        None,
        None,
        false,
        ShadowMode::Disabled,
    );

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(Color::RED))),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(10.0, 20.0, 30.0)),
        None,
    ));
    entity.ellipse = Some(ellipse);

    let updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
    batch.add(time, &updater);
    batch.update(time);

    let mut result = BoundingSphere::default();
    let state = batch.get_bounding_sphere(&updater, &mut result);
    assert_eq!(state, BoundingSphereState::Done);
    assert_eq!(result.center, Cartesian3::new(10.0, 20.0, 30.0).into());

    batch.remove_all_primitives();
}
