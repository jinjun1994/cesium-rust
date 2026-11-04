//! Column vectors, metres, radians, right-handed ECEF and negative-Z eye space.
use glam::{DMat3, DMat4, DVec3, Mat4, Vec4};
use thiserror::Error;
mod associative_array;
mod attribute_type;
mod axis_aligned_bounding_box;
mod bounding_rectangle;
mod box_geometry;
mod cartesian;
mod catmull_rom_spline;
mod circle_geometry;
mod color;
mod color_geometry_instance_attribute;
mod constant_spline;
mod coplanar_polygon_geometry;
mod core_math;
mod corridor_geometry;
mod corridor_geometry_library;
mod culling_volume;
mod cylinder_geometry;
mod distance_display_condition;
mod distance_display_condition_geometry_instance_attribute;
mod double_ended_priority_queue;
mod doubly_linked_list;
mod ellipsoid;
mod ellipsoid_geodesic;
mod ellipsoid_geometry;
mod ellipsoid_rhumb_line;
mod ellipsoid_tangent_plane;
mod ellipsoidal_occluder;
mod ellipse_geometry;
mod encoded_cartesian;
mod errors;
mod frustum;
mod frustum_geometry;
mod geometry;
mod geometry_pipeline;
mod hermite_spline;
mod index_datatype;
mod intersections;
mod julian_date;
mod linear_spline;
mod matrices;
mod occluder;
mod oriented_bounding_box;
mod plane;
mod plane_geometry;
mod pixel_format;
mod polygon_geometry;
mod polygon_geometry_library;
mod polygon_outline_geometry;
mod polygon_pipeline;
mod polynomial;
mod polyline_geometry;
mod polyline_pipeline;
mod polyline_volume_geometry;
mod polyline_volume_geometry_library;
mod polyline_volume_outline_geometry;
mod projection;
mod quaternion;
mod queue;
mod ray;
mod rectangle;
mod scalar;
mod show_geometry_instance_attribute;
mod simple_polyline_geometry;
mod spline_common;
mod sphere;
mod sphere_geometry;
mod stereographic;
mod time_constants;
mod time_interval;
mod time_interval_collection;
mod tiling;
mod tridiagonal_system_solver;
mod vertex_format;
mod wall_geometry;
mod winding_order;
pub use associative_array::AssociativeArray;
pub use attribute_type::AttributeType;
pub use axis_aligned_bounding_box::AxisAlignedBoundingBox;
pub use bounding_rectangle::BoundingRectangle;
pub use box_geometry::{BoxGeometry, BoxOutlineGeometry};
pub use cartesian::{Cartesian2, Cartesian3, Cartesian4};
pub use catmull_rom_spline::CatmullRomSpline;
pub use circle_geometry::{CircleGeometry, CircleOutlineGeometry};
pub use color::Color;
pub use color_geometry_instance_attribute::ColorGeometryInstanceAttribute;
pub use constant_spline::ConstantSpline;
pub use coplanar_polygon_geometry::{
    CoplanarPolygonGeometry, CoplanarPolygonOutlineGeometry, PolygonHierarchy,
};
pub use core_math::*;
pub use corridor_geometry::{
    CorridorGeometry, CorridorGeometryOptions, CorridorOutlineGeometry,
    CorridorOutlineGeometryOptions,
};
pub use corridor_geometry_library::{
    CorridorCorner, CorridorGeometryLibrary, CorridorParameters, CorridorPositions,
};
pub use culling_volume::{BoundingVolume, CullingVolume, Intersect};
pub use cylinder_geometry::{CylinderGeometry, CylinderOutlineGeometry};
pub use distance_display_condition::DistanceDisplayCondition;
pub use distance_display_condition_geometry_instance_attribute::DistanceDisplayConditionGeometryInstanceAttribute;
pub use double_ended_priority_queue::DoubleEndedPriorityQueue;
pub use doubly_linked_list::{DoublyLinkedList, DoublyLinkedListNode, NodeId};
pub use ellipsoid::Ellipsoid;
pub use ellipsoid_geodesic::EllipsoidGeodesic;
pub use ellipsoid_geometry::{EllipsoidGeometry, EllipsoidOutlineGeometry};
pub use ellipsoid_rhumb_line::EllipsoidRhumbLine;
pub use ellipsoid_tangent_plane::EllipsoidTangentPlane;
pub use ellipsoidal_occluder::EllipsoidalOccluder;
pub use ellipse_geometry::{EllipseGeometry, EllipseOutlineGeometry};
pub use encoded_cartesian::EncodedCartesian3;
pub use errors::{DeveloperError, RuntimeError};
pub use frustum::{
    OrthographicFrustum, OrthographicOffCenterFrustum, PerspectiveFrustum,
    PerspectiveOffCenterFrustum,
};
pub use frustum_geometry::{AnyFrustum, FrustumGeometry, FrustumOutlineGeometry};
pub use hermite_spline::HermiteSpline;
pub use geometry::{
    ArcType, ComponentDatatype, CornerType, Geometry, GeometryAttribute, GeometryAttributes,
    GeometryOffsetAttribute, PrimitiveType,
};
pub use geometry_pipeline::GeometryPipeline;
pub use index_datatype::{IndexBuffer, IndexDatatype};
pub use intersections::{
    grazing_altitude_location, line_segment_plane, line_segment_sphere, line_segment_triangle,
    ray_axis_aligned_box, ray_plane, ray_sphere, ray_triangle, ray_triangle_parametric,
    triangle_plane_intersection, TrianglePlaneIntersection,
};
pub use julian_date::{GregorianDate, JulianDate, LeapSecond, TimeStandard, LEAP_SECONDS};
pub use linear_spline::{LinearSpline, SplineInterpolate};
pub use matrices::{Matrix2, Matrix3, Matrix4};
pub use occluder::{Occluder, Visibility};
pub use oriented_bounding_box::OrientedBoundingBox;
pub use plane::Plane;
pub use plane_geometry::{PlaneGeometry, PlaneOutlineGeometry};
pub use pixel_format::{PixelDatatype, PixelFormat};
pub use polygon_geometry::{
    PolygonGeometry, PolygonGeometryFromPositionsOptions, PolygonGeometryOptions,
};
pub use polygon_geometry_library::PolygonGeometryLibrary;
pub use polygon_outline_geometry::{
    PolygonOutlineGeometry, PolygonOutlineGeometryFromPositionsOptions,
    PolygonOutlineGeometryOptions,
};
pub use polygon_pipeline::PolygonPipeline;
pub use polynomial::{
    cubic_discriminant, cubic_real_roots, quadratic_discriminant, quadratic_real_roots,
    quartic_discriminant, quartic_real_roots, CubicRealPolynomial, QuadraticRealPolynomial,
    QuarticRealPolynomial,
};
pub use polyline_geometry::{PolylineGeometry, PolylineGeometryOptions};
pub use polyline_pipeline::{
    PolylineArcOptions, PolylineHeight, PolylinePipeline, PolylineSegments,
};
pub use polyline_volume_geometry::{PolylineVolumeGeometry, PolylineVolumeGeometryOptions};
pub use polyline_volume_geometry_library::PolylineVolumeGeometryLibrary;
pub use polyline_volume_outline_geometry::{
    PolylineVolumeOutlineGeometry, PolylineVolumeOutlineGeometryOptions,
};
pub use projection::{GeographicProjection, MapProjection, WebMercatorProjection};
pub use quaternion::{HeadingPitchRoll, Quaternion};
pub use queue::Queue;
pub use ray::Ray;
pub use rectangle::GeoRectangle;
pub use scalar::{HeadingPitchRange, NearFarScalar};
pub use show_geometry_instance_attribute::ShowGeometryInstanceAttribute;
pub use simple_polyline_geometry::{SimplePolylineGeometry, SimplePolylineGeometryOptions};
pub use sphere::PlaneIntersection;
pub use sphere_geometry::{SphereGeometry, SphereOutlineGeometry};
pub use stereographic::{
    Stereographic, HALF_UNIT_SPHERE, NORTH_POLE, NORTH_POLE_TANGENT_PLANE, SOUTH_POLE,
    SOUTH_POLE_TANGENT_PLANE,
};
pub use time_constants::{TimeConstants, *};
pub use time_interval::TimeInterval;
pub use time_interval_collection::TimeIntervalCollection;
pub use tiling::{GeographicTilingScheme, NativeRectangle, TilingScheme, WebMercatorTilingScheme};
pub use tridiagonal_system_solver::TridiagonalSystemSolver;
pub use vertex_format::VertexFormat;
pub use wall_geometry::{WallGeometry, WallOutlineGeometry};
pub use winding_order::WindingOrder;

