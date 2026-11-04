use std::f64::consts::FRAC_PI_2;

use cesium_math::{
    Cartesian2, Cartesian3, Ellipsoid, GeoRectangle, GeographicProjection, GeometryOffsetAttribute,
    Matrix2, RectangleOutlineGeometry, RectangleOutlineGeometryOptions, EPSILON14, EPSILON6,
    EPSILON9,
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
    let options = RectangleOutlineGeometryOptions {
        rectangle,
        granularity: Some(1.0),
        ellipsoid: None,
        height: None,
        rotation: None,
        extruded_height: None,
        offset_attribute: None,
    };
    let geom = RectangleOutlineGeometry::new(options).unwrap();
    let m = RectangleOutlineGeometry::create_geometry(&geom)
        .unwrap()
        .unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 8 * 3);
    assert_eq!(m.indices.len(), 8 * 2);

    let expected_nw: Cartesian3 = Ellipsoid::WGS84
        .cartographic_to_cartesian(rectangle.northwest())
        .unwrap()
        .into();
    let actual_nw = Cartesian3::new(positions[0], positions[1], positions[2]);
    assert!(cartesian_approx_eq(&actual_nw, &expected_nw, EPSILON9));
}

#[test]
fn test_computes_positions_across_idl() {
    let rectangle = GeoRectangle::from_degrees(179.0, -1.0, -179.0, 1.0);
    let options = RectangleOutlineGeometryOptions {
        rectangle,
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        extruded_height: None,
        offset_attribute: None,
    };
    let geom = RectangleOutlineGeometry::new(options).unwrap();
    let m = RectangleOutlineGeometry::create_geometry(&geom)
        .unwrap()
        .unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 8 * 3);
    assert_eq!(m.indices.len(), 8 * 2);

    let expected_nw: Cartesian3 = Ellipsoid::WGS84
        .cartographic_to_cartesian(rectangle.northwest())
        .unwrap()
        .into();
    let actual_nw = Cartesian3::new(positions[0], positions[1], positions[2]);
    assert!(cartesian_approx_eq(&actual_nw, &expected_nw, EPSILON9));
}

#[test]
fn test_computes_positions_at_north_pole() {
    let rectangle = GeoRectangle::from_degrees(-180.0, 89.0, -179.0, 90.0);
    let options = RectangleOutlineGeometryOptions {
        rectangle,
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        extruded_height: None,
        offset_attribute: None,
    };
    let geom = RectangleOutlineGeometry::new(options).unwrap();
    let m = RectangleOutlineGeometry::create_geometry(&geom)
        .unwrap()
        .unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 5 * 3);
    assert_eq!(m.indices.len(), 5 * 2);
}

#[test]
fn test_computes_positions_at_south_pole() {
    let rectangle = GeoRectangle::from_degrees(-180.0, -90.0, -179.0, -89.0);
    let options = RectangleOutlineGeometryOptions {
        rectangle,
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        extruded_height: None,
        offset_attribute: None,
    };
    let geom = RectangleOutlineGeometry::new(options).unwrap();
    let m = RectangleOutlineGeometry::create_geometry(&geom)
        .unwrap()
        .unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 5 * 3);
    assert_eq!(m.indices.len(), 5 * 2);
}

#[test]
fn test_compute_positions_with_rotation() {
    let rectangle = GeoRectangle::new(-1.0, -1.0, 1.0, 1.0);
    let angle = FRAC_PI_2;
    let options = RectangleOutlineGeometryOptions {
        rectangle,
        rotation: Some(angle),
        granularity: Some(1.0),
        ellipsoid: None,
        height: None,
        extruded_height: None,
        offset_attribute: None,
    };
    let geom = RectangleOutlineGeometry::new(options).unwrap();
    let m = RectangleOutlineGeometry::create_geometry(&geom)
        .unwrap()
        .unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 8 * 3);
    assert_eq!(m.indices.len(), 8 * 2);

    let unrotated_nw = rectangle.northwest();
    let projection = GeographicProjection::default();
    let projected_nw = projection.project(unrotated_nw).unwrap();
    let rot_m = Matrix2::from_rotation(angle);
    let rot_v = rot_m.multiply_by_vector(Cartesian2::new(projected_nw.x, projected_nw.y));
    let rotated_nw_carto = projection
        .unproject(glam::DVec3::new(rot_v.x, rot_v.y, projected_nw.z))
        .unwrap();
    let rotated_nw: Cartesian3 = Ellipsoid::WGS84
        .cartographic_to_cartesian(rotated_nw_carto)
        .unwrap()
        .into();

    let actual = Cartesian3::new(positions[0], positions[1], positions[2]);
    assert!(cartesian_approx_eq(&actual, &rotated_nw, EPSILON6));
}

