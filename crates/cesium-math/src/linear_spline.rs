//! Linear spline matching CesiumJS `Core/LinearSpline`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::cartesian::Cartesian3;
use crate::errors::DeveloperError;
use crate::spline_common::{clamp_time, find_time_interval, wrap_time};

pub trait SplineInterpolate: Clone {
    fn lerp(p0: &Self, p1: &Self, t: f64) -> Self;
}

impl SplineInterpolate for f64 {
    fn lerp(p0: &Self, p1: &Self, t: f64) -> Self {
        (1.0 - t) * p0 + t * p1
    }
}

impl SplineInterpolate for Cartesian3 {
    fn lerp(p0: &Self, p1: &Self, t: f64) -> Self {
        Cartesian3::lerp(*p0, *p1, t)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearSpline<T: SplineInterpolate> {
    pub times: Vec<f64>,
    pub points: Vec<T>,
}

impl<T: SplineInterpolate> LinearSpline<T> {
    pub fn new(times: Vec<f64>, points: Vec<T>) -> Result<Self, DeveloperError> {
        if points.len() < 2 {
            return Err(DeveloperError::new(
                "points.length must be greater than or equal to 2.",
            ));
        }
        if times.len() != points.len() {
            return Err(DeveloperError::new(
                "times.length must be equal to points.length.",
            ));
        }
        Ok(Self { times, points })
    }

    pub fn evaluate(&self, time: f64) -> Result<T, DeveloperError> {
        let i = find_time_interval(&self.times, time)?;
        let t = (time - self.times[i]) / (self.times[i + 1] - self.times[i]);
        Ok(T::lerp(&self.points[i], &self.points[i + 1], t))
    }

    pub fn find_time_interval(&self, time: f64) -> Result<usize, DeveloperError> {
        find_time_interval(&self.times, time)
    }

    pub fn wrap_time(&self, time: f64) -> f64 {
        wrap_time(&self.times, time)
    }

    pub fn clamp_time(&self, time: f64) -> f64 {
        clamp_time(&self.times, time)
    }
}
