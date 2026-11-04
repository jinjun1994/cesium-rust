//! Matrices matching CesiumJS `Core/Matrix2`, `Core/Matrix3`, and `Core/Matrix4`.
//! Stored in column-major order; constructor takes elements in row-major order.
//! Adapted from CesiumJS (Apache-2.0).

#![allow(
    clippy::too_many_arguments,
    clippy::needless_range_loop,
    clippy::manual_is_multiple_of,
    clippy::chunks_exact_to_as_chunks
)]

use crate::{
    cartesian::{Cartesian2, Cartesian3, Cartesian4},
    quaternion::{HeadingPitchRoll, Quaternion},
    MathError,
};
use glam::{DMat2, DMat3, DMat4, DVec2, DVec3, DVec4};

// ============================================================================
// Matrix2
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix2 {
    /// Column-major storage: [col0row0, col0row1, col1row0, col1row1]
    pub values: [f64; 4],
}

impl Default for Matrix2 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Matrix2 {
    pub const PACKED_LENGTH: usize = 4;

    pub const ZERO: Self = Self {
        values: [0.0, 0.0, 0.0, 0.0],
    };

    pub const IDENTITY: Self = Self {
        values: [1.0, 0.0, 0.0, 1.0],
    };

    #[inline]
    pub const fn new(c0r0: f64, c1r0: f64, c0r1: f64, c1r1: f64) -> Self {
        Self {
            values: [c0r0, c0r1, c1r0, c1r1],
        }
    }

    #[inline]
    pub const fn from_column_major_array(values: [f64; 4]) -> Self {
        Self { values }
    }

    #[inline]
    pub const fn from_row_major_array(values: [f64; 4]) -> Self {
        Self::new(values[0], values[1], values[2], values[3])
    }

