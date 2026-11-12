//! Direct port and verification against CesiumJS `StaticGeometryPerMaterialBatchSpec.js` (Apache-2.0).

use std::rc::Rc;

use cesium_math::{
    BoundingSphere, BoundingSphereState, BoxGeometryUpdater, Cartesian2,
    Cartesian3, CheckerboardMaterialProperty, Color, ColorMaterialProperty,
    ConstantPositionProperty, ConstantProperty, DistanceDisplayCondition,
    EllipseGeometryUpdater, EllipseGraphics, EllipseGraphicsOptions, Entity,
    GeometryVisualizerScene, GridMaterialProperty, JulianDate, PolylineArrowMaterialProperty,
    PolylineGeometryUpdater, PolylineGraphics, PolylineGraphicsOptions, ShadowMode,
    StaticGeometryPerMaterialBatch, StripeMaterialProperty, TimeInterval,
    TimeIntervalCollection, TimeIntervalCollectionProperty,
};

#[test]
fn handles_shared_material_being_invalidated_with_geometry() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGeometryPerMaterialBatch::new(
        Rc::clone(&scene.primitives),
        None,
        None,
        false,
        ShadowMode::Disabled,
    );

    let mut ellipse1 = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(GridMaterialProperty::default())),
        ..Default::default()
    });

    let mut entity1 = Entity::new();
    entity1.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity1.ellipse = Some(ellipse1.clone());

    let ellipse2 = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(3.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        material: Some(Rc::new(GridMaterialProperty::default())),
        ..Default::default()
    });

    let mut entity2 = Entity::new();
    entity2.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity2.ellipse = Some(ellipse2);

    let updater1 = EllipseGeometryUpdater::new(entity1.clone(), None).unwrap();
    let updater2 = EllipseGeometryUpdater::new(entity2.clone(), None).unwrap();

    batch.add(time, &updater1);
    batch.add(time, &updater2);

    assert!(batch.update(time));
    assert_eq!(scene.primitives.borrow().len(), 1);

    // Invalidate shared material by changing cell_alpha on entity1
    let mut modified_grid = GridMaterialProperty::default();
    modified_grid.cell_alpha = Some(ConstantProperty::from_value(0.5));
    ellipse1.material = Some(Rc::new(modified_grid));
    entity1.ellipse = Some(ellipse1);

    let updater1_modified = EllipseGeometryUpdater::new(entity1, None).unwrap();
    batch.add(time, &updater1_modified);

    assert!(batch.update(time));
    assert_eq!(scene.primitives.borrow().len(), 2);

    batch.remove_all_primitives();
    assert_eq!(scene.primitives.borrow().len(), 0);
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
        extruded_height: Some(Rc::new(ConstantProperty::from_value(20.0))),
        material: Some(Rc::new(GridMaterialProperty::default())),
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
    let mut batch = StaticGeometryPerMaterialBatch::new(
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
        material: Some(Rc::new(GridMaterialProperty::default())),
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
    let mut batch = StaticGeometryPerMaterialBatch::new(
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
        assert_eq!(attributes.show, Some([1]));
    }

    assert!(batch.update(out_of_range_time));

    {
        let prims = scene.primitives.borrow();
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attributes.show, Some([0]));
    }

    batch.remove_all_primitives();
}

#[test]
fn handles_shared_material_being_invalidated_for_polyline() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGeometryPerMaterialBatch::new(
        Rc::clone(&scene.primitives),
        None,
        None,
        false,
        ShadowMode::Disabled,
    );

    let mut polyline1 = PolylineGraphics::from_options(PolylineGraphicsOptions {
        positions: Some(Rc::new(ConstantProperty::from_value(vec![
            Cartesian3::new(6378137.0, 0.0, 0.0),
            Cartesian3::new(6378137.0, 1000.0, 0.0),
        ]))),
        material: Some(Rc::new(PolylineArrowMaterialProperty::default())),
        ..Default::default()
    });

    let mut entity1 = Entity::new();
    entity1.polyline = Some(polyline1.clone());

    let polyline2 = PolylineGraphics::from_options(PolylineGraphicsOptions {
        positions: Some(Rc::new(ConstantProperty::from_value(vec![
            Cartesian3::new(6378137.0, 0.0, 0.0),
            Cartesian3::new(6378137.0, 1000.0, 0.0),
        ]))),
        material: Some(Rc::new(PolylineArrowMaterialProperty::default())),
        ..Default::default()
    });

    let mut entity2 = Entity::new();
    entity2.polyline = Some(polyline2);

    let updater1 = PolylineGeometryUpdater::new(entity1.clone(), None).unwrap();
    let updater2 = PolylineGeometryUpdater::new(entity2.clone(), None).unwrap();

    batch.add(time, &updater1);
    batch.add(time, &updater2);

    assert!(batch.update(time));
    assert_eq!(scene.primitives.borrow().len(), 1);

    // Modify polyline1 material to RED
    polyline1.material = Some(Rc::new(PolylineArrowMaterialProperty::from_color(
        Color::RED,
    )));
    entity1.polyline = Some(polyline1);

    let updater1_modified = PolylineGeometryUpdater::new(entity1, None).unwrap();
    batch.add(time, &updater1_modified);

    assert!(batch.update(time));
    assert_eq!(scene.primitives.borrow().len(), 2);

    batch.remove_all_primitives();
}

