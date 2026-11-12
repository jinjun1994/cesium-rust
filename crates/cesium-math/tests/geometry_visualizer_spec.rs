//! Direct port and verification against CesiumJS `GeometryVisualizerSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    BoundingSphere, BoundingSphereState, Cartesian3, ClassificationType, Color,
    ColorGeometryInstanceAttribute, ColorMaterialProperty, ConstantPositionProperty,
    ConstantProperty, EllipseGraphics, EllipseGraphicsOptions, Entity,
    EntityCollection, GeometryVisualizer, GeometryVisualizerScene, GridMaterialProperty,
    JulianDate, PrimitiveAppearanceType, ShadowMode, ShowGeometryInstanceAttribute,
};

#[test]
fn can_create_and_destroy() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = GeometryVisualizer::new(
        Some(scene.clone()),
        Some(Rc::clone(&objects)),
        Some(Rc::clone(&scene.primitives)),
        Some(Rc::clone(&scene.ground_primitives)),
    )
    .unwrap();

    assert!(visualizer.update(Some(time)).unwrap());
    assert_eq!(scene.primitives.borrow().len(), 0);
    assert!(!visualizer.is_destroyed());

    visualizer.destroy();
    assert!(visualizer.is_destroyed());
}

#[test]
fn creates_and_removes_static_color_open_geometry() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = GeometryVisualizer::new(
        Some(scene.clone()),
        Some(Rc::clone(&objects)),
        Some(Rc::clone(&scene.primitives)),
        Some(Rc::clone(&scene.ground_primitives)),
    )
    .unwrap();

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(Color::WHITE))),
        height: Some(Rc::new(ConstantProperty::from_value(0.0))),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(ellipse);
    let _ = objects.borrow_mut().add(entity.clone());

    assert!(visualizer.update(Some(time)).unwrap());

    {
        let prims = scene.primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity);
        assert!(attributes.is_some());
        let attrs = attributes.unwrap();
        assert_eq!(attrs.show, Some(ShowGeometryInstanceAttribute::to_value(true)));
        assert_eq!(
            attrs.color,
            Some(ColorGeometryInstanceAttribute::to_value(&Color::WHITE))
        );
        assert_eq!(
            primitive.appearance,
            PrimitiveAppearanceType::PerInstanceColor { closed: true }
        );
    }

    objects.borrow_mut().remove(&entity);
    assert!(visualizer.update(Some(time)).unwrap());
    assert_eq!(scene.primitives.borrow().len(), 0);

    visualizer.destroy();
}

#[test]
fn creates_and_removes_static_material_open_geometry() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = GeometryVisualizer::new(
        Some(scene.clone()),
        Some(Rc::clone(&objects)),
        Some(Rc::clone(&scene.primitives)),
        Some(Rc::clone(&scene.ground_primitives)),
    )
    .unwrap();

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(GridMaterialProperty::default())),
        height: Some(Rc::new(ConstantProperty::from_value(1.0))),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(ellipse);
    let _ = objects.borrow_mut().add(entity.clone());

    assert!(visualizer.update(Some(time)).unwrap());

    {
        let prims = scene.primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity);
        assert!(attributes.is_some());
        let attrs = attributes.unwrap();
        assert_eq!(attrs.show, Some(ShowGeometryInstanceAttribute::to_value(true)));
        assert_eq!(attrs.color, None);
        assert_eq!(
            primitive.appearance,
            PrimitiveAppearanceType::Material { closed: false }
        );
    }

    objects.borrow_mut().remove(&entity);
    assert!(visualizer.update(Some(time)).unwrap());
    assert_eq!(scene.primitives.borrow().len(), 0);

    visualizer.destroy();
}

