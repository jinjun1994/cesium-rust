use crate::{
    Cartesian2, Cartesian3, Cartesian4, DeveloperError, Ellipsoid, HeadingPitchRoll, MapProjection,
    Matrix3, Matrix4, Quaternion, EPSILON14, EPSILON6,
};
use glam::DVec3;

/// Axis in a local reference frame centered at a point on the ellipsoid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalFrameAxis {
    East,
    North,
    Up,
    West,
    South,
    Down,
}

impl LocalFrameAxis {
    /// Computes the cross product axis according to Cesium's reference frame rules.
    pub fn cross(self, other: Self) -> Result<Self, DeveloperError> {
        match (self, other) {
            (LocalFrameAxis::Up, LocalFrameAxis::South) => Ok(LocalFrameAxis::East),
            (LocalFrameAxis::Up, LocalFrameAxis::North) => Ok(LocalFrameAxis::West),
            (LocalFrameAxis::Up, LocalFrameAxis::West) => Ok(LocalFrameAxis::South),
            (LocalFrameAxis::Up, LocalFrameAxis::East) => Ok(LocalFrameAxis::North),

            (LocalFrameAxis::Down, LocalFrameAxis::South) => Ok(LocalFrameAxis::West),
            (LocalFrameAxis::Down, LocalFrameAxis::North) => Ok(LocalFrameAxis::East),
            (LocalFrameAxis::Down, LocalFrameAxis::West) => Ok(LocalFrameAxis::North),
            (LocalFrameAxis::Down, LocalFrameAxis::East) => Ok(LocalFrameAxis::South),

            (LocalFrameAxis::South, LocalFrameAxis::Up) => Ok(LocalFrameAxis::West),
            (LocalFrameAxis::South, LocalFrameAxis::Down) => Ok(LocalFrameAxis::East),
            (LocalFrameAxis::South, LocalFrameAxis::West) => Ok(LocalFrameAxis::Down),
            (LocalFrameAxis::South, LocalFrameAxis::East) => Ok(LocalFrameAxis::Up),

            (LocalFrameAxis::North, LocalFrameAxis::Up) => Ok(LocalFrameAxis::East),
            (LocalFrameAxis::North, LocalFrameAxis::Down) => Ok(LocalFrameAxis::West),
            (LocalFrameAxis::North, LocalFrameAxis::West) => Ok(LocalFrameAxis::Up),
            (LocalFrameAxis::North, LocalFrameAxis::East) => Ok(LocalFrameAxis::Down),

            (LocalFrameAxis::West, LocalFrameAxis::Up) => Ok(LocalFrameAxis::North),
            (LocalFrameAxis::West, LocalFrameAxis::Down) => Ok(LocalFrameAxis::South),
            (LocalFrameAxis::West, LocalFrameAxis::North) => Ok(LocalFrameAxis::Down),
            (LocalFrameAxis::West, LocalFrameAxis::South) => Ok(LocalFrameAxis::Up),

            (LocalFrameAxis::East, LocalFrameAxis::Up) => Ok(LocalFrameAxis::South),
            (LocalFrameAxis::East, LocalFrameAxis::Down) => Ok(LocalFrameAxis::North),
            (LocalFrameAxis::East, LocalFrameAxis::North) => Ok(LocalFrameAxis::Up),
            (LocalFrameAxis::East, LocalFrameAxis::South) => Ok(LocalFrameAxis::Down),

            _ => Err(DeveloperError::new(
                "firstAxis and secondAxis must be orthogonal valid axes.",
            )),
        }
    }

    #[inline]
    fn degenerate_vector(self) -> Cartesian3 {
        match self {
            LocalFrameAxis::North => Cartesian3::new(-1.0, 0.0, 0.0),
            LocalFrameAxis::East => Cartesian3::new(0.0, 1.0, 0.0),
            LocalFrameAxis::Up => Cartesian3::new(0.0, 0.0, 1.0),
            LocalFrameAxis::South => Cartesian3::new(1.0, 0.0, 0.0),
            LocalFrameAxis::West => Cartesian3::new(0.0, -1.0, 0.0),
            LocalFrameAxis::Down => Cartesian3::new(0.0, 0.0, -1.0),
        }
    }
}

/// Type alias for local-frame-to-fixed-frame generator functions.
pub type LocalFrameToFixedFrameFn = fn(Cartesian3, Option<Ellipsoid>) -> Result<Matrix4, DeveloperError>;

