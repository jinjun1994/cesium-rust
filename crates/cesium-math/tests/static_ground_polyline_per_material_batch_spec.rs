//! Direct port and verification against CesiumJS `StaticGroundPolylinePerMaterialBatchSpec.js` (Apache-2.0).

use std::rc::Rc;

use cesium_math::{
    BoundingSphere, BoundingSphereState, Cartesian3, ClassificationType, Color,
    ColorMaterialProperty, ConstantProperty, DistanceDisplayCondition, Entity,
    GeometryVisualizerScene, JulianDate, PolylineGeometryUpdater, PolylineGraphics,
    PolylineGraphicsOptions, PolylineOutlineMaterialProperty,
    StaticGroundPolylinePerMaterialBatch, TimeInterval, TimeIntervalCollection,
    TimeIntervalCollectionProperty,
};

fn create_ground_polyline() -> PolylineGraphics {
    PolylineGraphics::from_options(PolylineGraphicsOptions {
        clamp_to_ground: Some(Rc::new(ConstantProperty::from_value(true))),
        positions: Some(Rc::new(ConstantProperty::from_value(vec![
            Cartesian3::new(6378137.0, 0.0, 0.0),
            Cartesian3::new(6378137.0, 1000.0, 0.0),
            Cartesian3::new(6378137.0, 1000.0, 1000.0),
        ]))),
        ..Default::default()
    })
}

#[test]
fn handles_shared_material_being_invalidated_with_geometry() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundPolylinePerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        false,
    );

    let mut polyline1 = create_ground_polyline();
    polyline1.material = Some(Rc::new(PolylineOutlineMaterialProperty::default()));

    let mut entity1 = Entity::new();
    entity1.polyline = Some(polyline1.clone());

    let mut polyline2 = create_ground_polyline();
    polyline2.material = Some(Rc::new(PolylineOutlineMaterialProperty::default()));

    let mut entity2 = Entity::new();
    entity2.polyline = Some(polyline2);

    let updater1 = PolylineGeometryUpdater::new(entity1.clone(), None).unwrap();
    let updater2 = PolylineGeometryUpdater::new(entity2, None).unwrap();

    batch.add(time, &updater1);
    batch.add(time, &updater2);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 1);

    let mut modified_mat = PolylineOutlineMaterialProperty::default();
    modified_mat.outline_width = Some(ConstantProperty::from_value(0.5));
    polyline1.material = Some(Rc::new(modified_mat));
    entity1.polyline = Some(polyline1);

    let updater1_mod = PolylineGeometryUpdater::new(entity1, None).unwrap();
    batch.add(time, &updater1_mod);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 2);

    batch.remove_all_primitives();
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

    let mut polyline = create_ground_polyline();
    polyline.material = Some(Rc::new(ColorMaterialProperty::from_dynamic_color(Rc::new(
        color_prop,
    ))));

    let mut entity = Entity::new();
    entity.availability = Some(entity_availability);
    entity.polyline = Some(polyline);

    let scene = GeometryVisualizerScene::new();
    let mut batch = StaticGroundPolylinePerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        false,
    );

    let updater = PolylineGeometryUpdater::new(entity.clone(), None).unwrap();
    batch.add(valid_time, &updater);

    assert!(batch.update(valid_time));

    {
        let prims = scene.ground_primitives.borrow();
        assert_eq!(prims.len(), 1);
        let attributes = prims[0].get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attributes.color, Some([255, 0, 0, 255]));
    }

    assert!(batch.update(out_of_range_time));

    {
        let prims = scene.ground_primitives.borrow();
        let attributes = prims[0].get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attributes.color, Some([255, 255, 255, 255]));
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

    let mut entity_availability = TimeIntervalCollection::new();
    entity_availability.add_interval(TimeInterval::new(
        JulianDate::from_iso8601("2018-02-14T04:00:00Z").unwrap(),
        JulianDate::from_iso8601("2018-02-14T04:30:00Z").unwrap(),
        true,
        true,
    ));

    let mut polyline = create_ground_polyline();
    polyline.distance_display_condition = Some(Rc::new(ddc_prop));

    let mut entity = Entity::new();
    entity.availability = Some(entity_availability);
    entity.polyline = Some(polyline);

    let scene = GeometryVisualizerScene::new();
    let mut batch = StaticGroundPolylinePerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        false,
    );

    let updater = PolylineGeometryUpdater::new(entity.clone(), None).unwrap();
    batch.add(valid_time, &updater);

    assert!(batch.update(valid_time));

    {
        let prims = scene.ground_primitives.borrow();
        assert_eq!(prims.len(), 1);
        let attributes = prims[0].get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attributes.distance_display_condition, Some([1.0, 2.0]));
    }

    assert!(batch.update(out_of_range_time));

    {
        let prims = scene.ground_primitives.borrow();
        let attributes = prims[0].get_geometry_instance_attributes(&entity).unwrap();
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

    let mut polyline = create_ground_polyline();
    polyline.show = Some(Rc::new(show_prop));

    let mut entity = Entity::new();
    entity.availability = Some(entity_availability);
    entity.polyline = Some(polyline);

    let scene = GeometryVisualizerScene::new();
    let mut batch = StaticGroundPolylinePerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        false,
    );

    let updater = PolylineGeometryUpdater::new(entity.clone(), None).unwrap();
    batch.add(valid_time, &updater);

    assert!(batch.update(valid_time));

    {
        let prims = scene.ground_primitives.borrow();
        assert_eq!(prims.len(), 1);
        let attributes = prims[0].get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attributes.show, Some([1]));
    }

    assert!(batch.update(out_of_range_time));

    {
        let prims = scene.ground_primitives.borrow();
        let attributes = prims[0].get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attributes.show, Some([0]));
    }

    batch.remove_all_primitives();
}

