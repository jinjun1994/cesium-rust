//! Cartesian coordinate types matching CesiumJS `Core/Cartesian2`, `Core/Cartesian3`, and `Core/Cartesian4`.
//! Adapted from CesiumJS (Apache-2.0).

#![allow(
    clippy::manual_is_multiple_of,
    clippy::chunks_exact_to_as_chunks,
    clippy::needless_range_loop
)]

use crate::{
    core_math::{equals_epsilon, RADIANS_PER_DEGREE},
    Cartographic, Ellipsoid, MathError,
};
use glam::{DVec2, DVec3, DVec4};
use std::ops::{Add, Div, Mul, Neg, Sub};

// ============================================================================
// Cartesian2
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cartesian2 {
    pub x: f64,
    pub y: f64,
}

impl Cartesian2 {
    pub const PACKED_LENGTH: usize = 2;

    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const UNIT_X: Self = Self::new(1.0, 0.0);
    pub const UNIT_Y: Self = Self::new(0.0, 1.0);

    #[inline]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn from_elements(x: f64, y: f64) -> Self {
        Self::new(x, y)
    }

    pub fn from_array(values: &[f64], starting_index: usize) -> Result<Self, MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let slice = values.get(starting_index..end).ok_or(MathError)?;
        Ok(Self::new(slice[0], slice[1]))
    }

    pub fn to_array(&self, output: &mut [f64], starting_index: usize) -> Result<(), MathError> {
        self.pack(output, starting_index)
    }

    pub fn pack(&self, output: &mut [f64], starting_index: usize) -> Result<(), MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let target = output.get_mut(starting_index..end).ok_or(MathError)?;
        target[0] = self.x;
        target[1] = self.y;
        Ok(())
    }

    pub fn unpack(array: &[f64], starting_index: usize) -> Result<Self, MathError> {
        Self::from_array(array, starting_index)
    }

    #[inline]
    pub fn maximum_component(&self) -> f64 {
        self.x.max(self.y)
    }

    #[inline]
    pub fn minimum_component(&self) -> f64 {
        self.x.min(self.y)
    }

    #[inline]
    pub fn minimum_by_component(first: Self, second: Self) -> Self {
        Self::new(first.x.min(second.x), first.y.min(second.y))
    }

    #[inline]
    pub fn maximum_by_component(first: Self, second: Self) -> Self {
        Self::new(first.x.max(second.x), first.y.max(second.y))
    }

    #[inline]
    pub fn magnitude_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn magnitude(&self) -> f64 {
        self.magnitude_squared().sqrt()
    }

    #[inline]
    pub fn distance(&self, other: &Self) -> f64 {
        (*self - *other).magnitude()
    }

    #[inline]
    pub fn distance_squared(&self, other: &Self) -> f64 {
        (*self - *other).magnitude_squared()
    }

    #[inline]
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag == 0.0 || !mag.is_finite() {
            Self::ZERO
        } else {
            Self::new(self.x / mag, self.y / mag)
        }
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn cross(&self, other: &Self) -> f64 {
        self.x * other.y - self.y * other.x
    }

    #[inline]
    pub fn multiply_components(&self, other: &Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y)
    }

    #[inline]
    pub fn divide_by_components(&self, other: &Self) -> Self {
        Self::new(self.x / other.x, self.y / other.y)
    }

    #[inline]
    pub fn multiply_by_scalar(&self, scalar: f64) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }

    #[inline]
    pub fn divide_by_scalar(&self, scalar: f64) -> Self {
        Self::new(self.x / scalar, self.y / scalar)
    }

    #[inline]
    pub fn negate(&self) -> Self {
        Self::new(-self.x, -self.y)
    }

    #[inline]
    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    #[inline]
    pub fn lerp(start: Self, end: Self, t: f64) -> Self {
        Self::new(
            (1.0 - t) * start.x + t * end.x,
            (1.0 - t) * start.y + t * end.y,
        )
    }

    #[inline]
    pub fn angle_between(&self, other: &Self) -> f64 {
        let a = self.normalize();
        let b = other.normalize();
        let cross = a.cross(&b).abs();
        let dot = a.dot(&b);
        cross.atan2(dot)
    }

    #[inline]
    pub fn equals(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }

    #[inline]
    pub fn equals_epsilon(
        &self,
        other: &Self,
        relative_epsilon: Option<f64>,
        absolute_epsilon: Option<f64>,
    ) -> bool {
        equals_epsilon(self.x, other.x, relative_epsilon, absolute_epsilon)
            && equals_epsilon(self.y, other.y, relative_epsilon, absolute_epsilon)
    }

    #[inline]
    pub fn from_cartesian3(cartesian3: Cartesian3) -> Self {
        Self::new(cartesian3.x, cartesian3.y)
    }

    #[inline]
    pub fn from_cartesian4(cartesian4: Cartesian4) -> Self {
        Self::new(cartesian4.x, cartesian4.y)
    }
}

