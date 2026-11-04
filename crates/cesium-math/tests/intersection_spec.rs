//! Reference scenarios from CesiumJS `IntersectionTestsSpec.js` (Apache-2.0).
use cesium_math::{
    line_segment_plane, line_segment_sphere, line_segment_triangle, ray_axis_aligned_box,
    ray_plane, ray_sphere, ray_triangle, ray_triangle_parametric, triangle_plane_intersection,
    RayInterval, Sphere,
};
use glam::DVec3;

#[test]
fn ray_plane_upstream_hit_behind_and_parallel() {
    let origin = 2.0 * DVec3::X;
    assert_eq!(ray_plane(origin, -DVec3::X, DVec3::X, -1.0), Some(DVec3::X));
    assert_eq!(ray_plane(origin, DVec3::X, DVec3::X, -1.0), None);
    assert_eq!(ray_plane(origin, DVec3::Y, DVec3::X, -1.0), None);
    assert_eq!(
        ray_plane(DVec3::X, DVec3::X, DVec3::X, -1.0),
        Some(DVec3::X)
    );
}

#[test]
fn segment_plane_upstream_hit_misses_and_endpoints() {
    let p = DVec3::new(1.0, 1.0, 0.0);
    let q = DVec3::new(1.0, 3.0, 0.0);
    assert_eq!(
        line_segment_plane(p, q, DVec3::Y, -2.0),
        Some(DVec3::new(1.0, 2.0, 0.0))
    );
    assert_eq!(
        line_segment_plane(-2.0 * DVec3::X, -5.0 * DVec3::X, DVec3::X, 0.0),
        None
    );
    assert_eq!(
        line_segment_plane(5.0 * DVec3::X, 2.0 * DVec3::X, DVec3::X, 0.0),
        None
    );
    assert_eq!(line_segment_plane(-DVec3::Y, DVec3::Y, DVec3::X, 0.0), None);
    assert_eq!(
        line_segment_plane(DVec3::ZERO, DVec3::X, DVec3::X, 0.0),
        Some(DVec3::ZERO)
    );
    assert_eq!(
        line_segment_plane(DVec3::X, DVec3::ZERO, DVec3::X, 0.0),
        Some(DVec3::ZERO)
    );
}

#[test]
fn axis_aligned_box_parallel_inside_and_behind() {
    let min = DVec3::splat(-1.0);
    let max = DVec3::ONE;
    assert_eq!(
        ray_axis_aligned_box(2.0 * DVec3::X, -DVec3::X, min, max),
        Some(RayInterval {
            start: 1.0,
            stop: 3.0
        })
    );
    assert_eq!(
        ray_axis_aligned_box(2.0 * DVec3::X, DVec3::X, min, max),
        Some(RayInterval {
            start: -3.0,
            stop: -1.0
        })
    );
    assert_eq!(
        ray_axis_aligned_box(DVec3::ZERO, DVec3::Y, min, max),
        Some(RayInterval {
            start: -1.0,
            stop: 1.0
        })
    );
    assert_eq!(
        ray_axis_aligned_box(DVec3::new(2.0, 0.0, 0.0), DVec3::Y, min, max),
        None
    );
    assert_eq!(
        ray_axis_aligned_box(DVec3::new(1.0, 0.0, 0.0), DVec3::Y, min, max),
        Some(RayInterval {
            start: -1.0,
            stop: 1.0
        })
    );
    assert_eq!(
        ray_axis_aligned_box(DVec3::ZERO, DVec3::ZERO, min, max),
        None
    );
    assert_eq!(ray_axis_aligned_box(DVec3::ZERO, DVec3::X, max, min), None);
}

#[test]
fn triangle_plane_all_six_vertex_classifications() {
    let tri = [
        DVec3::new(0.0, 0.0, 0.0),
        DVec3::new(2.0, 0.0, 0.0),
        DVec3::new(0.0, 2.0, 0.0),
    ];
    for singled in 0..3 {
        for one_behind in [true, false] {
            let mut points = tri;
            for (i, p) in points.iter_mut().enumerate() {
                p.z = if (i == singled) == one_behind {
                    -1.0
                } else {
                    1.0
                };
            }
            let split = triangle_plane_intersection(points, DVec3::Z, 0.0).unwrap();
            assert_eq!(&split.positions[..3], &points);
            assert!(split.positions[3].z.abs() < 1e-14 && split.positions[4].z.abs() < 1e-14);
            let expected_start = if one_behind {
                singled
            } else {
                (singled + 1) % 3
            };
            assert_eq!(split.indices[0] as usize, expected_start);
            let mut area = 0.0;
            for indices in split.indices.as_chunks::<3>().0 {
                let a = split.positions[indices[0] as usize];
                let b = split.positions[indices[1] as usize];
                let c = split.positions[indices[2] as usize];
                assert!(a.z.min(b.z).min(c.z) >= 0.0 || a.z.max(b.z).max(c.z) <= 0.0);
                area += (b - a).cross(c - a).length() * 0.5;
            }
            let original_area = (points[1] - points[0])
                .cross(points[2] - points[0])
                .length()
                * 0.5;
            assert!((area - original_area).abs() < 1e-12);
        }
    }
    assert_eq!(triangle_plane_intersection(tri, DVec3::Z, -1.0), None);
    assert_eq!(triangle_plane_intersection(tri, DVec3::Z, 1.0), None);
}

fn triangle() -> [DVec3; 3] {
    [-DVec3::X, DVec3::X, DVec3::Y]
}

