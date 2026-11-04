//! Catmull-Rom spline matching CesiumJS `Core/CatmullRomSpline`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::cartesian::Cartesian3;
use crate::errors::DeveloperError;
use crate::spline_common::{clamp_time, find_time_interval, wrap_time};

#[derive(Clone, Debug, PartialEq)]
pub struct CatmullRomSpline {
    pub times: Vec<f64>,
    pub points: Vec<Cartesian3>,
    pub first_tangent: Cartesian3,
    pub last_tangent: Cartesian3,
}

impl CatmullRomSpline {
    pub fn new(
        times: Vec<f64>,
        points: Vec<Cartesian3>,
        first_tangent: Option<Cartesian3>,
        last_tangent: Option<Cartesian3>,
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

        let start_tan = if let Some(tan) = first_tangent {
            tan
        } else if n > 2 {
            (points[1] * 2.0 - points[2] - points[0]) * 0.5
        } else {
            points[1] - points[0]
        };

        let end_tan = if let Some(tan) = last_tangent {
            tan
        } else if n > 2 {
            (points[n - 1] - points[n - 2] * 2.0 + points[n - 3]) * 0.5
        } else {
            points[1] - points[0]
        };

        Ok(Self {
            times,
            points,
            first_tangent: start_tan,
            last_tangent: end_tan,
        })
    }

    pub fn evaluate(&self, time: f64) -> Result<Cartesian3, DeveloperError> {
        let n = self.points.len();
        if n < 3 {
            let u = (time - self.times[0]) / (self.times[1] - self.times[0]);
            return Ok(Cartesian3::lerp(self.points[0], self.points[1], u));
        }

        let i = find_time_interval(&self.times, time)?;
        let u = (time - self.times[i]) / (self.times[i + 1] - self.times[i]);

        let u2 = u * u;
        let u3 = u2 * u;

        if i == 0 {
            let p0 = self.points[0];
            let p1 = self.points[1];
            let p2 = self.first_tangent;
            let p3 = (self.points[2] - p0) * 0.5;

            let c0 = 2.0 * u3 - 3.0 * u2 + 1.0;
            let c1 = -2.0 * u3 + 3.0 * u2;
            let c2 = u3 - 2.0 * u2 + u;
            let c3 = u3 - u2;

            Ok(p0 * c0 + p1 * c1 + p2 * c2 + p3 * c3)
        } else if i == n - 2 {
            let p0 = self.points[i];
            let p1 = self.points[i + 1];
            let p2 = (p1 - self.points[i - 1]) * 0.5;
            let p3 = self.last_tangent;

            let c0 = 2.0 * u3 - 3.0 * u2 + 1.0;
            let c1 = -2.0 * u3 + 3.0 * u2;
            let c2 = u3 - 2.0 * u2 + u;
            let c3 = u3 - u2;

            Ok(p0 * c0 + p1 * c1 + p2 * c2 + p3 * c3)
        } else {
            let p0 = self.points[i - 1];
            let p1 = self.points[i];
            let p2 = self.points[i + 1];
            let p3 = self.points[i + 2];

            let c0 = -0.5 * u3 + 1.0 * u2 - 0.5 * u;
            let c1 = 1.5 * u3 - 2.5 * u2 + 1.0;
            let c2 = -1.5 * u3 + 2.0 * u2 + 0.5 * u;
            let c3 = 0.5 * u3 - 0.5 * u2;

            Ok(p0 * c0 + p1 * c1 + p2 * c2 + p3 * c3)
        }
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
