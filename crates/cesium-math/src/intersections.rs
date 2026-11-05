//! Intersection operations adapted from CesiumJS `IntersectionTests.js` (Apache-2.0).
use crate::{
    quadratic_real_roots, quartic_real_roots, ray_ellipsoid_interval, RayInterval, Sphere,
};
use glam::{DMat3, DVec3};

fn add_cancellation(a: f64, b: f64, tolerance: f64) -> f64 {
    let sum = a + b;
    if a.signum() != b.signum() && (sum / a.abs().max(b.abs())).abs() < tolerance {
        0.0
    } else {
        sum
    }
}

fn cesium_sign(value: f64) -> f64 {
    if value == 0.0 {
        0.0
    } else {
        value.signum()
    }
}

// Cesium divides each coordinate by the magnitude. Multiplying by the
// reciprocal changes rounding enough to affect the ill-conditioned quartic.
fn normalize_div(value: DVec3) -> Option<DVec3> {
    let length = value.length();
    if length == 0.0 || !length.is_finite() {
        return None;
    }
    let result = value / length;
    result.is_finite().then_some(result)
}

// Solves Cesium's constrained quadratic vector equation for x=0, w=1.
fn quadratic_vector_solutions(a: DMat3, b: DVec3) -> Vec<DVec3> {
    let cols = a.to_cols_array_2d();
    let l2 = cols[1][1] - cols[2][2];
    let l1 = b.y;
    let l0 = cols[2][2];
    let r1 = add_cancellation(cols[2][1], cols[1][2], 1e-15);
    let r0 = b.z;
    let mut solutions = Vec::new();
    if r0 == 0.0 && r1 == 0.0 {
        for cosine in quadratic_real_roots(l2, l1, l0) {
            let sine = (1.0 - cosine * cosine).max(0.0).sqrt();
            solutions.push(DVec3::new(0.0, cosine, -sine));
            solutions.push(DVec3::new(0.0, cosine, sine));
        }
        return solutions;
    }
    let (r0sq, r1sq, l2sq, r0r1) = (r0 * r0, r1 * r1, l2 * l2, r0 * r1);
    let c4 = l2sq + r1sq;
    let c3 = 2.0 * (l1 * l2 + r0r1);
    let c2 = 2.0 * l0 * l2 + l1 * l1 - r1sq + r0sq;
    let c1 = 2.0 * (l0 * l1 - r0r1);
    let c0 = l0 * l0 - r0sq;
    if c4 == 0.0 && c3 == 0.0 && c2 == 0.0 && c1 == 0.0 {
        return solutions;
    }
    let cosines = quartic_real_roots(c4, c3, c2, c1, c0);
    let mut i = 0;
    while i < cosines.len() {
        let cosine = cosines[i];
        let sine = (1.0 - cosine * cosine).max(0.0).sqrt();
        let left = if cesium_sign(l2) == cesium_sign(l0) {
            add_cancellation(l2 * cosine * cosine + l0, l1 * cosine, 1e-12)
        } else if cesium_sign(l0) == cesium_sign(l1 * cosine) {
            add_cancellation(l2 * cosine * cosine, l1 * cosine + l0, 1e-12)
        } else {
            add_cancellation(l2 * cosine * cosine + l1 * cosine, l0, 1e-12)
        };
        let right = add_cancellation(r1 * cosine, r0, 1e-15);
        let product = left * right;
        if product < 0.0 {
            solutions.push(DVec3::new(0.0, cosine, sine));
        } else if product > 0.0 {
            solutions.push(DVec3::new(0.0, cosine, -sine));
        } else if sine != 0.0 {
            solutions.push(DVec3::new(0.0, cosine, -sine));
            solutions.push(DVec3::new(0.0, cosine, sine));
            i += 1;
        } else {
            solutions.push(DVec3::new(0.0, cosine, sine));
        }
        i += 1;
    }
    solutions
}

