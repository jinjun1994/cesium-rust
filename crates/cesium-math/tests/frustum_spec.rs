//! Perspective frustum cases adapted from CesiumJS (Apache-2.0).
use cesium_math::{PerspectiveFrustum, PerspectiveOffCenterFrustum, PlaneIntersection, Sphere};
use glam::{DVec3, DVec4};

fn frustum() -> PerspectiveFrustum {
    PerspectiveFrustum {
        fov: std::f64::consts::FRAC_PI_2,
        aspect_ratio: 2.0,
        near: 1.0,
        far: 10.0,
        x_offset: 0.0,
        y_offset: 0.0,
    }
}

#[test]
fn aspect_ratio_offsets_and_pixel_dimensions() {
    let mut f = frustum();
    assert!((f.fovy().unwrap() - 2.0 * 0.5_f64.atan()).abs() < 1e-15);
    assert!((f.sse_denominator().unwrap() - 1.0).abs() < 1e-15);
    let off = f.off_center().unwrap();
    assert!((off.right - 1.0).abs() < 1e-15 && (off.top - 0.5).abs() < 1e-15);
    let pixels = f.pixel_dimensions(200, 100, 5.0, 2.0).unwrap();
    assert!((pixels.x - 0.1).abs() < 1e-15 && (pixels.y - 0.1).abs() < 1e-15);
    f.x_offset = 0.2;
    f.y_offset = -0.1;
    let offset = f.off_center().unwrap();
    assert!((offset.left - (off.left + 0.2)).abs() < 1e-15);
    assert!((offset.top - (off.top - 0.1)).abs() < 1e-15);
    f.aspect_ratio = 0.5;
    assert_eq!(f.fovy().unwrap(), f.fov);
    assert!(f.pixel_dimensions(0, 1, 1.0, 1.0).is_err());
    assert!(f.pixel_dimensions(1, 1, 1.0, 0.0).is_err());
}

#[test]
fn matrices_map_near_and_far_to_opengl_depth() {
    let f = PerspectiveOffCenterFrustum {
        left: -0.5,
        right: 1.0,
        bottom: -0.25,
        top: 0.75,
        near: 1.0,
        far: 10.0,
    };
    let m = f.projection_matrix().unwrap();
    for (p, expected) in [
        (
            DVec4::new(f.left, f.bottom, -f.near, 1.0),
            DVec3::new(-1.0, -1.0, -1.0),
        ),
        (
            DVec4::new(f.right, f.top, -f.near, 1.0),
            DVec3::new(1.0, 1.0, -1.0),
        ),
        (
            DVec4::new(0.0, 0.0, -f.far, 1.0),
            DVec3::new(-1.0 / 3.0, -0.5, 1.0),
        ),
    ] {
        let q = m * p;
        assert!((q.truncate() / q.w - expected).length() < 1e-14);
    }
    let infinite = f.infinite_projection_matrix().unwrap();
    let q = infinite * DVec4::new(0.0, 0.0, -1e12, 1.0);
    assert!((q.z / q.w - 1.0).abs() < 1e-10);
}

#[test]
fn translated_rotated_culling_and_invalid_parameters() {
    let f = frustum();
    let position = DVec3::new(10.0, 20.0, 30.0);
    let c = f
        .compute_culling_volume(position, DVec3::X, DVec3::Z)
        .unwrap();
    assert_eq!(
        c.compute_visibility(&Sphere {
            center: position + DVec3::X * 5.0,
            radius: 0.1
        }),
        PlaneIntersection::Inside
    );
    assert_eq!(
        c.compute_visibility(&Sphere {
            center: position - DVec3::X * 5.0,
            radius: 0.1
        }),
        PlaneIntersection::Outside
    );
    assert!(f
        .compute_culling_volume(position, DVec3::X, DVec3::X)
        .is_err());
    let mut invalid = f;
    invalid.near = 0.0;
    assert!(invalid.projection_matrix().is_err());
    invalid = f;
    invalid.fov = std::f64::consts::PI;
    assert!(invalid.off_center().is_err());
}

#[test]
fn upstream_pack_offsets_and_epsilon_comparison() {
    let f = PerspectiveFrustum {
        fov: 1.0,
        aspect_ratio: 2.0,
        near: 3.0,
        far: 4.0,
        x_offset: 5.0,
        y_offset: 6.0,
    };
    let mut values = [9.0; 8];
    f.pack(&mut values, 1).unwrap();
    assert_eq!(values, [9.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 9.0]);
    assert_eq!(PerspectiveFrustum::unpack(&values, 1).unwrap(), f);
    assert!(f.pack(&mut values, usize::MAX).is_err());
    assert!(PerspectiveFrustum::unpack(&values, 3).is_err());
    let base = frustum();
    let mut close = base;
    close.fov += 0.01;
    close.near += 0.01;
    close.far += 0.01;
    close.aspect_ratio += 0.01;
    assert!(base.equals_epsilon(close, 0.1, 0.1).unwrap());
    close = base;
    close.aspect_ratio += 0.1;
    assert!(!base.equals_epsilon(close, 0.01, 0.01).unwrap());
    close = base;
    close.x_offset = 0.01;
    assert!(!base.equals_epsilon(close, 1e-4, 1e-4).unwrap());
    let a = base.off_center().unwrap();
    let mut b = a;
    b.left += 1e-6;
    assert!(a.equals_epsilon(b, 0.0, 1e-5));
    assert!(!a.equals_epsilon(b, 0.0, 1e-7));
    assert!(!a.equals_epsilon(a, -1.0, 1e-5));
}