    pub fn from_array(values: &[f64], starting_index: usize) -> Result<Self, MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let slice = values.get(starting_index..end).ok_or(MathError)?;
        Ok(Self::from_column_major_array([
            slice[0], slice[1], slice[2], slice[3],
        ]))
    }

    pub fn pack(&self, output: &mut [f64], starting_index: usize) -> Result<(), MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let target = output.get_mut(starting_index..end).ok_or(MathError)?;
        target.copy_from_slice(&self.values);
        Ok(())
    }

    pub fn unpack(array: &[f64], starting_index: usize) -> Result<Self, MathError> {
        Self::from_array(array, starting_index)
    }

    #[inline]
    pub fn from_scale(scale: Cartesian2) -> Self {
        Self::new(scale.x, 0.0, 0.0, scale.y)
    }

    #[inline]
    pub fn from_uniform_scale(scale: f64) -> Self {
        Self::new(scale, 0.0, 0.0, scale)
    }

    #[inline]
    pub fn from_rotation(angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(cos, -sin, sin, cos)
    }

    #[inline]
    pub fn get_element(&self, row: usize, column: usize) -> Result<f64, MathError> {
        if row >= 2 || column >= 2 {
            return Err(MathError);
        }
        Ok(self.values[column * 2 + row])
    }

    #[inline]
    pub fn set_element(&mut self, row: usize, column: usize, value: f64) -> Result<(), MathError> {
        if row >= 2 || column >= 2 {
            return Err(MathError);
        }
        self.values[column * 2 + row] = value;
        Ok(())
    }

    #[inline]
    pub fn get_column(&self, column: usize) -> Result<Cartesian2, MathError> {
        if column >= 2 {
            return Err(MathError);
        }
        let i = column * 2;
        Ok(Cartesian2::new(self.values[i], self.values[i + 1]))
    }

    #[inline]
    pub fn set_column(&mut self, column: usize, cartesian: Cartesian2) -> Result<(), MathError> {
        if column >= 2 {
            return Err(MathError);
        }
        let i = column * 2;
        self.values[i] = cartesian.x;
        self.values[i + 1] = cartesian.y;
        Ok(())
    }

    #[inline]
    pub fn get_row(&self, row: usize) -> Result<Cartesian2, MathError> {
        if row >= 2 {
            return Err(MathError);
        }
        Ok(Cartesian2::new(self.values[row], self.values[row + 2]))
    }

    #[inline]
    pub fn set_row(&mut self, row: usize, cartesian: Cartesian2) -> Result<(), MathError> {
        if row >= 2 {
            return Err(MathError);
        }
        self.values[row] = cartesian.x;
        self.values[row + 2] = cartesian.y;
        Ok(())
    }

    #[inline]
    pub fn multiply(&self, other: &Self) -> Self {
        let m1 = DMat2::from_cols_array(&self.values);
        let m2 = DMat2::from_cols_array(&other.values);
        Self::from_column_major_array((m1 * m2).to_cols_array())
    }

    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        let mut out = [0.0; 4];
        for i in 0..4 {
            out[i] = self.values[i] + other.values[i];
        }
        Self::from_column_major_array(out)
    }

    #[inline]
    pub fn subtract(&self, other: &Self) -> Self {
        let mut out = [0.0; 4];
        for i in 0..4 {
            out[i] = self.values[i] - other.values[i];
        }
        Self::from_column_major_array(out)
    }

    #[inline]
    pub fn multiply_by_vector(&self, vector: Cartesian2) -> Cartesian2 {
        let m = DMat2::from_cols_array(&self.values);
        let v = DVec2::new(vector.x, vector.y);
        let r = m * v;
        Cartesian2::new(r.x, r.y)
    }

    #[inline]
    pub fn multiply_by_scalar(&self, scalar: f64) -> Self {
        let mut out = [0.0; 4];
        for i in 0..4 {
            out[i] = self.values[i] * scalar;
        }
        Self::from_column_major_array(out)
    }

    #[inline]
    pub fn multiply_by_scale(&self, scale: Cartesian2) -> Self {
        Self::from_column_major_array([
            self.values[0] * scale.x,
            self.values[1] * scale.x,
            self.values[2] * scale.y,
            self.values[3] * scale.y,
        ])
    }

    #[inline]
    pub fn multiply_by_uniform_scale(&self, scale: f64) -> Self {
        self.multiply_by_scalar(scale)
    }

    #[inline]
    pub fn negate(&self) -> Self {
        self.multiply_by_scalar(-1.0)
    }

    #[inline]
    pub fn transpose(&self) -> Self {
        Self::from_column_major_array([
            self.values[0],
            self.values[2],
            self.values[1],
            self.values[3],
        ])
    }

    #[inline]
    pub fn determinant(&self) -> f64 {
        self.values[0] * self.values[3] - self.values[1] * self.values[2]
    }

    pub fn inverse(&self) -> Result<Self, MathError> {
        let det = self.determinant();
        if det.abs() < 1e-15 || !det.is_finite() {
            return Err(MathError);
        }
        let inv_det = 1.0 / det;
        Ok(Self::from_column_major_array([
            self.values[3] * inv_det,
            -self.values[1] * inv_det,
            -self.values[2] * inv_det,
            self.values[0] * inv_det,
        ]))
    }

    pub fn inverse_transpose(&self) -> Result<Self, MathError> {
        self.inverse().map(|m| m.transpose())
    }

    #[inline]
    pub fn equals(&self, other: &Self) -> bool {
        self.values == other.values
    }

    #[inline]
    pub fn equals_epsilon(&self, other: &Self, epsilon: f64) -> bool {
        self.values
            .iter()
            .zip(other.values.iter())
            .all(|(a, b)| (a - b).abs() <= epsilon)
    }
}

// ============================================================================
// Matrix3
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix3 {
    /// Column-major storage: [col0row0, col0row1, col0row2, col1row0, col1row1, col1row2, col2row0, col2row1, col2row2]
    pub values: [f64; 9],
}