#[test]
fn creates_and_removes_static_color_closed_geometry() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = GeometryVisualizer::new(
        Some(scene.clone()),
        Some(Rc::clone(&objects)),
        Some(Rc::clone(&scene.primitives)),
        Some(Rc::clone(&scene.ground_primitives)),
    )
    .unwrap();

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(Color::WHITE))),
        extruded_height: Some(Rc::new(ConstantProperty::from_value(1000.0))),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(ellipse);
    let _ = objects.borrow_mut().add(entity.clone());

    assert!(visualizer.update(Some(time)).unwrap());

    {
        let prims = scene.primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity);
        assert!(attributes.is_some());
        let attrs = attributes.unwrap();
        assert_eq!(attrs.show, Some(ShowGeometryInstanceAttribute::to_value(true)));
        assert_eq!(
            attrs.color,
            Some(ColorGeometryInstanceAttribute::to_value(&Color::WHITE))
        );
        assert_eq!(
            primitive.appearance,
            PrimitiveAppearanceType::PerInstanceColor { closed: true }
        );
    }

    objects.borrow_mut().remove(&entity);
    assert!(visualizer.update(Some(time)).unwrap());
    assert_eq!(scene.primitives.borrow().len(), 0);

    visualizer.destroy();
}

#[test]
fn creates_and_removes_static_material_closed_geometry() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = GeometryVisualizer::new(
        Some(scene.clone()),
        Some(Rc::clone(&objects)),
        Some(Rc::clone(&scene.primitives)),
        Some(Rc::clone(&scene.ground_primitives)),
    )
    .unwrap();

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(GridMaterialProperty::default())),
        extruded_height: Some(Rc::new(ConstantProperty::from_value(1000.0))),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(ellipse);
    let _ = objects.borrow_mut().add(entity.clone());

    assert!(visualizer.update(Some(time)).unwrap());

    {
        let prims = scene.primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity);
        assert!(attributes.is_some());
        let attrs = attributes.unwrap();
        assert_eq!(attrs.show, Some(ShowGeometryInstanceAttribute::to_value(true)));
        assert_eq!(attrs.color, None);
        assert_eq!(
            primitive.appearance,
            PrimitiveAppearanceType::Material { closed: true }
        );
    }

    objects.borrow_mut().remove(&entity);
    assert!(visualizer.update(Some(time)).unwrap());
    assert_eq!(scene.primitives.borrow().len(), 0);

    visualizer.destroy();
}

#[test]
fn creates_and_removes_static_outline_geometry() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = GeometryVisualizer::new(
        Some(scene.clone()),
        Some(Rc::clone(&objects)),
        Some(Rc::clone(&scene.primitives)),
        Some(Rc::clone(&scene.ground_primitives)),
    )
    .unwrap();

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        outline: Some(Rc::new(ConstantProperty::from_value(true))),
        outline_color: Some(Rc::new(ConstantProperty::from_value(Color::BLUE))),
        fill: Some(Rc::new(ConstantProperty::from_value(false))),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(ellipse);
    let _ = objects.borrow_mut().add(entity.clone());

    assert!(visualizer.update(Some(time)).unwrap());

    {
        let prims = scene.primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity);
        assert!(attributes.is_some());
        let attrs = attributes.unwrap();
        assert_eq!(attrs.show, Some(ShowGeometryInstanceAttribute::to_value(true)));
        assert_eq!(
            attrs.color,
            Some(ColorGeometryInstanceAttribute::to_value(&Color::BLUE))
        );
        assert_eq!(
            primitive.appearance,
            PrimitiveAppearanceType::PerInstanceColor { closed: false }
        );
    }

    objects.borrow_mut().remove(&entity);
    assert!(visualizer.update(Some(time)).unwrap());
    assert_eq!(scene.primitives.borrow().len(), 0);

    visualizer.destroy();
}

#[test]
fn creates_and_removes_geometry_with_shadows() {
    for shadows in [
        ShadowMode::Disabled,
        ShadowMode::Enabled,
        ShadowMode::CastOnly,
        ShadowMode::ReceiveOnly,
    ] {
        let scene = GeometryVisualizerScene::new();
        let objects = Rc::new(RefCell::new(EntityCollection::new()));
        let time = JulianDate::now();

        let mut visualizer = GeometryVisualizer::new(
            Some(scene.clone()),
            Some(Rc::clone(&objects)),
            Some(Rc::clone(&scene.primitives)),
            Some(Rc::clone(&scene.ground_primitives)),
        )
        .unwrap();

        let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
            semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
            semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
            material: Some(Rc::new(ColorMaterialProperty::from_color(Color::WHITE))),
            extruded_height: Some(Rc::new(ConstantProperty::from_value(1000.0))),
            shadows: Some(Rc::new(ConstantProperty::from_value(shadows))),
            ..Default::default()
        });

        let mut entity = Entity::new();
        entity.position = Some(ConstantPositionProperty::new(
            Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
            None,
        ));
        entity.ellipse = Some(ellipse);
        let _ = objects.borrow_mut().add(entity.clone());

        assert!(visualizer.update(Some(time)).unwrap());

        {
            let prims = scene.primitives.borrow();
            assert_eq!(prims.len(), 1);
            assert_eq!(prims[0].shadows, shadows);
        }

        objects.borrow_mut().remove(&entity);
        assert!(visualizer.update(Some(time)).unwrap());
        assert_eq!(scene.primitives.borrow().len(), 0);

        visualizer.destroy();
    }
}

