use cesium_math::{
    ClassificationType, Color, ComponentDatatype, Geometry, GeometryAttribute, GeometryAttributes,
    GeometryInstance, GeometryInstanceAttribute, Matrix4, PrimitiveType,
};
use cesium_renderer::ground_primitive::{
    set_ground_primitive_terrain_heights_initialized_for_test, EllipsoidSurfaceAppearance,
    GroundPrimitive, GroundPrimitiveAppearance, GroundPrimitiveOptions,
};
use std::collections::HashMap;

fn create_dummy_geometry() -> Geometry {
    Geometry {
        attributes: GeometryAttributes {
            position: Some(GeometryAttribute {
                component_datatype: ComponentDatatype::Float,
                components_per_attribute: 3,
                normalize: false,
                values: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            }),
            ..Default::default()
        },
        indices: vec![0, 1],
        primitive_type: PrimitiveType::Lines,
        bounding_sphere: None,
    }
}

fn create_dummy_instance(id: &str, color: Option<Color>) -> GeometryInstance {
    let mut attributes = HashMap::new();
    if let Some(c) = color {
        attributes.insert(
            "color".to_string(),
            GeometryInstanceAttribute {
                component_datatype: ComponentDatatype::UnsignedByte,
                components_per_attribute: 4,
                normalize: true,
                value: vec![
                    (c.red * 255.0) as f64,
                    (c.green * 255.0) as f64,
                    (c.blue * 255.0) as f64,
                    (c.alpha * 255.0) as f64,
                ],
            },
        );
    }
    attributes.insert(
        "show".to_string(),
        GeometryInstanceAttribute {
            component_datatype: ComponentDatatype::UnsignedByte,
            components_per_attribute: 1,
            normalize: false,
            value: vec![1.0],
        },
    );
    attributes.insert(
        "width".to_string(),
        GeometryInstanceAttribute {
            component_datatype: ComponentDatatype::Float,
            components_per_attribute: 1,
            normalize: false,
            value: vec![4.0],
        },
    );

    GeometryInstance::new(
        create_dummy_geometry(),
        Some(Matrix4::IDENTITY),
        Some(id.to_string()),
        Some(attributes),
    )
}

#[test]
fn default_constructs() {
    let primitive = GroundPrimitive::new(GroundPrimitiveOptions::default()).unwrap();

    assert!(primitive.geometry_instances.is_none());
    assert!(primitive.appearance.is_none());
    assert!(primitive.show);
    assert!(!primitive.vertex_cache_optimize());
    assert!(!primitive.interleave());
    assert!(primitive.release_geometry_instances());
    assert!(primitive.allow_picking());
    assert!(primitive.asynchronous());
    assert!(primitive.compress_vertices());
    assert_eq!(primitive.classification_type, ClassificationType::Both);
    assert!(!primitive.debug_show_bounding_volume);
    assert!(!primitive.debug_show_shadow_volume);
    assert!(!primitive.ready());
    assert!(!primitive.is_destroyed());
}

#[test]
fn constructs_with_options() {
    let instance = create_dummy_instance("rect-1", Some(Color::RED));
    let primitive = GroundPrimitive::new(GroundPrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        appearance: Some(GroundPrimitiveAppearance::EllipsoidSurface(
            EllipsoidSurfaceAppearance::default(),
        )),
        show: Some(false),
        vertex_cache_optimize: Some(true),
        interleave: Some(true),
        release_geometry_instances: Some(false),
        allow_picking: Some(false),
        asynchronous: Some(false),
        compress_vertices: Some(false),
        classification_type: Some(ClassificationType::Terrain),
        debug_show_bounding_volume: Some(true),
        debug_show_shadow_volume: Some(true),
    })
    .unwrap();

    assert!(primitive.geometry_instances.is_some());
    assert!(matches!(
        primitive.appearance,
        Some(GroundPrimitiveAppearance::EllipsoidSurface(_))
    ));
    assert!(!primitive.show);
    assert!(primitive.vertex_cache_optimize());
    assert!(primitive.interleave());
    assert!(!primitive.release_geometry_instances());
    assert!(!primitive.allow_picking());
    assert!(!primitive.asynchronous());
    assert!(!primitive.compress_vertices());
    assert_eq!(primitive.classification_type, ClassificationType::Terrain);
    assert!(primitive.debug_show_bounding_volume);
    assert!(primitive.debug_show_shadow_volume);
    assert!(!primitive.ready());
}