impl Default for Matrix3 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Matrix3 {
    pub const PACKED_LENGTH: usize = 9;

    pub const ZERO: Self = Self { values: [0.0; 9] };

    pub const IDENTITY: Self = Self {
        values: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    };

    #[inline]
    pub const fn new(
        c0r0: f64,
        c1r0: f64,
        c2r0: f64,
        c0r1: f64,
        c1r1: f64,
        c2r1: f64,
        c0r2: f64,
        c1r2: f64,
        c2r2: f64,
    ) -> Self {
        Self {
            values: [c0r0, c0r1, c0r2, c1r0, c1r1, c1r2, c2r0, c2r1, c2r2],
        }
    }

    #[inline]
    pub const fn from_column_major_array(values: [f64; 9]) -> Self {
        Self { values }
    }

    #[inline]
    pub const fn from_row_major_array(v: [f64; 9]) -> Self {
        Self::new(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8])
    }

    pub fn from_array(values: &[f64], starting_index: usize) -> Result<Self, MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let slice = values.get(starting_index..end).ok_or(MathError)?;
        let mut out = [0.0; 9];
        out.copy_from_slice(slice);
        Ok(Self::from_column_major_array(out))
    }

    pub fn pack(&self, output: &mut [f64], starting_index: usize) -> Result<(), MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let target = output.get_mut(starting_index..end).ok_or(MathError)?;
        target.copy_from_slice(&self.values);
        Ok(())
    }

    pub fn unpack(array: &[f64], starting_index: usize) -> Result<Self, MathError> {
        Self::from_array(array, starting_index)
    }

    pub fn from_quaternion(q: &Quaternion) -> Self {
        let x2 = q.x * q.x;
        let xy = q.x * q.y;
        let xz = q.x * q.z;
        let xw = q.x * q.w;
        let y2 = q.y * q.y;
        let yz = q.y * q.z;
        let yw = q.y * q.w;
        let z2 = q.z * q.z;
        let zw = q.z * q.w;
        let w2 = q.w * q.w;

        let m00 = x2 - y2 - z2 + w2;
        let m01 = 2.0 * (xy - zw);
        let m02 = 2.0 * (xz + yw);

        let m10 = 2.0 * (xy + zw);
        let m11 = -x2 + y2 - z2 + w2;
        let m12 = 2.0 * (yz - xw);

        let m20 = 2.0 * (xz - yw);
        let m21 = 2.0 * (yz + xw);
        let m22 = -x2 - y2 + z2 + w2;

        Self::new(m00, m01, m02, m10, m11, m12, m20, m21, m22)
    }

    pub fn from_heading_pitch_roll(hpr: &HeadingPitchRoll) -> Self {
        let (sin_theta, cos_theta) = (-hpr.pitch).sin_cos();
        let (sin_psi, cos_psi) = (-hpr.heading).sin_cos();
        let (sin_phi, cos_phi) = hpr.roll.sin_cos();

        let m00 = cos_theta * cos_psi;
        let m01 = -cos_phi * sin_psi + sin_phi * sin_theta * cos_psi;
        let m02 = sin_phi * sin_psi + cos_phi * sin_theta * cos_psi;

        let m10 = cos_theta * sin_psi;
        let m11 = cos_phi * cos_psi + sin_phi * sin_theta * sin_psi;
        let m12 = -sin_phi * cos_psi + cos_phi * sin_theta * sin_psi;

        let m20 = -sin_theta;
        let m21 = sin_phi * cos_theta;
        let m22 = cos_phi * cos_theta;

        Self::new(m00, m01, m02, m10, m11, m12, m20, m21, m22)
    }

    #[inline]
    pub fn from_scale(scale: Cartesian3) -> Self {
        Self::new(
            scale.x, 0.0, 0.0, 0.0, scale.y, 0.0, 0.0, 0.0, scale.z,
        )
    }

    #[inline]
    pub fn from_uniform_scale(scale: f64) -> Self {
        Self::new(scale, 0.0, 0.0, 0.0, scale, 0.0, 0.0, 0.0, scale)
    }

    #[inline]
    pub fn from_cross_product(v: Cartesian3) -> Self {
        Self::new(0.0, -v.z, v.y, v.z, 0.0, -v.x, -v.y, v.x, 0.0)
    }

    #[inline]
    pub fn from_rotation_x(angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(1.0, 0.0, 0.0, 0.0, cos, -sin, 0.0, sin, cos)
    }

    #[inline]
    pub fn from_rotation_y(angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(cos, 0.0, sin, 0.0, 1.0, 0.0, -sin, 0.0, cos)
    }

    #[inline]
    pub fn from_rotation_z(angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(cos, -sin, 0.0, sin, cos, 0.0, 0.0, 0.0, 1.0)
    }

    #[inline]
    pub fn get_element(&self, row: usize, column: usize) -> Result<f64, MathError> {
        if row >= 3 || column >= 3 {
            return Err(MathError);
        }
        Ok(self.values[column * 3 + row])
    }

    #[inline]
    pub fn set_element(&mut self, row: usize, column: usize, value: f64) -> Result<(), MathError> {
        if row >= 3 || column >= 3 {
            return Err(MathError);
        }
        self.values[column * 3 + row] = value;
        Ok(())
    }

    #[inline]
    pub fn get_column(&self, column: usize) -> Result<Cartesian3, MathError> {
        if column >= 3 {
            return Err(MathError);
        }
        let i = column * 3;
        Ok(Cartesian3::new(
            self.values[i],
            self.values[i + 1],
            self.values[i + 2],
        ))
    }

    #[inline]
    pub fn set_column(&mut self, column: usize, cartesian: Cartesian3) -> Result<(), MathError> {
        if column >= 3 {
            return Err(MathError);
        }
        let i = column * 3;
        self.values[i] = cartesian.x;
        self.values[i + 1] = cartesian.y;
        self.values[i + 2] = cartesian.z;
        Ok(())
    }

    #[inline]
    pub fn get_row(&self, row: usize) -> Result<Cartesian3, MathError> {
        if row >= 3 {
            return Err(MathError);
        }
        Ok(Cartesian3::new(
            self.values[row],
            self.values[row + 3],
            self.values[row + 6],
        ))
    }

    #[inline]
    pub fn set_row(&mut self, row: usize, cartesian: Cartesian3) -> Result<(), MathError> {
        if row >= 3 {
            return Err(MathError);
        }
        self.values[row] = cartesian.x;
        self.values[row + 3] = cartesian.y;
        self.values[row + 6] = cartesian.z;
        Ok(())
    }

    #[inline]
    pub fn multiply(&self, other: &Self) -> Self {
        let m1 = DMat3::from_cols_array(&self.values);
        let m2 = DMat3::from_cols_array(&other.values);
        Self::from_column_major_array((m1 * m2).to_cols_array())
    }

    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        let mut out = [0.0; 9];
        for i in 0..9 {
            out[i] = self.values[i] + other.values[i];
        }
        Self::from_column_major_array(out)
    }

    #[inline]
    pub fn subtract(&self, other: &Self) -> Self {
        let mut out = [0.0; 9];
        for i in 0..9 {
            out[i] = self.values[i] - other.values[i];
        }
        Self::from_column_major_array(out)
    }

    #[inline]
    pub fn multiply_by_vector(&self, vector: Cartesian3) -> Cartesian3 {
        let m = DMat3::from_cols_array(&self.values);
        let v = DVec3::new(vector.x, vector.y, vector.z);
        let r = m * v;
        Cartesian3::new(r.x, r.y, r.z)
    }

    #[inline]
    pub fn multiply_by_scalar(&self, scalar: f64) -> Self {
        let mut out = [0.0; 9];
        for i in 0..9 {
            out[i] = self.values[i] * scalar;
        }
        Self::from_column_major_array(out)
    }

    #[inline]
    pub fn multiply_by_scale(&self, scale: Cartesian3) -> Self {
        let mut out = self.values;
        out[0] *= scale.x;
        out[1] *= scale.x;
        out[2] *= scale.x;
        out[3] *= scale.y;
        out[4] *= scale.y;
        out[5] *= scale.y;
        out[6] *= scale.z;
        out[7] *= scale.z;
        out[8] *= scale.z;
        Self::from_column_major_array(out)
    }

    #[inline]
    pub fn multiply_by_uniform_scale(&self, scale: f64) -> Self {
        self.multiply_by_scalar(scale)
    }

    #[inline]
    pub fn negate(&self) -> Self {
        self.multiply_by_scalar(-1.0)
    }

    #[inline]
    pub fn transpose(&self) -> Self {
        let m = DMat3::from_cols_array(&self.values);
        Self::from_column_major_array(m.transpose().to_cols_array())
    }

    #[inline]
    pub fn determinant(&self) -> f64 {
        DMat3::from_cols_array(&self.values).determinant()
    }

    pub fn inverse(&self) -> Result<Self, MathError> {
        let m = DMat3::from_cols_array(&self.values);
        let det = m.determinant();
        if det.abs() < 1e-15 || !det.is_finite() {
            return Err(MathError);
        }
        Ok(Self::from_column_major_array(m.inverse().to_cols_array()))
    }

    pub fn inverse_transpose(&self) -> Result<Self, MathError> {
        self.inverse().map(|m| m.transpose())
    }

    #[inline]
    pub fn equals(&self, other: &Self) -> bool {
        self.values == other.values
    }

    #[inline]
    pub fn equals_epsilon(&self, other: &Self, epsilon: f64) -> bool {
        self.values
            .iter()
            .zip(other.values.iter())
            .all(|(a, b)| (a - b).abs() <= epsilon)
    }
}

