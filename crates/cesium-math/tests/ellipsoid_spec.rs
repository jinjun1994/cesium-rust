//! Reference cases from CesiumJS EllipsoidSpec (Apache-2.0).
#![allow(clippy::excessive_precision)]
use cesium_math::{Cartographic, Ellipsoid};
use glam::DVec3;

#[test]
fn construction_and_scaled_space() {
    let e = Ellipsoid::new(DVec3::new(1.0, 2.0, 3.0)).unwrap();
    assert_eq!(Ellipsoid::default().radii(), DVec3::ZERO);
    assert_eq!(Ellipsoid::default().one_over_radii(), DVec3::ZERO);
    assert_eq!(e.radii_squared(), DVec3::new(1.0, 4.0, 9.0));
    assert_eq!(e.radii_to_fourth(), DVec3::new(1.0, 16.0, 81.0));
    assert_eq!(e.one_over_radii(), DVec3::new(1.0, 0.5, 1.0 / 3.0));
    assert_eq!((e.minimum_radius(), e.maximum_radius()), (1.0, 3.0));
    let p = DVec3::new(4.0, 6.0, 9.0);
    assert_eq!(e.from_scaled_space(e.to_scaled_space(p)), p);
    assert!(Ellipsoid::new(DVec3::new(-1.0, 1.0, 1.0)).is_err());
}

#[test]
fn upstream_normals_and_cartographic_reference() {
    let e = Ellipsoid::WGS84;
    let position = DVec3::new(4582719.8827300891, -4582719.8827300882, 1725510.4250797231);
    let normal = DVec3::new(
        0.6829975339864266,
        -0.68299753398642649,
        0.25889908678270795,
    );
    assert!(
        e.geodetic_surface_normal(position)
            .unwrap()
            .distance(normal)
            < 1e-15
    );
    assert!(e.geodetic_surface_normal(DVec3::ZERO).is_none());
    let cartographic = Cartographic::degrees(-45.0, 15.0, 330000.0).unwrap();
    let n = DVec3::new(
        0.68301270189221941,
        -0.6830127018922193,
        0.25881904510252074,
    );
    assert!(Ellipsoid::geodetic_surface_normal_cartographic(cartographic).distance(n) < 1e-15);
    assert!(
        e.cartographic_to_cartesian(cartographic)
            .unwrap()
            .distance(position)
            < 1e-7
    );
}

#[test]
fn upstream_triaxial_surface_projection() {
    let e = Ellipsoid::new(DVec3::new(1.0, 2.0, 3.0)).unwrap();
    let p = DVec3::new(4.0, 5.0, 6.0);
    let geodetic = DVec3::new(0.2680893773941855, 1.1160466902266495, 2.3559801120411263);
    let geocentric = DVec3::new(0.7807200583588266, 0.9759000729485333, 1.1710800875382399);
    assert!(e.scale_to_geodetic_surface(p).unwrap().distance(geodetic) < 1e-15);
    assert!(
        e.scale_to_geocentric_surface(p)
            .unwrap()
            .distance(geocentric)
            < 1e-15
    );
    for (axis, radius) in [(DVec3::X, 1.0), (DVec3::Y, 2.0), (DVec3::Z, 3.0)] {
        assert!(
            (e.scale_to_geodetic_surface(axis * 8.0).unwrap() - axis * radius).length() < 1e-12
        );
    }
    assert!(e.scale_to_geodetic_surface(DVec3::ZERO).is_none());
}

#[test]
fn triaxial_roundtrip_above_and_below_surface() {
    let ellipsoid = Ellipsoid::new(DVec3::new(6.0, 5.0, 4.0)).unwrap();
    for (lon, lat, height) in [
        (0.0, 0.0, 0.0),
        (35.0, 45.0, 1.5),
        (-110.0, -30.0, -0.5),
        (50.0, 90.0, 2.0),
    ] {
        let c = Cartographic::degrees(lon, lat, height).unwrap();
        let p = ellipsoid.cartographic_to_cartesian(c).unwrap();
        let restored = ellipsoid.cartesian_to_cartographic(p).unwrap();
        assert!(
            ellipsoid
                .cartographic_to_cartesian(restored)
                .unwrap()
                .distance(p)
                < 1e-10
        );
        assert!((restored.height - height).abs() < 1e-10);
    }
}