pub const WGS84_A: f64 = 6_378_137.0;
pub const WGS84_B: f64 = 6_356_752.314_245_179;
const E2: f64 = 1.0 - (WGS84_B * WGS84_B) / (WGS84_A * WGS84_A);

#[derive(Debug, Error)]
#[error("invalid coordinate, camera, or degenerate transform")]
pub struct MathError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cartographic {
    pub longitude: f64,
    pub latitude: f64,
    pub height: f64,
}
impl Cartographic {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(longitude: f64, latitude: f64, height: f64) -> Self {
        Self {
            longitude,
            latitude,
            height,
        }
    }
    pub const fn from_radians(longitude: f64, latitude: f64) -> Self {
        Self {
            longitude,
            latitude,
            height: 0.0,
        }
    }
    pub fn radians(longitude: f64, latitude: f64, height: f64) -> Result<Self, MathError> {
        if !longitude.is_finite()
            || !latitude.is_finite()
            || !height.is_finite()
            || latitude.abs() > std::f64::consts::FRAC_PI_2
        {
            return Err(MathError);
        }
        Ok(Self {
            longitude,
            latitude,
            height,
        })
    }
    pub fn degrees(lon: f64, lat: f64, height: f64) -> Result<Self, MathError> {
        Self::radians(lon.to_radians(), lat.to_radians(), height)
    }
    pub fn to_ecef(self) -> DVec3 {
        let (sin_lat, cos_lat) = self.latitude.sin_cos();
        let (sin_lon, cos_lon) = self.longitude.sin_cos();
        let n = WGS84_A / (1.0 - E2 * sin_lat * sin_lat).sqrt();
        DVec3::new(
            (n + self.height) * cos_lat * cos_lon,
            (n + self.height) * cos_lat * sin_lon,
            (n * (1.0 - E2) + self.height) * sin_lat,
        )
    }
    /// Rejects the ambiguous deep interior (<1 metre from centre).
    pub fn from_ecef(p: DVec3) -> Result<Self, MathError> {
        if !p.is_finite() || p.length() < 1.0 {
            return Err(MathError);
        }
        let xy = p.x.hypot(p.y);
        if xy < 1e-9 {
            return Self::radians(
                0.0,
                p.z.signum() * std::f64::consts::FRAC_PI_2,
                p.z.abs() - WGS84_B,
            );
        }
        // Bowring initial estimate, followed by a fixed-point correction.
        let theta = (p.z * WGS84_A).atan2(xy * WGS84_B);
        let ep2 = (WGS84_A * WGS84_A - WGS84_B * WGS84_B) / (WGS84_B * WGS84_B);
        let mut lat = (p.z + ep2 * WGS84_B * theta.sin().powi(3))
            .atan2(xy - E2 * WGS84_A * theta.cos().powi(3));
        for _ in 0..12 {
            let n = WGS84_A / (1.0 - E2 * lat.sin().powi(2)).sqrt();
            let next = (p.z + E2 * n * lat.sin()).atan2(xy);
            if (next - lat).abs() < 1e-15 {
                lat = next;
                break;
            }
            lat = next;
        }
        let n = WGS84_A / (1.0 - E2 * lat.sin().powi(2)).sqrt();
        let h = xy * lat.cos() + p.z * lat.sin() - n * (1.0 - E2 * lat.sin().powi(2));
        Self::radians(p.y.atan2(p.x), lat, h)
    }
    pub fn enu_to_ecef(self) -> DMat4 {
        let (s, c) = self.longitude.sin_cos();
        let (sl, cl) = self.latitude.sin_cos();
        DMat4::from_cols(
            DVec3::new(-s, c, 0.0).extend(0.0),
            DVec3::new(-sl * c, -sl * s, cl).extend(0.0),
            DVec3::new(cl * c, cl * s, sl).extend(0.0),
            self.to_ecef().extend(1.0),
        )
    }
}