#[test]
fn auto_infers_per_instance_color_appearance() {
    let instance = create_dummy_instance("rect-1", Some(Color::BLUE));
    let primitive = GroundPrimitive::new(GroundPrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        appearance: None,
        ..Default::default()
    })
    .unwrap();

    assert!(matches!(
        primitive.appearance,
        Some(GroundPrimitiveAppearance::PerInstanceColor(_))
    ));
}

#[test]
fn releases_geometry_instances_when_release_geometry_instances_is_true() {
    let instance = create_dummy_instance("rect-1", Some(Color::GREEN));
    let mut primitive = GroundPrimitive::new(GroundPrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        release_geometry_instances: Some(true),
        ..Default::default()
    })
    .unwrap();

    assert!(primitive.geometry_instances.is_some());
    primitive.update().unwrap();
    assert!(primitive.ready());
    assert!(primitive.geometry_instances.is_none());
}

#[test]
fn does_not_release_geometry_instances_when_release_geometry_instances_is_false() {
    let instance = create_dummy_instance("rect-1", Some(Color::GREEN));
    let mut primitive = GroundPrimitive::new(GroundPrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        release_geometry_instances: Some(false),
        ..Default::default()
    })
    .unwrap();

    assert!(primitive.geometry_instances.is_some());
    primitive.update().unwrap();
    assert!(primitive.ready());
    assert!(primitive.geometry_instances.is_some());
}

#[test]
fn does_not_render_when_geometry_instances_is_undefined() {
    let mut primitive = GroundPrimitive::new(GroundPrimitiveOptions::default()).unwrap();
    primitive.update().unwrap();
    assert!(!primitive.ready());
}

#[test]
fn becomes_ready_when_show_is_false() {
    let instance = create_dummy_instance("rect-1", Some(Color::GREEN));
    let mut primitive = GroundPrimitive::new(GroundPrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        show: Some(false),
        ..Default::default()
    })
    .unwrap();

    assert!(!primitive.ready());
    primitive.update().unwrap();
    assert!(primitive.ready());
}

#[test]
fn get_and_modify_per_instance_attributes() {
    let instance = create_dummy_instance("rect-1", Some(Color::GREEN));
    let mut primitive = GroundPrimitive::new(GroundPrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        ..Default::default()
    })
    .unwrap();

    primitive.update().unwrap();

    let attr_arc = primitive
        .get_geometry_instance_attributes("rect-1")
        .unwrap()
        .unwrap();

    // Verify initial values
    {
        let attr = attr_arc.lock().unwrap();
        assert_eq!(attr.show, Some(true));
        assert_eq!(attr.width, Some(4.0));
        assert!(attr.color.is_some());
    }

    // Modify color
    {
        let mut attr = attr_arc.lock().unwrap();
        attr.set_color(Some(Color::CYAN)).unwrap();
        assert_eq!(attr.color, Some(Color::CYAN));

        // Modify show
        attr.set_show(Some(false)).unwrap();
        assert_eq!(attr.show, Some(false));
    }

    // Same object returned each time
    let attr_arc2 = primitive
        .get_geometry_instance_attributes("rect-1")
        .unwrap()
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&attr_arc, &attr_arc2));
}

#[test]
fn get_bounding_sphere_from_per_instance_attribute() {
    let instance = create_dummy_instance("rect-1", Some(Color::YELLOW));
    let mut primitive = GroundPrimitive::new(GroundPrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        ..Default::default()
    })
    .unwrap();

    primitive.update().unwrap();

    let sphere = primitive.get_bounding_sphere("rect-1");
    assert!(sphere.is_some());
    assert!(sphere.unwrap().radius > 0.0);
}