impl From<DVec2> for Cartesian2 {
    #[inline]
    fn from(v: DVec2) -> Self {
        Self::new(v.x, v.y)
    }
}

impl From<Cartesian2> for DVec2 {
    #[inline]
    fn from(c: Cartesian2) -> Self {
        DVec2::new(c.x, c.y)
    }
}

impl From<[f64; 2]> for Cartesian2 {
    #[inline]
    fn from(arr: [f64; 2]) -> Self {
        Self::new(arr[0], arr[1])
    }
}

impl From<Cartesian2> for [f64; 2] {
    #[inline]
    fn from(c: Cartesian2) -> Self {
        [c.x, c.y]
    }
}

impl Add for Cartesian2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Cartesian2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f64> for Cartesian2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f64) -> Self {
        self.multiply_by_scalar(rhs)
    }
}

impl Div<f64> for Cartesian2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f64) -> Self {
        self.divide_by_scalar(rhs)
    }
}

impl Neg for Cartesian2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        self.negate()
    }
}

// ============================================================================
// Cartesian3
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cartesian3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Cartesian3 {
    pub const PACKED_LENGTH: usize = 3;

    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    pub const UNIT_X: Self = Self::new(1.0, 0.0, 0.0);
    pub const UNIT_Y: Self = Self::new(0.0, 1.0, 0.0);
    pub const UNIT_Z: Self = Self::new(0.0, 0.0, 1.0);

    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn from_elements(x: f64, y: f64, z: f64) -> Self {
        Self::new(x, y, z)
    }

    pub fn from_array(values: &[f64], starting_index: usize) -> Result<Self, MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let slice = values.get(starting_index..end).ok_or(MathError)?;
        Ok(Self::new(slice[0], slice[1], slice[2]))
    }

    pub fn to_array(&self, output: &mut [f64], starting_index: usize) -> Result<(), MathError> {
        self.pack(output, starting_index)
    }

    pub fn pack(&self, output: &mut [f64], starting_index: usize) -> Result<(), MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let target = output.get_mut(starting_index..end).ok_or(MathError)?;
        target[0] = self.x;
        target[1] = self.y;
        target[2] = self.z;
        Ok(())
    }

    pub fn unpack(array: &[f64], starting_index: usize) -> Result<Self, MathError> {
        Self::from_array(array, starting_index)
    }

    pub fn pack_array(
        cartesians: &[Self],
        output: &mut [f64],
        starting_index: usize,
    ) -> Result<(), MathError> {
        let total = cartesians
            .len()
            .checked_mul(Self::PACKED_LENGTH)
            .and_then(|l| starting_index.checked_add(l))
            .ok_or(MathError)?;
        if output.len() < total {
            return Err(MathError);
        }
        for (i, c) in cartesians.iter().enumerate() {
            c.pack(output, starting_index + i * Self::PACKED_LENGTH)?;
        }
        Ok(())
    }

    pub fn unpack_array(array: &[f64], starting_index: usize) -> Result<Vec<Self>, MathError> {
        if starting_index > array.len() {
            return Err(MathError);
        }
        let remaining = &array[starting_index..];
        if remaining.len() % Self::PACKED_LENGTH != 0 {
            return Err(MathError);
        }
        let count = remaining.len() / Self::PACKED_LENGTH;
        let mut result = Vec::with_capacity(count);
        for chunk in remaining.chunks_exact(Self::PACKED_LENGTH) {
            result.push(Self::new(chunk[0], chunk[1], chunk[2]));
        }
        Ok(result)
    }

    #[inline]
    pub fn maximum_component(&self) -> f64 {
        self.x.max(self.y.max(self.z))
    }

    #[inline]
    pub fn minimum_component(&self) -> f64 {
        self.x.min(self.y.min(self.z))
    }

    #[inline]
    pub fn minimum_by_component(first: Self, second: Self) -> Self {
        Self::new(
            first.x.min(second.x),
            first.y.min(second.y),
            first.z.min(second.z),
        )
    }

    #[inline]
    pub fn maximum_by_component(first: Self, second: Self) -> Self {
        Self::new(
            first.x.max(second.x),
            first.y.max(second.y),
            first.z.max(second.z),
        )
    }

    #[inline]
    pub fn magnitude_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline]
    pub fn magnitude(&self) -> f64 {
        self.magnitude_squared().sqrt()
    }

    #[inline]
    pub fn distance(&self, other: &Self) -> f64 {
        (*self - *other).magnitude()
    }

    #[inline]
    pub fn distance_squared(&self, other: &Self) -> f64 {
        (*self - *other).magnitude_squared()
    }

    #[inline]
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag == 0.0 || !mag.is_finite() {
            Self::ZERO
        } else {
            Self::new(self.x / mag, self.y / mag, self.z / mag)
        }
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    #[inline]
    pub fn multiply_components(&self, other: &Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }

    #[inline]
    pub fn divide_by_components(&self, other: &Self) -> Self {
        Self::new(self.x / other.x, self.y / other.y, self.z / other.z)
    }

    #[inline]
    pub fn multiply_by_scalar(&self, scalar: f64) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }

    #[inline]
    pub fn divide_by_scalar(&self, scalar: f64) -> Self {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }

    #[inline]
    pub fn negate(&self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }

    #[inline]
    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    #[inline]
    pub fn lerp(start: Self, end: Self, t: f64) -> Self {
        Self::new(
            (1.0 - t) * start.x + t * end.x,
            (1.0 - t) * start.y + t * end.y,
            (1.0 - t) * start.z + t * end.z,
        )
    }

    #[inline]
    pub fn angle_between(&self, other: &Self) -> f64 {
        let a = self.normalize();
        let b = other.normalize();
        let cosine = a.dot(&b);
        let sine = a.cross(&b).magnitude();
        sine.atan2(cosine)
    }

    #[inline]
    pub fn most_orthogonal_axis(&self) -> Self {
        let f = self.normalize().abs();
        if f.x <= f.y {
            if f.x <= f.z {
                Self::UNIT_X
            } else {
                Self::UNIT_Z
            }
        } else if f.y <= f.z {
            Self::UNIT_Y
        } else {
            Self::UNIT_Z
        }
    }

    #[inline]
    pub fn project_vector(&self, other: &Self) -> Self {
        let d = other.dot(other);
        if d == 0.0 || !d.is_finite() {
            Self::ZERO
        } else {
            let scalar = self.dot(other) / d;
            other.multiply_by_scalar(scalar)
        }
    }

    #[inline]
    pub fn from_spherical(clock: f64, cone: f64, magnitude: Option<f64>) -> Self {
        let mag = magnitude.unwrap_or(1.0);
        let radial = mag * cone.sin();
        Self::new(radial * clock.cos(), radial * clock.sin(), mag * cone.cos())
    }

    pub fn from_radians(
        longitude: f64,
        latitude: f64,
        height: Option<f64>,
        ellipsoid: Option<&Ellipsoid>,
    ) -> Result<Self, MathError> {
        let default_ellipsoid = Ellipsoid::WGS84;
        let ell = ellipsoid.unwrap_or(&default_ellipsoid);
        let h = height.unwrap_or(0.0);
        let carto = Cartographic::radians(longitude, latitude, h)?;
        let pos = ell.cartographic_to_cartesian(carto).ok_or(MathError)?;
        Ok(Self::new(pos.x, pos.y, pos.z))
    }

    pub fn from_degrees(
        longitude: f64,
        latitude: f64,
        height: Option<f64>,
        ellipsoid: Option<&Ellipsoid>,
    ) -> Result<Self, MathError> {
        Self::from_radians(
            longitude * RADIANS_PER_DEGREE,
            latitude * RADIANS_PER_DEGREE,
            height,
            ellipsoid,
        )
    }

    pub fn from_radians_array(
        coordinates: &[f64],
        ellipsoid: Option<&Ellipsoid>,
    ) -> Result<Vec<Self>, MathError> {
        if coordinates.len() < 2 || coordinates.len() % 2 != 0 {
            return Err(MathError);
        }
        let mut result = Vec::with_capacity(coordinates.len() / 2);
        for chunk in coordinates.chunks_exact(2) {
            result.push(Self::from_radians(chunk[0], chunk[1], None, ellipsoid)?);
        }
        Ok(result)
    }

    pub fn from_degrees_array(
        coordinates: &[f64],
        ellipsoid: Option<&Ellipsoid>,
    ) -> Result<Vec<Self>, MathError> {
        if coordinates.len() < 2 || coordinates.len() % 2 != 0 {
            return Err(MathError);
        }
        let mut result = Vec::with_capacity(coordinates.len() / 2);
        for chunk in coordinates.chunks_exact(2) {
            result.push(Self::from_degrees(chunk[0], chunk[1], None, ellipsoid)?);
        }
        Ok(result)
    }

    pub fn from_radians_array_heights(
        coordinates: &[f64],
        ellipsoid: Option<&Ellipsoid>,
    ) -> Result<Vec<Self>, MathError> {
        if coordinates.len() < 3 || coordinates.len() % 3 != 0 {
            return Err(MathError);
        }
        let mut result = Vec::with_capacity(coordinates.len() / 3);
        for chunk in coordinates.chunks_exact(3) {
            result.push(Self::from_radians(
                chunk[0],
                chunk[1],
                Some(chunk[2]),
                ellipsoid,
            )?);
        }
        Ok(result)
    }

    pub fn from_degrees_array_heights(
        coordinates: &[f64],
        ellipsoid: Option<&Ellipsoid>,
    ) -> Result<Vec<Self>, MathError> {
        if coordinates.len() < 3 || coordinates.len() % 3 != 0 {
            return Err(MathError);
        }
        let mut result = Vec::with_capacity(coordinates.len() / 3);
        for chunk in coordinates.chunks_exact(3) {
            result.push(Self::from_degrees(
                chunk[0],
                chunk[1],
                Some(chunk[2]),
                ellipsoid,
            )?);
        }
        Ok(result)
    }

    #[inline]
    pub fn equals(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }

    #[inline]
    pub fn equals_epsilon(
        &self,
        other: &Self,
        relative_epsilon: Option<f64>,
        absolute_epsilon: Option<f64>,
    ) -> bool {
        equals_epsilon(self.x, other.x, relative_epsilon, absolute_epsilon)
            && equals_epsilon(self.y, other.y, relative_epsilon, absolute_epsilon)
            && equals_epsilon(self.z, other.z, relative_epsilon, absolute_epsilon)
    }

    #[inline]
    pub fn from_cartesian2(cartesian2: Cartesian2) -> Self {
        Self::new(cartesian2.x, cartesian2.y, 0.0)
    }

    #[inline]
    pub fn from_cartesian4(cartesian4: Cartesian4) -> Self {
        Self::new(cartesian4.x, cartesian4.y, cartesian4.z)
    }
}

