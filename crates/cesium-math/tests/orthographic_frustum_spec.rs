//! Cases adapted from CesiumJS orthographic frustum Specs (Apache-2.0).
use cesium_math::{OrthographicFrustum, OrthographicOffCenterFrustum, PlaneIntersection, Sphere};
use glam::{DVec2, DVec3, DVec4};

#[test]
fn upstream_projection_and_dimensions() {
    let f = OrthographicFrustum {
        width: 4.0,
        aspect_ratio: 2.0,
        near: 1.0,
        far: 11.0,
    };
    let off = f.off_center().unwrap();
    assert_eq!(
        (off.left, off.right, off.bottom, off.top),
        (-2.0, 2.0, -1.0, 1.0)
    );
    assert_eq!(
        f.pixel_dimensions(200, 100, 2.0).unwrap(),
        DVec2::splat(0.04)
    );
    let m = f.projection_matrix().unwrap();
    let near = m * DVec4::new(-2.0, -1.0, -1.0, 1.0);
    let far = m * DVec4::new(2.0, 1.0, -11.0, 1.0);
    assert!((near - DVec4::new(-1.0, -1.0, -1.0, 1.0)).length() < 1e-14);
    assert!((far - DVec4::new(1.0, 1.0, 1.0, 1.0)).length() < 1e-14);
    let mut data = [0.0; 6];
    f.pack(&mut data, 1).unwrap();
    assert_eq!(data, [0.0, 4.0, 2.0, 1.0, 11.0, 0.0]);
    assert_eq!(OrthographicFrustum::unpack(&data, 1).unwrap(), f);
    assert!(f.pack(&mut data, usize::MAX).is_err());
}

#[test]
fn off_center_culling_all_six_planes() {
    let f = OrthographicOffCenterFrustum {
        left: -1.0,
        right: 2.0,
        bottom: -2.0,
        top: 3.0,
        near: 1.0,
        far: 5.0,
    };
    let p = DVec3::new(10.0, 20.0, 30.0);
    let c = f.compute_culling_volume(p, -DVec3::Z, DVec3::Y).unwrap();
    let center = p + DVec3::new(0.0, 0.0, -3.0);
    assert_eq!(
        c.compute_visibility(&Sphere {
            center,
            radius: 0.1
        }),
        PlaneIntersection::Inside
    );
    for offset in [
        DVec3::new(-2.0, 0.0, -3.0),
        DVec3::new(3.0, 0.0, -3.0),
        DVec3::new(0.0, -3.0, -3.0),
        DVec3::new(0.0, 4.0, -3.0),
        DVec3::new(0.0, 0.0, 0.0),
        DVec3::new(0.0, 0.0, -6.0),
    ] {
        assert_eq!(
            c.compute_visibility(&Sphere {
                center: p + offset,
                radius: 0.1
            }),
            PlaneIntersection::Outside
        );
    }
    assert_eq!(
        c.compute_visibility(&Sphere {
            center: p + DVec3::new(2.0, 0.0, -3.0),
            radius: 0.5
        }),
        PlaneIntersection::Intersecting
    );
    assert!(f.compute_culling_volume(p, -DVec3::Z, DVec3::Z).is_err());
}

#[test]
fn rejects_invalid_frustum_and_buffer_sizes() {
    let mut f = OrthographicFrustum {
        width: 4.0,
        aspect_ratio: 2.0,
        near: 1.0,
        far: 11.0,
    };
    assert!(f.pixel_dimensions(0, 100, 1.0).is_err());
    assert!(f.pixel_dimensions(100, 100, 0.0).is_err());
    f.aspect_ratio = 0.0;
    assert!(f.projection_matrix().is_err());
    f.aspect_ratio = 2.0;
    f.near = 0.0;
    assert!(f.off_center().is_err());
}

#[test]
fn upstream_epsilon_uses_absolute_and_relative_tolerances() {
    let base = OrthographicFrustum {
        width: 2.0,
        aspect_ratio: 1.0,
        near: 1.0,
        far: 3.0,
    };
    let close = OrthographicFrustum {
        width: 2.01,
        aspect_ratio: 1.01,
        near: 1.01,
        far: 3.01,
    };
    assert!(base.equals_epsilon(close, 0.1, 0.1).unwrap());
    let different = OrthographicFrustum {
        aspect_ratio: 1.1,
        ..base
    };
    assert!(!base.equals_epsilon(different, 0.01, 0.01).unwrap());
    let a = base.off_center().unwrap();
    let mut b = a;
    b.near += 0.001;
    assert!(a.equals_epsilon(b, 0.002, 0.0));
    assert!(!a.equals_epsilon(b, 0.0001, 0.0));
    assert!(!a.equals_epsilon(a, f64::NAN, 0.0));
}

#[test]
fn equality_and_clone() {
    let f1 = OrthographicFrustum {
        width: 2.0,
        aspect_ratio: 1.0,
        near: 1.0,
        far: 3.0,
    };
    let f2 = f1;
    assert!(f1.equals(Some(&f2)));
    assert!(!f1.equals(Some(&OrthographicFrustum { width: 3.0, ..f1 })));
    assert!(!f1.equals(None));

    let off1 = f1.off_center().unwrap();
    let off2 = off1;
    assert!(off1.equals(Some(&off2)));
    assert!(!off1.equals(Some(&OrthographicOffCenterFrustum {
        left: -5.0,
        ..off1
    })));
    assert!(!off1.equals(None));
}
