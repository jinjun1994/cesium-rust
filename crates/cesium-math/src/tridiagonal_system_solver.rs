//! Tridiagonal system solver using Thomas algorithm matching CesiumJS `Core/TridiagonalSystemSolver`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::cartesian::Cartesian3;
use crate::errors::DeveloperError;

pub struct TridiagonalSystemSolver;

impl TridiagonalSystemSolver {
    pub fn solve(
        lower: &[f64],
        diagonal: &[f64],
        upper: &[f64],
        right: &[Cartesian3],
    ) -> Result<Vec<Cartesian3>, DeveloperError> {
        let n = diagonal.len();
        if right.len() != n {
            return Err(DeveloperError::new(
                "diagonal and right must have the same lengths.",
            ));
        }
        if lower.len() != upper.len() {
            return Err(DeveloperError::new(
                "lower and upper must have the same lengths.",
            ));
        }
        if n == 0 || lower.len() != n - 1 {
            return Err(DeveloperError::new(
                "lower and upper must be one less than the length of diagonal.",
            ));
        }

        if n == 1 {
            if diagonal[0].abs() < 1e-15 {
                return Err(DeveloperError::new("Singular matrix in tridiagonal solver."));
            }
            return Ok(vec![right[0] / diagonal[0]]);
        }

        let mut c = vec![0.0; n - 1];
        let mut d = vec![Cartesian3::ZERO; n];
        let mut x = vec![Cartesian3::ZERO; n];

        c[0] = upper[0] / diagonal[0];
        d[0] = right[0] * (1.0 / diagonal[0]);

        for i in 1..(n - 1) {
            let denom = diagonal[i] - c[i - 1] * lower[i - 1];
            let scalar = 1.0 / denom;
            c[i] = upper[i] * scalar;
            let temp = right[i] - d[i - 1] * lower[i - 1];
            d[i] = temp * scalar;
        }

        let last_idx = n - 1;
        let denom = diagonal[last_idx] - c[last_idx - 1] * lower[last_idx - 1];
        let scalar = 1.0 / denom;
        let temp = right[last_idx] - d[last_idx - 1] * lower[last_idx - 1];
        d[last_idx] = temp * scalar;

        x[last_idx] = d[last_idx];
        for i in (0..=(last_idx - 1)).rev() {
            x[i] = d[i] - x[i + 1] * c[i];
        }

        Ok(x)
    }
}