/// Finds Cesium's minimum geodetic altitude location along a ray for a triaxial ellipsoid.
/// The direction is normalized internally, matching the Cesium `Ray` constructor.
/// Returns `None` for invalid input or when no candidate solution is available.
pub fn grazing_altitude_location(origin: DVec3, direction: DVec3, radii: DVec3) -> Option<DVec3> {
    if !origin.is_finite()
        || !direction.is_finite()
        || !radii.is_finite()
        || radii.min_element() <= 0.0
    {
        return None;
    }
    let direction = normalize_div(direction)?;
    if origin != DVec3::ZERO {
        let normal = normalize_div(origin * (DVec3::ONE / (radii * radii)))?;
        if direction.dot(normal) >= 0.0 {
            return Some(origin);
        }
    }
    let intersects = ray_ellipsoid_interval(origin, direction, radii).is_some();
    let first = normalize_div(direction * (DVec3::ONE / radii))?;
    let abs = first.abs();
    let reference = if abs.x <= abs.y && abs.x <= abs.z {
        DVec3::X
    } else if abs.y <= abs.z {
        DVec3::Y
    } else {
        DVec3::Z
    };
    let second = normalize_div(reference.cross(first))?;
    let third = normalize_div(first.cross(second))?;
    let basis = DMat3::from_cols(first, second, third);
    let inverse_radii = DMat3::from_diagonal(DVec3::ONE / radii);
    let radii_matrix = DMat3::from_diagonal(radii);
    let cross = DMat3::from_cols(
        DVec3::new(0.0, -direction.z, direction.y),
        DVec3::new(direction.z, 0.0, -direction.x),
        DVec3::new(-direction.y, direction.x, 0.0),
    );
    let temp = basis.transpose() * inverse_radii * cross;
    let a = temp * radii_matrix * basis;
    let b = -(temp * origin);
    let solutions = quadratic_vector_solutions(a, b);
    let mut best: Option<(DVec3, f64)> = None;
    for solution in solutions {
        let surface = radii_matrix * basis * solution;
        let Some(v) = normalize_div(surface - origin) else {
            continue;
        };
        let value = v.dot(direction);
        if value.is_finite() && best.is_none_or(|(_, current)| value > current) {
            best = Some((surface, value));
        }
    }
    let (surface, value) = best?;
    let normal = normalize_div(surface * (DVec3::ONE / (radii * radii)))?;
    let value = value.clamp(0.0, 1.0);
    let altitude = (surface - origin).length()
        * (1.0 - value * value).sqrt()
        * if intersects { -1.0 } else { 1.0 };
    // Reconstruct the ellipsoid surface point from its geodetic normal as
    // Cesium's cartographicToCartesian does, before adding the altitude.
    let scaled_normal = radii * radii * normal;
    let surface_from_normal = scaled_normal / normal.dot(scaled_normal).sqrt();
    let point = surface_from_normal + normal * altitude;
    point.is_finite().then_some(point)
}

/// Intersects a finite segment with `normal.dot(point) + distance = 0`.
pub fn line_segment_plane(start: DVec3, end: DVec3, normal: DVec3, distance: f64) -> Option<DVec3> {
    if !start.is_finite() || !end.is_finite() || !normal.is_finite() || !distance.is_finite() {
        return None;
    }
    let delta = end - start;
    let denominator = normal.dot(delta);
    if denominator.abs() < 1e-6 {
        return None;
    }
    let t = -(distance + normal.dot(start)) / denominator;
    let point = start + t * delta;
    ((0.0..=1.0).contains(&t) && point.is_finite()).then_some(point)
}

/// Intersection with the supporting line of an axis-aligned box. Like Cesium's
/// `rayAxisAlignedBoundingBox`, the returned interval may be behind the origin.
/// Parallel axes are handled explicitly, including rays on a box face.
pub fn ray_axis_aligned_box(
    origin: DVec3,
    direction: DVec3,
    minimum: DVec3,
    maximum: DVec3,
) -> Option<RayInterval> {
    if !origin.is_finite()
        || !direction.is_finite()
        || !minimum.is_finite()
        || !maximum.is_finite()
        || minimum.cmpgt(maximum).any()
        || direction == DVec3::ZERO
    {
        return None;
    }
    let mut near = f64::NEG_INFINITY;
    let mut far = f64::INFINITY;
    for axis in 0..3 {
        if direction[axis] == 0.0 {
            if origin[axis] < minimum[axis] || origin[axis] > maximum[axis] {
                return None;
            }
            continue;
        }
        let a = (minimum[axis] - origin[axis]) / direction[axis];
        let b = (maximum[axis] - origin[axis]) / direction[axis];
        near = near.max(a.min(b));
        far = far.min(a.max(b));
        if near > far {
            return None;
        }
    }
    Some(RayInterval {
        start: near,
        stop: far,
    })
}

