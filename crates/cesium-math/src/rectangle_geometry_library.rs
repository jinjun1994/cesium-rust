//! RectangleGeometryLibrary matching CesiumJS `Core/RectangleGeometryLibrary`.
//! Adapted from CesiumJS (Apache-2.0).

use std::f64::consts::{FRAC_PI_2, TAU};

use glam::DVec3;

use crate::cartesian::{Cartesian2, Cartesian3};
use crate::ellipsoid::Ellipsoid;
use crate::errors::DeveloperError;
use crate::matrices::{Matrix2, Matrix3};
use crate::projection::GeographicProjection;
use crate::rectangle::GeoRectangle;
use crate::Cartographic;

#[derive(Clone, Debug, PartialEq)]
pub struct RectangleOptionsComputed {
    pub gran_y_cos: f64,
    pub gran_y_sin: f64,
    pub gran_x_cos: f64,
    pub gran_x_sin: f64,
    pub nw_corner: Cartographic,
    pub bounding_rectangle: GeoRectangle,
    pub width: usize,
    pub height: usize,
    pub north_cap: bool,
    pub south_cap: bool,
    pub st_gran_y_cos: Option<f64>,
    pub st_gran_x_cos: Option<f64>,
    pub st_gran_y_sin: Option<f64>,
    pub st_gran_x_sin: Option<f64>,
    pub st_nw_corner: Option<Cartographic>,
    pub st_west: Option<f64>,
    pub st_south: Option<f64>,
    pub lon_scalar: f64,
    pub lat_scalar: f64,
    pub tangent_rotation_matrix: Matrix3,
}

pub struct RectangleGeometryLibrary;

struct RotationOptions {
    north: f64,
    south: f64,
    east: f64,
    west: f64,
    gran_y_cos: f64,
    gran_y_sin: f64,
    gran_x_cos: f64,
    gran_x_sin: f64,
    nw_corner: Cartographic,
}

fn get_rotation_options(
    nw_corner: Cartographic,
    rotation: f64,
    granularity_x: f64,
    granularity_y: f64,
    center_cartesian: DVec3,
    width: usize,
    height: usize,
) -> RotationOptions {
    let cos_rotation = rotation.cos();
    let gran_y_cos = granularity_y * cos_rotation;
    let gran_x_cos = granularity_x * cos_rotation;

    let sin_rotation = rotation.sin();
    let gran_y_sin = granularity_y * sin_rotation;
    let gran_x_sin = granularity_x * sin_rotation;

    let proj = GeographicProjection::new(Ellipsoid::WGS84).unwrap_or_default();
    let nw_cartesian = proj.project(nw_corner).unwrap_or(DVec3::ZERO);

    let diff_x = nw_cartesian.x - center_cartesian.x;
    let diff_y = nw_cartesian.y - center_cartesian.y;
    let rotation_matrix = Matrix2::from_rotation(rotation);
    let rot_v = rotation_matrix.multiply_by_vector(Cartesian2::new(diff_x, diff_y));

    let nw_cartesian_rotated = DVec3::new(
        rot_v.x + center_cartesian.x,
        rot_v.y + center_cartesian.y,
        nw_cartesian.z,
    );
    let nw_corner_unproj = proj.unproject(nw_cartesian_rotated).unwrap_or(nw_corner);

    let w = (width - 1) as f64;
    let h = (height - 1) as f64;

    let latitude = nw_corner_unproj.latitude;
    let latitude0 = latitude + w * gran_x_sin;
    let latitude1 = latitude - gran_y_cos * h;
    let latitude2 = latitude - gran_y_cos * h + w * gran_x_sin;

    let north = latitude.max(latitude0).max(latitude1).max(latitude2);
    let south = latitude.min(latitude0).min(latitude1).min(latitude2);

    let longitude = nw_corner_unproj.longitude;
    let longitude0 = longitude + w * gran_x_cos;
    let longitude1 = longitude + h * gran_y_sin;
    let longitude2 = longitude + h * gran_y_sin + w * gran_x_cos;

    let east = longitude.max(longitude0).max(longitude1).max(longitude2);
    let west = longitude.min(longitude0).min(longitude1).min(longitude2);

    RotationOptions {
        north,
        south,
        east,
        west,
        gran_y_cos,
        gran_y_sin,
        gran_x_cos,
        gran_x_sin,
        nw_corner: nw_corner_unproj,
    }
}

