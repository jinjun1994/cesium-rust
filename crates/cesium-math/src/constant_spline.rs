//! Constant spline matching CesiumJS `Core/ConstantSpline`.
//! Adapted from CesiumJS (Apache-2.0).

#[derive(Clone, Debug, PartialEq)]
pub struct ConstantSpline<T: Clone> {
    pub value: T,
}

impl<T: Clone> ConstantSpline<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    #[inline]
    pub fn evaluate(&self, _time: f64) -> T {
        self.value.clone()
    }

    #[inline]
    pub fn clamp_time(&self, _time: f64) -> f64 {
        0.0
    }

    #[inline]
    pub fn wrap_time(&self, _time: f64) -> f64 {
        0.0
    }
}
