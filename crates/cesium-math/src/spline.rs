//! Spline base interface and utilities matching CesiumJS `Core/Spline`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::cartesian::Cartesian3;
use crate::errors::DeveloperError;
use crate::quaternion::Quaternion;
use crate::spline_common::{clamp_time, find_time_interval_with_start, wrap_time};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplinePointType {
    Number,
    Cartesian3,
    Quaternion,
}

pub trait SplinePoint: Sized + 'static {
    fn spline_point_type() -> Result<SplinePointType, DeveloperError>;
}

impl SplinePoint for f64 {
    fn spline_point_type() -> Result<SplinePointType, DeveloperError> {
        Ok(SplinePointType::Number)
    }
}

impl SplinePoint for Cartesian3 {
    fn spline_point_type() -> Result<SplinePointType, DeveloperError> {
        Ok(SplinePointType::Cartesian3)
    }
}

impl SplinePoint for Quaternion {
    fn spline_point_type() -> Result<SplinePointType, DeveloperError> {
        Ok(SplinePointType::Quaternion)
    }
}

pub struct InvalidPointType;

impl SplinePoint for InvalidPointType {
    fn spline_point_type() -> Result<SplinePointType, DeveloperError> {
        Err(DeveloperError::new(
            "point must be a Cartesian3, Quaternion, or number.",
        ))
    }
}

pub struct Spline;

impl Spline {
    pub fn new() -> Result<Self, DeveloperError> {
        Err(DeveloperError::new(
            "Spline is an interface and cannot be instantiated directly.",
        ))
    }

    pub fn get_point_type<T: SplinePoint>(_point: &T) -> Result<SplinePointType, DeveloperError> {
        T::spline_point_type()
    }

    pub fn find_time_interval(
        times: &[f64],
        time: f64,
        start_index: Option<usize>,
    ) -> Result<usize, DeveloperError> {
        find_time_interval_with_start(times, time, start_index)
    }

    pub fn wrap_time(times: &[f64], time: f64) -> f64 {
        wrap_time(times, time)
    }

    pub fn clamp_time(times: &[f64], time: f64) -> f64 {
        clamp_time(times, time)
    }
}