impl RectangleGeometryLibrary {
    pub fn compute_options(
        rectangle: &GeoRectangle,
        granularity: f64,
        rotation: f64,
        st_rotation: f64,
    ) -> Result<RectangleOptionsComputed, DeveloperError> {
        let mut east = rectangle.east;
        let mut west = rectangle.west;
        let mut north = rectangle.north;
        let mut south = rectangle.south;

        let mut north_cap = false;
        let mut south_cap = false;

        if north == FRAC_PI_2 {
            north_cap = true;
        }
        if south == -FRAC_PI_2 {
            south_cap = true;
        }

        let dy = north - south;
        let dx = if west > east {
            TAU - west + east
        } else {
            east - west
        };

        let width = ((dx / granularity).ceil() as usize) + 1;
        let height = ((dy / granularity).ceil() as usize) + 1;
        let granularity_x = dx / ((width - 1) as f64);
        let granularity_y = dy / ((height - 1) as f64);

        let nw_corner = rectangle.northwest();
        let mut center = rectangle.center();
        let proj = GeographicProjection::new(Ellipsoid::WGS84).unwrap_or_default();
        let mut center_cartesian = DVec3::ZERO;

        if rotation != 0.0 || st_rotation != 0.0 {
            if center.longitude < nw_corner.longitude {
                center.longitude += TAU;
            }
            center_cartesian = proj.project(center).unwrap_or(DVec3::ZERO);
        }

        let mut gran_y_cos = granularity_y;
        let mut gran_x_cos = granularity_x;
        let mut gran_y_sin = 0.0;
        let mut gran_x_sin = 0.0;

        let mut bounding_rectangle = *rectangle;
        let mut nw_corner_computed = nw_corner;

        if rotation != 0.0 {
            let rotation_options = get_rotation_options(
                nw_corner,
                rotation,
                granularity_x,
                granularity_y,
                center_cartesian,
                width,
                height,
            );
            north = rotation_options.north;
            south = rotation_options.south;
            east = rotation_options.east;
            west = rotation_options.west;

            if north < -FRAC_PI_2 || north > FRAC_PI_2 || south < -FRAC_PI_2 || south > FRAC_PI_2 {
                return Err(DeveloperError::new(
                    "Rotated rectangle is invalid. It crosses over either the north or south pole.",
                ));
            }

            gran_y_cos = rotation_options.gran_y_cos;
            gran_y_sin = rotation_options.gran_y_sin;
            gran_x_cos = rotation_options.gran_x_cos;
            gran_x_sin = rotation_options.gran_x_sin;
            nw_corner_computed = rotation_options.nw_corner;

            bounding_rectangle.north = north;
            bounding_rectangle.south = south;
            bounding_rectangle.east = east;
            bounding_rectangle.west = west;
        }

        let mut st_gran_y_cos = None;
        let mut st_gran_x_cos = None;
        let mut st_gran_y_sin = None;
        let mut st_gran_x_sin = None;
        let mut st_nw_corner = None;
        let mut st_west = None;
        let mut st_south = None;

        if st_rotation != 0.0 {
            let combined_rotation = rotation - st_rotation;
            let st_nw_corner_initial = bounding_rectangle.northwest();

            let st_rotation_options = get_rotation_options(
                st_nw_corner_initial,
                combined_rotation,
                granularity_x,
                granularity_y,
                center_cartesian,
                width,
                height,
            );

            st_gran_y_cos = Some(st_rotation_options.gran_y_cos);
            st_gran_x_cos = Some(st_rotation_options.gran_x_cos);
            st_gran_y_sin = Some(st_rotation_options.gran_y_sin);
            st_gran_x_sin = Some(st_rotation_options.gran_x_sin);
            st_nw_corner = Some(st_rotation_options.nw_corner);
            st_west = Some(st_rotation_options.west);
            st_south = Some(st_rotation_options.south);
        }

        let rect_w = if rectangle.west > rectangle.east {
            TAU - rectangle.west + rectangle.east
        } else {
            rectangle.east - rectangle.west
        };
        let rect_h = rectangle.north - rectangle.south;
        let lon_scalar = if rect_w > 0.0 { 1.0 / rect_w } else { 0.0 };
        let lat_scalar = if rect_h > 0.0 { 1.0 / rect_h } else { 0.0 };

        Ok(RectangleOptionsComputed {
            gran_y_cos,
            gran_y_sin,
            gran_x_cos,
            gran_x_sin,
            nw_corner: nw_corner_computed,
            bounding_rectangle,
            width,
            height,
            north_cap,
            south_cap,
            st_gran_y_cos,
            st_gran_x_cos,
            st_gran_y_sin,
            st_gran_x_sin,
            st_nw_corner,
            st_west,
            st_south,
            lon_scalar,
            lat_scalar,
            tangent_rotation_matrix: Matrix3::IDENTITY,
        })
    }

