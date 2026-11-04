use crate::{
    barycentric_coordinates, barycentric_coordinates_2d, BarycentricCoordinates, Cartesian2,
    Cartesian3,
};

/// Determines if a point is inside a triangle.
///
/// Returns `true` if the point is strictly inside the triangle; otherwise `false`.
pub fn point_inside_triangle<T: BarycentricCoordinates>(
    point: &T,
    p0: &T,
    p1: &T,
    p2: &T,
) -> bool {
    if let Some(coords) = point.barycentric_coordinates(p0, p1, p2) {
        coords.x > 0.0 && coords.y > 0.0 && coords.z > 0.0
    } else {
        false
    }
}

/// Determines if a 2D point is inside a 2D triangle.
pub fn point_inside_triangle_2d(
    point: &Cartesian2,
    p0: &Cartesian2,
    p1: &Cartesian2,
    p2: &Cartesian2,
) -> bool {
    if let Some(coords) = barycentric_coordinates_2d(point, p0, p1, p2) {
        coords.x > 0.0 && coords.y > 0.0 && coords.z > 0.0
    } else {
        false
    }
}

/// Determines if a 3D point is inside a 3D triangle.
pub fn point_inside_triangle_3d(
    point: &Cartesian3,
    p0: &Cartesian3,
    p1: &Cartesian3,
    p2: &Cartesian3,
) -> bool {
    if let Some(coords) = barycentric_coordinates(point, p0, p1, p2) {
        coords.x > 0.0 && coords.y > 0.0 && coords.z > 0.0
    } else {
        false
    }
}