#[test]
fn test_throws_if_rotated_rectangle_is_invalid() {
    let options = RectangleOutlineGeometryOptions {
        rectangle: GeoRectangle::new(-FRAC_PI_2, 1.0, FRAC_PI_2, FRAC_PI_2),
        rotation: Some(FRAC_PI_2),
        ellipsoid: None,
        granularity: None,
        height: None,
        extruded_height: None,
        offset_attribute: None,
    };
    let geom = RectangleOutlineGeometry::new(options).unwrap();
    assert!(RectangleOutlineGeometry::create_geometry(&geom).is_err());
}

#[test]
fn test_throws_if_north_is_less_than_south() {
    let options = RectangleOutlineGeometryOptions {
        rectangle: GeoRectangle::new(-FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, -FRAC_PI_2),
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        extruded_height: None,
        offset_attribute: None,
    };
    assert!(RectangleOutlineGeometry::new(options).is_err());
}

#[test]
fn test_computes_positions_extruded() {
    let rectangle = GeoRectangle::new(-2.0, -1.0, 0.0, 1.0);
    let options = RectangleOutlineGeometryOptions {
        rectangle,
        granularity: Some(1.0),
        extruded_height: Some(2.0),
        ellipsoid: None,
        height: None,
        rotation: None,
        offset_attribute: None,
    };
    let geom = RectangleOutlineGeometry::new(options).unwrap();
    let m = RectangleOutlineGeometry::create_geometry(&geom)
        .unwrap()
        .unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 16 * 3);
    assert_eq!(m.indices.len(), 20 * 2);
}

#[test]
fn test_computes_positions_extruded_at_north_pole() {
    let rectangle = GeoRectangle::from_degrees(-180.0, 89.0, -179.0, 90.0);
    let options = RectangleOutlineGeometryOptions {
        rectangle,
        extruded_height: Some(2.0),
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        offset_attribute: None,
    };
    let geom = RectangleOutlineGeometry::new(options).unwrap();
    let m = RectangleOutlineGeometry::create_geometry(&geom)
        .unwrap()
        .unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 10 * 3);
    assert_eq!(m.indices.len(), 13 * 2);
}

#[test]
fn test_computes_positions_extruded_at_south_pole() {
    let rectangle = GeoRectangle::from_degrees(-180.0, -90.0, -179.0, -89.0);
    let options = RectangleOutlineGeometryOptions {
        rectangle,
        extruded_height: Some(2.0),
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        offset_attribute: None,
    };
    let geom = RectangleOutlineGeometry::new(options).unwrap();
    let m = RectangleOutlineGeometry::create_geometry(&geom)
        .unwrap()
        .unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 10 * 3);
    assert_eq!(m.indices.len(), 13 * 2);
}

