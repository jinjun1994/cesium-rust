//! QuaternionSpline matching CesiumJS `Core/QuaternionSpline`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::errors::DeveloperError;
use crate::quaternion::Quaternion;
use crate::spline_common::{
    clamp_time, find_time_interval, find_time_interval_with_start, wrap_time,
};

#[derive(Clone, Debug, PartialEq)]
pub struct QuaternionSpline {
    pub times: Vec<f64>,
    pub points: Vec<Quaternion>,
}

impl QuaternionSpline {
    pub fn new(times: Vec<f64>, points: Vec<Quaternion>) -> Result<Self, DeveloperError> {
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

    pub fn evaluate(&self, time: f64) -> Result<Quaternion, DeveloperError> {
        let i = find_time_interval(&self.times, time)?;
        let dt = self.times[i + 1] - self.times[i];
        let u = if dt > 0.0 {
            (time - self.times[i]) / dt
        } else {
            0.0
        };

        let q0 = self.points[i];
        let q1 = self.points[i + 1];
        Ok(Quaternion::slerp(&q0, &q1, u))
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