#[test]
fn upstream_near_center_cartographic() {
    let e = Ellipsoid::WGS84;
    let c = e
        .cartesian_to_cartographic(DVec3::new(1e-50, 1e-60, 1e-70))
        .unwrap();
    assert!((c.longitude - 9.999999999999999e-11).abs() < 1e-24);
    assert!((c.latitude - 1.0067394967422763e-20).abs() < 1e-34);
    assert_eq!(c.height, -6378137.0);
    assert!(e.cartesian_to_cartographic(DVec3::splat(1e-150)).is_none());
    assert!(e.cartesian_to_cartographic(DVec3::ZERO).is_none());
}

#[test]
fn surface_normal_axis_intersection_and_local_curvature() {
    let e = Ellipsoid::WGS84;
    let p = e
        .cartographic_to_cartesian(Cartographic::degrees(35.23, 33.23, 0.0).unwrap())
        .unwrap();
    let axis = e
        .surface_normal_intersection_with_z_axis(p, 0.0)
        .unwrap()
        .unwrap();
    let normal = e.geodetic_surface_normal(p).unwrap();
    let expected = p.z - normal.z * p.x / normal.x;
    assert!((axis.z - expected).abs() < 1e-8);
    assert!(e
        .surface_normal_intersection_with_z_axis(p, e.radii().z)
        .unwrap()
        .is_none());
    assert!(Ellipsoid::new(DVec3::new(1.0, 2.0, 3.0))
        .unwrap()
        .surface_normal_intersection_with_z_axis(p, 0.0)
        .is_err());
    for lat in [0.0, 90.0] {
        let point = e
            .cartographic_to_cartesian(Cartographic::degrees(0.0, lat, 0.0).unwrap())
            .unwrap();
        let (east, north) = e.local_curvature(point).unwrap();
        let (a, b) = (e.maximum_radius(), e.minimum_radius());
        let expected = if lat == 0.0 {
            (1.0 / a, a / (b * b))
        } else {
            (b / (a * a), b / (a * a))
        };
        assert!((east - expected.0).abs() < 1e-15 && (north - expected.1).abs() < 1e-15);
    }
}

#[test]
fn upstream_oblate_and_prolate_surface_areas() {
    use std::f64::consts::{FRAC_PI_2, PI, TAU};
    for (a, c) in [(4.0_f64, 3.0_f64), (3.0_f64, 4.0_f64)] {
        let e = Ellipsoid::new(DVec3::new(a, a, c)).unwrap();
        let expected = if a > c {
            let eccentricity = (1.0 - c * c / (a * a)).sqrt();
            TAU * a * a
                + PI * c * c / eccentricity * ((1.0 + eccentricity) / (1.0 - eccentricity)).ln()
        } else {
            let eccentricity = (1.0 - a * a / (c * c)).sqrt();
            TAU * a * a + TAU * a * c / eccentricity * eccentricity.asin()
        };
        let actual = e.surface_area(-PI, -FRAC_PI_2, PI, FRAC_PI_2).unwrap();
        assert!(
            (actual - expected).abs() <= 1e-3 * expected.abs(),
            "{actual} vs {expected}"
        );
    }
}

#[test]
fn sphere_area_and_antimeridian() {
    use std::f64::consts::{FRAC_PI_2, PI};
    let e = Ellipsoid::UNIT_SPHERE;
    let full = e.surface_area(-PI, -FRAC_PI_2, PI, FRAC_PI_2).unwrap();
    assert!((full - 4.0 * PI).abs() < 1e-11);
    let west = 170_f64.to_radians();
    let east = -170_f64.to_radians();
    let south = -30_f64.to_radians();
    let north = 60_f64.to_radians();
    let expected = 20_f64.to_radians() * (north.sin() - south.sin());
    assert!((e.surface_area(west, south, east, north).unwrap() - expected).abs() < 1e-13);
    assert_eq!(e.surface_area(0.0, 0.0, 0.0, 1.0).unwrap(), 0.0);
    assert!(e.surface_area(0.0, 1.0, 1.0, -1.0).is_err());
    assert!(e.surface_area(0.0, f64::NAN, 1.0, 1.0).is_err());
    assert_eq!(Ellipsoid::MOON.radii(), DVec3::splat(1_737_400.0));
    assert_eq!(
        Ellipsoid::MARS.radii(),
        DVec3::new(3_396_190.0, 3_396_190.0, 3_376_200.0)
    );
}