#[test]
fn test_compute_positions_with_rotation_extruded() {
    let rectangle = GeoRectangle::new(-1.0, -1.0, 1.0, 1.0);
    let angle = FRAC_PI_2;
    let options = RectangleOutlineGeometryOptions {
        rectangle,
        rotation: Some(angle),
        granularity: Some(1.0),
        extruded_height: Some(2.0),
        ellipsoid: None,
        height: None,
        offset_attribute: None,
    };
    let geom = RectangleOutlineGeometry::new(options).unwrap();
    let m = RectangleOutlineGeometry::create_geometry(&geom)
        .unwrap()
        .unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 16 * 3);
    assert_eq!(m.indices.len(), 20 * 2);

    let unrotated_nw = rectangle.northwest();
    let projection = GeographicProjection::default();
    let projected_nw = projection.project(unrotated_nw).unwrap();
    let rot_m = Matrix2::from_rotation(angle);
    let rot_v = rot_m.multiply_by_vector(Cartesian2::new(projected_nw.x, projected_nw.y));
    let mut rotated_nw_carto = projection
        .unproject(glam::DVec3::new(rot_v.x, rot_v.y, projected_nw.z))
        .unwrap();
    rotated_nw_carto.height = 2.0;
    let rotated_nw: Cartesian3 = Ellipsoid::WGS84
        .cartographic_to_cartesian(rotated_nw_carto)
        .unwrap()
        .into();

    let actual = Cartesian3::new(positions[0], positions[1], positions[2]);
    assert!(cartesian_approx_eq(&actual, &rotated_nw, EPSILON6));
}

#[test]
fn test_computes_non_extruded_rectangle_if_height_is_small() {
    let rectangle = GeoRectangle::new(-2.0, -1.0, 0.0, 1.0);
    let options = RectangleOutlineGeometryOptions {
        rectangle,
        granularity: Some(1.0),
        extruded_height: Some(EPSILON14),
        ellipsoid: None,
        height: None,
        rotation: None,
        offset_attribute: None,
    };
    let geom = RectangleOutlineGeometry::new(options).unwrap();
    let m = RectangleOutlineGeometry::create_geometry(&geom)
        .unwrap()
        .unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 8 * 3);
    assert_eq!(m.indices.len(), 8 * 2);
}

#[test]
fn test_undefined_is_returned_if_any_side_are_of_length_zero() {
    let r0 = GeoRectangle::from_degrees(-80.0, 39.0, -80.0, 42.0);
    let r1 = GeoRectangle::from_degrees(-81.0, 42.0, -80.0, 42.0);
    let r2 = GeoRectangle::from_degrees(-80.0, 39.0, -80.0, 39.0);

    let geom0 = RectangleOutlineGeometry::new(RectangleOutlineGeometryOptions {
        rectangle: r0,
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        extruded_height: None,
        offset_attribute: None,
    })
    .unwrap();
    let geom1 = RectangleOutlineGeometry::new(RectangleOutlineGeometryOptions {
        rectangle: r1,
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        extruded_height: None,
        offset_attribute: None,
    })
    .unwrap();
    let geom2 = RectangleOutlineGeometry::new(RectangleOutlineGeometryOptions {
        rectangle: r2,
        ellipsoid: None,
        granularity: None,
        height: None,
        rotation: None,
        extruded_height: None,
        offset_attribute: None,
    })
    .unwrap();

    assert!(RectangleOutlineGeometry::create_geometry(&geom0)
        .unwrap()
        .is_none());
    assert!(RectangleOutlineGeometry::create_geometry(&geom1)
        .unwrap()
        .is_none());
    assert!(RectangleOutlineGeometry::create_geometry(&geom2)
        .unwrap()
        .is_none());
}

#[test]
fn test_computes_offset_attribute() {
    let rectangle = GeoRectangle::new(-2.0, -1.0, 0.0, 1.0);
    let options = RectangleOutlineGeometryOptions {
        rectangle,
        granularity: Some(1.0),
        offset_attribute: Some(GeometryOffsetAttribute::Top),
        ellipsoid: None,
        height: None,
        rotation: None,
        extruded_height: None,
    };
    let geom = RectangleOutlineGeometry::new(options).unwrap();
    let m = RectangleOutlineGeometry::create_geometry(&geom)
        .unwrap()
        .unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 8 * 3);

    let offset = &m.attributes.apply_offset.unwrap().values;
    assert_eq!(offset.len(), 8);
    for &v in offset {
        assert_eq!(v, 1.0);
    }
}

