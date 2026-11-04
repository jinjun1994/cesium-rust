use crate::{
    core_math, Cartesian3, Iau2000Orientation, IauOrientationParameters, JulianDate, Matrix3,
    Quaternion,
};

pub type IauComputeFn = fn(&JulianDate) -> IauOrientationParameters;

/// The Axes representing the orientation of a celestial globe from IAU/IAG Working Group reports.
#[derive(Debug, Clone, Copy)]
pub struct IauOrientationAxes {
    compute_fn: IauComputeFn,
}

impl Default for IauOrientationAxes {
    fn default() -> Self {
        Self::new(None)
    }
}

impl IauOrientationAxes {
    pub fn new(compute_fn: Option<IauComputeFn>) -> Self {
        Self {
            compute_fn: compute_fn.unwrap_or(Iau2000Orientation::compute_moon),
        }
    }

    /// Computes a rotation matrix from ICRF to a Globe's Fixed axes at the given date.
    pub fn evaluate(&self, date: &JulianDate) -> Matrix3 {
        let alpha_delta_w = (self.compute_fn)(date);
        let prec_mtx = compute_rotation_matrix(alpha_delta_w.right_ascension, alpha_delta_w.declination);

        let rot = core_math::zero_to_two_pi(alpha_delta_w.rotation);
        let quat = Quaternion::from_axis_angle(Cartesian3::UNIT_Z, rot);
        let rot_mtx = Matrix3::from_quaternion(&quat.conjugate());

        rot_mtx.multiply(&prec_mtx)
    }
}

fn compute_rotation_matrix(alpha: f64, delta: f64) -> Matrix3 {
    let x_axis = Cartesian3::new(
        (alpha + std::f64::consts::FRAC_PI_2).cos(),
        (alpha + std::f64::consts::FRAC_PI_2).sin(),
        0.0,
    );

    let cos_dec = delta.cos();
    let z_axis = Cartesian3::new(
        cos_dec * alpha.cos(),
        cos_dec * alpha.sin(),
        delta.sin(),
    );

    let y_axis = z_axis.cross(&x_axis);

    Matrix3::from_row_major_array([
        x_axis.x, x_axis.y, x_axis.z,
        y_axis.x, y_axis.y, y_axis.z,
        z_axis.x, z_axis.y, z_axis.z,
    ])
}