/// Contains functions for transforming positions to various reference frames.
pub struct FixedFrameTransforms;

impl FixedFrameTransforms {
    /// An immutable matrix that swaps x, y, z for 2D.
    pub const SWIZZLE_3D_TO_2D_MATRIX: Matrix4 = Matrix4::new(
        0.0, 0.0, 1.0, 0.0,
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    /// Computes a 4x4 transformation matrix from a reference frame centered at origin to fixed frame.
    pub fn local_frame_to_fixed_frame(
        first_axis: LocalFrameAxis,
        second_axis: LocalFrameAxis,
        origin: Cartesian3,
        ellipsoid: Option<Ellipsoid>,
    ) -> Result<Matrix4, DeveloperError> {
        if !origin.x.is_finite() || !origin.y.is_finite() || !origin.z.is_finite() {
            return Err(DeveloperError::new("origin has a NaN or non-finite component"));
        }

        let third_axis = first_axis.cross(second_axis)?;

        let (first_cart, second_cart, third_cart) = if origin.equals_epsilon(
            &Cartesian3::ZERO,
            Some(EPSILON14),
            None,
        ) {
            (
                first_axis.degenerate_vector(),
                second_axis.degenerate_vector(),
                third_axis.degenerate_vector(),
            )
        } else if origin.x.abs() <= EPSILON14 && origin.y.abs() <= EPSILON14 {
            let sign = if origin.z >= 0.0 { 1.0 } else { -1.0 };
            let apply_sign = |axis: LocalFrameAxis| {
                let v = axis.degenerate_vector();
                if axis != LocalFrameAxis::East && axis != LocalFrameAxis::West {
                    v * sign
                } else {
                    v
                }
            };
            (
                apply_sign(first_axis),
                apply_sign(second_axis),
                apply_sign(third_axis),
            )
        } else {
            let el = ellipsoid.unwrap_or(Ellipsoid::WGS84);
            let up_dvec = el
                .geodetic_surface_normal(DVec3::new(origin.x, origin.y, origin.z))
                .unwrap_or(DVec3::Z);
            let up = Cartesian3::new(up_dvec.x, up_dvec.y, up_dvec.z);

            let east_unnormalized = Cartesian3::new(-origin.y, origin.x, 0.0);
            let east = east_unnormalized.normalize();
            let north = up.cross(&east);

            let down = -up;
            let west = -east;
            let south = -north;

            let lookup = |axis: LocalFrameAxis| match axis {
                LocalFrameAxis::East => east,
                LocalFrameAxis::North => north,
                LocalFrameAxis::Up => up,
                LocalFrameAxis::West => west,
                LocalFrameAxis::South => south,
                LocalFrameAxis::Down => down,
            };

            (lookup(first_axis), lookup(second_axis), lookup(third_axis))
        };

        Ok(Matrix4::from_column_major_array([
            first_cart.x,
            first_cart.y,
            first_cart.z,
            0.0,
            second_cart.x,
            second_cart.y,
            second_cart.z,
            0.0,
            third_cart.x,
            third_cart.y,
            third_cart.z,
            0.0,
            origin.x,
            origin.y,
            origin.z,
            1.0,
        ]))
    }

    /// Computes east-north-up to fixed frame transform.
    #[inline]
    pub fn east_north_up_to_fixed_frame(
        origin: Cartesian3,
        ellipsoid: Option<Ellipsoid>,
    ) -> Result<Matrix4, DeveloperError> {
        Self::local_frame_to_fixed_frame(
            LocalFrameAxis::East,
            LocalFrameAxis::North,
            origin,
            ellipsoid,
        )
    }

    /// Computes north-east-down to fixed frame transform.
    #[inline]
    pub fn north_east_down_to_fixed_frame(
        origin: Cartesian3,
        ellipsoid: Option<Ellipsoid>,
    ) -> Result<Matrix4, DeveloperError> {
        Self::local_frame_to_fixed_frame(
            LocalFrameAxis::North,
            LocalFrameAxis::East,
            origin,
            ellipsoid,
        )
    }

    /// Computes north-up-east to fixed frame transform.
    #[inline]
    pub fn north_up_east_to_fixed_frame(
        origin: Cartesian3,
        ellipsoid: Option<Ellipsoid>,
    ) -> Result<Matrix4, DeveloperError> {
        Self::local_frame_to_fixed_frame(
            LocalFrameAxis::North,
            LocalFrameAxis::Up,
            origin,
            ellipsoid,
        )
    }

    /// Computes north-west-up to fixed frame transform.
    #[inline]
    pub fn north_west_up_to_fixed_frame(
        origin: Cartesian3,
        ellipsoid: Option<Ellipsoid>,
    ) -> Result<Matrix4, DeveloperError> {
        Self::local_frame_to_fixed_frame(
            LocalFrameAxis::North,
            LocalFrameAxis::West,
            origin,
            ellipsoid,
        )
    }

    /// Computes heading-pitch-roll to fixed frame transformation.
    pub fn heading_pitch_roll_to_fixed_frame(
        origin: Cartesian3,
        heading_pitch_roll: &HeadingPitchRoll,
        ellipsoid: Option<Ellipsoid>,
        fixed_frame_transform: Option<LocalFrameToFixedFrameFn>,
    ) -> Result<Matrix4, DeveloperError> {
        let hpr_quaternion = Quaternion::from_heading_pitch_roll(heading_pitch_roll);
        let hpr_matrix = Matrix4::from_translation_quaternion_rotation_scale(
            Cartesian3::ZERO,
            hpr_quaternion,
            Cartesian3::new(1.0, 1.0, 1.0),
        );
        let fft = fixed_frame_transform.unwrap_or(Self::east_north_up_to_fixed_frame);
        let base_transform = fft(origin, ellipsoid)?;
        Ok(base_transform.multiply(&hpr_matrix))
    }

    /// Computes heading-pitch-roll quaternion.
    pub fn heading_pitch_roll_quaternion(
        origin: Cartesian3,
        heading_pitch_roll: &HeadingPitchRoll,
        ellipsoid: Option<Ellipsoid>,
        fixed_frame_transform: Option<LocalFrameToFixedFrameFn>,
    ) -> Result<Quaternion, DeveloperError> {
        let transform = Self::heading_pitch_roll_to_fixed_frame(
            origin,
            heading_pitch_roll,
            ellipsoid,
            fixed_frame_transform,
        )?;
        let rotation = transform.get_matrix3();
        Ok(Quaternion::from_rotation_matrix(&rotation))
    }

    /// Computes heading-pitch-roll angles from a transform.
    pub fn fixed_frame_to_heading_pitch_roll(
        transform: &Matrix4,
        ellipsoid: Option<Ellipsoid>,
        fixed_frame_transform: Option<LocalFrameToFixedFrameFn>,
    ) -> Result<HeadingPitchRoll, DeveloperError> {
        let center = transform.get_translation();
        if center == Cartesian3::ZERO {
            return Ok(HeadingPitchRoll::new(0.0, 0.0, 0.0));
        }

        let fft = fixed_frame_transform.unwrap_or(Self::east_north_up_to_fixed_frame);
        let base_frame = fft(center, ellipsoid)?;
        let to_fixed_frame = base_frame
            .inverse_transformation()
            .map_err(|_| DeveloperError::new("Failed to invert fixed frame transform"))?;

        let mut transform_copy = *transform;
        transform_copy.set_scale(Cartesian3::new(1.0, 1.0, 1.0));
        transform_copy.set_translation(Cartesian3::ZERO);

        let local_frame = to_fixed_frame.multiply(&transform_copy);
        let rotation = local_frame.get_matrix3();
        let quaternion_rot = Quaternion::from_rotation_matrix(&rotation).normalize();

        Ok(HeadingPitchRoll::from_quaternion(&quaternion_rot))
    }

    /// Transform a point from model coordinates to OpenGL window coordinates.
    pub fn point_to_gl_window_coordinates(
        model_view_projection_matrix: &Matrix4,
        viewport_transformation: &Matrix4,
        point: Cartesian3,
    ) -> Cartesian2 {
        let tmp = model_view_projection_matrix.multiply_by_vector(Cartesian4::new(
            point.x, point.y, point.z, 1.0,
        ));
        let tmp_normalized = tmp.multiply_by_scalar(1.0 / tmp.w);
        let win = viewport_transformation.multiply_by_vector(tmp_normalized);
        Cartesian2::new(win.x, win.y)
    }

    /// Transform a point from model coordinates to window coordinates.
    pub fn point_to_window_coordinates(
        model_view_projection_matrix: &Matrix4,
        viewport_transformation: &Matrix4,
        point: Cartesian3,
    ) -> Cartesian2 {
        let mut result = Self::point_to_gl_window_coordinates(
            model_view_projection_matrix,
            viewport_transformation,
            point,
        );
        result.y = 2.0 * viewport_transformation.values[5] - result.y;
        result
    }

    /// Transform position and velocity to a rotation matrix.
    pub fn rotation_matrix_from_position_velocity(
        position: Cartesian3,
        velocity: Cartesian3,
        ellipsoid: Option<Ellipsoid>,
    ) -> Result<Matrix3, DeveloperError> {
        let el = ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let pos_dvec = DVec3::new(position.x, position.y, position.z);
        let normal_dvec = el
            .geodetic_surface_normal(pos_dvec)
            .unwrap_or(DVec3::Z);
        let normal = Cartesian3::new(normal_dvec.x, normal_dvec.y, normal_dvec.z);

        let mut right = velocity.cross(&normal);
        if right.equals_epsilon(&Cartesian3::ZERO, Some(EPSILON6), None) {
            right = Cartesian3::UNIT_X;
        }

        let up = right.cross(&velocity).normalize();
        let right_final = (-velocity.cross(&up)).normalize();

        Ok(Matrix3::from_column_major_array([
            velocity.x,
            velocity.y,
            velocity.z,
            right_final.x,
            right_final.y,
            right_final.z,
            up.x,
            up.y,
            up.z,
        ]))
    }

    /// Transforms a 3D matrix into 2D map projection coordinates.
    pub fn basis_to_2d<P: MapProjection>(
        projection: &P,
        matrix: &Matrix4,
    ) -> Result<Matrix4, DeveloperError> {
        let rtc_center = matrix.get_translation();
        let ellipsoid = projection.ellipsoid();

        let projected_position = if rtc_center == Cartesian3::ZERO {
            Cartesian3::ZERO
        } else {
            let center_dvec = DVec3::new(rtc_center.x, rtc_center.y, rtc_center.z);
            let cartographic = ellipsoid
                .cartesian_to_cartographic(center_dvec)
                .ok_or_else(|| DeveloperError::new("Failed to convert Cartesian3 to Cartographic"))?;

            let proj = projection
                .project(cartographic)
                .ok_or_else(|| DeveloperError::new("Projection failed"))?;
            Cartesian3::new(proj.z, proj.x, proj.y)
        };

        let from_enu = Self::east_north_up_to_fixed_frame(rtc_center, Some(ellipsoid))?;
        let to_enu = from_enu
            .inverse_transformation()
            .map_err(|_| DeveloperError::new("Failed to invert from_enu"))?;

        let rotation = matrix.get_matrix3();
        let local = to_enu.multiply_by_matrix3(&rotation);

        let mut result = Self::SWIZZLE_3D_TO_2D_MATRIX.multiply(&local);
        result.set_translation(projected_position);
        Ok(result)
    }

    /// Computes model matrix to transform vertices centered at origin to 2D.
    pub fn ellipsoid_to_2d_model_matrix<P: MapProjection>(
        projection: &P,
        center: Cartesian3,
    ) -> Result<Matrix4, DeveloperError> {
        let ellipsoid = projection.ellipsoid();
        let from_enu = Self::east_north_up_to_fixed_frame(center, Some(ellipsoid))?;
        let to_enu = from_enu
            .inverse_transformation()
            .map_err(|_| DeveloperError::new("Failed to invert from_enu"))?;

        let center_dvec = DVec3::new(center.x, center.y, center.z);
        let cartographic = ellipsoid
            .cartesian_to_cartographic(center_dvec)
            .ok_or_else(|| DeveloperError::new("Failed to convert Cartesian3 to Cartographic"))?;

        let proj = projection
            .project(cartographic)
            .ok_or_else(|| DeveloperError::new("Projection failed"))?;
        let projected_position = Cartesian3::new(proj.z, proj.x, proj.y);

        let translation = Matrix4::from_translation(projected_position);
        let res = Self::SWIZZLE_3D_TO_2D_MATRIX.multiply(&to_enu);
        Ok(translation.multiply(&res))
    }
}