#[test]
fn updates_with_sampled_depth_fail_color_out_of_range() {
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

    let polyline = PolylineGraphics::from_options(PolylineGraphicsOptions {
        positions: Some(Rc::new(ConstantProperty::from_value(vec![
            Cartesian3::new(6378137.0, 0.0, 0.0),
            Cartesian3::new(6378137.0, 1000.0, 0.0),
        ]))),
        material: Some(Rc::new(PolylineArrowMaterialProperty::default())),
        depth_fail_material: Some(Rc::new(ColorMaterialProperty::from_dynamic_color(Rc::new(
            color_prop,
        )))),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.availability = Some(entity_availability);
    entity.polyline = Some(polyline);

    let scene = GeometryVisualizerScene::new();
    let mut batch = StaticGeometryPerMaterialBatch::new(
        Rc::clone(&scene.primitives),
        None,
        None,
        false,
        ShadowMode::Disabled,
    );

    let updater = PolylineGeometryUpdater::new(entity.clone(), None).unwrap();
    batch.add(valid_time, &updater);

    assert!(batch.update(valid_time));

    {
        let prims = scene.primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attributes.depth_fail_color, Some([255, 0, 0, 255]));
    }

    assert!(batch.update(out_of_range_time));

    {
        let prims = scene.primitives.borrow();
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attributes.depth_fail_color, Some([255, 255, 255, 255]));
    }

    batch.remove_all_primitives();
}

#[test]
fn shows_only_one_primitive_while_rebuilding_primitive() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGeometryPerMaterialBatch::new(
        Rc::clone(&scene.primitives),
        None,
        None,
        false,
        ShadowMode::Disabled,
    );

    fn build_entity() -> Entity {
        let grid = GridMaterialProperty::new(
            Some(ConstantProperty::from_value(Color::YELLOW)),
            Some(ConstantProperty::from_value(0.3)),
            Some(ConstantProperty::from_value(Cartesian2::new(8.0, 8.0))),
            Some(ConstantProperty::from_value(Cartesian2::new(2.0, 2.0))),
            None,
        );

        let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
            semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
            semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
            height: Some(Rc::new(ConstantProperty::from_value(0.0))),
            material: Some(Rc::new(grid)),
            ..Default::default()
        });

        let mut entity = Entity::new();
        entity.position = Some(ConstantPositionProperty::new(
            Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
            None,
        ));
        entity.ellipse = Some(ellipse);
        entity
    }

    let entity1 = build_entity();
    let entity2 = build_entity();

    let updater1 = EllipseGeometryUpdater::new(entity1, None).unwrap();
    let updater2 = EllipseGeometryUpdater::new(entity2, None).unwrap();

    batch.add(time, &updater1);
    assert!(batch.update(time));
    assert_eq!(scene.primitives.borrow().len(), 1);
    assert!(scene.primitives.borrow()[0].show);

    batch.add(time, &updater2);
    assert!(batch.update(time));
    assert_eq!(scene.primitives.borrow().len(), 1);
    assert!(scene.primitives.borrow()[0].show);

    batch.remove_all_primitives();
}