pub fn ray_ellipsoid(origin: DVec3, direction: DVec3) -> Option<f64> {
    if !origin.is_finite() || !direction.is_finite() || direction.length_squared() < 1e-30 {
        return None;
    }
    let axes = DVec3::new(WGS84_A, WGS84_A, WGS84_B);
    let o = origin / axes;
    let d = direction / axes;
    let a = d.dot(d);
    let b = 2.0 * o.dot(d);
    let c = o.dot(o) - 1.0;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return None;
    }
    let q = -0.5 * (b + disc.sqrt().copysign(b));
    let (a_t, b_t) = if q == 0.0 { (0.0, 0.0) } else { (q / a, c / q) };
    [a_t, b_t]
        .into_iter()
        .filter(|t| *t >= 0.0 && t.is_finite())
        .min_by(f64::total_cmp)
}

/// A numeric interval, including ray intersection parameters.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RayInterval {
    pub start: f64,
    pub stop: f64,
}

impl RayInterval {
    pub const fn new(start: f64, stop: f64) -> Self {
        Self { start, stop }
    }
}

/// Cesium's generic numeric interval, also used for ray intersections.
pub type Interval = RayInterval;

/// The portion of a ray inside an ellipsoid, expressed in ray parameters.
/// An origin inside has a start of zero; an origin on the surface pointing
/// out or tangent has no interval. Direction is not normalized. Radii must
/// be finite and strictly positive. Adapted from CesiumJS (Apache-2.0).
pub fn ray_ellipsoid_interval(
    origin: DVec3,
    direction: DVec3,
    radii: DVec3,
) -> Option<RayInterval> {
    if !origin.is_finite()
        || !direction.is_finite()
        || !radii.is_finite()
        || radii.min_element() <= 0.0
        || direction.length_squared() < 1e-30
    {
        return None;
    }
    let q = origin / radii;
    let w = direction / radii;
    let q2 = q.length_squared();
    let qw = q.dot(w);
    let w2 = w.length_squared();
    if !q2.is_finite() || !qw.is_finite() || !w2.is_finite() || w2 == 0.0 {
        return None;
    }
    let interval = if q2 > 1.0 {
        if qw >= 0.0 {
            return None;
        }
        let difference = q2 - 1.0;
        let product = w2 * difference;
        let qw2 = qw * qw;
        if qw2 < product {
            return None;
        }
        if qw2 == product {
            let root = (difference / w2).sqrt();
            RayInterval {
                start: root,
                stop: root,
            }
        } else {
            let temp = -qw + (qw2 - product).sqrt();
            let a = temp / w2;
            let b = difference / temp;
            RayInterval {
                start: a.min(b),
                stop: a.max(b),
            }
        }
    } else if q2 < 1.0 {
        let discriminant = qw * qw - w2 * (q2 - 1.0);
        RayInterval {
            start: 0.0,
            stop: (-qw + discriminant.sqrt()) / w2,
        }
    } else if qw < 0.0 {
        RayInterval {
            start: 0.0,
            stop: -qw / w2,
        }
    } else {
        return None;
    };
    (interval.start.is_finite() && interval.stop.is_finite()).then_some(interval)
}

