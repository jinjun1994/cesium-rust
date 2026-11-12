use std::collections::HashMap;

use cesium_math::{
    BoxGeometry, Cartesian3, CircleGeometry, CylinderGeometry, GeometryInstance,
    GeometryInstanceAttribute, Matrix4, Rectangle, RectangleGeometry,
    RectangleGeometryOptions, SceneMode, SphereGeometry, VertexFormat,
};
use cesium_renderer::scene_primitive::{
    Primitive, PrimitiveAppearance, PrimitiveOptions, PrimitivePasses, PrimitiveRenderFrameState,
};

fn make_instance(
    id: &str,
    geometry: cesium_math::Geometry,
    model_matrix: Matrix4,
) -> GeometryInstance {
    let mut attributes = HashMap::new();
    attributes.insert(
        "color".to_string(),
        GeometryInstanceAttribute {
            component_datatype: cesium_math::ComponentDatatype::Float,
            components_per_attribute: 4,
            normalize: false,
            value: vec![1.0, 1.0, 0.0, 1.0],
        },
    );

    GeometryInstance {
        geometry,
        model_matrix,
        id: Some(id.to_string()),
        attributes,
    }
}

#[test]
fn renders_box_geometry_in_3d_columbus_view_and_2d() {
    let box_geom = BoxGeometry::from_dimensions(
        Cartesian3::new(100.0, 100.0, 200.0),
        Some(VertexFormat::DEFAULT),
    )
    .create_geometry()
    .expect("box geometry creation");

    let instance = make_instance("box", box_geom, Matrix4::IDENTITY);
    let mut primitive = Primitive::new(PrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        appearance: Some(PrimitiveAppearance::new(true, false).with_flat(true)),
        asynchronous: Some(false),
        release_geometry_instances: Some(false),
        ..Default::default()
    });

    // 3D mode
    let mut frame_state_3d = PrimitiveRenderFrameState {
        mode: SceneMode::Scene3D,
        passes: PrimitivePasses {
            render: true,
            pick: false,
        },
        ..Default::default()
    };
    primitive.update(&mut frame_state_3d);
    assert!(primitive.ready);
    assert_eq!(frame_state_3d.command_list.len(), 1);

    // Columbus View mode
    let mut frame_state_cv = PrimitiveRenderFrameState {
        mode: SceneMode::ColumbusView,
        passes: PrimitivePasses {
            render: true,
            pick: false,
        },
        ..Default::default()
    };
    primitive.update(&mut frame_state_cv);
    assert_eq!(frame_state_cv.command_list.len(), 1);

    // 2D mode
    let mut frame_state_2d = PrimitiveRenderFrameState {
        mode: SceneMode::Scene2D,
        passes: PrimitivePasses {
            render: true,
            pick: false,
        },
        ..Default::default()
    };
    primitive.update(&mut frame_state_2d);
    assert_eq!(frame_state_2d.command_list.len(), 1);
}

#[test]
fn renders_sphere_cylinder_and_circle_geometries() {
    let sphere_geom = SphereGeometry::new(Some(50.0), Some(16), Some(16), None)
        .create_geometry()
        .expect("sphere geometry creation");
    let cylinder_geom = CylinderGeometry::new(100.0, 20.0, 20.0, None, None)
        .create_geometry()
        .expect("cylinder geometry creation");
    let circle_geom = CircleGeometry::new(
        Cartesian3::ZERO,
        50.0,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .create_geometry()
    .expect("circle geometry creation");

    let instances = vec![
        make_instance("sphere", sphere_geom, Matrix4::IDENTITY),
        make_instance(
            "cylinder",
            cylinder_geom,
            Matrix4::from_translation(Cartesian3::new(200.0, 0.0, 0.0)),
        ),
        make_instance(
            "circle",
            circle_geom,
            Matrix4::from_translation(Cartesian3::new(-200.0, 0.0, 0.0)),
        ),
    ];

    let mut primitive = Primitive::new(PrimitiveOptions {
        geometry_instances: Some(instances),
        appearance: Some(PrimitiveAppearance::new(false, false)),
        asynchronous: Some(false),
        release_geometry_instances: Some(false),
        ..Default::default()
    });

    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state);
    assert!(primitive.ready);
    assert_eq!(frame_state.command_list.len(), 1);

    // Verify instance attribute retrieval for picking/styling
    let attr = primitive
        .get_geometry_instance_attribute("cylinder", "color")
        .expect("found cylinder color attribute");
    assert_eq!(attr.value, vec![1.0, 1.0, 0.0, 1.0]);
}

#[test]
fn renders_rectangle_geometry_and_handles_closed_translucency_two_passes() {
    let rect_options = RectangleGeometry::new(RectangleGeometryOptions {
        rectangle: Rectangle::from_degrees(-10.0, -10.0, 10.0, 10.0),
        vertex_format: Some(VertexFormat::DEFAULT),
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    })
    .expect("valid rectangle options");
    let rect_geom = RectangleGeometry::create_geometry(&rect_options)
        .expect("valid create_geometry call")
        .expect("rectangle geometry creation");

    let instance = make_instance("rect", rect_geom, Matrix4::IDENTITY);
    // Closed + translucent triggers two render passes (back face, then front face).
    let mut primitive = Primitive::new(PrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        appearance: Some(PrimitiveAppearance::new(true, true)),
        asynchronous: Some(false),
        ..Default::default()
    });

    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state);
    assert!(primitive.ready);
    assert_eq!(frame_state.command_list.len(), 2);
}

#[test]
fn respects_scene_3d_only_picking_and_destroy() {
    let box_geom = BoxGeometry::from_dimensions(Cartesian3::new(10.0, 10.0, 10.0), None)
        .create_geometry()
        .expect("box geometry");

    let instance = make_instance("box_pick", box_geom, Matrix4::IDENTITY);
    let mut primitive = Primitive::new(PrimitiveOptions {
        geometry_instances: Some(vec![instance]),
        asynchronous: Some(false),
        ..Default::default()
    });

    // Pick-only pass generates draw commands when allow_picking is true
    let mut pick_frame_state = PrimitiveRenderFrameState {
        mode: SceneMode::Scene3D,
        passes: PrimitivePasses {
            render: false,
            pick: true,
        },
        ..Default::default()
    };
    primitive.update(&mut pick_frame_state);
    assert_eq!(pick_frame_state.command_list.len(), 1);

    // scene_3d_only suppresses non-3D modes
    let mut frame_state_3d_only_in_2d = PrimitiveRenderFrameState {
        mode: SceneMode::Scene2D,
        scene_3d_only: true,
        passes: PrimitivePasses {
            render: true,
            pick: false,
        },
        ..Default::default()
    };
    primitive.update(&mut frame_state_3d_only_in_2d);
    assert!(frame_state_3d_only_in_2d.command_list.is_empty());

    primitive.destroy();
    assert!(primitive.is_destroyed());
    let mut frame_state_after_destroy = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state_after_destroy);
    assert!(frame_state_after_destroy.command_list.is_empty());
}