#[test]
fn removes_all_updaters() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGeometryPerMaterialBatch::new(
        Rc::clone(&scene.primitives),
        None,
        None,
        false,
        ShadowMode::Disabled,
    );

    let entity1 = {
        let stripe = StripeMaterialProperty::new(
            None,
            Some(ConstantProperty::from_value(Color::YELLOW)),
            Some(ConstantProperty::from_value(Color::BLUE)),
            None,
            None,
        );
        let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
            semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
            semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
            material: Some(Rc::new(stripe)),
            ..Default::default()
        });
        let mut entity = Entity::new();
        entity.position = Some(ConstantPositionProperty::new(
            Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
            None,
        ));
        entity.ellipse = Some(ellipse);
        entity
    };

    let entity2 = {
        let checkerboard = CheckerboardMaterialProperty::new(
            Some(ConstantProperty::from_value(Color::YELLOW)),
            Some(ConstantProperty::from_value(Color::BLUE)),
            None,
        );
        let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
            semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
            semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
            material: Some(Rc::new(checkerboard)),
            ..Default::default()
        });
        let mut entity = Entity::new();
        entity.position = Some(ConstantPositionProperty::new(
            Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
            None,
        ));
        entity.ellipse = Some(ellipse);
        entity
    };

    let entity3 = {
        let stripe = StripeMaterialProperty::new(
            None,
            Some(ConstantProperty::from_value(Color::YELLOW)),
            Some(ConstantProperty::from_value(Color::BLUE)),
            None,
            None,
        );
        let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
            semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
            semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
            material: Some(Rc::new(stripe)),
            ..Default::default()
        });
        let mut entity = Entity::new();
        entity.position = Some(ConstantPositionProperty::new(
            Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
            None,
        ));
        entity.ellipse = Some(ellipse);
        entity
    };

    let entity4 = {
        let checkerboard = CheckerboardMaterialProperty::new(
            Some(ConstantProperty::from_value(Color::YELLOW)),
            Some(ConstantProperty::from_value(Color::BLUE)),
            None,
        );
        let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
            semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
            semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
            material: Some(Rc::new(checkerboard)),
            ..Default::default()
        });
        let mut entity = Entity::new();
        entity.position = Some(ConstantPositionProperty::new(
            Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
            None,
        ));
        entity.ellipse = Some(ellipse);
        entity
    };

    let updater1 = EllipseGeometryUpdater::new(entity1.clone(), None).unwrap();
    let updater2 = EllipseGeometryUpdater::new(entity2.clone(), None).unwrap();
    let updater3 = EllipseGeometryUpdater::new(entity3.clone(), None).unwrap();
    let updater4 = EllipseGeometryUpdater::new(entity4.clone(), None).unwrap();
    let empty_updater = BoxGeometryUpdater::new(entity1.clone(), None).unwrap();

    batch.add(time, &updater1);
    batch.add(time, &updater2);
    batch.add(time, &updater3);
    batch.add(time, &updater4);

    assert!(batch.update(time));
    assert_eq!(scene.primitives.borrow().len(), 2);

    batch.remove(&updater1);
    batch.remove(&updater2);
    batch.remove(&empty_updater);
    batch.remove(&updater3);
    batch.remove(&updater4);

    assert!(batch.update(time));
    assert_eq!(scene.primitives.borrow().len(), 0);

    batch.remove_all_primitives();
}

#[test]
fn has_correct_show_attribute_after_rebuilding_primitive() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGeometryPerMaterialBatch::new(
        Rc::clone(&scene.primitives),
        None,
        None,
        false,
        ShadowMode::Disabled,
    );

    let grid = GridMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::YELLOW)),
        Some(ConstantProperty::from_value(0.3)),
        Some(ConstantProperty::from_value(Cartesian2::new(8.0, 8.0))),
        Some(ConstantProperty::from_value(Cartesian2::new(2.0, 2.0))),
        None,
    );

    let mut entity1 = Entity::new();
    entity1.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity1.ellipse = Some(EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(grid.clone())),
        ..Default::default()
    }));

    let mut entity2 = Entity::new();
    entity2.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity2.ellipse = Some(EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(grid)),
        ..Default::default()
    }));

    let mut updater1 = EllipseGeometryUpdater::new(entity1.clone(), None).unwrap();
    batch.add(time, &updater1);

    assert!(batch.update(time));
    assert_eq!(scene.primitives.borrow().len(), 1);
    {
        let prims = scene.primitives.borrow();
        let attrs = prims[0].get_geometry_instance_attributes(&entity1).unwrap();
        assert_eq!(attrs.show, Some([1]));
    }

    // Set entity1.show = false
    entity1.show = false;
    updater1.on_entity_property_changed(&entity1, "isShowing");
    batch.add(time, &updater1);

    assert!(batch.update(time));
    assert_eq!(scene.primitives.borrow().len(), 1);
    {
        let prims = scene.primitives.borrow();
        let attrs = prims[0].get_geometry_instance_attributes(&entity1).unwrap();
        assert_eq!(attrs.show, Some([0]));
    }

    let updater2 = EllipseGeometryUpdater::new(entity2.clone(), None).unwrap();
    batch.add(time, &updater2);

    assert!(batch.update(time));
    assert_eq!(scene.primitives.borrow().len(), 1);
    {
        let prims = scene.primitives.borrow();
        let attrs1 = prims[0].get_geometry_instance_attributes(&entity1).unwrap();
        assert_eq!(attrs1.show, Some([0]));

        let attrs2 = prims[0].get_geometry_instance_attributes(&entity2).unwrap();
        assert_eq!(attrs2.show, Some([1]));
    }

    batch.remove_all_primitives();
}

#[test]
fn computes_bounding_sphere() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGeometryPerMaterialBatch::new(
        Rc::clone(&scene.primitives),
        None,
        None,
        false,
        ShadowMode::Disabled,
    );

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(GridMaterialProperty::default())),
        ..Default::default()
    }));

    let updater = EllipseGeometryUpdater::new(entity, None).unwrap();
    batch.add(time, &updater);
    assert!(batch.update(time));

    let mut result = BoundingSphere::default();
    let state = batch.get_bounding_sphere(&updater, &mut result);
    assert_eq!(state, BoundingSphereState::Done);
    assert_eq!(result.center, Cartesian3::new(1234.0, 5678.0, 9101112.0).into());
    assert_eq!(result.radius, 10.0);
}