impl From<DMat3> for Matrix3 {
    #[inline]
    fn from(m: DMat3) -> Self {
        Self::from_column_major_array(m.to_cols_array())
    }
}

impl From<Matrix3> for DMat3 {
    #[inline]
    fn from(m: Matrix3) -> Self {
        DMat3::from_cols_array(&m.values)
    }
}

// ============================================================================
// Matrix4
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix4 {
    /// Column-major storage: 16 elements
    pub values: [f64; 16],
}

impl Default for Matrix4 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Matrix4 {
    pub const PACKED_LENGTH: usize = 16;

    pub const ZERO: Self = Self { values: [0.0; 16] };

    pub const IDENTITY: Self = Self {
        values: [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    };

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        c0r0: f64,
        c1r0: f64,
        c2r0: f64,
        c3r0: f64,
        c0r1: f64,
        c1r1: f64,
        c2r1: f64,
        c3r1: f64,
        c0r2: f64,
        c1r2: f64,
        c2r2: f64,
        c3r2: f64,
        c0r3: f64,
        c1r3: f64,
        c2r3: f64,
        c3r3: f64,
    ) -> Self {
        Self {
            values: [
                c0r0, c0r1, c0r2, c0r3, c1r0, c1r1, c1r2, c1r3, c2r0, c2r1, c2r2, c2r3, c3r0,
                c3r1, c3r2, c3r3,
            ],
        }
    }

