use crate::DeveloperError;

/// An algorithm for performing Lagrange polynomial interpolation.
pub struct LagrangePolynomialApproximation;

impl LagrangePolynomialApproximation {
    pub const TYPE: &'static str = "Lagrange";

    /// Given the desired degree, returns the number of data points required for interpolation.
    #[inline]
    pub fn get_required_data_points(degree: usize) -> usize {
        (degree + 1).max(2)
    }

    /// Interpolates values using Lagrange Polynomial Approximation.
    pub fn interpolate_order_zero(
        x: f64,
        x_table: &[f64],
        y_table: &[f64],
        y_stride: usize,
    ) -> Result<Vec<f64>, DeveloperError> {
        let length = x_table.len();
        if length == 0 {
            return Err(DeveloperError::new("xTable must not be empty."));
        }
        if y_stride == 0 {
            return Err(DeveloperError::new(
                "There must be at least 1 dependent variable for each independent variable.",
            ));
        }

        let mut result = vec![0.0; y_stride];

        for i in 0..length {
            let mut coefficient = 1.0;

            for j in 0..length {
                if j != i {
                    let diff_x = x_table[i] - x_table[j];
                    if diff_x == 0.0 {
                        return Err(DeveloperError::new("Elements in xTable must be distinct."));
                    }
                    coefficient *= (x - x_table[j]) / diff_x;
                }
            }

            for j in 0..y_stride {
                result[j] += coefficient * y_table[i * y_stride + j];
            }
        }

        Ok(result)
    }
}