#[test]
fn shows_only_one_primitive_while_rebuilding_primitive() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundPolylinePerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        false,
    );

    let mut polyline1 = create_ground_polyline();
    polyline1.material = Some(Rc::new(PolylineOutlineMaterialProperty::default()));
    let mut entity1 = Entity::new();
    entity1.polyline = Some(polyline1);

    let mut polyline2 = create_ground_polyline();
    polyline2.material = Some(Rc::new(PolylineOutlineMaterialProperty::default()));
    let mut entity2 = Entity::new();
    entity2.polyline = Some(polyline2);

    let updater1 = PolylineGeometryUpdater::new(entity1, None).unwrap();
    let updater2 = PolylineGeometryUpdater::new(entity2, None).unwrap();

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
fn computes_bounding_sphere() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundPolylinePerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        false,
    );

    let mut polyline = create_ground_polyline();
    polyline.material = Some(Rc::new(PolylineOutlineMaterialProperty::default()));
    let mut entity = Entity::new();
    entity.polyline = Some(polyline);

    let updater = PolylineGeometryUpdater::new(entity, None).unwrap();

    let mut result_sphere = BoundingSphere::default();
    assert_eq!(
        batch.get_bounding_sphere(&updater, &mut result_sphere),
        BoundingSphereState::Failed
    );

    batch.add(time, &updater);
    assert_eq!(
        batch.get_bounding_sphere(&updater, &mut result_sphere),
        BoundingSphereState::Pending
    );

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 1);
    assert_eq!(
        batch.get_bounding_sphere(&updater, &mut result_sphere),
        BoundingSphereState::Done
    );
    assert!(result_sphere.radius > 0.0);

    batch.remove_all_primitives();
}

#[test]
fn does_not_crash_when_removing_entity_with_pending_show_update() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundPolylinePerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        false,
    );

    let mut polyline = create_ground_polyline();
    polyline.material = Some(Rc::new(PolylineOutlineMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::YELLOW)),
        Some(ConstantProperty::from_value(Color::BLACK)),
        Some(ConstantProperty::from_value(2.0)),
    )));

    let mut entity = Entity::new();
    entity.polyline = Some(polyline);

    let updater = PolylineGeometryUpdater::new(entity.clone(), None).unwrap();
    batch.add(time, &updater);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 1);

    entity.show = false;
    batch.remove(&updater);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 0);

    batch.remove_all_primitives();
}

#[test]
fn has_correct_show_attribute_after_rebuilding_primitive() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundPolylinePerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        false,
    );

    let mut polyline1 = create_ground_polyline();
    polyline1.material = Some(Rc::new(PolylineOutlineMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::YELLOW)),
        Some(ConstantProperty::from_value(Color::BLACK)),
        Some(ConstantProperty::from_value(2.0)),
    )));
    let mut entity1 = Entity::new();
    entity1.polyline = Some(polyline1);

    let mut polyline2 = create_ground_polyline();
    polyline2.material = Some(Rc::new(PolylineOutlineMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::YELLOW)),
        Some(ConstantProperty::from_value(Color::BLACK)),
        Some(ConstantProperty::from_value(2.0)),
    )));
    let mut entity2 = Entity::new();
    entity2.polyline = Some(polyline2);

    let updater1 = PolylineGeometryUpdater::new(entity1.clone(), None).unwrap();
    batch.add(time, &updater1);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 1);
    {
        let prims = scene.ground_primitives.borrow();
        let attrs = prims[0].get_geometry_instance_attributes(&entity1).unwrap();
        assert_eq!(attrs.show, Some([1]));
    }

    entity1.show = false;
    let updater1_hidden = PolylineGeometryUpdater::new(entity1.clone(), None).unwrap();
    batch.add(time, &updater1_hidden);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 1);
    {
        let prims = scene.ground_primitives.borrow();
        let attrs = prims[0].get_geometry_instance_attributes(&entity1).unwrap();
        assert_eq!(attrs.show, Some([0]));
    }

    let updater2 = PolylineGeometryUpdater::new(entity2.clone(), None).unwrap();
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
