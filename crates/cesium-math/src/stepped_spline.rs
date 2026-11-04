//! SteppedSpline matching CesiumJS `Core/SteppedSpline`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::errors::DeveloperError;
use crate::spline_common::{
    clamp_time, find_time_interval, find_time_interval_with_start, wrap_time,
};

#[derive(Clone, Debug, PartialEq)]
pub struct SteppedSpline<T: Clone> {
    pub times: Vec<f64>,
    pub points: Vec<T>,
}

impl<T: Clone> SteppedSpline<T> {
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
        Ok(self.points[i].clone())
    }

    pub fn find_time_interval(&self, time: f64) -> Result<usize, DeveloperError> {
        find_time_interval(&self.times, time)
    }

    pub fn find_time_interval_with_start(
        &self,
        time: f64,
        start_index: Option<usize>,
    ) -> Result<usize, DeveloperError> {
        find_time_interval_with_start(&self.times, time, start_index)
    }

    pub fn wrap_time(&self, time: f64) -> f64 {
        wrap_time(&self.times, time)
    }

    pub fn clamp_time(&self, time: f64) -> f64 {
        clamp_time(&self.times, time)
    }
}
