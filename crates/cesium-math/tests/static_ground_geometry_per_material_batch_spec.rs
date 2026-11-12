//! Direct port and verification against CesiumJS `StaticGroundGeometryPerMaterialBatchSpec.js` (Apache-2.0).

use std::rc::Rc;

use cesium_math::{
    Cartesian2, Cartesian3, ClassificationType, Color, ColorMaterialProperty,
    ConstantPositionProperty, ConstantProperty, DistanceDisplayCondition, EllipseGeometryUpdater,
    EllipseGraphics, EllipseGraphicsOptions, Entity, GeometryVisualizerScene,
    GridMaterialProperty, JulianDate, StaticGroundGeometryPerMaterialBatch, TimeInterval,
    TimeIntervalCollection, TimeIntervalCollectionProperty,
};

#[test]
fn handles_shared_material_being_invalidated_with_geometry() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundGeometryPerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        None,
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
        Some(Cartesian3::new(123.0, 456.0, 789.0)),
        None,
    ));
    entity2.ellipse = Some(ellipse2);

    let updater1 = EllipseGeometryUpdater::new(entity1.clone(), None).unwrap();
    let updater2 = EllipseGeometryUpdater::new(entity2, None).unwrap();

    batch.add(time, &updater1);
    batch.add(time, &updater2);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 1);

    let mut modified_grid = GridMaterialProperty::default();
    modified_grid.cell_alpha = Some(ConstantProperty::from_value(0.5));
    ellipse1.material = Some(Rc::new(modified_grid));
    entity1.ellipse = Some(ellipse1);

    let updater1_mod = EllipseGeometryUpdater::new(entity1, None).unwrap();
    batch.add(time, &updater1_mod);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 2);

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

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
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
    let mut batch = StaticGroundGeometryPerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        None,
    );

    let updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
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

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
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
    let mut batch = StaticGroundGeometryPerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        None,
    );

    let updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
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
fn handles_removing_entity_before_show_update() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundGeometryPerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        None,
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

    let mut updater = EllipseGeometryUpdater::new(entity.clone(), None).unwrap();
    batch.add(time, &updater);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 1);

    entity.show = false;
    updater.on_entity_property_changed(&entity, "isShowing");
    batch.remove(&updater);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 0);

    batch.remove_all_primitives();
}

#[test]
fn shows_only_one_primitive_while_rebuilding_primitive() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundGeometryPerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        None,
    );

    fn build_entity(x: f64, y: f64, z: f64) -> Entity {
        let grid = GridMaterialProperty::new(
            Some(ConstantProperty::from_value(Color::YELLOW)),
            Some(ConstantProperty::from_value(0.3)),
            Some(ConstantProperty::from_value(Cartesian2::new(8.0, 8.0))),
            Some(ConstantProperty::from_value(Cartesian2::new(2.0, 2.0))),
            None,
        );

        let mut e = Entity::new();
        e.position = Some(ConstantPositionProperty::new(
            Some(Cartesian3::new(x, y, z)),
            None,
        ));
        e.ellipse = Some(EllipseGraphics::from_options(EllipseGraphicsOptions {
            semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
            semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
            material: Some(Rc::new(grid)),
            ..Default::default()
        }));
        e
    }

    let entity1 = build_entity(1234.0, 5678.0, 9101112.0);
    let entity2 = build_entity(123.0, 456.0, 789.0);

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
fn batches_overlapping_entities_with_the_same_material_separately() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundGeometryPerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
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
        material: Some(Rc::new(GridMaterialProperty::default())),
        ..Default::default()
    }));

    let mut entity2 = Entity::new();
    entity2.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity2.ellipse = Some(EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(3.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        material: Some(Rc::new(GridMaterialProperty::default())),
        ..Default::default()
    }));

    let updater1 = EllipseGeometryUpdater::new(entity1, None).unwrap();
    let updater2 = EllipseGeometryUpdater::new(entity2, None).unwrap();

    batch.add(time, &updater1);
    batch.add(time, &updater2);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 2);

    batch.remove_all_primitives();
}

#[test]
fn batches_nonoverlapping_entities_that_both_use_color_materials() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundGeometryPerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
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
        material: Some(Rc::new(ColorMaterialProperty::from_color(Color::RED))),
        ..Default::default()
    }));

    let mut entity2 = Entity::new();
    entity2.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(123.0, 456.0, 789.0)),
        None,
    ));
    entity2.ellipse = Some(EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(Color::BLUE))),
        ..Default::default()
    }));

    let updater1 = EllipseGeometryUpdater::new(entity1, None).unwrap();
    let updater2 = EllipseGeometryUpdater::new(entity2, None).unwrap();

    batch.add(time, &updater1);
    batch.add(time, &updater2);

    assert!(batch.update(time));
    assert_eq!(scene.ground_primitives.borrow().len(), 1);

    batch.remove_all_primitives();
}

#[test]
fn has_correct_show_attribute_after_rebuilding_primitive() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundGeometryPerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        None,
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
        Some(Cartesian3::new(123.0, 456.0, 789.0)),
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

#[test]
fn does_not_immediately_remove_empty_batch_reused_before_next_update() {
    let time = JulianDate::now();
    let scene = GeometryVisualizerScene::new();

    let mut batch = StaticGroundGeometryPerMaterialBatch::new(
        Rc::clone(&scene.ground_primitives),
        ClassificationType::Both,
        None,
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

    let material_before = scene.ground_primitives.borrow()[0].id.clone();
    batch.remove(&updater);
    batch.add(time, &updater);
    assert!(batch.update(time));

    {
        let prims = scene.ground_primitives.borrow();
        assert_eq!(prims.len(), 1);
        assert_eq!(prims[0].id, material_before);
    }

    batch.remove_all_primitives();
}