/// Three indexed triangles, ordered with the negative side first as in Cesium.
#[derive(Clone, Debug, PartialEq)]
pub struct TrianglePlaneIntersection {
    pub positions: [DVec3; 5],
    pub indices: [u32; 9],
}

pub fn triangle_plane_intersection(
    vertices: [DVec3; 3],
    normal: DVec3,
    distance: f64,
) -> Option<TrianglePlaneIntersection> {
    if vertices.iter().any(|p| !p.is_finite()) || !normal.is_finite() || !distance.is_finite() {
        return None;
    }
    let behind = vertices.map(|p| normal.dot(p) + distance < 0.0);
    let count = behind.iter().filter(|&&b| b).count();
    if count != 1 && count != 2 {
        return None;
    }
    let single = behind.iter().position(|&b| b == (count == 1))?;
    let next = (single + 1) % 3;
    let last = (single + 2) % 3;
    let (u1, u2, indices) = if count == 1 {
        (
            line_segment_plane(vertices[single], vertices[next], normal, distance)?,
            line_segment_plane(vertices[single], vertices[last], normal, distance)?,
            [
                single as u32,
                3,
                4,
                next as u32,
                last as u32,
                4,
                next as u32,
                4,
                3,
            ],
        )
    } else {
        (
            line_segment_plane(vertices[next], vertices[single], normal, distance)?,
            line_segment_plane(vertices[last], vertices[single], normal, distance)?,
            [
                next as u32,
                last as u32,
                4,
                next as u32,
                4,
                3,
                single as u32,
                3,
                4,
            ],
        )
    };
    Some(TrianglePlaneIntersection {
        positions: [vertices[0], vertices[1], vertices[2], u1, u2],
        indices,
    })
}

/// Intersects a ray with the plane `normal.dot(point) + distance = 0`.
/// Direction is not implicitly normalized; the returned point is independent of its magnitude.
pub fn ray_plane(origin: DVec3, direction: DVec3, normal: DVec3, distance: f64) -> Option<DVec3> {
    if !origin.is_finite() || !direction.is_finite() || !normal.is_finite() || !distance.is_finite()
    {
        return None;
    }
    let denominator = normal.dot(direction);
    if denominator.abs() < 1e-15 {
        return None;
    }
    let t = (-distance - normal.dot(origin)) / denominator;
    let point = origin + direction * t;
    (t >= 0.0 && point.is_finite()).then_some(point)
}

/// Intersects the supporting line with a triangle; a negative result is behind the ray origin.
pub fn ray_triangle_parametric(
    origin: DVec3,
    direction: DVec3,
    vertices: [DVec3; 3],
    cull_back_faces: bool,
) -> Option<f64> {
    if !origin.is_finite() || !direction.is_finite() || vertices.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let edge0 = vertices[1] - vertices[0];
    let edge1 = vertices[2] - vertices[0];
    let p = direction.cross(edge1);
    let det = edge0.dot(p);
    if if cull_back_faces {
        det < 1e-6
    } else {
        det.abs() < 1e-6
    } {
        return None;
    }
    let inv = 1.0 / det;
    let from_vertex = origin - vertices[0];
    let u = from_vertex.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = from_vertex.cross(edge0);
    let v = direction.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = edge1.dot(q) * inv;
    t.is_finite().then_some(t)
}

pub fn ray_triangle(
    origin: DVec3,
    direction: DVec3,
    vertices: [DVec3; 3],
    cull_back_faces: bool,
) -> Option<DVec3> {
    let t = ray_triangle_parametric(origin, direction, vertices, cull_back_faces)?;
    (t >= 0.0).then_some(origin + direction * t)
}

pub fn line_segment_triangle(
    start: DVec3,
    end: DVec3,
    vertices: [DVec3; 3],
    cull_back_faces: bool,
) -> Option<DVec3> {
    let delta = end - start;
    let length = delta.length();
    if !length.is_finite() || length == 0.0 {
        return None;
    }
    let direction = delta / length;
    let t = ray_triangle_parametric(start, direction, vertices, cull_back_faces)?;
    (0.0..=length).contains(&t).then_some(start + direction * t)
}