    pub fn compute_position(
        computed_options: &RectangleOptionsComputed,
        ellipsoid: &Ellipsoid,
        compute_st: bool,
        row: usize,
        col: usize,
    ) -> (Cartesian3, Option<Cartesian2>) {
        let radii_squared = ellipsoid.radii_squared();
        let nw_corner = computed_options.nw_corner;
        let rectangle = computed_options.bounding_rectangle;

        let r = row as f64;
        let c = col as f64;

        let mut st_latitude = nw_corner.latitude - computed_options.gran_y_cos * r
            + c * computed_options.gran_x_sin;
        let cos_latitude = st_latitude.cos();
        let n_z = st_latitude.sin();
        let k_z = radii_squared.z * n_z;

        let mut st_longitude = nw_corner.longitude + r * computed_options.gran_y_sin
            + c * computed_options.gran_x_cos;
        let n_x = cos_latitude * st_longitude.cos();
        let n_y = cos_latitude * st_longitude.sin();

        let k_x = radii_squared.x * n_x;
        let k_y = radii_squared.y * n_y;

        let gamma = (k_x * n_x + k_y * n_y + k_z * n_z).sqrt();

        let position = Cartesian3::new(k_x / gamma, k_y / gamma, k_z / gamma);

        let mut st = None;
        if compute_st {
            if let Some(st_nw) = computed_options.st_nw_corner {
                let st_gran_y_cos = computed_options.st_gran_y_cos.unwrap_or(0.0);
                let st_gran_x_sin = computed_options.st_gran_x_sin.unwrap_or(0.0);
                let st_gran_y_sin = computed_options.st_gran_y_sin.unwrap_or(0.0);
                let st_gran_x_cos = computed_options.st_gran_x_cos.unwrap_or(0.0);
                let st_west = computed_options.st_west.unwrap_or(0.0);
                let st_south = computed_options.st_south.unwrap_or(0.0);

                st_latitude = st_nw.latitude - st_gran_y_cos * r + c * st_gran_x_sin;
                st_longitude = st_nw.longitude + r * st_gran_y_sin + c * st_gran_x_cos;

                let x = (st_longitude - st_west) * computed_options.lon_scalar;
                let y = (st_latitude - st_south) * computed_options.lat_scalar;
                st = Some(Cartesian2::new(x, y));
            } else {
                let x = (st_longitude - rectangle.west) * computed_options.lon_scalar;
                let y = (st_latitude - rectangle.south) * computed_options.lat_scalar;
                st = Some(Cartesian2::new(x, y));
            }
        }

        (position, st)
    }
}