#[test]
fn creates_and_removes_geometry_with_classification_types() {
    for ctype in [
        ClassificationType::Terrain,
        ClassificationType::Cesium3DTile,
        ClassificationType::Both,
    ] {
        let scene = GeometryVisualizerScene::new();
        let objects = Rc::new(RefCell::new(EntityCollection::new()));
        let time = JulianDate::now();

        let mut visualizer = GeometryVisualizer::new(
            Some(scene.clone()),
            Some(Rc::clone(&objects)),
            Some(Rc::clone(&scene.primitives)),
            Some(Rc::clone(&scene.ground_primitives)),
        )
        .unwrap();

        let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
            semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
            semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
            material: Some(Rc::new(ColorMaterialProperty::from_color(Color::WHITE))),
            classification_type: Some(Rc::new(ConstantProperty::from_value(ctype))),
            ..Default::default()
        });

        let mut entity = Entity::new();
        entity.position = Some(ConstantPositionProperty::new(
            Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
            None,
        ));
        entity.ellipse = Some(ellipse);
        let _ = objects.borrow_mut().add(entity.clone());

        assert!(visualizer.update(Some(time)).unwrap());

        {
            let gprims = scene.ground_primitives.borrow();
            assert_eq!(gprims.len(), 1);
            assert_eq!(gprims[0].classification_type, Some(ctype));
        }

        objects.borrow_mut().remove(&entity);
        assert!(visualizer.update(Some(time)).unwrap());
        assert_eq!(scene.ground_primitives.borrow().len(), 0);

        visualizer.destroy();
    }
}

#[test]
fn correctly_handles_geometry_changing_batches() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = GeometryVisualizer::new(
        Some(scene.clone()),
        Some(Rc::clone(&objects)),
        Some(Rc::clone(&scene.primitives)),
        Some(Rc::clone(&scene.ground_primitives)),
    )
    .unwrap();

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(Color::WHITE))),
        height: Some(Rc::new(ConstantProperty::from_value(0.0))),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(ellipse);
    let _ = objects.borrow_mut().add(entity.clone());

    assert!(visualizer.update(Some(time)).unwrap());

    {
        let prims = scene.primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity);
        assert!(attributes.is_some());
        let attrs = attributes.unwrap();
        assert_eq!(attrs.show, Some(ShowGeometryInstanceAttribute::to_value(true)));
        assert_eq!(
            attrs.color,
            Some(ColorGeometryInstanceAttribute::to_value(&Color::WHITE))
        );
        assert_eq!(
            primitive.appearance,
            PrimitiveAppearanceType::PerInstanceColor { closed: true }
        );
    }

    // Switch material to Grid
    if let Some(ref mut el) = entity.ellipse {
        el.material = Some(Rc::new(GridMaterialProperty::default()));
    }
    objects.borrow_mut().remove(&entity);
    let _ = objects.borrow_mut().add(entity.clone());
    assert!(visualizer.update(Some(time)).unwrap());

    {
        let prims = scene.primitives.borrow();
        assert_eq!(prims.len(), 1);
        let primitive = &prims[0];
        let attributes = primitive.get_geometry_instance_attributes(&entity);
        assert!(attributes.is_some());
        let attrs = attributes.unwrap();
        assert_eq!(attrs.show, Some(ShowGeometryInstanceAttribute::to_value(true)));
        assert_eq!(attrs.color, None);
        assert_eq!(
            primitive.appearance,
            PrimitiveAppearanceType::Material { closed: true }
        );
    }

    objects.borrow_mut().remove(&entity);
    assert!(visualizer.update(Some(time)).unwrap());
    assert_eq!(scene.primitives.borrow().len(), 0);

    visualizer.destroy();
}

