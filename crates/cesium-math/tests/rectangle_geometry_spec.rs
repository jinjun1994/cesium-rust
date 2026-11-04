use std::f64::consts::{FRAC_PI_2, PI};

use cesium_math::{
    Cartesian2, Cartesian3, Ellipsoid, GeoRectangle, GeographicProjection,
    GeometryOffsetAttribute, Matrix2, RectangleGeometry, RectangleGeometryOptions, VertexFormat,
    EPSILON14, EPSILON15, EPSILON6, EPSILON7, EPSILON8, EPSILON9,
};

fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps
}

fn cartesian_approx_eq(a: &Cartesian3, b: &Cartesian3, eps: f64) -> bool {
    approx_eq(a.x, b.x, eps) && approx_eq(a.y, b.y, eps) && approx_eq(a.z, b.z, eps)
}

#[test]
fn test_computes_positions() {
    let rectangle = GeoRectangle::new(-2.0, -1.0, 0.0, 1.0);
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        granularity: Some(1.0),
        ellipsoid: None,
        height: None,
        rotation: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    let length = positions.len();

    assert_eq!(positions.len(), 9 * 3);
    assert_eq!(m.indices.len(), 8 * 3);

    let ellipsoid = Ellipsoid::WGS84;
    let expected_nw_corner: Cartesian3 = ellipsoid
        .cartographic_to_cartesian(rectangle.northwest())
        .unwrap()
        .into();
    let expected_se_corner: Cartesian3 = ellipsoid
        .cartographic_to_cartesian(rectangle.southeast())
        .unwrap()
        .into();

    let actual_nw = Cartesian3::new(positions[0], positions[1], positions[2]);
    let actual_se = Cartesian3::new(
        positions[length - 3],
        positions[length - 2],
        positions[length - 1],
    );

    assert!(cartesian_approx_eq(
        &actual_nw,
        &expected_nw_corner,
        EPSILON9
    ));
    assert!(cartesian_approx_eq(
        &actual_se,
        &expected_se_corner,
        EPSILON9
    ));
}

#[test]
fn test_computes_positions_across_idl() {
    let rectangle = GeoRectangle::from_degrees(179.0, -1.0, -179.0, 1.0);
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    let length = positions.len();

    assert_eq!(positions.len(), 9 * 3);
    assert_eq!(m.indices.len(), 8 * 3);

    let ellipsoid = Ellipsoid::WGS84;
    let expected_nw_corner: Cartesian3 = ellipsoid
        .cartographic_to_cartesian(rectangle.northwest())
        .unwrap()
        .into();
    let expected_se_corner: Cartesian3 = ellipsoid
        .cartographic_to_cartesian(rectangle.southeast())
        .unwrap()
        .into();

    let actual_nw = Cartesian3::new(positions[0], positions[1], positions[2]);
    let actual_se = Cartesian3::new(
        positions[length - 3],
        positions[length - 2],
        positions[length - 1],
    );

    assert!(cartesian_approx_eq(
        &actual_nw,
        &expected_nw_corner,
        EPSILON8
    ));
    assert!(cartesian_approx_eq(
        &actual_se,
        &expected_se_corner,
        EPSILON8
    ));
}

#[test]
fn test_computes_positions_at_north_pole() {
    let rectangle = GeoRectangle::from_degrees(-180.0, 89.0, -179.0, 90.0);
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 5 * 3);
    assert_eq!(m.indices.len(), 3 * 3);
}

#[test]
fn test_computes_positions_at_south_pole() {
    let rectangle = GeoRectangle::from_degrees(-180.0, -90.0, -179.0, -89.0);
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 5 * 3);
    assert_eq!(m.indices.len(), 3 * 3);
}