pub fn linear_part(m: DMat4) -> DMat3 {
    DMat3::from_cols(
        m.x_axis.truncate(),
        m.y_axis.truncate(),
        m.z_axis.truncate(),
    )
}
/// sqrt(norm1 * normInfinity), a conservative spectral norm bound (including shear).
pub fn scale_bound(a: DMat3) -> f64 {
    let c = [a.x_axis.abs(), a.y_axis.abs(), a.z_axis.abs()];
    let n1 = c.iter().map(|v| v.element_sum()).fold(0.0, f64::max);
    let ni = (c[0] + c[1] + c[2]).max_element();
    (n1 * ni).sqrt()
}
pub fn relative_model_view(model: DMat4, eye: DVec3, camera_to_ecef: DMat3) -> Mat4 {
    let r = camera_to_ecef.transpose();
    let a = r * linear_part(model);
    let t = r * (model.w_axis.truncate() - eye);
    DMat4::from_cols(
        a.x_axis.extend(0.0),
        a.y_axis.extend(0.0),
        a.z_axis.extend(0.0),
        t.extend(1.0),
    )
    .as_mat4()
}
pub fn reverse_z(fovy: f64, aspect: f64, near: f64) -> Result<Mat4, MathError> {
    if fovy <= 0.0
        || !fovy.is_finite()
        || !(0.0..std::f64::consts::PI).contains(&fovy)
        || !aspect.is_finite()
        || aspect <= 0.0
        || !near.is_finite()
        || near <= 0.0
    {
        return Err(MathError);
    }
    let f = (1.0 / (fovy * 0.5).tan()) as f32;
    Ok(Mat4::from_cols(
        Vec4::new(f / aspect as f32, 0.0, 0.0, 0.0),
        Vec4::new(0.0, f, 0.0, 0.0),
        Vec4::new(0.0, 0.0, 0.0, -1.0),
        Vec4::new(0.0, 0.0, near as f32, 0.0),
    ))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sphere {
    pub center: DVec3,
    pub radius: f64,
}

pub type BoundingSphere = Sphere;

impl Sphere {
    pub fn new(center: impl Into<DVec3>, radius: f64) -> Self {
        Self {
            center: center.into(),
            radius,
        }
    }
    pub fn transformed(self, m: DMat4) -> Self {
        Self {
            center: m.transform_point3(self.center),
            radius: self.radius * scale_bound(linear_part(m)),
        }
    }
    pub fn distance(self, p: DVec3) -> f64 {
        (p.distance(self.center) - self.radius).max(0.0)
    }
    /// Squared distance to the sphere, zero for points inside it.
    pub fn distance_squared_to(self, p: DVec3) -> f64 {
        self.distance(p).powi(2)
    }
    /// Near and far distances along the supplied unit direction.
    pub fn compute_plane_distances(self, position: DVec3, direction: DVec3) -> (f64, f64) {
        let center_distance = direction.dot(self.center - position);
        (center_distance - self.radius, center_distance + self.radius)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub eye: DVec3,
    pub camera_to_ecef: DMat3,
    pub fovy: f64,
    pub near: f64,
    pub width: u32,
    pub height: u32,
}
impl Camera {
    pub fn look_at(
        eye: DVec3,
        target: DVec3,
        up: DVec3,
        width: u32,
        height: u32,
    ) -> Result<Self, MathError> {
        let z = (eye - target).try_normalize().ok_or(MathError)?;
        let x = up.cross(z).try_normalize().ok_or(MathError)?;
        Ok(Self {
            eye,
            camera_to_ecef: DMat3::from_cols(x, z.cross(x), z),
            fovy: 60f64.to_radians(),
            near: 0.1,
            width,
            height,
        })
    }
    pub fn projection(self) -> Result<Mat4, MathError> {
        if self.width == 0 || self.height == 0 {
            return Err(MathError);
        }
        reverse_z(self.fovy, self.width as f64 / self.height as f64, self.near)
    }
    pub fn sse(self, sphere: Sphere, error: f64) -> f64 {
        error * self.height as f64
            / (2.0 * (self.fovy * 0.5).tan() * sphere.distance(self.eye).max(1e-6))
    }
    pub fn visible(self, s: Sphere) -> bool {
        let p = self.camera_to_ecef.transpose() * (s.center - self.eye);
        let ty = (self.fovy * 0.5).tan();
        let tx = ty * self.width as f64 / self.height.max(1) as f64;
        // Exact sphere/side-plane distances, conservative for all orientations.
        -p.z + s.radius >= self.near
            && p.x.abs() + p.z * tx <= s.radius * (1.0 + tx * tx).sqrt()
            && p.y.abs() + p.z * ty <= s.radius * (1.0 + ty * ty).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrips() {
        for lat in [-90.0, -89.999, -60.0, 0.0, 60.0, 89.999, 90.0] {
            for lon in [-180.0, -170.0, 0.0, 179.999] {
                for h in [-500.0, 0.0, 10000.0, 36_000_000.0] {
                    let a = Cartographic::degrees(lon, lat, h).unwrap();
                    let b = Cartographic::from_ecef(a.to_ecef()).unwrap();
                    assert!(a.to_ecef().distance(b.to_ecef()) < 1e-6);
                    assert!((a.height - b.height).abs() < 1e-6);
                }
            }
        }
    }
    #[test]
    fn enu_is_orthonormal() {
        let m = Cartographic::degrees(121.0, 31.0, 0.0)
            .unwrap()
            .enu_to_ecef();
        let a = linear_part(m);
        assert!((a.determinant() - 1.0).abs() < 1e-14);
        assert!((a.transpose() * a - DMat3::IDENTITY)
            .to_cols_array()
            .iter()
            .all(|v| v.abs() < 1e-14));
    }
    #[test]
    fn projection_and_relative_translation() {
        let p = reverse_z(1.0, 1.0, 0.1).unwrap();
        for d in [0.1, 1.0, 1e6] {
            let q = p * Vec4::new(0.0, 0.0, -d, 1.0);
            assert!((q.z / q.w - 0.1 / d).abs() < 1e-6);
        }
        let eye = DVec3::new(WGS84_A, 2.0, 3.0);
        let m = relative_model_view(
            DMat4::from_translation(eye + DVec3::X * 0.01),
            eye,
            DMat3::IDENTITY,
        );
        assert!((m.w_axis.x - 0.01).abs() < 1e-7);
        assert!(reverse_z(0.0, 1.0, 0.1).is_err());
    }
    #[test]
    fn ray_and_shear() {
        let t = ray_ellipsoid(DVec3::X * (WGS84_A + 100.0), -DVec3::X).unwrap();
        assert!((t - 100.0).abs() < 1e-7);
        assert!(ray_ellipsoid(DVec3::X * (WGS84_A + 100.0), DVec3::X).is_none());
        assert!(Cartographic::from_ecef(DVec3::ZERO).is_err());
        let a = DMat3::from_cols(DVec3::X, DVec3::new(2.0, 1.0, 0.0), DVec3::Z);
        assert!(scale_bound(a) >= a.mul_vec3(DVec3::new(1.0, 1.0, 0.0).normalize()).length());
    }
}