#[test]
fn correctly_handles_modifying_translucent_outline_color() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = GeometryVisualizer::new(
        Some(scene.clone()),
        Some(Rc::clone(&objects)),
        Some(Rc::clone(&scene.primitives)),
        Some(Rc::clone(&scene.ground_primitives)),
    )
    .unwrap();

    let mut color = Color::new(0.0, 0.0, 1.0, 0.5);
    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        fill: Some(Rc::new(ConstantProperty::from_value(false))),
        outline: Some(Rc::new(ConstantProperty::from_value(true))),
        outline_color: Some(Rc::new(ConstantProperty::from_value(color))),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(ellipse);
    let _ = objects.borrow_mut().add(entity.clone());

    assert!(visualizer.update(Some(time)).unwrap());

    {
        let prims = scene.primitives.borrow();
        let attrs = prims[0].get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(
            attrs.color,
            Some(ColorGeometryInstanceAttribute::to_value(&color))
        );
    }

    color = Color::new(1.0, 0.0, 0.0, 0.5);
    if let Some(ref mut el) = entity.ellipse {
        el.outline_color = Some(Rc::new(ConstantProperty::from_value(color)));
    }
    objects.borrow_mut().remove(&entity);
    let _ = objects.borrow_mut().add(entity.clone());
    assert!(visualizer.update(Some(time)).unwrap());

    {
        let prims = scene.primitives.borrow();
        let attrs = prims[0].get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(
            attrs.color,
            Some(ColorGeometryInstanceAttribute::to_value(&color))
        );
    }

    objects.borrow_mut().remove(&entity);
    visualizer.destroy();
}

#[test]
fn creates_and_removes_dynamic_geometry() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = GeometryVisualizer::new(
        Some(scene.clone()),
        Some(Rc::clone(&objects)),
        Some(Rc::clone(&scene.primitives)),
        Some(Rc::clone(&scene.ground_primitives)),
    )
    .unwrap();

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        extruded_height: Some(Rc::new(ConstantProperty::from_value(1000.0))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(Color::WHITE))),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    entity.ellipse = Some(ellipse);
    let _ = objects.borrow_mut().add(entity.clone());

    assert!(visualizer.update(Some(time)).unwrap());
    assert_eq!(scene.primitives.borrow().len(), 1);

    objects.borrow_mut().remove(&entity);
    assert!(visualizer.update(Some(time)).unwrap());
    assert_eq!(scene.primitives.borrow().len(), 0);

    visualizer.destroy();
}

#[test]
fn constructor_throws_without_required_args() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));

    assert!(GeometryVisualizer::new(None, Some(Rc::clone(&objects)), None, None).is_err());
    assert!(GeometryVisualizer::new(Some(scene), None, None, None).is_err());
}

#[test]
fn update_throws_without_time_parameter() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer = GeometryVisualizer::new(Some(scene), Some(objects), None, None).unwrap();
    assert!(visualizer.update(None).is_err());
}

#[test]
fn removes_the_listener_from_entity_collection_when_destroyed() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer =
        GeometryVisualizer::new(Some(scene), Some(Rc::clone(&objects)), None, None).unwrap();
    assert_eq!(objects.borrow().collection_changed.number_of_listeners(), 1);
    visualizer.destroy();
    assert_eq!(objects.borrow().collection_changed.number_of_listeners(), 0);
}

#[test]
fn computes_dynamic_geometry_bounding_sphere() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = GeometryVisualizer::new(
        Some(scene),
        Some(Rc::clone(&objects)),
        None,
        None,
    )
    .unwrap();

    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        height: Some(Rc::new(ConstantProperty::from_value(0.0))),
        ..Default::default()
    });

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::from_degrees(0.0, 0.0, Some(0.0), None).unwrap()),
        None,
    ));
    entity.ellipse = Some(ellipse);
    let _ = objects.borrow_mut().add(entity.clone());

    assert!(visualizer.update(Some(time)).unwrap());

    let mut result = BoundingSphere::default();
    let state = visualizer
        .get_bounding_sphere(Some(&entity), Some(&mut result))
        .unwrap();
    assert_eq!(state, BoundingSphereState::Done);
    assert_eq!(
        result.center,
        Cartesian3::from_degrees(0.0, 0.0, Some(0.0), None).unwrap().into()
    );
    assert_eq!(result.radius, 2.0);

    // Throws on missing args
    assert!(visualizer.get_bounding_sphere(None, Some(&mut result)).is_err());
    assert!(visualizer.get_bounding_sphere(Some(&entity), None).is_err());

    visualizer.destroy();
}

