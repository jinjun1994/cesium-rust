//! Quaternion and HeadingPitchRoll matching CesiumJS `Core/Quaternion` and `Core/HeadingPitchRoll`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{
    cartesian::Cartesian3,
    core_math::{asin_clamped, EPSILON6, RADIANS_PER_DEGREE},
    matrices::Matrix3,
    MathError,
};
use glam::DQuat;

// ============================================================================
// HeadingPitchRoll
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HeadingPitchRoll {
    pub heading: f64,
    pub pitch: f64,
    pub roll: f64,
}

impl HeadingPitchRoll {
    #[inline]
    pub const fn new(heading: f64, pitch: f64, roll: f64) -> Self {
        Self {
            heading,
            pitch,
            roll,
        }
    }

    #[inline]
    pub fn from_degrees(heading: f64, pitch: f64, roll: f64) -> Self {
        Self::new(
            heading * RADIANS_PER_DEGREE,
            pitch * RADIANS_PER_DEGREE,
            roll * RADIANS_PER_DEGREE,
        )
    }

    pub fn from_quaternion(q: &Quaternion) -> Self {
        let test = 2.0 * (q.w * q.y - q.z * q.x);
        let denominator_roll = 1.0 - 2.0 * (q.x * q.x + q.y * q.y);
        let numerator_roll = 2.0 * (q.w * q.x + q.y * q.z);
        let denominator_heading = 1.0 - 2.0 * (q.y * q.y + q.z * q.z);
        let numerator_heading = 2.0 * (q.w * q.z + q.x * q.y);

        let heading = -numerator_heading.atan2(denominator_heading);
        let roll = numerator_roll.atan2(denominator_roll);
        let pitch = -asin_clamped(test);

        Self::new(heading, pitch, roll)
    }

    #[inline]
    pub fn equals(&self, other: &Self) -> bool {
        self.heading == other.heading && self.pitch == other.pitch && self.roll == other.roll
    }

    #[inline]
    pub fn equals_epsilon(&self, other: &Self, epsilon: f64) -> bool {
        (self.heading - other.heading).abs() <= epsilon
            && (self.pitch - other.pitch).abs() <= epsilon
            && (self.roll - other.roll).abs() <= epsilon
    }
}

