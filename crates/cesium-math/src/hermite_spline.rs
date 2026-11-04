//! Hermite spline matching CesiumJS `Core/HermiteSpline`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::cartesian::Cartesian3;
use crate::errors::DeveloperError;
use crate::spline_common::{
    clamp_time, find_time_interval, find_time_interval_with_start, wrap_time,
};
use crate::tridiagonal_system_solver::TridiagonalSystemSolver;

#[derive(Clone, Debug, PartialEq)]
pub struct HermiteSpline {
    pub times: Vec<f64>,
    pub points: Vec<Cartesian3>,
    pub in_tangents: Vec<Cartesian3>,
    pub out_tangents: Vec<Cartesian3>,
}

impl HermiteSpline {
    pub fn new(
        times: Vec<f64>,
        points: Vec<Cartesian3>,
        in_tangents: Vec<Cartesian3>,
        out_tangents: Vec<Cartesian3>,
    ) -> Result<Self, DeveloperError> {
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
        if in_tangents.len() != points.len() - 1 || out_tangents.len() != points.len() - 1 {
            return Err(DeveloperError::new(
                "inTangents or outTangents length is not equal to points.length - 1.",
            ));
        }
        Ok(Self {
            times,
            points,
            in_tangents,
            out_tangents,
        })
    }

    pub fn create_natural_cubic(
        times: Vec<f64>,
        points: Vec<Cartesian3>,
    ) -> Result<Self, DeveloperError> {
        Self::create_natural_spline(times, points)
    }

    pub fn create_natural_spline(
        times: Vec<f64>,
        points: Vec<Cartesian3>,
    ) -> Result<Self, DeveloperError> {
        let n = points.len();
        if n < 2 {
            return Err(DeveloperError::new(
                "points.length must be greater than or equal to 2.",
            ));
        }
        if times.len() != n {
            return Err(DeveloperError::new(
                "times.length must be equal to points.length.",
            ));
        }

        let lower = vec![1.0; n - 1];
        let upper = vec![1.0; n - 1];
        let mut diagonal = vec![4.0; n];
        let mut right = vec![Cartesian3::ZERO; n];

        diagonal[0] = 2.0;
        right[0] = (points[1] - points[0]) * 3.0;

        for i in 1..(n - 1) {
            right[i] = (points[i + 1] - points[i - 1]) * 3.0;
        }

        diagonal[n - 1] = 2.0;
        right[n - 1] = (points[n - 1] - points[n - 2]) * 3.0;

        let tangents = TridiagonalSystemSolver::solve(&lower, &diagonal, &upper, &right)?;

        let mut in_tangents = Vec::with_capacity(n - 1);
        let mut out_tangents = Vec::with_capacity(n - 1);
        for i in 0..(n - 1) {
            out_tangents.push(tangents[i]);
            in_tangents.push(tangents[i + 1]);
        }

        Ok(Self {
            times,
            points,
            in_tangents,
            out_tangents,
        })
    }

    pub fn create_clamped_spline(
        times: Vec<f64>,
        points: Vec<Cartesian3>,
        first_tangent: Cartesian3,
        last_tangent: Cartesian3,
    ) -> Result<Self, DeveloperError> {
        let n = points.len();
        if n < 2 {
            return Err(DeveloperError::new(
                "points.length must be greater than or equal to 2.",
            ));
        }
        if times.len() != n {
            return Err(DeveloperError::new(
                "times.length must be equal to points.length.",
            ));
        }

        if n == 2 {
            return Ok(Self {
                times,
                points,
                in_tangents: vec![last_tangent],
                out_tangents: vec![first_tangent],
            });
        }

        let mut lower = vec![1.0; n - 1];
        let mut upper = vec![1.0; n - 1];
        let mut diagonal = vec![4.0; n];
        let mut right = vec![Cartesian3::ZERO; n];

        lower[0] = 1.0;
        diagonal[0] = 1.0;
        upper[0] = 0.0;
        right[0] = first_tangent;

        for i in 1..(n - 2) {
            lower[i] = 1.0;
            upper[i] = 1.0;
            diagonal[i] = 4.0;
            right[i] = (points[i + 1] - points[i - 1]) * 3.0;
        }

        let penultimate = n - 2;
        lower[penultimate] = 0.0;
        upper[penultimate] = 1.0;
        diagonal[penultimate] = 4.0;
        right[penultimate] = (points[n - 1] - points[n - 3]) * 3.0;

        diagonal[n - 1] = 1.0;
        right[n - 1] = last_tangent;

        let tangents = TridiagonalSystemSolver::solve(&lower, &diagonal, &upper, &right)?;

        let mut in_tangents = Vec::with_capacity(n - 1);
        let mut out_tangents = Vec::with_capacity(n - 1);
        for i in 0..(n - 1) {
            out_tangents.push(tangents[i]);
            in_tangents.push(tangents[i + 1]);
        }

        Ok(Self {
            times,
            points,
            in_tangents,
            out_tangents,
        })
    }

    pub fn evaluate(&self, time: f64) -> Result<Cartesian3, DeveloperError> {
        let i = find_time_interval(&self.times, time)?;
        let dt = self.times[i + 1] - self.times[i];
        let u = (time - self.times[i]) / dt;

        let u2 = u * u;
        let u3 = u2 * u;

        let c_p0 = 2.0 * u3 - 3.0 * u2 + 1.0;
        let c_p1 = -2.0 * u3 + 3.0 * u2;
        let c_out = (u3 - 2.0 * u2 + u) * dt;
        let c_in = (u3 - u2) * dt;

        Ok(self.points[i] * c_p0
            + self.points[i + 1] * c_p1
            + self.out_tangents[i] * c_out
            + self.in_tangents[i] * c_in)
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