#[test]
fn triaxial_area_matches_independent_midpoint_integral() {
    let (a, b, c) = (1.0_f64, 2.0_f64, 3.0_f64);
    let ellipsoid = Ellipsoid::new(DVec3::new(a, b, c)).unwrap();
    let (west, south, east, north) = (-1.1, -0.7, 2.1, 0.9);
    let actual = ellipsoid.surface_area(west, south, east, north).unwrap();
    let n = 600;
    let dlon = (east - west) / n as f64;
    let dlat = (north - south) / n as f64;
    let mut expected = 0.0;
    for i in 0..n {
        let lat = south + (i as f64 + 0.5) * dlat;
        for j in 0..n {
            let lon = west + (j as f64 + 0.5) * dlon;
            // Cross product of derivatives of the parametric ellipsoid.
            let d_lat = DVec3::new(
                -a * lat.sin() * lon.cos(),
                -b * lat.sin() * lon.sin(),
                c * lat.cos(),
            );
            let d_lon = DVec3::new(-a * lat.cos() * lon.sin(), b * lat.cos() * lon.cos(), 0.0);
            expected += d_lat.cross(d_lon).length() * dlat * dlon;
        }
    }
    assert!(
        (actual - expected).abs() / expected < 1e-5,
        "{actual} vs {expected}"
    );
}

#[test]
fn east_north_up_origin_poles_and_triaxial_surface() {
    let e = Ellipsoid::new(DVec3::new(1.0, 2.0, 3.0)).unwrap();
    for (origin, expected_east, expected_north, expected_up) in [
        (DVec3::ZERO, DVec3::Y, -DVec3::X, DVec3::Z),
        (3.0 * DVec3::Z, DVec3::Y, -DVec3::X, DVec3::Z),
        (-3.0 * DVec3::Z, DVec3::Y, DVec3::X, -DVec3::Z),
    ] {
        let frame = e.east_north_up_to_fixed_frame(origin).unwrap();
        assert_eq!(frame.x_axis.truncate(), expected_east);
        assert_eq!(frame.y_axis.truncate(), expected_north);
        assert_eq!(frame.z_axis.truncate(), expected_up);
        assert_eq!(frame.w_axis.truncate(), origin);
    }
    let origin = DVec3::new(1.0, 2.0, 3.0);
    let frame = Ellipsoid::WGS84
        .east_north_up_to_fixed_frame(origin)
        .unwrap();
    let east = frame.x_axis.truncate();
    let north = frame.y_axis.truncate();
    let up = frame.z_axis.truncate();
    assert!(
        east.dot(north).abs() < 1e-14 && east.dot(up).abs() < 1e-14 && north.dot(up).abs() < 1e-14
    );
    assert!(east.cross(north).distance(up) < 1e-14);
    assert!(up.distance(Ellipsoid::WGS84.geodetic_surface_normal(origin).unwrap()) < 1e-14);
    // Cesium defines east from origin.x/y even for a triaxial ellipsoid.
    let triaxial = e.east_north_up_to_fixed_frame(origin).unwrap();
    let triaxial_up = e.geodetic_surface_normal(origin).unwrap();
    let triaxial_east = DVec3::new(-origin.y, origin.x, 0.0).normalize();
    assert!(triaxial.x_axis.truncate().distance(triaxial_east) < 1e-14);
    assert!(
        triaxial
            .y_axis
            .truncate()
            .distance(triaxial_up.cross(triaxial_east))
            < 1e-14
    );
}

