//! Cases adapted from CesiumJS CullingVolumeSpec (Apache-2.0).
use cesium_math::{
    AxisAlignedBoundingBox, BoundingVolume, CullingVolume, PerspectiveFrustum,
    PlaneIntersection as I, Sphere,
};
use glam::DVec3;

fn frustum() -> CullingVolume {
    PerspectiveFrustum {
        fov: std::f64::consts::PI / 3.0,
        aspect_ratio: 1.0,
        near: 1.0,
        far: 2.0,
        x_offset: 0.0,
        y_offset: 0.0,
    }
    .compute_culling_volume(DVec3::ZERO, -DVec3::Z, DVec3::Y)
    .unwrap()
}

fn check<V: BoundingVolume>(c: &CullingVolume, volume: &V, expected: I) {
    assert_eq!(c.compute_visibility(volume), expected);
    let mask = c.compute_visibility_with_plane_mask(volume, CullingVolume::MASK_INDETERMINATE);
    match expected {
        I::Outside => assert_eq!(mask, CullingVolume::MASK_OUTSIDE),
        I::Inside => assert_eq!(mask, CullingVolume::MASK_INSIDE),
        I::Intersecting => {
            assert!(mask != CullingVolume::MASK_INSIDE && mask != CullingVolume::MASK_OUTSIDE)
        }
    }
    assert_eq!(c.compute_visibility_with_plane_mask(volume, mask), mask);
}

#[test]
fn frustum_classifies_spheres_and_boxes_against_all_sides() {
    let c = frustum();
    let b = |min: DVec3, max: DVec3| AxisAlignedBoundingBox::from_corners(min, max);
    check(
        &c,
        &b(DVec3::new(-0.5, 0.0, -1.75), DVec3::new(0.5, 0.0, -1.25)),
        I::Inside,
    );
    for (min, max) in [
        (DVec3::new(-0.5, 0.0, -2.5), DVec3::new(0.5, 0.0, -1.5)),
        (DVec3::new(-0.5, 0.0, -1.5), DVec3::new(0.5, 0.0, -0.5)),
        (DVec3::new(-1.5, 0.0, -1.5), DVec3::new(0.0, 0.0, -1.25)),
        (DVec3::new(0.0, 0.0, -1.5), DVec3::new(1.5, 0.0, -1.25)),
        (DVec3::new(-0.5, 0.0, -1.75), DVec3::new(0.5, 2.0, -1.25)),
        (DVec3::new(-0.5, -2.0, -1.5), DVec3::new(0.5, 0.0, -1.25)),
    ] {
        check(&c, &b(min, max), I::Intersecting);
    }
    let sphere = |center, radius| Sphere { center, radius };
    check(&c, &sphere(DVec3::new(0.0, 0.0, -1.5), 0.25), I::Inside);
    for center in [
        DVec3::new(0.0, 0.0, -2.5),
        DVec3::new(0.0, 0.0, -0.2),
        DVec3::new(-4.0, 0.0, -1.5),
        DVec3::new(4.0, 0.0, -1.5),
        DVec3::new(0.0, 4.0, -1.5),
        DVec3::new(0.0, -4.0, -1.5),
    ] {
        check(&c, &sphere(center, 0.1), I::Outside);
        check(
            &c,
            &b(center - DVec3::splat(0.1), center + DVec3::splat(0.1)),
            I::Outside,
        );
    }
}

#[test]
fn bounding_sphere_cube_and_parent_masks() {
    let center = DVec3::new(1000.0, 2000.0, 3000.0);
    let c = CullingVolume::from_bounding_sphere(Sphere {
        center,
        radius: 100.0,
    })
    .unwrap();
    assert_eq!(c.planes.len(), 6);
    check(
        &c,
        &Sphere {
            center,
            radius: 50.0,
        },
        I::Inside,
    );
    for axis in [
        DVec3::X,
        -DVec3::X,
        DVec3::Y,
        -DVec3::Y,
        DVec3::Z,
        -DVec3::Z,
    ] {
        check(
            &c,
            &Sphere {
                center: center + axis * 150.0,
                radius: 50.0,
            },
            I::Intersecting,
        );
        check(
            &c,
            &Sphere {
                center: center + axis * 200.0,
                radius: 50.0,
            },
            I::Outside,
        );
    }
    let parent = Sphere {
        center: center + DVec3::X * 80.0,
        radius: 40.0,
    };
    let child = Sphere {
        center: center + DVec3::X * 80.0,
        radius: 10.0,
    };
    let mask = c.compute_visibility_with_plane_mask(&parent, CullingVolume::MASK_INDETERMINATE);
    assert_ne!(mask, 0);
    assert_eq!(
        c.compute_visibility_with_plane_mask(&child, mask),
        CullingVolume::MASK_INSIDE
    );
}
