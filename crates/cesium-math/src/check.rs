use crate::DeveloperError;

/// Runtime precondition and parameter checks.
pub struct Check;

impl Check {
    /// Asserts that a value is defined (Some).
    #[inline]
    pub fn defined<T>(name: &str, test: Option<&T>) -> Result<(), DeveloperError> {
        if test.is_none() {
            Err(DeveloperError::new(format!(
                "{} is required, actual value was undefined",
                name
            )))
        } else {
            Ok(())
        }
    }

    /// Asserts that a number is valid and not NaN.
    #[inline]
    pub fn type_of_number(name: &str, test: f64) -> Result<(), DeveloperError> {
        if test.is_nan() {
            Err(DeveloperError::new(format!(
                "{} must be a valid number",
                name
            )))
        } else {
            Ok(())
        }
    }

    /// Asserts boolean type.
    #[inline]
    pub fn type_of_bool(_name: &str, _test: bool) -> Result<(), DeveloperError> {
        Ok(())
    }
}