#[test]
fn test_computes_all_attributes() {
    let options = RectangleGeometryOptions {
        rectangle: GeoRectangle::new(-2.0, -1.0, 0.0, 1.0),
        vertex_format: Some(VertexFormat::ALL),
        granularity: Some(1.0),
        ellipsoid: None,
        height: None,
        rotation: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let num_vertices = 9;
    let num_triangles = 8;
    assert_eq!(
        m.attributes.position.as_ref().unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(
        m.attributes.st.as_ref().unwrap().values.len(),
        num_vertices * 2
    );
    assert_eq!(
        m.attributes.normal.as_ref().unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(
        m.attributes.tangent.as_ref().unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(
        m.attributes.bitangent.as_ref().unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(m.indices.len(), num_triangles * 3);
}

#[test]
fn test_compute_positions_with_rotation() {
    let rectangle = GeoRectangle::new(-1.0, -1.0, 1.0, 1.0);
    let angle = FRAC_PI_2;
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        rotation: Some(angle),
        granularity: Some(1.0),
        ellipsoid: None,
        height: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    let length = positions.len();

    assert_eq!(length, 9 * 3);
    assert_eq!(m.indices.len(), 8 * 3);

    let unrotated_se_corner = rectangle.southeast();
    let projection = GeographicProjection::default();
    let projected_se_corner = projection.project(unrotated_se_corner).unwrap();
    let rotation = Matrix2::from_rotation(angle);
    let rotated_projected = rotation.multiply_by_vector(Cartesian2::new(
        projected_se_corner.x,
        projected_se_corner.y,
    ));
    let rotated_se_corner_cartographic = projection
        .unproject(glam::DVec3::new(
            rotated_projected.x,
            rotated_projected.y,
            projected_se_corner.z,
        ))
        .unwrap();
    let rotated_se_corner: Cartesian3 = Ellipsoid::WGS84
        .cartographic_to_cartesian(rotated_se_corner_cartographic)
        .unwrap()
        .into();
    let actual = Cartesian3::new(
        positions[length - 3],
        positions[length - 2],
        positions[length - 1],
    );
    assert!(cartesian_approx_eq(&actual, &rotated_se_corner, EPSILON6));
}

#[test]
fn test_compute_vertices_with_pi_rotation() {
    let rectangle = GeoRectangle::new(-1.0, -1.0, 1.0, 1.0);
    let options = RectangleGeometryOptions {
        rectangle,
        rotation: Some(PI),
        granularity: Some(1.0),
        vertex_format: None,
        ellipsoid: None,
        height: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    let length = positions.len();

    assert_eq!(length, 9 * 3);
    assert_eq!(m.indices.len(), 8 * 3);

    let unrotated_nw: Cartesian3 = Ellipsoid::WGS84
        .cartographic_to_cartesian(rectangle.northwest())
        .unwrap()
        .into();
    let unrotated_se: Cartesian3 = Ellipsoid::WGS84
        .cartographic_to_cartesian(rectangle.southeast())
        .unwrap()
        .into();

    let actual_first = Cartesian3::new(positions[0], positions[1], positions[2]);
    assert!(cartesian_approx_eq(&actual_first, &unrotated_se, EPSILON8));

    let actual_last = Cartesian3::new(
        positions[length - 3],
        positions[length - 2],
        positions[length - 1],
    );
    assert!(cartesian_approx_eq(&actual_last, &unrotated_nw, EPSILON8));
}

#[test]
fn test_compute_texture_coordinates_with_rotation() {
    let rectangle = GeoRectangle::new(-1.0, -1.0, 1.0, 1.0);
    let angle = FRAC_PI_2;
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_AND_ST),
        st_rotation: Some(angle),
        granularity: Some(1.0),
        ellipsoid: None,
        height: None,
        rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    let st = &m.attributes.st.unwrap().values;
    let length = st.len();

    assert_eq!(positions.len(), 9 * 3);
    assert_eq!(length, 9 * 2);
    assert_eq!(m.indices.len(), 8 * 3);

    assert!(approx_eq(st[length - 2], 0.0, EPSILON14));
    assert!(approx_eq(st[length - 1], 0.0, EPSILON14));
}

#[test]
fn test_compute_texture_coordinate_rotation_with_rectangle_rotation() {
    let rectangle = GeoRectangle::new(-1.0, -1.0, 1.0, 1.0);
    let angle = 30.0f64.to_radians();
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_AND_ST),
        rotation: Some(angle),
        st_rotation: Some(angle),
        granularity: Some(1.0),
        ellipsoid: None,
        height: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let st = &m.attributes.st.unwrap().values;

    assert!(approx_eq(st[0], 0.0, 1e-10));
    assert!(approx_eq(st[1], 1.0, 1e-10));
    assert!(approx_eq(st[4], 1.0, 1e-10));
    assert!(approx_eq(st[5], 1.0, 1e-10));
    assert!(approx_eq(st[12], 0.0, 1e-10));
    assert!(approx_eq(st[13], 0.0, 1e-10));
    assert!(approx_eq(st[16], 1.0, 1e-10));
    assert!(approx_eq(st[17], 0.0, 1e-10));
}

#[test]
fn test_throws_if_rotated_rectangle_is_invalid() {
    let options = RectangleGeometryOptions {
        rectangle: GeoRectangle::new(-FRAC_PI_2, 1.0, FRAC_PI_2, FRAC_PI_2),
        rotation: Some(FRAC_PI_2),
        vertex_format: None,
        ellipsoid: None,
        granularity: None,
        height: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    assert!(RectangleGeometry::create_geometry(&geom).is_err());
}

#[test]
fn test_throws_if_north_is_less_than_south() {
    let options = RectangleGeometryOptions {
        rectangle: GeoRectangle::new(-FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, -FRAC_PI_2),
        vertex_format: None,
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    assert!(RectangleGeometry::new(options).is_err());
}

#[test]
fn test_computes_positions_extruded() {
    let rectangle = GeoRectangle::new(-2.0, -1.0, 0.0, 1.0);
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        granularity: Some(1.0),
        extruded_height: Some(2.0),
        ellipsoid: None,
        height: None,
        rotation: None,
        st_rotation: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 42 * 3);
    assert_eq!(m.indices.len(), 32 * 3);
}

#[test]
fn test_computes_positions_extruded_at_north_pole() {
    let rectangle = GeoRectangle::from_degrees(-180.0, 89.0, -179.0, 90.0);
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        extruded_height: Some(2.0),
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        st_rotation: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 26 * 3);
    assert_eq!(m.indices.len(), 16 * 3);
}

#[test]
fn test_computes_positions_extruded_at_south_pole() {
    let rectangle = GeoRectangle::from_degrees(-180.0, -90.0, -179.0, -89.0);
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        extruded_height: Some(2.0),
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        st_rotation: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 26 * 3);
    assert_eq!(m.indices.len(), 16 * 3);
}

#[test]
fn test_computes_all_attributes_extruded() {
    let options = RectangleGeometryOptions {
        rectangle: GeoRectangle::new(-2.0, -1.0, 0.0, 1.0),
        vertex_format: Some(VertexFormat::ALL),
        granularity: Some(1.0),
        extruded_height: Some(2.0),
        ellipsoid: None,
        height: None,
        rotation: None,
        st_rotation: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let num_vertices = 42;
    let num_triangles = 32;
    assert_eq!(
        m.attributes.position.as_ref().unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(
        m.attributes.st.as_ref().unwrap().values.len(),
        num_vertices * 2
    );
    assert_eq!(
        m.attributes.normal.as_ref().unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(
        m.attributes.tangent.as_ref().unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(
        m.attributes.bitangent.as_ref().unwrap().values.len(),
        num_vertices * 3
    );
    assert_eq!(m.indices.len(), num_triangles * 3);
}

#[test]
fn test_compute_positions_with_rotation_extruded() {
    let rectangle = GeoRectangle::new(-1.0, -1.0, 1.0, 1.0);
    let angle = FRAC_PI_2;
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        rotation: Some(angle),
        granularity: Some(1.0),
        extruded_height: Some(2.0),
        ellipsoid: None,
        height: None,
        st_rotation: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 42 * 3);
    assert_eq!(m.indices.len(), 32 * 3);

    let unrotated_se = rectangle.southeast();
    let projection = GeographicProjection::default();
    let projected_se = projection.project(unrotated_se).unwrap();
    let rot_m = Matrix2::from_rotation(angle);
    let rot_v = rot_m.multiply_by_vector(Cartesian2::new(projected_se.x, projected_se.y));
    let rotated_se_carto = projection
        .unproject(glam::DVec3::new(rot_v.x, rot_v.y, projected_se.z))
        .unwrap();
    let rotated_se: Cartesian3 = Ellipsoid::WGS84
        .cartographic_to_cartesian(rotated_se_carto)
        .unwrap()
        .into();
    let actual = Cartesian3::new(positions[51], positions[52], positions[53]);
    assert!(cartesian_approx_eq(&actual, &rotated_se, EPSILON6));
}

#[test]
fn test_computes_non_extruded_rectangle_if_height_is_small() {
    let rectangle = GeoRectangle::new(-2.0, -1.0, 0.0, 1.0);
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        granularity: Some(1.0),
        extruded_height: Some(EPSILON14),
        ellipsoid: None,
        height: None,
        rotation: None,
        st_rotation: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 9 * 3);
    assert_eq!(m.indices.len(), 8 * 3);
}

#[test]
fn test_computes_offset_attribute() {
    let rectangle = GeoRectangle::new(-2.0, -1.0, 0.0, 1.0);
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        granularity: Some(1.0),
        offset_attribute: Some(GeometryOffsetAttribute::Top),
        ellipsoid: None,
        height: None,
        rotation: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 9 * 3);

    let offset = &m.attributes.apply_offset.unwrap().values;
    assert_eq!(offset.len(), 9);
    for &v in offset {
        assert_eq!(v, 1.0);
    }
}

#[test]
fn test_computes_offset_attribute_extruded_for_top_vertices() {
    let rectangle = GeoRectangle::new(-2.0, -1.0, 0.0, 1.0);
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        granularity: Some(1.0),
        extruded_height: Some(2.0),
        offset_attribute: Some(GeometryOffsetAttribute::Top),
        ellipsoid: None,
        height: None,
        rotation: None,
        st_rotation: None,
        shadow_volume: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 42 * 3);

    let offset = &m.attributes.apply_offset.unwrap().values;
    assert_eq!(offset.len(), 42);

    let mut expected = vec![0.0; 42];
    for v in expected.iter_mut().take(9) {
        *v = 1.0;
    }
    for i in (18..42).step_by(2) {
        expected[i] = 1.0;
    }
    assert_eq!(offset, &expected);
}

#[test]
fn test_computes_offset_attribute_extruded_for_all_vertices() {
    let rectangle = GeoRectangle::new(-2.0, -1.0, 0.0, 1.0);
    let options = RectangleGeometryOptions {
        rectangle,
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        granularity: Some(1.0),
        extruded_height: Some(2.0),
        offset_attribute: Some(GeometryOffsetAttribute::All),
        ellipsoid: None,
        height: None,
        rotation: None,
        st_rotation: None,
        shadow_volume: None,
    };
    let geom = RectangleGeometry::new(options).unwrap();
    let m = RectangleGeometry::create_geometry(&geom).unwrap().unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 42 * 3);

    let offset = &m.attributes.apply_offset.unwrap().values;
    assert_eq!(offset.len(), 42);

    let expected = vec![1.0; 42];
    assert_eq!(offset, &expected);
}

#[test]
fn test_undefined_is_returned_if_any_side_are_of_length_zero() {
    let r0 = GeoRectangle::from_degrees(-80.0, 39.0, -80.0, 42.0);
    let r1 = GeoRectangle::from_degrees(-81.0, 42.0, -80.0, 42.0);
    let r2 = GeoRectangle::from_degrees(-80.0, 39.0, -80.0, 39.0);

    let geom0 = RectangleGeometry::new(RectangleGeometryOptions {
        rectangle: r0,
        vertex_format: None,
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    })
    .unwrap();
    let geom1 = RectangleGeometry::new(RectangleGeometryOptions {
        rectangle: r1,
        vertex_format: None,
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    })
    .unwrap();
    let geom2 = RectangleGeometry::new(RectangleGeometryOptions {
        rectangle: r2,
        vertex_format: None,
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    })
    .unwrap();

    assert!(RectangleGeometry::create_geometry(&geom0).unwrap().is_none());
    assert!(RectangleGeometry::create_geometry(&geom1).unwrap().is_none());
    assert!(RectangleGeometry::create_geometry(&geom2).unwrap().is_none());
}

#[test]
fn test_computing_rectangle_property() {
    let rectangle = GeoRectangle::from_degrees(-1.0, -1.0, 1.0, 1.0);
    let geometry = RectangleGeometry::new(RectangleGeometryOptions {
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        rectangle,
        granularity: Some(1.0),
        ellipsoid: None,
        height: None,
        rotation: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    })
    .unwrap();

    let r = geometry.rectangle();
    assert_eq!(r.north.to_degrees(), 1.0);
    assert_eq!(r.south.to_degrees(), -1.0);
    assert_eq!(r.east.to_degrees(), 1.0);
    assert_eq!(r.west.to_degrees(), -1.0);
}

#[test]
#[allow(clippy::approx_constant)]
fn test_computing_rectangle_property_with_rotation() {
    let rectangle = GeoRectangle::from_degrees(-1.0, -1.0, 1.0, 1.0);
    let geometry = RectangleGeometry::new(RectangleGeometryOptions {
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        rectangle,
        granularity: Some(1.0),
        rotation: Some(45.0f64.to_radians()),
        ellipsoid: None,
        height: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    })
    .unwrap();

    let r = geometry.rectangle();
    assert!(approx_eq(
        r.north.to_degrees(),
        1.414213562373095,
        EPSILON15
    ));
    assert!(approx_eq(
        r.south.to_degrees(),
        -1.414213562373095,
        EPSILON15
    ));
    assert!(approx_eq(
        r.east.to_degrees(),
        1.414213562373095,
        EPSILON15
    ));
    assert!(approx_eq(
        r.west.to_degrees(),
        -1.4142135623730951,
        EPSILON15
    ));
}

#[test]
fn test_computing_texture_coordinate_rotation_points_property() {
    let rectangle = GeoRectangle::from_degrees(-1.0, -1.0, 1.0, 1.0);
    let geometry = RectangleGeometry::new(RectangleGeometryOptions {
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        rectangle,
        granularity: Some(1.0),
        rotation: Some(90.0f64.to_radians()),
        ellipsoid: None,
        height: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    })
    .unwrap();

    let pts = geometry.texture_coordinate_rotation_points();
    assert_eq!(pts.len(), 6);
    assert!(approx_eq(pts[0], 0.0, EPSILON7));
    assert!(approx_eq(pts[1], 0.0, EPSILON7));
    assert!(approx_eq(pts[2], 0.0, EPSILON7));
    assert!(approx_eq(pts[3], 1.0, EPSILON7));
    assert!(approx_eq(pts[4], 1.0, EPSILON7));
    assert!(approx_eq(pts[5], 0.0, EPSILON7));
}

#[test]
fn test_compute_rectangle_static() {
    let options = RectangleGeometryOptions {
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        rectangle: GeoRectangle::from_degrees(-1.0, -1.0, 1.0, 1.0),
        granularity: Some(1.0),
        ellipsoid: Some(Ellipsoid::UNIT_SPHERE),
        rotation: Some(PI),
        height: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    };
    let geometry = RectangleGeometry::new(options.clone()).unwrap();
    let expected = geometry.rectangle();
    let result = RectangleGeometry::compute_rectangle(&options).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_computing_rectangle_property_with_zero_rotation() {
    let geometry = RectangleGeometry::new(RectangleGeometryOptions {
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        rectangle: GeoRectangle::MAXIMUM,
        granularity: Some(1.0),
        rotation: Some(0.0),
        ellipsoid: None,
        height: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    })
    .unwrap();
    assert!(RectangleGeometry::create_geometry(&geometry).is_ok());
}

#[test]
fn test_can_create_rectangle_geometry_where_nw_and_center_opposite_idl() {
    let rectangle = GeoRectangle::new(
        PI - 0.005,
        (PI / 6.0) + 0.02,
        0.01 - PI,
        (PI / 6.0) + 0.04,
    );
    let geometry = RectangleGeometry::new(RectangleGeometryOptions {
        rectangle,
        rotation: Some(0.5),
        vertex_format: None,
        ellipsoid: None,
        granularity: None,
        height: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    })
    .unwrap();

    assert!(RectangleGeometry::create_geometry(&geometry).is_ok());
}

#[test]
fn test_pack_and_unpack() {
    let rectangle = RectangleGeometry::new(RectangleGeometryOptions {
        vertex_format: Some(VertexFormat::POSITION_ONLY),
        rectangle: GeoRectangle::new(-2.0, -1.0, 0.0, 1.0),
        granularity: Some(1.0),
        ellipsoid: Some(Ellipsoid::UNIT_SPHERE),
        height: None,
        rotation: None,
        st_rotation: None,
        extruded_height: None,
        shadow_volume: None,
        offset_attribute: None,
    })
    .unwrap();

    let packed_instance = [
        -2.0, -1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
        0.0, -1.0,
    ];

    assert_eq!(
        RectangleGeometry::PACKED_LENGTH,
        packed_instance.len()
    );

    let mut array = vec![0.0; RectangleGeometry::PACKED_LENGTH];
    rectangle.pack(&mut array, 0).unwrap();
    assert_eq!(&array, &packed_instance);

    let unpacked = RectangleGeometry::unpack(&packed_instance, 0).unwrap();
    assert_eq!(unpacked, rectangle);
}