#[test]
fn upstream_from_cartesian3_equals_and_display() {
    let default_e = Ellipsoid::from_cartesian3(None).unwrap();
    assert_eq!(default_e.radii(), DVec3::ZERO);
    assert_eq!(default_e.radii_squared(), DVec3::ZERO);
    assert_eq!(default_e.radii_to_fourth(), DVec3::ZERO);
    assert_eq!(default_e.one_over_radii(), DVec3::ZERO);
    assert_eq!(default_e.one_over_radii_squared(), DVec3::ZERO);
    assert_eq!(default_e.minimum_radius(), 0.0);
    assert_eq!(default_e.maximum_radius(), 0.0);

    let e = Ellipsoid::from_cartesian3(Some(DVec3::new(1.0, 2.0, 3.0))).unwrap();
    assert_eq!(e.radii(), DVec3::new(1.0, 2.0, 3.0));

    let same = Ellipsoid::new(DVec3::new(1.0, 2.0, 3.0)).unwrap();
    let diff = Ellipsoid::new(DVec3::new(1.0, 2.0, 4.0)).unwrap();
    assert!(e.equals(Some(&same)));
    assert!(!e.equals(Some(&diff)));
    assert!(!e.equals(None));

    assert_eq!(format!("{e}"), "(1, 2, 3)");
}

#[test]
fn upstream_array_conversions_and_scaled_space_aliases() {
    let e = Ellipsoid::WGS84;
    let space_cartesian = DVec3::new(4582719.8827300891, -4582719.8827300882, 1725510.4250797231);
    let space_cartographic = Cartographic::degrees(-45.0, 15.0, 330000.0).unwrap();
    let surface_cartographic = Cartographic::degrees(-45.0, 15.0, 0.0).unwrap();
    let surface_cartesian = e.cartographic_to_cartesian(surface_cartographic).unwrap();

    let cartesians = e.cartographic_array_to_cartesian_array(&[space_cartographic, surface_cartographic]);
    assert_eq!(cartesians.len(), 2);
    assert!(cartesians[0].unwrap().distance(space_cartesian) < 1e-7);
    assert!(cartesians[1].unwrap().distance(surface_cartesian) < 1e-7);

    let cartographics = e.cartesian_array_to_cartographic_array(&[space_cartesian, surface_cartesian]);
    assert_eq!(cartographics.len(), 2);
    let c0 = cartographics[0].unwrap();
    let c1 = cartographics[1].unwrap();
    assert!((c0.longitude - space_cartographic.longitude).abs() < 1e-7);
    assert!((c0.latitude - space_cartographic.latitude).abs() < 1e-7);
    assert!((c0.height - space_cartographic.height).abs() < 1e-7);
    assert!((c1.longitude - surface_cartographic.longitude).abs() < 1e-7);
    assert!((c1.latitude - surface_cartographic.latitude).abs() < 1e-7);

    let norm = Ellipsoid::geocentric_surface_normal(DVec3::new(2.0, 0.0, 0.0));
    assert_eq!(norm, DVec3::X);

    let scaled_e = Ellipsoid::new(DVec3::new(2.0, 3.0, 4.0)).unwrap();
    let p = DVec3::new(4.0, 6.0, 8.0);
    assert_eq!(scaled_e.transform_position_to_scaled_space(p), DVec3::new(2.0, 2.0, 2.0));
    assert_eq!(scaled_e.transform_position_from_scaled_space(DVec3::new(2.0, 2.0, 2.0)), p);

    let rev_e = Ellipsoid::new(DVec3::new(4.0, 4.0, 3.0)).unwrap();
    assert_eq!(rev_e.squared_x_over_squared_z(), 16.0 / 9.0);
}

#[test]
fn upstream_pack_unpack_and_get_methods() {
    let e = Ellipsoid::WGS84;
    let mut packed = [0.0; 5];
    e.pack(&mut packed, 1).unwrap();
    assert_eq!(packed[1], e.radii().x);
    assert_eq!(packed[2], e.radii().y);
    assert_eq!(packed[3], e.radii().z);

    let unpacked = Ellipsoid::unpack(&packed, 1).unwrap();
    assert_eq!(unpacked, e);

    let p = e
        .cartographic_to_cartesian(Cartographic::degrees(35.23, 33.23, 0.0).unwrap())
        .unwrap();
    let z_isect = e.get_surface_normal_intersection_with_z_axis(p, None).unwrap();
    assert!(z_isect.is_some());

    let curv = e.get_local_curvature(p);
    assert!(curv.is_some());
}
