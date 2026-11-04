//! Perspective frusta adapted from CesiumJS (Apache-2.0).
use crate::{CullingVolume, MathError, Plane};
use glam::{DMat4, DVec2, DVec3, DVec4};

fn equal_values(left: &[f64], right: &[f64], relative: f64, absolute: f64) -> bool {
    relative.is_finite()
        && absolute.is_finite()
        && relative >= 0.0
        && absolute >= 0.0
        && left.iter().zip(right).all(|(&a, &b)| {
            let difference = (a - b).abs();
            difference <= absolute || difference <= relative * a.abs().max(b.abs())
        })
}

fn validate_camera(position: DVec3, direction: DVec3, up: DVec3) -> Result<(), MathError> {
    if !position.is_finite()
        || !direction.is_finite()
        || !up.is_finite()
        || (direction.length() - 1.0).abs() > 1e-6
        || (up.length() - 1.0).abs() > 1e-6
        || direction.dot(up).abs() > 1e-6
    {
        return Err(MathError);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PerspectiveOffCenterFrustum {
    pub left: f64,
    pub right: f64,
    pub bottom: f64,
    pub top: f64,
    pub near: f64,
    pub far: f64,
}

impl PerspectiveOffCenterFrustum {
    pub fn equals_epsilon(self, other: Self, relative: f64, absolute: f64) -> bool {
        equal_values(
            &[
                self.left,
                self.right,
                self.bottom,
                self.top,
                self.near,
                self.far,
            ],
            &[
                other.left,
                other.right,
                other.bottom,
                other.top,
                other.near,
                other.far,
            ],
            relative,
            absolute,
        )
    }
    fn validate(self) -> Result<(), MathError> {
        if [
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        ]
        .iter()
        .any(|x| !x.is_finite())
            || self.left >= self.right
            || self.bottom >= self.top
            || self.near <= 0.0
            || self.near >= self.far
        {
            return Err(MathError);
        }
        Ok(())
    }
    /// OpenGL clip-space depth [-1, 1], matching Cesium's projection matrix.
    pub fn projection_matrix(self) -> Result<DMat4, MathError> {
        self.validate()?;
        let (l, r, b, t, n, f) = (
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        );
        Ok(DMat4::from_cols(
            DVec4::new(2.0 * n / (r - l), 0.0, 0.0, 0.0),
            DVec4::new(0.0, 2.0 * n / (t - b), 0.0, 0.0),
            DVec4::new(
                (r + l) / (r - l),
                (t + b) / (t - b),
                -(f + n) / (f - n),
                -1.0,
            ),
            DVec4::new(0.0, 0.0, -2.0 * f * n / (f - n), 0.0),
        ))
    }
    /// Infinite-far OpenGL clip-space projection, matching Cesium.
    pub fn infinite_projection_matrix(self) -> Result<DMat4, MathError> {
        self.validate()?;
        let (l, r, b, t, n) = (self.left, self.right, self.bottom, self.top, self.near);
        Ok(DMat4::from_cols(
            DVec4::new(2.0 * n / (r - l), 0.0, 0.0, 0.0),
            DVec4::new(0.0, 2.0 * n / (t - b), 0.0, 0.0),
            DVec4::new((r + l) / (r - l), (t + b) / (t - b), -1.0, -1.0),
            DVec4::new(0.0, 0.0, -2.0 * n, 0.0),
        ))
    }
    pub fn pixel_dimensions(
        self,
        width: u32,
        height: u32,
        distance: f64,
        pixel_ratio: f64,
    ) -> Result<DVec2, MathError> {
        self.validate()?;
        if width == 0
            || height == 0
            || !distance.is_finite()
            || !pixel_ratio.is_finite()
            || pixel_ratio <= 0.0
        {
            return Err(MathError);
        }
        Ok(DVec2::new(
            2.0 * pixel_ratio * distance * self.right / self.near / width as f64,
            2.0 * pixel_ratio * distance * self.top / self.near / height as f64,
        ))
    }
    pub fn compute_culling_volume(
        self,
        position: DVec3,
        direction: DVec3,
        up: DVec3,
    ) -> Result<CullingVolume, MathError> {
        self.validate()?;
        validate_camera(position, direction, up)?;
        let right = direction.cross(up);
        let near = direction * self.near;
        let normals = [
            (near + right * self.left).cross(up),
            up.cross(near + right * self.right),
            right.cross(near + up * self.bottom),
            (near + up * self.top).cross(right),
        ];
        let mut planes = Vec::with_capacity(6);
        for normal in normals {
            planes.push(Plane::from_point_normal(
                position,
                normal.try_normalize().ok_or(MathError)?,
            )?);
        }
        planes.push(Plane::from_point_normal(position + near, direction)?);
        planes.push(Plane::from_point_normal(
            position + direction * self.far,
            -direction,
        )?);
        Ok(CullingVolume::new(planes))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrthographicOffCenterFrustum {
    pub left: f64,
    pub right: f64,
    pub bottom: f64,
    pub top: f64,
    pub near: f64,
    pub far: f64,
}

impl OrthographicOffCenterFrustum {
    pub fn equals_epsilon(self, other: Self, relative: f64, absolute: f64) -> bool {
        equal_values(
            &[
                self.left,
                self.right,
                self.bottom,
                self.top,
                self.near,
                self.far,
            ],
            &[
                other.left,
                other.right,
                other.bottom,
                other.top,
                other.near,
                other.far,
            ],
            relative,
            absolute,
        )
    }
    fn validate(self) -> Result<(), MathError> {
        if [
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        ]
        .iter()
        .any(|x| !x.is_finite())
            || self.left >= self.right
            || self.bottom >= self.top
            || self.near <= 0.0
            || self.near >= self.far
        {
            return Err(MathError);
        }
        Ok(())
    }
    /// OpenGL clip-space depth [-1, 1], matching Cesium.
    pub fn projection_matrix(self) -> Result<DMat4, MathError> {
        self.validate()?;
        let (l, r, b, t, n, f) = (
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        );
        Ok(DMat4::from_cols(
            DVec4::new(2.0 / (r - l), 0.0, 0.0, 0.0),
            DVec4::new(0.0, 2.0 / (t - b), 0.0, 0.0),
            DVec4::new(0.0, 0.0, -2.0 / (f - n), 0.0),
            DVec4::new(
                -(r + l) / (r - l),
                -(t + b) / (t - b),
                -(f + n) / (f - n),
                1.0,
            ),
        ))
    }
    pub fn pixel_dimensions(
        self,
        width: u32,
        height: u32,
        pixel_ratio: f64,
    ) -> Result<DVec2, MathError> {
        self.validate()?;
        if width == 0 || height == 0 || !pixel_ratio.is_finite() || pixel_ratio <= 0.0 {
            return Err(MathError);
        }
        Ok(DVec2::new(
            pixel_ratio * (self.right - self.left) / width as f64,
            pixel_ratio * (self.top - self.bottom) / height as f64,
        ))
    }
    pub fn compute_culling_volume(
        self,
        position: DVec3,
        direction: DVec3,
        up: DVec3,
    ) -> Result<CullingVolume, MathError> {
        self.validate()?;
        validate_camera(position, direction, up)?;
        let right = direction.cross(up).try_normalize().ok_or(MathError)?;
        let near_center = position + direction * self.near;
        let points_and_normals = [
            (near_center + right * self.left, right),
            (near_center + right * self.right, -right),
            (near_center + up * self.bottom, up),
            (near_center + up * self.top, -up),
            (near_center, direction),
            (position + direction * self.far, -direction),
        ];
        let planes = points_and_normals
            .into_iter()
            .map(|(point, normal)| Plane::from_point_normal(point, normal))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CullingVolume::new(planes))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrthographicFrustum {
    pub width: f64,
    pub aspect_ratio: f64,
    pub near: f64,
    pub far: f64,
}

impl OrthographicFrustum {
    pub const PACKED_LENGTH: usize = 4;
    pub fn equals_epsilon(
        self,
        other: Self,
        relative: f64,
        absolute: f64,
    ) -> Result<bool, MathError> {
        let left = self.off_center()?;
        let right = other.off_center()?;
        Ok(equal_values(
            &[self.width, self.aspect_ratio],
            &[other.width, other.aspect_ratio],
            relative,
            absolute,
        ) && left.equals_epsilon(right, relative, absolute))
    }
    pub fn off_center(self) -> Result<OrthographicOffCenterFrustum, MathError> {
        if !self.width.is_finite()
            || self.width <= 0.0
            || !self.aspect_ratio.is_finite()
            || self.aspect_ratio <= 0.0
        {
            return Err(MathError);
        }
        let right = self.width * 0.5;
        let top = right / self.aspect_ratio;
        let result = OrthographicOffCenterFrustum {
            left: -right,
            right,
            bottom: -top,
            top,
            near: self.near,
            far: self.far,
        };
        result.validate()?;
        Ok(result)
    }
    pub fn projection_matrix(self) -> Result<DMat4, MathError> {
        self.off_center()?.projection_matrix()
    }
    pub fn pixel_dimensions(
        self,
        width: u32,
        height: u32,
        pixel_ratio: f64,
    ) -> Result<DVec2, MathError> {
        self.off_center()?
            .pixel_dimensions(width, height, pixel_ratio)
    }
    pub fn compute_culling_volume(
        self,
        position: DVec3,
        direction: DVec3,
        up: DVec3,
    ) -> Result<CullingVolume, MathError> {
        self.off_center()?
            .compute_culling_volume(position, direction, up)
    }
    pub fn pack(self, output: &mut [f64], offset: usize) -> Result<(), MathError> {
        let end = offset.checked_add(Self::PACKED_LENGTH).ok_or(MathError)?;
        output
            .get_mut(offset..end)
            .ok_or(MathError)?
            .copy_from_slice(&[self.width, self.aspect_ratio, self.near, self.far]);
        Ok(())
    }
    pub fn unpack(input: &[f64], offset: usize) -> Result<Self, MathError> {
        let end = offset.checked_add(Self::PACKED_LENGTH).ok_or(MathError)?;
        let v = input.get(offset..end).ok_or(MathError)?;
        Ok(Self {
            width: v[0],
            aspect_ratio: v[1],
            near: v[2],
            far: v[3],
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PerspectiveFrustum {
    pub fov: f64,
    pub aspect_ratio: f64,
    pub near: f64,
    pub far: f64,
    pub x_offset: f64,
    pub y_offset: f64,
}

impl PerspectiveFrustum {
    pub const PACKED_LENGTH: usize = 6;
    pub fn pack(self, output: &mut [f64], offset: usize) -> Result<(), MathError> {
        let end = offset.checked_add(Self::PACKED_LENGTH).ok_or(MathError)?;
        output
            .get_mut(offset..end)
            .ok_or(MathError)?
            .copy_from_slice(&[
                self.fov,
                self.aspect_ratio,
                self.near,
                self.far,
                self.x_offset,
                self.y_offset,
            ]);
        Ok(())
    }
    pub fn unpack(input: &[f64], offset: usize) -> Result<Self, MathError> {
        let end = offset.checked_add(Self::PACKED_LENGTH).ok_or(MathError)?;
        let v = input.get(offset..end).ok_or(MathError)?;
        Ok(Self {
            fov: v[0],
            aspect_ratio: v[1],
            near: v[2],
            far: v[3],
            x_offset: v[4],
            y_offset: v[5],
        })
    }
    pub fn equals_epsilon(
        self,
        other: Self,
        relative: f64,
        absolute: f64,
    ) -> Result<bool, MathError> {
        let left = self.off_center()?;
        let right = other.off_center()?;
        Ok(equal_values(
            &[self.fov, self.aspect_ratio],
            &[other.fov, other.aspect_ratio],
            relative,
            absolute,
        ) && left.equals_epsilon(right, relative, absolute))
    }
    pub fn off_center(self) -> Result<PerspectiveOffCenterFrustum, MathError> {
        if !self.fov.is_finite()
            || self.fov <= 0.0
            || self.fov >= std::f64::consts::PI
            || !self.aspect_ratio.is_finite()
            || self.aspect_ratio <= 0.0
            || !self.x_offset.is_finite()
            || !self.y_offset.is_finite()
        {
            return Err(MathError);
        }
        let top = self.near * (self.fovy()? * 0.5).tan();
        let right = self.aspect_ratio * top;
        let result = PerspectiveOffCenterFrustum {
            left: -right + self.x_offset,
            right: right + self.x_offset,
            bottom: -top + self.y_offset,
            top: top + self.y_offset,
            near: self.near,
            far: self.far,
        };
        result.validate()?;
        Ok(result)
    }
    pub fn fovy(self) -> Result<f64, MathError> {
        if !self.fov.is_finite()
            || self.fov <= 0.0
            || self.fov >= std::f64::consts::PI
            || !self.aspect_ratio.is_finite()
            || self.aspect_ratio <= 0.0
        {
            return Err(MathError);
        }
        Ok(if self.aspect_ratio <= 1.0 {
            self.fov
        } else {
            2.0 * ((self.fov * 0.5).tan() / self.aspect_ratio).atan()
        })
    }
    pub fn sse_denominator(self) -> Result<f64, MathError> {
        Ok(2.0 * (self.fovy()? * 0.5).tan())
    }
    pub fn projection_matrix(self) -> Result<DMat4, MathError> {
        self.off_center()?.projection_matrix()
    }
    pub fn infinite_projection_matrix(self) -> Result<DMat4, MathError> {
        self.off_center()?.infinite_projection_matrix()
    }
    pub fn pixel_dimensions(
        self,
        width: u32,
        height: u32,
        distance: f64,
        pixel_ratio: f64,
    ) -> Result<DVec2, MathError> {
        self.off_center()?
            .pixel_dimensions(width, height, distance, pixel_ratio)
    }
    pub fn compute_culling_volume(
        self,
        position: DVec3,
        direction: DVec3,
        up: DVec3,
    ) -> Result<CullingVolume, MathError> {
        self.off_center()?
            .compute_culling_volume(position, direction, up)
    }
}