#[test]
fn ray_triangle_faces_edges_parallel_and_behind() {
    let tri = triangle();
    assert_eq!(
        ray_triangle(DVec3::Z, -DVec3::Z, tri, true),
        Some(DVec3::ZERO)
    );
    assert_eq!(
        ray_triangle(-DVec3::Z, DVec3::Z, tri, false),
        Some(DVec3::ZERO)
    );
    assert_eq!(ray_triangle(-DVec3::Z, DVec3::Z, tri, true), None);
    for origin in [
        DVec3::new(0.0, -1.0, 1.0),
        DVec3::new(1.0, 1.0, 1.0),
        DVec3::new(-1.0, 1.0, 1.0),
    ] {
        assert_eq!(ray_triangle(origin, -DVec3::Z, tri, false), None);
    }
    assert_eq!(ray_triangle(DVec3::Z, DVec3::X, tri, false), None);
    assert_eq!(ray_triangle(DVec3::Z, DVec3::Z, tri, false), None);
    assert_eq!(
        ray_triangle_parametric(DVec3::Z, DVec3::Z, tri, false),
        Some(-1.0)
    );
    assert_eq!(
        ray_triangle(DVec3::X + DVec3::Z, -DVec3::Z, tri, false),
        Some(DVec3::X)
    );
}

#[test]
fn segment_triangle_respects_both_endpoints_and_culling() {
    let tri = triangle();
    assert_eq!(
        line_segment_triangle(DVec3::Z, -DVec3::Z, tri, true),
        Some(DVec3::ZERO)
    );
    assert_eq!(
        line_segment_triangle(-DVec3::Z, DVec3::Z, tri, false),
        Some(DVec3::ZERO)
    );
    assert_eq!(line_segment_triangle(-DVec3::Z, DVec3::Z, tri, true), None);
    assert_eq!(
        line_segment_triangle(DVec3::Z, 2.0 * DVec3::Z, tri, false),
        None
    );
    assert_eq!(
        line_segment_triangle(DVec3::Z, 0.5 * DVec3::Z, tri, false),
        None
    );
    assert_eq!(
        line_segment_triangle(DVec3::Z, DVec3::ZERO, tri, false),
        Some(DVec3::ZERO)
    );
    assert_eq!(
        line_segment_triangle(DVec3::ZERO, DVec3::ZERO, tri, false),
        None
    );
}

#[test]
fn ray_sphere_upstream_intervals_and_tangents() {
    let sphere = Sphere {
        center: DVec3::ZERO,
        radius: 1.0,
    };
    for axis in [DVec3::X, DVec3::Y, DVec3::Z] {
        for sign in [-1.0, 1.0] {
            assert_eq!(
                ray_sphere(2.0 * sign * axis, -sign * axis, sphere),
                Some(RayInterval {
                    start: 1.0,
                    stop: 3.0
                })
            );
            assert_eq!(ray_sphere(2.0 * sign * axis, sign * axis, sphere), None);
        }
    }
    assert_eq!(
        ray_sphere(DVec3::new(1.0, 1.0, 0.0), -DVec3::X, sphere),
        Some(RayInterval {
            start: 1.0,
            stop: 1.0
        })
    );
    assert_eq!(ray_sphere(DVec3::X, DVec3::Z, sphere), None);
    assert_eq!(
        ray_sphere(0.5 * DVec3::X, DVec3::X, sphere),
        Some(RayInterval {
            start: 0.0,
            stop: 0.5
        })
    );
    assert_eq!(
        ray_sphere(0.5 * DVec3::X, -DVec3::X, sphere),
        Some(RayInterval {
            start: 0.0,
            stop: 1.5
        })
    );
}

#[test]
fn segment_sphere_clips_intersection_to_segment() {
    let sphere = Sphere {
        center: DVec3::ZERO,
        radius: 1.0,
    };
    assert_eq!(
        line_segment_sphere(-2.0 * DVec3::X, 2.0 * DVec3::X, sphere),
        Some(RayInterval {
            start: 1.0,
            stop: 3.0
        })
    );
    assert_eq!(
        line_segment_sphere(DVec3::ZERO, 0.5 * DVec3::X, sphere),
        Some(RayInterval {
            start: 0.0,
            stop: 0.5
        })
    );
    assert_eq!(
        line_segment_sphere(2.0 * DVec3::X, 3.0 * DVec3::X, sphere),
        None
    );
    assert_eq!(line_segment_sphere(DVec3::ZERO, DVec3::ZERO, sphere), None);
}

#[test]
fn intersection_parameter_scaling_and_invalid_inputs() {
    let sphere = Sphere {
        center: DVec3::new(5.0, 0.0, 0.0),
        radius: 1.0,
    };
    assert_eq!(
        ray_sphere(DVec3::ZERO, 2.0 * DVec3::X, sphere),
        Some(RayInterval {
            start: 2.0,
            stop: 3.0
        })
    );
    assert_eq!(ray_sphere(DVec3::ZERO, DVec3::ZERO, sphere), None);
    assert_eq!(
        ray_sphere(
            DVec3::ZERO,
            DVec3::X,
            Sphere {
                radius: -1.0,
                ..sphere
            }
        ),
        None
    );
    assert_eq!(
        ray_plane(DVec3::ZERO, DVec3::splat(f64::NAN), DVec3::X, 0.0),
        None
    );
    assert_eq!(
        ray_triangle(DVec3::ZERO, DVec3::Z, [DVec3::ZERO; 3], false),
        None
    );
}
