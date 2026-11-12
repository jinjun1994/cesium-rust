use cesium_math::{Cartesian3, Matrix4};
use cesium_renderer::geometry_3d_tile_content::{
    generate_geometry_tile_buffer, Geometry3DTileContent, GeometryTileBufferOptions,
};
use cesium_renderer::vector_3d_tile_geometry::{
    pack_boxes, pack_cylinders, pack_ellipsoids, pack_spheres,
};
use serde_json::json;

#[test]
fn throws_with_invalid_version() {
    let buf = generate_geometry_tile_buffer(&GeometryTileBufferOptions {
        version: Some(2),
        ..Default::default()
    });
    let err =
        Geometry3DTileContent::from_bytes(None, None, "tile.geom", &buf, 0, Matrix4::IDENTITY)
            .unwrap_err();
    assert!(
        err.to_string()
            .contains("Only Geometry tile version 1 is supported."),
        "unexpected error: {err}"
    );
}

#[test]
fn throws_with_empty_feature_table() {
    let buf = generate_geometry_tile_buffer(&GeometryTileBufferOptions {
        feature_table_json_override: Some(Vec::new()),
        ..Default::default()
    });
    let err =
        Geometry3DTileContent::from_bytes(None, None, "tile.geom", &buf, 0, Matrix4::IDENTITY)
            .unwrap_err();
    assert!(
        err.to_string()
            .contains("Feature table must have a byte length greater than zero"),
        "unexpected error: {err}"
    );
}

#[test]
fn throws_when_only_some_batch_ids_are_defined() {
    let buf = generate_geometry_tile_buffer(&GeometryTileBufferOptions {
        boxes: Some(pack_boxes(&[(Cartesian3::new(1.0, 2.0, 3.0), Matrix4::IDENTITY)])),
        spheres: Some(pack_spheres(&[(2.0, Cartesian3::ZERO)])),
        box_batch_ids: Some(vec![0]),
        ..Default::default()
    });
    let err =
        Geometry3DTileContent::from_bytes(None, None, "tile.geom", &buf, 0, Matrix4::IDENTITY)
            .unwrap_err();
    assert!(
        err.to_string()
            .contains("If one group of batch ids is defined, then all batch ids must be defined"),
        "unexpected error: {err}"
    );
}

#[test]
fn loads_empty_geometry_tile() {
    let buf = generate_geometry_tile_buffer(&GeometryTileBufferOptions {
        empty: true,
        ..Default::default()
    });
    let content =
        Geometry3DTileContent::from_bytes(None, None, "empty.geom", &buf, 0, Matrix4::IDENTITY)
            .unwrap();
    assert!(content.ready());
    assert_eq!(content.features_length(), 0);
    assert_eq!(content.triangles_length(), 0);
    assert_eq!(content.geometry_byte_length(), 0);
}

#[test]
fn loads_all_geometry_types_with_batch_table_and_destroys() {
    let buf = generate_geometry_tile_buffer(&GeometryTileBufferOptions {
        boxes: Some(pack_boxes(&[(Cartesian3::new(2.0, 2.0, 2.0), Matrix4::IDENTITY)])),
        cylinders: Some(pack_cylinders(&[(1.0, 4.0, Matrix4::IDENTITY)])),
        ellipsoids: Some(pack_ellipsoids(&[(
            Cartesian3::new(1.0, 2.0, 3.0),
            Matrix4::IDENTITY,
        )])),
        spheres: Some(pack_spheres(&[(1.5, Cartesian3::ZERO)])),
        box_batch_ids: Some(vec![0]),
        cylinder_batch_ids: Some(vec![1]),
        ellipsoid_batch_ids: Some(vec![2]),
        sphere_batch_ids: Some(vec![3]),
        rtc_center: Some([1.0, 2.0, 3.0]),
        batch_table_json: Some(json!({
            "name": ["box", "cyl", "ell", "sph"],
            "height": [10, 20, 30, 40]
        })),
        ..Default::default()
    });

    let mut content =
        Geometry3DTileContent::from_bytes(None, None, "all.geom", &buf, 0, Matrix4::IDENTITY)
            .unwrap();

    assert!(content.ready());
    assert_eq!(content.features_length(), 4);
    assert_eq!(content.points_length(), 0);
    assert_eq!(content.textures_byte_length(), 0);
    assert!(content.batch_table_byte_length() > 0);
    assert!(content.has_property(0, "name"));
    assert_eq!(
        content.get_feature(2).unwrap().get_property("name").unwrap(),
        &json!("ell")
    );

    assert!(!content.is_destroyed());
    content.destroy();
    assert!(content.is_destroyed());
    assert_eq!(content.features_length(), 0);
}