/// Returns the portion of the ray inside the sphere in parameters of `direction`.
pub fn ray_sphere(origin: DVec3, direction: DVec3, sphere: Sphere) -> Option<RayInterval> {
    if !origin.is_finite()
        || !direction.is_finite()
        || !sphere.center.is_finite()
        || !sphere.radius.is_finite()
        || sphere.radius < 0.0
    {
        return None;
    }
    let diff = origin - sphere.center;
    let a = direction.length_squared();
    if a == 0.0 || !a.is_finite() {
        return None;
    }
    let b = 2.0 * direction.dot(diff);
    let c = diff.length_squared() - sphere.radius * sphere.radius;
    let disc = b * b - 4.0 * a * c;
    if !disc.is_finite() || disc < 0.0 {
        return None;
    }
    let (near, far) = if disc == 0.0 {
        let t = -b / (2.0 * a);
        if t == 0.0 {
            return None;
        }
        (t, t)
    } else {
        let q = -0.5 * (b + disc.sqrt().copysign(b));
        let t0 = q / a;
        let t1 = c / q;
        (t0.min(t1), t0.max(t1))
    };
    if far < 0.0 || !near.is_finite() || !far.is_finite() {
        return None;
    }
    Some(RayInterval {
        start: near.max(0.0),
        stop: far,
    })
}

/// Returns the intersecting portion in distance units measured from `start`.
pub fn line_segment_sphere(start: DVec3, end: DVec3, sphere: Sphere) -> Option<RayInterval> {
    let delta = end - start;
    let length = delta.length();
    if !length.is_finite() || length == 0.0 {
        return None;
    }
    let hit = ray_sphere(start, delta / length, sphere)?;
    if hit.start > length {
        return None;
    }
    Some(RayInterval {
        start: hit.start,
        stop: hit.stop.min(length),
    })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IntersectionTests;

impl IntersectionTests {
    pub fn ray_plane(ray: &crate::Ray, plane: &crate::Plane) -> Option<DVec3> {
        ray_plane(ray.origin, ray.direction, plane.normal, plane.distance)
    }

    pub fn ray_triangle_parametric(
        ray: &crate::Ray,
        p0: DVec3,
        p1: DVec3,
        p2: DVec3,
        cull_back_faces: bool,
    ) -> Option<f64> {
        ray_triangle_parametric(ray.origin, ray.direction, [p0, p1, p2], cull_back_faces)
    }

    pub fn ray_triangle(
        ray: &crate::Ray,
        p0: DVec3,
        p1: DVec3,
        p2: DVec3,
        cull_back_faces: bool,
    ) -> Option<DVec3> {
        ray_triangle(ray.origin, ray.direction, [p0, p1, p2], cull_back_faces)
    }

    pub fn line_segment_triangle(
        v0: DVec3,
        v1: DVec3,
        p0: DVec3,
        p1: DVec3,
        p2: DVec3,
        cull_back_faces: bool,
    ) -> Option<DVec3> {
        line_segment_triangle(v0, v1, [p0, p1, p2], cull_back_faces)
    }

    pub fn ray_sphere(ray: &crate::Ray, sphere: &crate::Sphere) -> Option<RayInterval> {
        ray_sphere(ray.origin, ray.direction, *sphere)
    }

    pub fn line_segment_sphere(
        v0: DVec3,
        v1: DVec3,
        sphere: &crate::Sphere,
    ) -> Option<RayInterval> {
        line_segment_sphere(v0, v1, *sphere)
    }

    pub fn ray_ellipsoid(ray: &crate::Ray, ellipsoid: &crate::Ellipsoid) -> Option<RayInterval> {
        crate::ray_ellipsoid_interval(ray.origin, ray.direction, ellipsoid.radii())
    }

    pub fn grazing_altitude_location(
        ray: &crate::Ray,
        ellipsoid: &crate::Ellipsoid,
    ) -> Option<DVec3> {
        grazing_altitude_location(ray.origin, ray.direction, ellipsoid.radii())
    }

    pub fn line_segment_plane(v0: DVec3, v1: DVec3, plane: &crate::Plane) -> Option<DVec3> {
        line_segment_plane(v0, v1, plane.normal, plane.distance)
    }

    pub fn triangle_plane_intersection(
        p0: DVec3,
        p1: DVec3,
        p2: DVec3,
        plane: &crate::Plane,
    ) -> Option<TrianglePlaneIntersection> {
        triangle_plane_intersection([p0, p1, p2], plane.normal, plane.distance)
    }

    pub fn ray_axis_aligned_bounding_box(
        ray: &crate::Ray,
        box_: &crate::AxisAlignedBoundingBox,
    ) -> Option<RayInterval> {
        ray_axis_aligned_box(ray.origin, ray.direction, box_.minimum, box_.maximum)
    }
}
