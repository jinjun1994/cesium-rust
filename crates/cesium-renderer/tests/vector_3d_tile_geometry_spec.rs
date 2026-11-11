use cesium_math::{BoundingSphere, Cartesian3, ClassificationType, Color, Matrix4};
use cesium_renderer::commands::Pass;
use cesium_renderer::scene_primitive::{PrimitivePasses, PrimitiveRenderFrameState};
use cesium_renderer::vector_3d_tile_geometry::{
    pack_boxes, pack_cylinders, pack_ellipsoids, pack_spheres, Vector3DTileGeometry,
    Vector3DTileGeometryOptions,
};

#[test]
fn test_renders_single_box() {
    let dimensions = Cartesian3::new(1000.0, 1000.0, 1000.0);
    let boxes = pack_boxes(&[(dimensions, Matrix4::IDENTITY)]);
    let box_batch_ids = vec![0];
    let bv = BoundingSphere::new(glam::DVec3::ZERO, 1732.0);

    let geom = Vector3DTileGeometry::new(Vector3DTileGeometryOptions {
        boxes: Some(boxes),
        box_batch_ids: Some(box_batch_ids),
        bounding_volume: Some(bv),
        ..Default::default()
    });

    assert!(geom.ready);
    assert_eq!(geom.triangles_length, 12);
    assert_eq!(geom.batches.len(), 1);
    assert_eq!(geom.batches[0].count, 36);
    assert_eq!(geom.batches[0].batch_ids, vec![0]);
}

#[test]
fn test_renders_multiple_boxes() {
    let dimensions = Cartesian3::new(500.0, 500.0, 500.0);
    let m0 = Matrix4::from_translation(Cartesian3::new(dimensions.x, 0.0, 0.0));
    let m1 = Matrix4::from_translation(Cartesian3::new(-dimensions.x, 0.0, 0.0));
    let boxes = pack_boxes(&[(dimensions, m0), (dimensions, m1)]);
    let box_batch_ids = vec![0, 1];

    let geom = Vector3DTileGeometry::new(Vector3DTileGeometryOptions {
        boxes: Some(boxes),
        box_batch_ids: Some(box_batch_ids),
        ..Default::default()
    });

    assert_eq!(geom.triangles_length, 24);
    assert_eq!(geom.batches.len(), 2);
}

#[test]
fn test_renders_single_cylinder() {
    let radius = 100.0;
    let length = 200.0;
    let cylinders = pack_cylinders(&[(radius, length, Matrix4::IDENTITY)]);
    let cylinder_batch_ids = vec![0];

    let geom = Vector3DTileGeometry::new(Vector3DTileGeometryOptions {
        cylinders: Some(cylinders),
        cylinder_batch_ids: Some(cylinder_batch_ids),
        ..Default::default()
    });

    assert!(geom.ready);
    assert!(geom.triangles_length > 0);
    assert_eq!(geom.batches.len(), 1);
}

#[test]
fn test_renders_multiple_cylinders() {
    let radius = 50.0;
    let length = 100.0;
    let m0 = Matrix4::from_translation(Cartesian3::new(radius as f64, 0.0, 0.0));
    let m1 = Matrix4::from_translation(Cartesian3::new(-radius as f64, 0.0, 0.0));
    let cylinders = pack_cylinders(&[(radius, length, m0), (radius, length, m1)]);
    let cylinder_batch_ids = vec![0, 1];

    let geom = Vector3DTileGeometry::new(Vector3DTileGeometryOptions {
        cylinders: Some(cylinders),
        cylinder_batch_ids: Some(cylinder_batch_ids),
        ..Default::default()
    });

    assert_eq!(geom.batches.len(), 2);
    assert!(geom.triangles_length > 0);
}

#[test]
fn test_renders_single_ellipsoid() {
    let radii = Cartesian3::new(500.0, 500.0, 500.0);
    let ellipsoids = pack_ellipsoids(&[(radii, Matrix4::IDENTITY)]);
    let ellipsoid_batch_ids = vec![0];

    let geom = Vector3DTileGeometry::new(Vector3DTileGeometryOptions {
        ellipsoids: Some(ellipsoids),
        ellipsoid_batch_ids: Some(ellipsoid_batch_ids),
        ..Default::default()
    });

    assert!(geom.ready);
    assert!(geom.triangles_length > 0);
    assert_eq!(geom.batches.len(), 1);
}

#[test]
fn test_renders_multiple_ellipsoids() {
    let radii = Cartesian3::new(500.0, 500.0, 500.0);
    let m0 = Matrix4::from_translation(Cartesian3::new(radii.x, 0.0, 0.0));
    let m1 = Matrix4::from_translation(Cartesian3::new(-radii.x, 0.0, 0.0));
    let ellipsoids = pack_ellipsoids(&[(radii, m0), (radii, m1)]);
    let ellipsoid_batch_ids = vec![0, 1];

    let geom = Vector3DTileGeometry::new(Vector3DTileGeometryOptions {
        ellipsoids: Some(ellipsoids),
        ellipsoid_batch_ids: Some(ellipsoid_batch_ids),
        ..Default::default()
    });

    assert_eq!(geom.batches.len(), 2);
    assert!(geom.triangles_length > 0);
}

