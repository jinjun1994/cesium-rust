use crate::{Cartesian2, Cartesian3, EPSILON14};

/// Computes the barycentric coordinates for a 3D point with respect to a triangle.
///
/// Returns `None` if the triangle is degenerate.
pub fn barycentric_coordinates(
    point: &Cartesian3,
    p0: &Cartesian3,
    p1: &Cartesian3,
    p2: &Cartesian3,
) -> Option<Cartesian3> {
    if point.equals_epsilon(p0, Some(EPSILON14), None) {
        return Some(Cartesian3::UNIT_X);
    }
    if point.equals_epsilon(p1, Some(EPSILON14), None) {
        return Some(Cartesian3::UNIT_Y);
    }
    if point.equals_epsilon(p2, Some(EPSILON14), None) {
        return Some(Cartesian3::UNIT_Z);
    }

    let v0 = *p1 - *p0;
    let v1 = *p2 - *p0;
    let v2 = *point - *p0;

    let dot00 = v0.dot(&v0);
    let dot01 = v0.dot(&v1);
    let dot02 = v0.dot(&v2);
    let dot11 = v1.dot(&v1);
    let dot12 = v1.dot(&v2);

    let q = dot00 * dot11 - dot01 * dot01;

    // Triangle is degenerate
    if q == 0.0 {
        return None;
    }

    let y = (dot11 * dot02 - dot01 * dot12) / q;
    let z = (dot00 * dot12 - dot01 * dot02) / q;
    let x = 1.0 - y - z;

    Some(Cartesian3::new(x, y, z))
}

/// Computes the barycentric coordinates for a 2D point with respect to a triangle.
///
/// Returns `None` if the triangle is degenerate.
pub fn barycentric_coordinates_2d(
    point: &Cartesian2,
    p0: &Cartesian2,
    p1: &Cartesian2,
    p2: &Cartesian2,
) -> Option<Cartesian3> {
    if point.equals_epsilon(p0, Some(EPSILON14), None) {
        return Some(Cartesian3::UNIT_X);
    }
    if point.equals_epsilon(p1, Some(EPSILON14), None) {
        return Some(Cartesian3::UNIT_Y);
    }
    if point.equals_epsilon(p2, Some(EPSILON14), None) {
        return Some(Cartesian3::UNIT_Z);
    }

    let v0 = *p1 - *p0;
    let v1 = *p2 - *p0;
    let v2 = *point - *p0;

    let dot00 = v0.dot(&v0);
    let dot01 = v0.dot(&v1);
    let dot02 = v0.dot(&v2);
    let dot11 = v1.dot(&v1);
    let dot12 = v1.dot(&v2);

    let q = dot00 * dot11 - dot01 * dot01;

    // Triangle is degenerate
    if q == 0.0 {
        return None;
    }

    let y = (dot11 * dot02 - dot01 * dot12) / q;
    let z = (dot00 * dot12 - dot01 * dot02) / q;
    let x = 1.0 - y - z;

    Some(Cartesian3::new(x, y, z))
}

/// Trait for computing barycentric coordinates on 2D and 3D points.
pub trait BarycentricCoordinates {
    fn barycentric_coordinates(&self, p0: &Self, p1: &Self, p2: &Self) -> Option<Cartesian3>;
}

impl BarycentricCoordinates for Cartesian3 {
    fn barycentric_coordinates(&self, p0: &Self, p1: &Self, p2: &Self) -> Option<Cartesian3> {
        barycentric_coordinates(self, p0, p1, p2)
    }
}

impl BarycentricCoordinates for Cartesian2 {
    fn barycentric_coordinates(&self, p0: &Self, p1: &Self, p2: &Self) -> Option<Cartesian3> {
        barycentric_coordinates_2d(self, p0, p1, p2)
    }
}
