//! Cases adapted from CesiumJS OrientedBoundingBoxSpec (Apache-2.0).
use cesium_math::{
    Cartographic, CullingVolume, Ellipsoid, GeoRectangle, Occluder, OrientedBoundingBox as O,
    Plane, PlaneIntersection as I, Sphere,
};
use glam::{DMat3, DMat4, DQuat, DVec3};

#[test]
fn transformation_corners_and_pack() {
    let transform = DMat4::from_scale_rotation_translation(
        DVec3::new(1.0, 2.0, 3.0),
        DQuat::from_rotation_z(0.4),
        DVec3::new(1.0, 2.0, 3.0),
    );
    let b = O::from_transformation(transform).unwrap();
    assert_eq!(b.center, DVec3::new(1.0, 2.0, 3.0));
    assert_eq!(b.transformation(), transform);
    let corners = b.corners();
    for (i, corner) in corners.into_iter().enumerate() {
        let local = DVec3::new(
            if i & 4 == 0 { -0.5 } else { 0.5 },
            if i & 2 == 0 { -0.5 } else { 0.5 },
            if i & 1 == 0 { -0.5 } else { 0.5 },
        );
        assert!(corner.distance(transform.transform_point3(local)) < 1e-14);
    }
    let mut packed = [9.0; 14];
    b.pack(&mut packed, 1).unwrap();
    assert_eq!(O::unpack(&packed, 1).unwrap(), b);
    assert!(b.pack(&mut packed, usize::MAX).is_err());
    assert_eq!(
        O::from_transformation(DMat4::from_scale(DVec3::ZERO)).unwrap(),
        O::default()
    );
}

#[test]
fn distance_to_rotated_and_degenerate_boxes() {
    let rotation = DMat3::from_rotation_y(0.7) * DMat3::from_rotation_z(-0.4);
    let center = DVec3::new(4.0, 3.0, 2.0);
    for scale in [
        DVec3::new(2.0, 3.0, 4.0),
        DVec3::new(0.0, 3.0, 4.0),
        DVec3::new(2.0, 0.0, 0.0),
        DVec3::ZERO,
    ] {
        let b = O::new(center, rotation * DMat3::from_diagonal(scale));
        let local = DVec3::new(4.0, -5.0, 6.0);
        let excess = (local.abs() - scale).max(DVec3::ZERO);
        assert!(
            (b.distance_squared_to(center + rotation * local).unwrap() - excess.length_squared())
                .abs()
                < 1e-12
        );
        assert_eq!(b.distance_squared_to(center).unwrap(), 0.0);
    }
    let sheared = O::new(
        center,
        DMat3::from_cols(DVec3::X, DVec3::new(1.0, 1.0, 0.0), DVec3::Z),
    );
    assert!(sheared.distance_squared_to(center).is_none());
}

#[test]
fn planes_distance_intervals_and_culling() {
    let b = O::new(
        DVec3::new(4.0, 3.0, 2.0),
        DMat3::from_rotation_z(0.5) * DMat3::from_diagonal(DVec3::new(2.0, 3.0, 4.0)),
    );
    let direction = DVec3::new(0.3, 0.4, 0.5).normalize();
    let position = DVec3::new(-3.0, 1.0, 4.0);
    let interval = b.compute_plane_distances(position, direction);
    let projections: Vec<_> = b
        .corners()
        .iter()
        .map(|p| direction.dot(*p - position))
        .collect();
    assert!(
        (interval.start - projections.iter().copied().fold(f64::INFINITY, f64::min)).abs() < 1e-14
    );
    assert!(
        (interval.stop
            - projections
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max))
        .abs()
            < 1e-14
    );
    let plane = Plane::new(DVec3::X, -4.0).unwrap();
    assert_eq!(b.intersect_plane(plane), I::Intersecting);
    let c = CullingVolume::from_bounding_sphere(Sphere {
        center: b.center,
        radius: 10.0,
    })
    .unwrap();
    assert_eq!(c.compute_visibility(&b), I::Inside);
    let point = O::default();
    assert_eq!(point.intersect_plane(Plane::ORIGIN_YZ), I::Outside);
}

#[test]
fn upstream_pca_axes_and_translated_rotations() {
    let points = [
        DVec3::X * 2.0,
        DVec3::Y * 3.0,
        DVec3::Z * 4.0,
        -DVec3::X * 2.0,
        -DVec3::Y * 3.0,
        -DVec3::Z * 4.0,
    ];
    let b = O::from_points(&points).unwrap();
    assert_eq!(b.center, DVec3::ZERO);
    assert_eq!(b.half_axes, DMat3::from_diagonal(DVec3::new(2.0, 3.0, 4.0)));
    let rotation = DMat3::from_rotation_z(std::f64::consts::FRAC_PI_4);
    let translation = DVec3::new(-40.0, 20.0, -30.0);
    let transformed = points.map(|p| rotation * p + translation);
    let b = O::from_points(&transformed).unwrap();
    assert!(b.center.distance(translation) < 1e-13);
    let expected = DMat3::from_rotation_z(-std::f64::consts::FRAC_PI_4)
        * DMat3::from_diagonal(DVec3::new(3.0, 2.0, 4.0));
    assert!((b.half_axes - expected)
        .to_cols_array()
        .iter()
        .all(|x| x.abs() < 1e-13));
    for point in transformed {
        assert!(b.distance_squared_to(point).unwrap() < 1e-25);
    }
}