    #[inline]
    pub const fn from_column_major_array(values: [f64; 16]) -> Self {
        Self { values }
    }

    #[inline]
    pub const fn from_row_major_array(v: [f64; 16]) -> Self {
        Self::new(
            v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9], v[10], v[11], v[12],
            v[13], v[14], v[15],
        )
    }

    pub fn from_array(values: &[f64], starting_index: usize) -> Result<Self, MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let slice = values.get(starting_index..end).ok_or(MathError)?;
        let mut out = [0.0; 16];
        out.copy_from_slice(slice);
        Ok(Self::from_column_major_array(out))
    }

    pub fn pack(&self, output: &mut [f64], starting_index: usize) -> Result<(), MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let target = output.get_mut(starting_index..end).ok_or(MathError)?;
        target.copy_from_slice(&self.values);
        Ok(())
    }

    pub fn unpack(array: &[f64], starting_index: usize) -> Result<Self, MathError> {
        Self::from_array(array, starting_index)
    }

    #[inline]
    pub fn from_translation(translation: Cartesian3) -> Self {
        Self::new(
            1.0,
            0.0,
            0.0,
            translation.x,
            0.0,
            1.0,
            0.0,
            translation.y,
            0.0,
            0.0,
            1.0,
            translation.z,
            0.0,
            0.0,
            0.0,
            1.0,
        )
    }

    #[inline]
    pub fn from_scale(scale: Cartesian3) -> Self {
        Self::new(
            scale.x, 0.0, 0.0, 0.0, 0.0, scale.y, 0.0, 0.0, 0.0, 0.0, scale.z, 0.0, 0.0, 0.0,
            0.0, 1.0,
        )
    }

    #[inline]
    pub fn from_translation_quaternion_rotation_scale(
        translation: Cartesian3,
        rotation: Quaternion,
        scale: Cartesian3,
    ) -> Self {
        let rot = Matrix3::from_quaternion(&rotation);
        let rot_scale = rot.multiply_by_scale(scale);
        Self::from_rotation_translation(&rot_scale, translation)
    }

    #[inline]
    pub fn from_uniform_scale(scale: f64) -> Self {
        Self::from_scale(Cartesian3::new(scale, scale, scale))
    }

    pub fn from_rotation_translation(rotation: &Matrix3, translation: Cartesian3) -> Self {
        let r = rotation.values;
        Self::from_column_major_array([
            r[0],
            r[1],
            r[2],
            0.0,
            r[3],
            r[4],
            r[5],
            0.0,
            r[6],
            r[7],
            r[8],
            0.0,
            translation.x,
            translation.y,
            translation.z,
            1.0,
        ])
    }

    pub fn from_camera(
        eye: Cartesian3,
        target: Cartesian3,
        up: Cartesian3,
    ) -> Result<Self, MathError> {
        let z = (eye - target).normalize();
        if z.magnitude_squared() == 0.0 {
            return Err(MathError);
        }
        let x = up.cross(&z).normalize();
        if x.magnitude_squared() == 0.0 {
            return Err(MathError);
        }
        let y = z.cross(&x);
        let rot = Matrix3::new(x.x, x.y, x.z, y.x, y.y, y.z, z.x, z.y, z.z);
        let trans = Cartesian3::new(-x.dot(&eye), -y.dot(&eye), -z.dot(&eye));
        Ok(Self::from_rotation_translation(&rot, trans))
    }

    #[inline]
    pub fn get_matrix3(&self) -> Matrix3 {
        Matrix3::from_column_major_array([
            self.values[0],
            self.values[1],
            self.values[2],
            self.values[4],
            self.values[5],
            self.values[6],
            self.values[8],
            self.values[9],
            self.values[10],
        ])
    }

    #[inline]
    pub fn set_translation(&mut self, translation: Cartesian3) {
        self.values[12] = translation.x;
        self.values[13] = translation.y;
        self.values[14] = translation.z;
    }

    #[inline]
    pub fn set_scale(&mut self, scale: Cartesian3) {
        let current_scale = self.get_scale();
        if current_scale.x != 0.0 && current_scale.y != 0.0 && current_scale.z != 0.0 {
            let factor = Cartesian3::new(
                scale.x / current_scale.x,
                scale.y / current_scale.y,
                scale.z / current_scale.z,
            );
            *self = self.multiply_by_scale(factor);
        }
    }

    #[inline]
    pub fn get_translation(&self) -> Cartesian3 {
        Cartesian3::new(self.values[12], self.values[13], self.values[14])
    }

    #[inline]
    pub fn get_scale(&self) -> Cartesian3 {
        let c0 = Cartesian3::new(self.values[0], self.values[1], self.values[2]).magnitude();
        let c1 = Cartesian3::new(self.values[4], self.values[5], self.values[6]).magnitude();
        let c2 = Cartesian3::new(self.values[8], self.values[9], self.values[10]).magnitude();
        Cartesian3::new(c0, c1, c2)
    }

    #[inline]
    pub fn get_element(&self, row: usize, column: usize) -> Result<f64, MathError> {
        if row >= 4 || column >= 4 {
            return Err(MathError);
        }
        Ok(self.values[column * 4 + row])
    }

    #[inline]
    pub fn set_element(&mut self, row: usize, column: usize, value: f64) -> Result<(), MathError> {
        if row >= 4 || column >= 4 {
            return Err(MathError);
        }
        self.values[column * 4 + row] = value;
        Ok(())
    }

    #[inline]
    pub fn get_column(&self, column: usize) -> Result<Cartesian4, MathError> {
        if column >= 4 {
            return Err(MathError);
        }
        let i = column * 4;
        Ok(Cartesian4::new(
            self.values[i],
            self.values[i + 1],
            self.values[i + 2],
            self.values[i + 3],
        ))
    }

    #[inline]
    pub fn set_column(&mut self, column: usize, cartesian: Cartesian4) -> Result<(), MathError> {
        if column >= 4 {
            return Err(MathError);
        }
        let i = column * 4;
        self.values[i] = cartesian.x;
        self.values[i + 1] = cartesian.y;
        self.values[i + 2] = cartesian.z;
        self.values[i + 3] = cartesian.w;
        Ok(())
    }

    #[inline]
    pub fn get_row(&self, row: usize) -> Result<Cartesian4, MathError> {
        if row >= 4 {
            return Err(MathError);
        }
        Ok(Cartesian4::new(
            self.values[row],
            self.values[row + 4],
            self.values[row + 8],
            self.values[row + 12],
        ))
    }

    #[inline]
    pub fn set_row(&mut self, row: usize, cartesian: Cartesian4) -> Result<(), MathError> {
        if row >= 4 {
            return Err(MathError);
        }
        self.values[row] = cartesian.x;
        self.values[row + 4] = cartesian.y;
        self.values[row + 8] = cartesian.z;
        self.values[row + 12] = cartesian.w;
        Ok(())
    }

    #[inline]
    pub fn multiply(&self, other: &Self) -> Self {
        let m1 = DMat4::from_cols_array(&self.values);
        let m2 = DMat4::from_cols_array(&other.values);
        Self::from_column_major_array((m1 * m2).to_cols_array())
    }

    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        let mut out = [0.0; 16];
        for i in 0..16 {
            out[i] = self.values[i] + other.values[i];
        }
        Self::from_column_major_array(out)
    }

    #[inline]
    pub fn subtract(&self, other: &Self) -> Self {
        let mut out = [0.0; 16];
        for i in 0..16 {
            out[i] = self.values[i] - other.values[i];
        }
        Self::from_column_major_array(out)
    }

    #[inline]
    pub fn multiply_by_vector(&self, vector: Cartesian4) -> Cartesian4 {
        let m = DMat4::from_cols_array(&self.values);
        let v = DVec4::new(vector.x, vector.y, vector.z, vector.w);
        let r = m * v;
        Cartesian4::new(r.x, r.y, r.z, r.w)
    }

    #[inline]
    pub fn multiply_by_matrix3(&self, matrix: &Matrix3) -> Self {
        let m4 = Self::from_rotation_translation(matrix, Cartesian3::ZERO);
        self.multiply(&m4)
    }

    #[inline]
    pub fn multiply_by_point(&self, point: Cartesian3) -> Cartesian3 {
        let m = DMat4::from_cols_array(&self.values);
        let v = DVec3::new(point.x, point.y, point.z);
        let r = m.transform_point3(v);
        Cartesian3::new(r.x, r.y, r.z)
    }

    #[inline]
    pub fn multiply_by_point_as_vector(&self, vector: Cartesian3) -> Cartesian3 {
        let m = DMat4::from_cols_array(&self.values);
        let v = DVec3::new(vector.x, vector.y, vector.z);
        let r = m.transform_vector3(v);
        Cartesian3::new(r.x, r.y, r.z)
    }

    #[inline]
    pub fn multiply_by_direction(&self, vector: Cartesian3) -> Cartesian3 {
        self.multiply_by_point_as_vector(vector)
    }

    #[inline]
    pub fn multiply_by_scalar(&self, scalar: f64) -> Self {
        let mut out = [0.0; 16];
        for i in 0..16 {
            out[i] = self.values[i] * scalar;
        }
        Self::from_column_major_array(out)
    }

    #[inline]
    pub fn multiply_by_scale(&self, scale: Cartesian3) -> Self {
        let mut out = self.values;
        for i in 0..4 {
            out[i] *= scale.x;
            out[i + 4] *= scale.y;
            out[i + 8] *= scale.z;
        }
        Self::from_column_major_array(out)
    }

    #[inline]
    pub fn multiply_by_uniform_scale(&self, scale: f64) -> Self {
        self.multiply_by_scale(Cartesian3::new(scale, scale, scale))
    }

    #[inline]
    pub fn multiply_by_translation(&self, translation: Cartesian3) -> Self {
        let trans = Self::from_translation(translation);
        self.multiply(&trans)
    }

    #[inline]
    pub fn negate(&self) -> Self {
        self.multiply_by_scalar(-1.0)
    }

    #[inline]
    pub fn transpose(&self) -> Self {
        let m = DMat4::from_cols_array(&self.values);
        Self::from_column_major_array(m.transpose().to_cols_array())
    }

    #[inline]
    pub fn determinant(&self) -> f64 {
        DMat4::from_cols_array(&self.values).determinant()
    }

    pub fn inverse(&self) -> Result<Self, MathError> {
        let m = DMat4::from_cols_array(&self.values);
        let det = m.determinant();
        if det.abs() < 1e-15 || !det.is_finite() {
            return Err(MathError);
        }
        Ok(Self::from_column_major_array(m.inverse().to_cols_array()))
    }

    pub fn inverse_transformation(&self) -> Result<Self, MathError> {
        let r = self.get_matrix3().transpose();
        let t = self.get_translation();
        let inv_t = r.multiply_by_vector(t).negate();
        Ok(Self::from_rotation_translation(&r, inv_t))
    }

    pub fn inverse_transpose(&self) -> Result<Self, MathError> {
        self.inverse().map(|m| m.transpose())
    }

    #[inline]
    pub fn equals(&self, other: &Self) -> bool {
        self.values == other.values
    }

    #[inline]
    pub fn equals_epsilon(&self, other: &Self, epsilon: f64) -> bool {
        self.values
            .iter()
            .zip(other.values.iter())
            .all(|(a, b)| (a - b).abs() <= epsilon)
    }
}

impl From<DMat4> for Matrix4 {
    #[inline]
    fn from(m: DMat4) -> Self {
        Self::from_column_major_array(m.to_cols_array())
    }
}

impl std::ops::Index<usize> for Matrix2 {
    type Output = f64;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl std::ops::Index<usize> for Matrix3 {
    type Output = f64;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl std::ops::Index<usize> for Matrix4 {
    type Output = f64;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}


impl From<Matrix4> for DMat4 {
    #[inline]
    fn from(m: Matrix4) -> Self {
        DMat4::from_cols_array(&m.values)
    }
}