#[test]
fn test_computes_offset_attribute_extruded_for_top_vertices() {
    let rectangle = GeoRectangle::new(-2.0, -1.0, 0.0, 1.0);
    let options = RectangleOutlineGeometryOptions {
        rectangle,
        granularity: Some(1.0),
        extruded_height: Some(2.0),
        offset_attribute: Some(GeometryOffsetAttribute::Top),
        ellipsoid: None,
        height: None,
        rotation: None,
    };
    let geom = RectangleOutlineGeometry::new(options).unwrap();
    let m = RectangleOutlineGeometry::create_geometry(&geom)
        .unwrap()
        .unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 16 * 3);

    let offset = &m.attributes.apply_offset.unwrap().values;
    assert_eq!(offset.len(), 16);

    let mut expected = vec![0.0; 16];
    for v in expected.iter_mut().take(8) {
        *v = 1.0;
    }
    assert_eq!(offset, &expected);
}

#[test]
fn test_computes_offset_attribute_extruded_for_all_vertices() {
    let rectangle = GeoRectangle::new(-2.0, -1.0, 0.0, 1.0);
    let options = RectangleOutlineGeometryOptions {
        rectangle,
        granularity: Some(1.0),
        extruded_height: Some(2.0),
        offset_attribute: Some(GeometryOffsetAttribute::All),
        ellipsoid: None,
        height: None,
        rotation: None,
    };
    let geom = RectangleOutlineGeometry::new(options).unwrap();
    let m = RectangleOutlineGeometry::create_geometry(&geom)
        .unwrap()
        .unwrap();

    let positions = &m.attributes.position.unwrap().values;
    assert_eq!(positions.len(), 16 * 3);

    let offset = &m.attributes.apply_offset.unwrap().values;
    assert_eq!(offset.len(), 16);

    let expected = vec![1.0; 16];
    assert_eq!(offset, &expected);
}

#[test]
fn test_pack_and_unpack_extruded() {
    let ellipsoid = Ellipsoid::new(glam::DVec3::new(5.0, 6.0, 7.0)).unwrap();
    let rectangle = RectangleOutlineGeometry::new(RectangleOutlineGeometryOptions {
        rectangle: GeoRectangle::new(0.1, 0.2, 0.3, 0.4),
        ellipsoid: Some(ellipsoid),
        granularity: Some(8.0),
        height: Some(9.0),
        rotation: Some(10.0),
        extruded_height: Some(11.0),
        offset_attribute: None,
    })
    .unwrap();

    let packed_instance = [0.1, 0.2, 0.3, 0.4, 5.0, 6.0, 7.0, 8.0, 11.0, 10.0, 9.0, -1.0];
    assert_eq!(
        RectangleOutlineGeometry::PACKED_LENGTH,
        packed_instance.len()
    );

    let mut array = vec![0.0; RectangleOutlineGeometry::PACKED_LENGTH];
    rectangle.pack(&mut array, 0).unwrap();
    assert_eq!(&array, &packed_instance);

    let unpacked = RectangleOutlineGeometry::unpack(&packed_instance, 0).unwrap();
    assert_eq!(unpacked, rectangle);
}

#[test]
fn test_pack_and_unpack_at_height() {
    let ellipsoid = Ellipsoid::new(glam::DVec3::new(5.0, 6.0, 7.0)).unwrap();
    let rectangle = RectangleOutlineGeometry::new(RectangleOutlineGeometryOptions {
        rectangle: GeoRectangle::new(0.1, 0.2, 0.3, 0.4),
        ellipsoid: Some(ellipsoid),
        granularity: Some(8.0),
        height: Some(9.0),
        rotation: Some(10.0),
        extruded_height: None,
        offset_attribute: None,
    })
    .unwrap();

    let packed_instance = [0.1, 0.2, 0.3, 0.4, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 9.0, -1.0];
    assert_eq!(
        RectangleOutlineGeometry::PACKED_LENGTH,
        packed_instance.len()
    );

    let mut array = vec![0.0; RectangleOutlineGeometry::PACKED_LENGTH];
    rectangle.pack(&mut array, 0).unwrap();
    assert_eq!(&array, &packed_instance);

    let unpacked = RectangleOutlineGeometry::unpack(&packed_instance, 0).unwrap();
    assert_eq!(unpacked, rectangle);
}