#[test]
fn can_remove_entity_and_add_new_instance_with_same_id() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = GeometryVisualizer::new(
        Some(scene.clone()),
        Some(Rc::clone(&objects)),
        Some(Rc::clone(&scene.primitives)),
        Some(Rc::clone(&scene.ground_primitives)),
    )
    .unwrap();

    let mut entity1 = Entity::new();
    entity1.id = "test".to_string();
    entity1.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::from_degrees(0.0, 0.0, Some(0.0), None).unwrap()),
        None,
    ));
    let ellipse1 = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(
            Color::new(1.0, 0.65, 0.0, 1.0),
        ))),
        height: Some(Rc::new(ConstantProperty::from_value(0.0))),
        ..Default::default()
    });
    entity1.ellipse = Some(ellipse1);
    let _ = objects.borrow_mut().add(entity1.clone());

    assert!(visualizer.update(Some(time)).unwrap());
    objects.borrow_mut().remove(&entity1);

    let mut entity2 = Entity::new();
    entity2.id = "test".to_string();
    entity2.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::from_degrees(0.0, 0.0, Some(0.0), None).unwrap()),
        None,
    ));
    let ellipse2 = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(Color::BLUE))),
        height: Some(Rc::new(ConstantProperty::from_value(0.0))),
        ..Default::default()
    });
    entity2.ellipse = Some(ellipse2);
    let _ = objects.borrow_mut().add(entity2.clone());

    assert!(visualizer.update(Some(time)).unwrap());

    {
        let prims = scene.primitives.borrow();
        assert_eq!(prims.len(), 1);
        let attrs = prims[0].get_geometry_instance_attributes(&entity2).unwrap();
        assert_eq!(
            attrs.color,
            Some(ColorGeometryInstanceAttribute::to_value(&Color::BLUE))
        );
    }

    objects.borrow_mut().remove(&entity2);
    assert!(visualizer.update(Some(time)).unwrap());
    assert_eq!(scene.primitives.borrow().len(), 0);

    visualizer.destroy();
}

#[test]
fn sets_static_geometry_primitive_show_attribute_when_using_dynamic_properties() {
    let scene = GeometryVisualizerScene::new();
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = GeometryVisualizer::new(
        Some(scene.clone()),
        Some(Rc::clone(&objects)),
        Some(Rc::clone(&scene.primitives)),
        Some(Rc::clone(&scene.ground_primitives)),
    )
    .unwrap();

    let mut entity = Entity::new();
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 9101112.0)),
        None,
    ));
    let ellipse = EllipseGraphics::from_options(EllipseGraphicsOptions {
        semi_major_axis: Some(Rc::new(ConstantProperty::from_value(2.0))),
        semi_minor_axis: Some(Rc::new(ConstantProperty::from_value(1.0))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(Color::BLUE))),
        height: Some(Rc::new(ConstantProperty::from_value(0.0))),
        ..Default::default()
    });
    entity.ellipse = Some(ellipse);
    let _ = objects.borrow_mut().add(entity.clone());

    assert!(visualizer.update(Some(time)).unwrap());

    {
        let prims = scene.primitives.borrow();
        let attrs = prims[0].get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attrs.show, Some(ShowGeometryInstanceAttribute::to_value(true)));
    }

    entity.show = false;
    objects.borrow_mut().remove(&entity);
    let _ = objects.borrow_mut().add(entity.clone());
    assert!(visualizer.update(Some(time)).unwrap());

    {
        let prims = scene.primitives.borrow();
        let attrs = prims[0].get_geometry_instance_attributes(&entity).unwrap();
        assert_eq!(attrs.show, Some(ShowGeometryInstanceAttribute::to_value(false)));
    }

    objects.borrow_mut().remove(&entity);
    visualizer.destroy();
}