#[test]
fn update_throws_when_batched_instance_colors_are_different() {
    let instance1 = create_dummy_instance("rect-1", Some(Color::RED));
    let instance2 = create_dummy_instance("rect-2", Some(Color::BLUE));

    let mut primitive = GroundPrimitive::new(GroundPrimitiveOptions {
        geometry_instances: Some(vec![instance1, instance2]),
        ..Default::default()
    })
    .unwrap();

    let err = primitive.update().unwrap_err();
    assert!(err.message.contains("Batched instance colors must be identical"));
}

#[test]
fn update_throws_when_one_batched_instance_color_is_undefined() {
    let instance1 = create_dummy_instance("rect-1", Some(Color::RED));
    let instance2 = create_dummy_instance("rect-2", None);

    let mut primitive = GroundPrimitive::new(GroundPrimitiveOptions {
        geometry_instances: Some(vec![instance1, instance2]),
        ..Default::default()
    })
    .unwrap();

    let err = primitive.update().unwrap_err();
    assert!(err.message.contains("All batched instances must have a color attribute"));
}

#[test]
fn setting_per_instance_attribute_throws_when_value_is_undefined() {
    let instance = create_dummy_instance("rect-1", Some(Color::RED));
    let mut primitive = GroundPrimitive::new(GroundPrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        ..Default::default()
    })
    .unwrap();

    primitive.update().unwrap();
    let attr_arc = primitive
        .get_geometry_instance_attributes("rect-1")
        .unwrap()
        .unwrap();

    let mut attr = attr_arc.lock().unwrap();
    assert!(attr.set_color(None).is_err());
    assert!(attr.set_show(None).is_err());
}

#[test]
fn get_geometry_instance_attributes_throws_without_update() {
    let primitive = GroundPrimitive::new(GroundPrimitiveOptions::default()).unwrap();

    let err = primitive.get_geometry_instance_attributes("rect-1").unwrap_err();
    assert!(err.message.contains("must call update before calling getGeometryInstanceAttributes"));
}

#[test]
fn get_geometry_instance_attributes_throws_without_id() {
    let mut primitive = GroundPrimitive::new(GroundPrimitiveOptions::default()).unwrap();
    primitive.update().unwrap();

    let err = primitive.get_geometry_instance_attributes("").unwrap_err();
    assert!(err.message.contains("id is required"));
}

#[test]
fn get_geometry_instance_attributes_returns_none_if_id_does_not_exist() {
    let mut primitive = GroundPrimitive::new(GroundPrimitiveOptions::default()).unwrap();
    primitive.update().unwrap();

    let attr = primitive.get_geometry_instance_attributes("non-existent").unwrap();
    assert!(attr.is_none());
}

#[test]
fn is_destroyed_and_destroy() {
    let mut primitive = GroundPrimitive::new(GroundPrimitiveOptions::default()).unwrap();

    assert!(!primitive.is_destroyed());
    primitive.destroy();
    assert!(primitive.is_destroyed());

    let err = primitive.update().unwrap_err();
    assert!(err.message.contains("destroyed"));
}

#[test]
fn creating_synchronous_primitive_throws_if_initialize_terrain_heights_wasnt_called() {
    set_ground_primitive_terrain_heights_initialized_for_test(Some(false));

    let err = GroundPrimitive::new(GroundPrimitiveOptions {
        asynchronous: Some(false),
        ..Default::default()
    })
    .unwrap_err();

    assert!(err.message.contains("ApproximateTerrainHeights was not initialized"));

    // Reset override
    set_ground_primitive_terrain_heights_initialized_for_test(None);
}

#[test]
fn is_supported_checks_depth_texture() {
    assert!(GroundPrimitive::is_supported(true));
    assert!(!GroundPrimitive::is_supported(false));
}