#[test]
fn test_renders_single_sphere() {
    let radius = 500.0;
    let spheres = pack_spheres(&[(radius, Cartesian3::ZERO)]);
    let sphere_batch_ids = vec![0];

    let geom = Vector3DTileGeometry::new(Vector3DTileGeometryOptions {
        spheres: Some(spheres),
        sphere_batch_ids: Some(sphere_batch_ids),
        ..Default::default()
    });

    assert!(geom.ready);
    assert!(geom.triangles_length > 0);
    assert_eq!(geom.batches.len(), 1);
}

#[test]
fn test_renders_multiple_spheres() {
    let radius = 500.0;
    let spheres = pack_spheres(&[
        (radius, Cartesian3::new(radius as f64, 0.0, 0.0)),
        (radius, Cartesian3::new(-radius as f64, 0.0, 0.0)),
    ]);
    let sphere_batch_ids = vec![0, 1];

    let geom = Vector3DTileGeometry::new(Vector3DTileGeometryOptions {
        spheres: Some(spheres),
        sphere_batch_ids: Some(sphere_batch_ids),
        ..Default::default()
    });

    assert_eq!(geom.batches.len(), 2);
    assert!(geom.triangles_length > 0);
}

#[test]
fn test_renders_with_multiple_types_of_each_geometry() {
    let box_dim = Cartesian3::new(10.0, 10.0, 10.0);
    let boxes = pack_boxes(&[(box_dim, Matrix4::IDENTITY)]);
    let box_batch_ids = vec![0];

    let cyl = pack_cylinders(&[(5.0, 20.0, Matrix4::IDENTITY)]);
    let cyl_batch_ids = vec![1];

    let sph = pack_spheres(&[(10.0, Cartesian3::ZERO)]);
    let sph_batch_ids = vec![2];

    let geom = Vector3DTileGeometry::new(Vector3DTileGeometryOptions {
        boxes: Some(boxes),
        box_batch_ids: Some(box_batch_ids),
        cylinders: Some(cyl),
        cylinder_batch_ids: Some(cyl_batch_ids),
        spheres: Some(sph),
        sphere_batch_ids: Some(sph_batch_ids),
        ..Default::default()
    });

    assert_eq!(geom.batches.len(), 3);
    assert!(geom.triangles_length >= 12);
}

#[test]
fn test_renders_multiple_geometries_after_rebatch_and_color_update() {
    let box_dim = Cartesian3::new(10.0, 10.0, 10.0);
    let boxes = pack_boxes(&[(box_dim, Matrix4::IDENTITY)]);
    let box_batch_ids = vec![0];

    let mut geom = Vector3DTileGeometry::new(Vector3DTileGeometryOptions {
        boxes: Some(boxes),
        box_batch_ids: Some(box_batch_ids),
        ..Default::default()
    });

    assert_eq!(geom.batches[0].color, Color::WHITE);
    geom.update_commands(0, Color::BLUE);
    assert_eq!(geom.batches[0].color, Color::BLUE);
}

#[test]
fn test_renders_based_on_classification_type() {
    let boxes = pack_boxes(&[(Cartesian3::new(1.0, 1.0, 1.0), Matrix4::IDENTITY)]);
    let mut geom = Vector3DTileGeometry::new(Vector3DTileGeometryOptions {
        boxes: Some(boxes.clone()),
        box_batch_ids: Some(vec![0]),
        classification_type: Some(ClassificationType::Both),
        ..Default::default()
    });

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: true,
            pick: false,
        },
        ..Default::default()
    };

    geom.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 2);
    assert_eq!(frame_state.command_list[0].pass, Some(Pass::Globe));
    assert_eq!(
        frame_state.command_list[1].pass,
        Some(Pass::Cesium3dTileClassification)
    );

    // Terrain only
    frame_state.command_list.clear();
    geom.classification_type = ClassificationType::Terrain;
    geom.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 1);
    assert_eq!(frame_state.command_list[0].pass, Some(Pass::Globe));

    // Cesium3DTile only
    frame_state.command_list.clear();
    geom.classification_type = ClassificationType::Cesium3DTile;
    geom.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 1);
    assert_eq!(
        frame_state.command_list[0].pass,
        Some(Pass::Cesium3dTileClassification)
    );
}

#[test]
fn test_renders_wireframe_and_picks() {
    let boxes = pack_boxes(&[(Cartesian3::new(1.0, 1.0, 1.0), Matrix4::IDENTITY)]);
    let mut geom = Vector3DTileGeometry::new(Vector3DTileGeometryOptions {
        boxes: Some(boxes),
        box_batch_ids: Some(vec![0]),
        classification_type: Some(ClassificationType::Terrain),
        ..Default::default()
    });

    geom.debug_wireframe = true;

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: true,
            pick: true,
        },
        ..Default::default()
    };

    geom.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 1);
    let prog = frame_state.command_list[0].shader_program.as_ref().unwrap();
    assert_eq!(
        prog,
        &cesium_renderer::commands::ShaderProgramHandle::Name(
            "Vector3DTileGeometryWireframe".to_string()
        )
    );
}

#[test]
fn test_is_destroyed_and_destroy() {
    let boxes = pack_boxes(&[(Cartesian3::new(1.0, 1.0, 1.0), Matrix4::IDENTITY)]);
    let mut geom = Vector3DTileGeometry::new(Vector3DTileGeometryOptions {
        boxes: Some(boxes),
        box_batch_ids: Some(vec![0]),
        ..Default::default()
    });

    assert!(!geom.is_destroyed());
    geom.destroy();
    assert!(geom.is_destroyed());
    assert!(!geom.ready);
}