impl From<DVec3> for Cartesian3 {
    #[inline]
    fn from(v: DVec3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

impl From<Cartesian3> for DVec3 {
    #[inline]
    fn from(c: Cartesian3) -> Self {
        DVec3::new(c.x, c.y, c.z)
    }
}

impl From<[f64; 3]> for Cartesian3 {
    #[inline]
    fn from(arr: [f64; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
    }
}

impl From<Cartesian3> for [f64; 3] {
    #[inline]
    fn from(c: Cartesian3) -> Self {
        [c.x, c.y, c.z]
    }
}

impl Add for Cartesian3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Cartesian3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Cartesian3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f64) -> Self {
        self.multiply_by_scalar(rhs)
    }
}

impl Div<f64> for Cartesian3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f64) -> Self {
        self.divide_by_scalar(rhs)
    }
}

impl Neg for Cartesian3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        self.negate()
    }
}

// ============================================================================
// Cartesian4
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cartesian4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Cartesian4 {
    pub const PACKED_LENGTH: usize = 4;

    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const UNIT_X: Self = Self::new(1.0, 0.0, 0.0, 0.0);
    pub const UNIT_Y: Self = Self::new(0.0, 1.0, 0.0, 0.0);
    pub const UNIT_Z: Self = Self::new(0.0, 0.0, 1.0, 0.0);
    pub const UNIT_W: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    #[inline]
    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub const fn from_elements(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self::new(x, y, z, w)
    }

    pub fn from_array(values: &[f64], starting_index: usize) -> Result<Self, MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let slice = values.get(starting_index..end).ok_or(MathError)?;
        Ok(Self::new(slice[0], slice[1], slice[2], slice[3]))
    }

    pub fn to_array(&self, output: &mut [f64], starting_index: usize) -> Result<(), MathError> {
        self.pack(output, starting_index)
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

    pub fn pack_array(
        cartesians: &[Self],
        output: &mut [f64],
        starting_index: usize,
    ) -> Result<(), MathError> {
        let total = cartesians
            .len()
            .checked_mul(Self::PACKED_LENGTH)
            .and_then(|l| starting_index.checked_add(l))
            .ok_or(MathError)?;
        if output.len() < total {
            return Err(MathError);
        }
        for (i, c) in cartesians.iter().enumerate() {
            c.pack(output, starting_index + i * Self::PACKED_LENGTH)?;
        }
        Ok(())
    }

    pub fn unpack_array(array: &[f64], starting_index: usize) -> Result<Vec<Self>, MathError> {
        if starting_index > array.len() {
            return Err(MathError);
        }
        let remaining = &array[starting_index..];
        if remaining.len() % Self::PACKED_LENGTH != 0 {
            return Err(MathError);
        }
        let count = remaining.len() / Self::PACKED_LENGTH;
        let mut result = Vec::with_capacity(count);
        for chunk in remaining.chunks_exact(Self::PACKED_LENGTH) {
            result.push(Self::new(chunk[0], chunk[1], chunk[2], chunk[3]));
        }
        Ok(result)
    }

    #[inline]
    pub fn maximum_component(&self) -> f64 {
        self.x.max(self.y.max(self.z.max(self.w)))
    }

    #[inline]
    pub fn minimum_component(&self) -> f64 {
        self.x.min(self.y.min(self.z.min(self.w)))
    }

    #[inline]
    pub fn minimum_by_component(first: Self, second: Self) -> Self {
        Self::new(
            first.x.min(second.x),
            first.y.min(second.y),
            first.z.min(second.z),
            first.w.min(second.w),
        )
    }

    #[inline]
    pub fn maximum_by_component(first: Self, second: Self) -> Self {
        Self::new(
            first.x.max(second.x),
            first.y.max(second.y),
            first.z.max(second.z),
            first.w.max(second.w),
        )
    }

    #[inline]
    pub fn magnitude_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    #[inline]
    pub fn magnitude(&self) -> f64 {
        self.magnitude_squared().sqrt()
    }

    #[inline]
    pub fn distance(&self, other: &Self) -> f64 {
        (*self - *other).magnitude()
    }

    #[inline]
    pub fn distance_squared(&self, other: &Self) -> f64 {
        (*self - *other).magnitude_squared()
    }

    #[inline]
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag == 0.0 || !mag.is_finite() {
            Self::ZERO
        } else {
            Self::new(
                self.x / mag,
                self.y / mag,
                self.z / mag,
                self.w / mag,
            )
        }
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    #[inline]
    pub fn multiply_components(&self, other: &Self) -> Self {
        Self::new(
            self.x * other.x,
            self.y * other.y,
            self.z * other.z,
            self.w * other.w,
        )
    }

    #[inline]
    pub fn divide_by_components(&self, other: &Self) -> Self {
        Self::new(
            self.x / other.x,
            self.y / other.y,
            self.z / other.z,
            self.w / other.w,
        )
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
    pub fn negate(&self) -> Self {
        Self::new(-self.x, -self.y, -self.z, -self.w)
    }

    #[inline]
    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs(), self.w.abs())
    }

    #[inline]
    pub fn lerp(start: Self, end: Self, t: f64) -> Self {
        Self::new(
            (1.0 - t) * start.x + t * end.x,
            (1.0 - t) * start.y + t * end.y,
            (1.0 - t) * start.z + t * end.z,
            (1.0 - t) * start.w + t * end.w,
        )
    }

    #[inline]
    pub fn from_color(color: crate::color::Color) -> Self {
        Self::new(
            color.red as f64,
            color.green as f64,
            color.blue as f64,
            color.alpha as f64,
        )
    }

    #[inline]
    pub fn equals(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z && self.w == other.w
    }

    #[inline]
    pub fn equals_epsilon(
        &self,
        other: &Self,
        relative_epsilon: Option<f64>,
        absolute_epsilon: Option<f64>,
    ) -> bool {
        equals_epsilon(self.x, other.x, relative_epsilon, absolute_epsilon)
            && equals_epsilon(self.y, other.y, relative_epsilon, absolute_epsilon)
            && equals_epsilon(self.z, other.z, relative_epsilon, absolute_epsilon)
            && equals_epsilon(self.w, other.w, relative_epsilon, absolute_epsilon)
    }
}

impl From<DVec4> for Cartesian4 {
    #[inline]
    fn from(v: DVec4) -> Self {
        Self::new(v.x, v.y, v.z, v.w)
    }
}

impl From<Cartesian4> for DVec4 {
    #[inline]
    fn from(c: Cartesian4) -> Self {
        DVec4::new(c.x, c.y, c.z, c.w)
    }
}

impl From<[f64; 4]> for Cartesian4 {
    #[inline]
    fn from(arr: [f64; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}

impl From<Cartesian4> for [f64; 4] {
    #[inline]
    fn from(c: Cartesian4) -> Self {
        [c.x, c.y, c.z, c.w]
    }
}

impl Add for Cartesian4 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        )
    }
}

impl Sub for Cartesian4 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
            self.w - rhs.w,
        )
    }
}

impl Mul<f64> for Cartesian4 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f64) -> Self {
        self.multiply_by_scalar(rhs)
    }
}

impl Div<f64> for Cartesian4 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f64) -> Self {
        self.divide_by_scalar(rhs)
    }
}

impl Neg for Cartesian4 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        self.negate()
    }
}
