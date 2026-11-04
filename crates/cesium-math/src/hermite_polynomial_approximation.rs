use crate::{factorial, DeveloperError};

/// An algorithm for performing Hermite polynomial interpolation.
pub struct HermitePolynomialApproximation;

fn calculate_coefficient_term(
    x: f64,
    z_indices: &[usize],
    x_table: &[f64],
    deriv_order: usize,
    term_order: usize,
    reserved_indices: &mut Vec<usize>,
) -> f64 {
    if deriv_order > 0 {
        let mut result = 0.0;
        for i in 0..term_order {
            if !reserved_indices.contains(&i) {
                reserved_indices.push(i);
                result += calculate_coefficient_term(
                    x,
                    z_indices,
                    x_table,
                    deriv_order - 1,
                    term_order,
                    reserved_indices,
                );
                reserved_indices.pop();
            }
        }
        return result;
    }

    let mut result = 1.0;
    for i in 0..term_order {
        if !reserved_indices.contains(&i) {
            result *= x - x_table[z_indices[i]];
        }
    }
    result
}

#[inline]
fn compute_dim_two(z_len: usize, i: usize) -> usize {
    if i == 0 {
        0
    } else {
        z_len * i - (i * (i - 1)) / 2
    }
}

fn fill_coefficient_list(
    coefficients: &mut [f64],
    z_indices: &[usize],
    x_table: &[f64],
    y_table: &[f64],
    y_stride: usize,
    input_order: usize,
) -> usize {
    let z_len = z_indices.len();
    let tmp = (z_len * (z_len + 1)) / 2;
    let mut highest_non_zero: isize = -1;

    for s in 0..y_stride {
        let dim_one = s * tmp;

        for j in 0..z_len {
            let index = z_indices[j] * y_stride * (input_order + 1) + s;
            coefficients[dim_one + j] = y_table[index];
        }

        for i in 1..z_len {
            let mut coef_index = 0;
            let dim_two = compute_dim_two(z_len, i);
            let mut non_zero_coefficients = false;

            for j in 0..(z_len - i) {
                let zj = x_table[z_indices[j]];
                let zn = x_table[z_indices[j + i]];

                let numerator;
                let coefficient;
                if zn - zj <= 0.0 {
                    let index = z_indices[j] * y_stride * (input_order + 1) + y_stride * i + s;
                    numerator = y_table[index];
                    coefficient = numerator / factorial(i);
                    coefficients[dim_one + dim_two + coef_index] = coefficient;
                    coef_index += 1;
                } else {
                    let dim_two_minus_one = compute_dim_two(z_len, i - 1);
                    numerator = coefficients[dim_one + dim_two_minus_one + j + 1]
                        - coefficients[dim_one + dim_two_minus_one + j];
                    coefficient = numerator / (zn - zj);
                    coefficients[dim_one + dim_two + coef_index] = coefficient;
                    coef_index += 1;
                }
                non_zero_coefficients = non_zero_coefficients || numerator != 0.0;
            }

            if non_zero_coefficients {
                highest_non_zero = highest_non_zero.max(i as isize);
            }
        }
    }

    if highest_non_zero < 0 {
        0
    } else {
        highest_non_zero as usize
    }
}

impl HermitePolynomialApproximation {
    pub const TYPE: &'static str = "Hermite";

    /// Given the desired degree and input order, returns the number of data points required for interpolation.
    pub fn get_required_data_points(degree: usize, input_order: usize) -> usize {
        ((degree + 1) / (input_order + 1)).max(2)
    }

    /// Interpolates values using Hermite Polynomial Approximation.
    pub fn interpolate(
        x: f64,
        x_table: &[f64],
        y_table: &[f64],
        y_stride: usize,
        input_order: usize,
        output_order: usize,
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

        let result_length = y_stride * (output_order + 1);
        let mut result = vec![0.0; result_length];

        let z_indices_length = length * (input_order + 1);
        let mut z_indices = Vec::with_capacity(z_indices_length);
        for i in 0..length {
            for _ in 0..=input_order {
                z_indices.push(i);
            }
        }

        let tmp = (z_indices_length * (z_indices_length + 1)) / 2;
        let mut coefficients = vec![0.0; y_stride * tmp];
        let highest_non_zero_coef = fill_coefficient_list(
            &mut coefficients,
            &z_indices,
            x_table,
            y_table,
            y_stride,
            input_order,
        );

        let mut reserved_indices = Vec::new();
        let loop_stop = highest_non_zero_coef.min(output_order);

        for d in 0..=loop_stop {
            for i in d..=highest_non_zero_coef {
                reserved_indices.clear();
                let temp_term = calculate_coefficient_term(
                    x,
                    &z_indices,
                    x_table,
                    d,
                    i,
                    &mut reserved_indices,
                );
                let dim_two = compute_dim_two(z_indices_length, i);

                for s in 0..y_stride {
                    let dim_one = s * tmp;
                    let coef = coefficients[dim_one + dim_two];
                    result[s + d * y_stride] += coef * temp_term;
                }
            }
        }

        Ok(result)
    }

    /// Interpolates values using order zero Hermite Polynomial Approximation.
    pub fn interpolate_order_zero(
        x: f64,
        x_table: &[f64],
        y_table: &[f64],
        y_stride: usize,
    ) -> Result<Vec<f64>, DeveloperError> {
        Self::interpolate(x, x_table, y_table, y_stride, 0, 0)
    }
}