// ============================================================================
// Quaternion
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Quaternion {
    pub const PACKED_LENGTH: usize = 4;

    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const IDENTITY: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    #[inline]
    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    pub fn from_axis_angle(axis: Cartesian3, angle: f64) -> Self {
        let (s, c) = (angle * 0.5).sin_cos();
        let a = axis.multiply_by_scalar(s);
        Self::new(a.x, a.y, a.z, c)
    }

    pub fn from_heading_pitch_roll(hpr: &HeadingPitchRoll) -> Self {
        let roll_q = Self::from_axis_angle(Cartesian3::UNIT_X, hpr.roll);
        let pitch_q = Self::from_axis_angle(Cartesian3::UNIT_Y, -hpr.pitch);
        let heading_q = Self::from_axis_angle(Cartesian3::UNIT_Z, -hpr.heading);
        let pitch_roll = pitch_q.multiply(&roll_q);
        heading_q.multiply(&pitch_roll)
    }

    #[inline]
    pub fn from_matrix3(matrix: &Matrix3) -> Self {
        Self::from_rotation_matrix(matrix)
    }

    pub fn from_rotation_matrix(matrix: &Matrix3) -> Self {
        let m00 = matrix.get_element(0, 0).unwrap_or(0.0);
        let m11 = matrix.get_element(1, 1).unwrap_or(0.0);
        let m22 = matrix.get_element(2, 2).unwrap_or(0.0);
        let trace = m00 + m11 + m22;

        if trace > 0.0 {
            let root = (trace + 1.0).sqrt();
            let w = 0.5 * root;
            let inv_root = 0.5 / root;
            let x = (matrix.get_element(2, 1).unwrap() - matrix.get_element(1, 2).unwrap())
                * inv_root;
            let y = (matrix.get_element(0, 2).unwrap() - matrix.get_element(2, 0).unwrap())
                * inv_root;
            let z = (matrix.get_element(1, 0).unwrap() - matrix.get_element(0, 1).unwrap())
                * inv_root;
            Self::new(x, y, z, w)
        } else {
            let next = [1usize, 2, 0];
            let mut i = 0usize;
            if m11 > m00 {
                i = 1;
            }
            if m22 > m00 && m22 > m11 {
                i = 2;
            }
            let j = next[i];
            let k = next[j];

            let m_ii = matrix.get_element(i, i).unwrap();
            let m_jj = matrix.get_element(j, j).unwrap();
            let m_kk = matrix.get_element(k, k).unwrap();
            let root = (m_ii - m_jj - m_kk + 1.0).sqrt();

            let mut quat = [0.0; 3];
            quat[i] = 0.5 * root;
            let inv_root = 0.5 / root;
            let w = (matrix.get_element(j, k).unwrap() - matrix.get_element(k, j).unwrap())
                * inv_root;
            quat[j] = (matrix.get_element(i, j).unwrap() + matrix.get_element(j, i).unwrap())
                * inv_root;
            quat[k] = (matrix.get_element(i, k).unwrap() + matrix.get_element(k, i).unwrap())
                * inv_root;

            Self::new(-quat[0], -quat[1], -quat[2], w)
        }
    }

    pub fn from_array(values: &[f64], starting_index: usize) -> Result<Self, MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let slice = values.get(starting_index..end).ok_or(MathError)?;
        Ok(Self::new(slice[0], slice[1], slice[2], slice[3]))
    }

    pub fn pack(&self, output: &mut [f64], starting_index: usize) -> Result<(), MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let target = output.get_mut(starting_index..end).ok_or(MathError)?;
        target[0] = self.x;
        target[1] = self.y;
        target[2] = self.z;
        target[3] = self.w;
        Ok(())
    }

    pub fn unpack(array: &[f64], starting_index: usize) -> Result<Self, MathError> {
        Self::from_array(array, starting_index)
    }

    #[inline]
    pub fn multiply(&self, other: &Self) -> Self {
        let q1 = DQuat::from_xyzw(self.x, self.y, self.z, self.w);
        let q2 = DQuat::from_xyzw(other.x, other.y, other.z, other.w);
        let r = q1 * q2;
        Self::new(r.x, r.y, r.z, r.w)
    }

    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        Self::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
            self.w + other.w,
        )
    }

    #[inline]
    pub fn subtract(&self, other: &Self) -> Self {
        Self::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z,
            self.w - other.w,
        )
    }

    #[inline]
    pub fn negate(&self) -> Self {
        Self::new(-self.x, -self.y, -self.z, -self.w)
    }

    #[inline]
    pub fn multiply_by_scalar(&self, scalar: f64) -> Self {
        Self::new(
            self.x * scalar,
            self.y * scalar,
            self.z * scalar,
            self.w * scalar,
        )
    }

    #[inline]
    pub fn divide_by_scalar(&self, scalar: f64) -> Self {
        Self::new(
            self.x / scalar,
            self.y / scalar,
            self.z / scalar,
            self.w / scalar,
        )
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    #[inline]
    pub fn magnitude_squared(&self) -> f64 {
        self.dot(self)
    }

    #[inline]
    pub fn magnitude(&self) -> f64 {
        self.magnitude_squared().sqrt()
    }

    #[inline]
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag == 0.0 || !mag.is_finite() {
            Self::ZERO
        } else {
            self.divide_by_scalar(mag)
        }
    }

    #[inline]
    pub fn conjugate(&self) -> Self {
        Self::new(-self.x, -self.y, -self.z, self.w)
    }

    #[inline]
    pub fn inverse(&self) -> Self {
        let mag_sq = self.magnitude_squared();
        if mag_sq == 0.0 || !mag_sq.is_finite() {
            Self::ZERO
        } else {
            self.conjugate().divide_by_scalar(mag_sq)
        }
    }

    pub fn slerp(start: &Self, end: &Self, t: f64) -> Self {
        let q1 = DQuat::from_xyzw(start.x, start.y, start.z, start.w);
        let mut q2 = DQuat::from_xyzw(end.x, end.y, end.z, end.w);
        let mut dot = q1.dot(q2);
        if dot < 0.0 {
            q2 = -q2;
            dot = -dot;
        }
        if 1.0 - dot < EPSILON6 {
            let r = q1.lerp(q2, t);
            return Self::new(r.x, r.y, r.z, r.w);
        }
        let theta = dot.clamp(-1.0, 1.0).acos();
        let sin_theta = theta.sin();
        let s0 = ((1.0 - t) * theta).sin() / sin_theta;
        let s1 = (t * theta).sin() / sin_theta;
        let r = DQuat::from_xyzw(
            q1.x * s0 + q2.x * s1,
            q1.y * s0 + q2.y * s1,
            q1.z * s0 + q2.z * s1,
            q1.w * s0 + q2.w * s1,
        );
        Self::new(r.x, r.y, r.z, r.w)
    }

    #[inline]
    pub fn equals(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z && self.w == other.w
    }

    #[inline]
    pub fn equals_epsilon(&self, other: &Self, epsilon: f64) -> bool {
        (self.x - other.x).abs() <= epsilon
            && (self.y - other.y).abs() <= epsilon
            && (self.z - other.z).abs() <= epsilon
            && (self.w - other.w).abs() <= epsilon
    }
}

impl From<DQuat> for Quaternion {
    #[inline]
    fn from(q: DQuat) -> Self {
        Self::new(q.x, q.y, q.z, q.w)
    }
}

impl From<Quaternion> for DQuat {
    #[inline]
    fn from(q: Quaternion) -> Self {
        DQuat::from_xyzw(q.x, q.y, q.z, q.w)
    }
}