#[test]
fn pca_empty_single_line_and_ecef_points() {
    assert_eq!(O::from_points(&[]).unwrap(), O::default());
    let center = DVec3::new(6_000_000.0, -2_000_000.0, 1_000_000.0);
    let single = O::from_points(&[center]).unwrap();
    assert_eq!(single.center, center);
    assert_eq!(single.half_axes, DMat3::ZERO);
    let d = DVec3::new(1.0, 2.0, -3.0).normalize();
    let points = [center - d * 50.0, center + d * 15.0, center + d * 90.0];
    let b = O::from_points(&points).unwrap();
    for point in points {
        assert!(b.distance_squared_to(point).unwrap() < 1e-16);
    }
    assert!(O::from_points(&[DVec3::splat(f64::NAN)]).is_err());
}

#[test]
fn rectangle_zero_width_and_height_upstream_cases() {
    let r = GeoRectangle::radians(0.0, 0.0, 0.0, 0.0).unwrap();
    for (low, high, center, half_height) in [
        (0.0, 0.0, 1.0, 0.0),
        (1.0, 1.0, 2.0, 0.0),
        (-1.0, -1.0, 0.0, 0.0),
        (-1.0, 1.0, 1.0, 1.0),
    ] {
        let b = O::from_rectangle(r, low, high, Ellipsoid::UNIT_SPHERE).unwrap();
        assert!(b.center.distance(DVec3::X * center) < 1e-14);
        assert!((b.half_axes.z_axis.length() - half_height).abs() < 1e-14);
    }
    let triaxial = Ellipsoid::new(DVec3::new(1.0, 2.0, 1.0)).unwrap();
    assert!(O::from_rectangle(r, 0.0, 0.0, triaxial).is_err());
    assert!(O::from_rectangle(r, 1.0, -1.0, Ellipsoid::UNIT_SPHERE).is_err());
}

#[test]
fn rectangle_dense_samples_with_heights_and_antimeridian() {
    let cases = [
        (
            GeoRectangle::degrees(-20.0, -10.0, 30.0, 25.0).unwrap(),
            Ellipsoid::WGS84,
            -100.0,
            3000.0,
        ),
        (
            GeoRectangle::degrees(170.0, -20.0, -150.0, 30.0).unwrap(),
            Ellipsoid::UNIT_SPHERE,
            -0.1,
            0.3,
        ),
        (
            GeoRectangle::degrees(-135.0, -60.0, 135.0, 70.0).unwrap(),
            Ellipsoid::UNIT_SPHERE,
            -0.1,
            0.3,
        ),
        (
            GeoRectangle::degrees(-180.0, 30.0, 180.0, 80.0).unwrap(),
            Ellipsoid::UNIT_SPHERE,
            0.0,
            0.2,
        ),
    ];
    for (rect, ellipsoid, low, high) in cases {
        let b = O::from_rectangle(rect, low, high, ellipsoid).unwrap();
        let tolerance = ellipsoid.maximum_radius() * 1e-12;
        for i in 0..=30 {
            for j in 0..=20 {
                for height in [low, high] {
                    let p = Cartographic {
                        longitude: rect.west + rect.width() * i as f64 / 30.0,
                        latitude: rect.south + rect.height() * j as f64 / 20.0,
                        height,
                    };
                    let point = ellipsoid.cartographic_to_cartesian(p).unwrap();
                    let distance = b.distance_squared_to(point).unwrap().sqrt();
                    assert!(distance <= tolerance, "{rect:?}: {p:?} distance={distance}");
                }
            }
        }
    }
}

#[test]
fn upstream_occlusion_and_conservative_sheared_bounding_sphere() {
    let occluder = Occluder::new(
        Sphere {
            center: DVec3::new(0.0, 0.0, -1.5),
            radius: 0.5,
        },
        DVec3::ZERO,
    )
    .unwrap();
    let scale = 0.25 / 2.0_f64.sqrt();
    let hidden = O::new(
        DVec3::new(0.0, 0.0, -2.75),
        DMat3::from_diagonal(DVec3::splat(scale)),
    );
    assert!(hidden.is_occluded(occluder).unwrap());
    let occluder = Occluder::new(
        Sphere {
            center: DVec3::new(0.0, 0.0, -2.75),
            radius: 0.25,
        },
        DVec3::ZERO,
    )
    .unwrap();
    let front = O::new(
        DVec3::new(0.0, 0.0, -1.5),
        DMat3::from_diagonal(DVec3::splat(0.5 / 2.0_f64.sqrt())),
    );
    assert!(!front.is_occluded(occluder).unwrap());
    let shear = O::new(
        DVec3::ZERO,
        DMat3::from_cols(DVec3::X, -DVec3::X + DVec3::Y, DVec3::Z),
    );
    let sphere = Sphere::from_oriented_box(shear.center, shear.half_axes).unwrap();
    for corner in shear.corners() {
        assert!(corner.distance(sphere.center) <= sphere.radius + 1e-14);
    }
    assert!(
        sphere.radius
            > (shear.half_axes.x_axis + shear.half_axes.y_axis + shear.half_axes.z_axis).length()
    );
}

#[test]
fn constructor_default_and_parameter_values_and_equality() {
    let def = O::default();
    assert_eq!(def.center, DVec3::ZERO);
    assert_eq!(def.half_axes, DMat3::ZERO);

    let center = DVec3::new(1.0, 2.0, 3.0);
    let half_axes = DMat3::from_diagonal(DVec3::splat(2.0));
    let box_val = O::new(center, half_axes);
    assert_eq!(box_val.center, center);
    assert_eq!(box_val.half_axes, half_axes);

    assert!(box_val.equals(Some(&O::new(center, half_axes))));
    assert!(!box_val.equals(Some(&O::new(DVec3::new(2.0, 2.0, 3.0), half_axes))));
    assert!(!box_val.equals(Some(&O::new(center, DMat3::from_diagonal(DVec3::splat(3.0))))));
    assert!(!box_val.equals(None));

    let cloned = box_val;
    assert_eq!(cloned, box_val);
}
