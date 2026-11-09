use cesium_math::{
    Cartesian3, ClassificationType, Color, ComponentDatatype, Geometry,
    GeometryInstance, GeometryInstanceAttribute, GroundPolylineGeometry,
    GroundPolylineGeometryOptions, Matrix4,
};
use cesium_renderer::ground_polyline_primitive::{
    set_terrain_heights_initialized_for_test, GroundPolylineAppearance, GroundPolylinePrimitive,
    GroundPolylinePrimitiveOptions, PolylineColorAppearance,
};
use std::collections::HashMap;

fn create_dummy_polyline_geometry() -> Geometry {
    let positions = Cartesian3::from_degrees_array(&[0.01, 0.0, 0.03, 0.0], None).unwrap();
    let desc = GroundPolylineGeometry::new(GroundPolylineGeometryOptions {
        positions,
        width: Some(4.0),
        granularity: None,
        loop_: None,
        arc_type: None,
        ellipsoid: None,
    })
    .unwrap();
    GroundPolylineGeometry::create_geometry(&desc).unwrap()
}

fn create_dummy_instance(id: &str, with_color: bool) -> GeometryInstance {
    let mut attributes = HashMap::new();
    if with_color {
        attributes.insert(
            "color".to_string(),
            GeometryInstanceAttribute {
                component_datatype: ComponentDatatype::UnsignedByte,
                components_per_attribute: 4,
                normalize: true,
                value: vec![0.0, 255.0, 0.0, 255.0],
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
        create_dummy_polyline_geometry(),
        Some(Matrix4::IDENTITY),
        Some(id.to_string()),
        Some(attributes),
    )
}

#[test]
fn default_constructs() {
    let primitive =
        GroundPolylinePrimitive::new(GroundPolylinePrimitiveOptions::default()).unwrap();

    assert!(primitive.geometry_instances.is_none());
    assert!(matches!(
        primitive.appearance,
        GroundPolylineAppearance::Material(_)
    ));
    assert!(primitive.show);
    assert!(!primitive.interleave());
    assert!(primitive.release_geometry_instances());
    assert!(primitive.allow_picking());
    assert!(primitive.asynchronous());
    assert_eq!(primitive.classification_type, ClassificationType::Both);
    assert!(!primitive.debug_show_bounding_volume);
    assert!(!primitive.debug_show_shadow_volume());
    assert!(!primitive.ready());
    assert!(!primitive.is_destroyed());
}

#[test]
fn constructs_with_options() {
    let instance = create_dummy_instance("test-id", true);
    let primitive = GroundPolylinePrimitive::new(GroundPolylinePrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        appearance: Some(GroundPolylineAppearance::Color(
            PolylineColorAppearance::default(),
        )),
        show: Some(false),
        interleave: Some(true),
        release_geometry_instances: Some(false),
        allow_picking: Some(false),
        asynchronous: Some(false),
        classification_type: Some(ClassificationType::Terrain),
        debug_show_bounding_volume: Some(true),
        debug_show_shadow_volume: Some(true),
    })
    .unwrap();

    assert!(primitive.geometry_instances.is_some());
    assert!(matches!(
        primitive.appearance,
        GroundPolylineAppearance::Color(_)
    ));
    assert!(!primitive.show);
    assert!(primitive.interleave());
    assert!(!primitive.release_geometry_instances());
    assert!(!primitive.allow_picking());
    assert!(!primitive.asynchronous());
    assert_eq!(primitive.classification_type, ClassificationType::Terrain);
    assert!(primitive.debug_show_bounding_volume);
    assert!(primitive.debug_show_shadow_volume());
    assert!(!primitive.ready());
}

#[test]
fn releases_geometry_instances_when_release_geometry_instances_is_true() {
    let instance = create_dummy_instance("line-1", true);
    let mut primitive = GroundPolylinePrimitive::new(GroundPolylinePrimitiveOptions {
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
    let instance = create_dummy_instance("line-1", true);
    let mut primitive = GroundPolylinePrimitive::new(GroundPolylinePrimitiveOptions {
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
    let mut primitive =
        GroundPolylinePrimitive::new(GroundPolylinePrimitiveOptions::default()).unwrap();

    primitive.update().unwrap();
    assert!(!primitive.ready());
}

#[test]
fn becomes_ready_when_show_is_false() {
    let instance = create_dummy_instance("line-1", true);
    let mut primitive = GroundPolylinePrimitive::new(GroundPolylinePrimitiveOptions {
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
    let instance = create_dummy_instance("line-1", true);
    let mut primitive = GroundPolylinePrimitive::new(GroundPolylinePrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        ..Default::default()
    })
    .unwrap();

    primitive.update().unwrap();

    let attr_arc = primitive
        .get_geometry_instance_attributes("line-1")
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
        attr.set_color(Some(Color::RED)).unwrap();
        assert_eq!(attr.color, Some(Color::RED));

        // Modify width
        attr.set_width(Some(8.0)).unwrap();
        assert_eq!(attr.width, Some(8.0));

        // Modify show
        attr.set_show(Some(false)).unwrap();
        assert_eq!(attr.show, Some(false));
    }

    // Same object returned each time
    let attr_arc2 = primitive
        .get_geometry_instance_attributes("line-1")
        .unwrap()
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&attr_arc, &attr_arc2));
}

#[test]
fn update_throws_when_batched_instance_colors_are_missing() {
    let instance_no_color = create_dummy_instance("line-1", false);
    let mut primitive = GroundPolylinePrimitive::new(GroundPolylinePrimitiveOptions {
        geometry_instances: Some(vec![instance_no_color]),
        appearance: Some(GroundPolylineAppearance::Color(
            PolylineColorAppearance::default(),
        )),
        ..Default::default()
    })
    .unwrap();

    let err = primitive.update().unwrap_err();
    assert!(err.message.contains("All GeometryInstances must have color attributes"));
}

#[test]
fn setting_per_instance_attribute_throws_when_value_is_undefined() {
    let instance = create_dummy_instance("line-1", true);
    let mut primitive = GroundPolylinePrimitive::new(GroundPolylinePrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        ..Default::default()
    })
    .unwrap();

    primitive.update().unwrap();
    let attr_arc = primitive
        .get_geometry_instance_attributes("line-1")
        .unwrap()
        .unwrap();

    let mut attr = attr_arc.lock().unwrap();
    assert!(attr.set_color(None).is_err());
    assert!(attr.set_width(None).is_err());
    assert!(attr.set_show(None).is_err());
    assert!(attr.set_distance_display_condition(None).is_err());
}

#[test]
fn get_geometry_instance_attributes_throws_without_update() {
    let primitive =
        GroundPolylinePrimitive::new(GroundPolylinePrimitiveOptions::default()).unwrap();

    let err = primitive.get_geometry_instance_attributes("line-1").unwrap_err();
    assert!(err.message.contains("must call update before calling getGeometryInstanceAttributes"));
}

#[test]
fn get_geometry_instance_attributes_throws_without_id() {
    let mut primitive =
        GroundPolylinePrimitive::new(GroundPolylinePrimitiveOptions::default()).unwrap();
    primitive.update().unwrap();

    let err = primitive.get_geometry_instance_attributes("").unwrap_err();
    assert!(err.message.contains("id is required"));
}

#[test]
fn get_geometry_instance_attributes_returns_none_if_id_does_not_exist() {
    let mut primitive =
        GroundPolylinePrimitive::new(GroundPolylinePrimitiveOptions::default()).unwrap();
    primitive.update().unwrap();

    let attr = primitive.get_geometry_instance_attributes("non-existent").unwrap();
    assert!(attr.is_none());
}

#[test]
fn is_destroyed_and_destroy() {
    let mut primitive =
        GroundPolylinePrimitive::new(GroundPolylinePrimitiveOptions::default()).unwrap();

    assert!(!primitive.is_destroyed());
    primitive.destroy();
    assert!(primitive.is_destroyed());

    let err = primitive.update().unwrap_err();
    assert!(err.message.contains("destroyed"));
}

#[test]
fn creating_synchronous_primitive_throws_if_initialize_terrain_heights_wasnt_called() {
    set_terrain_heights_initialized_for_test(Some(false));

    let err = GroundPolylinePrimitive::new(GroundPolylinePrimitiveOptions {
        asynchronous: Some(false),
        ..Default::default()
    })
    .unwrap_err();

    assert!(err.message.contains("ApproximateTerrainHeights was not initialized"));

    // Reset override
    set_terrain_heights_initialized_for_test(None);
}
