use crate::DeveloperError;

/// An algorithm for performing linear interpolation.
pub struct LinearApproximation;

impl LinearApproximation {
    pub const TYPE: &'static str = "Linear";

    /// Given the desired degree, returns the number of data points required for interpolation.
    /// Since linear interpolation can only generate a first degree polynomial, this always returns 2.
    #[inline]
    pub fn get_required_data_points(_degree: usize) -> usize {
        2
    }

    /// Interpolates values using linear approximation.
    pub fn interpolate_order_zero(
        x: f64,
        x_table: &[f64],
        y_table: &[f64],
        y_stride: usize,
    ) -> Result<Vec<f64>, DeveloperError> {
        if x_table.len() != 2 {
            return Err(DeveloperError::new(
                "The xTable provided to the linear interpolator must have exactly two elements.",
            ));
        }
        if y_stride == 0 {
            return Err(DeveloperError::new(
                "There must be at least 1 dependent variable for each independent variable.",
            ));
        }

        let x0 = x_table[0];
        let x1 = x_table[1];

        if x0 == x1 {
            return Err(DeveloperError::new(
                "Divide by zero error: xTable[0] and xTable[1] are equal",
            ));
        }

        let mut result = vec![0.0; y_stride];
        for i in 0..y_stride {
            let y0 = y_table[i];
            let y1 = y_table[i + y_stride];
            result[i] = ((y1 - y0) * x + x1 * y0 - x0 * y1) / (x1 - x0);
        }

        Ok(result)
    }
}
