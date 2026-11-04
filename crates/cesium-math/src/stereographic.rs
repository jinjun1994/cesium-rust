//! Stereographic coordinates matching CesiumJS `Core/Stereographic`.
//! Adapted from CesiumJS (Apache-2.0).

use std::f64::consts::{FRAC_PI_2, PI};
use std::sync::LazyLock;

use glam::DVec3;

use crate::cartesian::{Cartesian2, Cartesian3};
use crate::core_math::{sign_not_zero, TWO_PI};
use crate::ellipsoid::Ellipsoid;
use crate::ellipsoid_tangent_plane::EllipsoidTangentPlane;
use crate::errors::DeveloperError;
use crate::intersections::ray_plane;
use crate::Cartographic;

pub static HALF_UNIT_SPHERE: LazyLock<Ellipsoid> =
    LazyLock::new(|| Ellipsoid::new(DVec3::splat(0.5)).unwrap());

pub static NORTH_POLE: Cartesian3 = Cartesian3::new(0.0, 0.0, 0.5);
pub static SOUTH_POLE: Cartesian3 = Cartesian3::new(0.0, 0.0, -0.5);

pub static NORTH_POLE_TANGENT_PLANE: LazyLock<EllipsoidTangentPlane> =
    LazyLock::new(|| EllipsoidTangentPlane::new(NORTH_POLE.into(), *HALF_UNIT_SPHERE).unwrap());

pub static SOUTH_POLE_TANGENT_PLANE: LazyLock<EllipsoidTangentPlane> =
    LazyLock::new(|| EllipsoidTangentPlane::new(SOUTH_POLE.into(), *HALF_UNIT_SPHERE).unwrap());

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stereographic {
    pub position: Cartesian2,
    pub tangent_plane: EllipsoidTangentPlane,
}

impl Default for Stereographic {
    fn default() -> Self {
        Self {
            position: Cartesian2::new(0.0, 0.0),
            tangent_plane: *NORTH_POLE_TANGENT_PLANE,
        }
    }
}

impl Stereographic {
    pub const HALF_UNIT_SPHERE: &'static LazyLock<Ellipsoid> = &HALF_UNIT_SPHERE;
    pub const NORTH_POLE: Cartesian3 = NORTH_POLE;
    pub const SOUTH_POLE: Cartesian3 = SOUTH_POLE;
    pub const NORTH_POLE_TANGENT_PLANE: &'static LazyLock<EllipsoidTangentPlane> =
        &NORTH_POLE_TANGENT_PLANE;
    pub const SOUTH_POLE_TANGENT_PLANE: &'static LazyLock<EllipsoidTangentPlane> =
        &SOUTH_POLE_TANGENT_PLANE;

    pub fn new(position: Option<Cartesian2>, tangent_plane: Option<EllipsoidTangentPlane>) -> Self {
        Self {
            position: position.unwrap_or_else(|| Cartesian2::new(0.0, 0.0)),
            tangent_plane: tangent_plane.unwrap_or(*NORTH_POLE_TANGENT_PLANE),
        }
    }

    pub fn x(&self) -> f64 {
        self.position.x
    }

    pub fn y(&self) -> f64 {
        self.position.y
    }

    pub fn ellipsoid(&self) -> Ellipsoid {
        self.tangent_plane.ellipsoid
    }

    pub fn conformal_latitude(&self) -> f64 {
        let r = self.position.magnitude();
        let d = 2.0 * self.tangent_plane.ellipsoid.maximum_radius();
        let sign = self.tangent_plane.plane.normal.z;
        sign * (FRAC_PI_2 - 2.0 * r.atan2(d))
    }

    pub fn longitude(&self) -> f64 {
        let mut lon = FRAC_PI_2 + self.position.y.atan2(self.position.x);
        if lon > PI {
            lon -= TWO_PI;
        }
        lon
    }

    pub fn get_latitude(&self, ellipsoid: Option<&Ellipsoid>) -> f64 {
        let target_ellipsoid = ellipsoid.copied().unwrap_or(Ellipsoid::WGS84);
        let carto = Cartographic {
            longitude: self.longitude(),
            latitude: self.conformal_latitude(),
            height: 0.0,
        };
        let cartesian = self
            .tangent_plane
            .ellipsoid
            .cartographic_to_cartesian(carto)
            .unwrap_or(DVec3::ZERO);
        target_ellipsoid
            .cartesian_to_cartographic(cartesian)
            .map(|c| c.latitude)
            .unwrap_or(0.0)
    }

    pub fn from_cartesian(cartesian: Cartesian3) -> Result<Self, DeveloperError> {
        let sign = sign_not_zero(cartesian.z);
        let (tangent_plane, origin) = if sign < 0.0 {
            (*SOUTH_POLE_TANGENT_PLANE, NORTH_POLE)
        } else {
            (*NORTH_POLE_TANGENT_PLANE, SOUTH_POLE)
        };

        let origin_dvec: DVec3 = origin.into();
        let ray_origin = tangent_plane
            .ellipsoid
            .scale_to_geocentric_surface(cartesian.into())
            .unwrap_or(cartesian.into());
        let ray_dir = (ray_origin - origin_dvec).normalize();

        let inter = ray_plane(
            ray_origin,
            ray_dir,
            tangent_plane.plane.normal,
            tangent_plane.plane.distance,
        )
        .ok_or_else(|| DeveloperError::new("Ray plane intersection failed"))?;

        let v = inter - origin_dvec;
        let x = tangent_plane.x_axis.dot(v);
        let y = sign * tangent_plane.y_axis.dot(v);

        Ok(Self {
            position: Cartesian2::new(x, y),
            tangent_plane,
        })
    }

    pub fn from_cartesian_array(
        cartesians: &[Cartesian3],
    ) -> Result<Vec<Self>, DeveloperError> {
        let mut result = Vec::with_capacity(cartesians.len());
        for &pt in cartesians {
            result.push(Self::from_cartesian(pt)?);
        }
        Ok(result)
    }
}
