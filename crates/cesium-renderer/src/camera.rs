//! Scene Camera implementation matching CesiumJS `Scene/Camera.js` (Apache-2.0).

use cesium_math::{
    zero_to_two_pi, Cartesian2, Cartesian3, DeveloperError, Ellipsoid,
    FixedFrameTransforms, HeadingPitchRange, HeadingPitchRoll, Matrix3, Matrix4,
    PerspectiveFrustum, Quaternion, Ray, SceneMode, PI, TWO_PI,
};
use glam::DVec3;

use crate::camera_flight_path::CameraFrustum;
use crate::cesium_widget::MapMode2D;

/// Canvas dimensions associated with a scene camera.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraCanvas {
    pub client_width: f64,
    pub client_height: f64,
}

impl Default for CameraCanvas {
    fn default() -> Self {
        Self {
            client_width: 512.0,
            client_height: 384.0,
        }
    }
}

/// Orientation specification options for [`Camera::set_view`].
#[derive(Clone, Debug)]
pub enum CameraOrientation {
    HeadingPitchRoll(HeadingPitchRoll),
    DirectionUp { direction: Cartesian3, up: Cartesian3 },
    HeadingPitchRange(HeadingPitchRange),
}

/// View options passed into [`Camera::set_view`].
#[derive(Clone, Debug, Default)]
pub struct CameraViewOptions {
    pub destination: Option<Cartesian3>,
    pub orientation: Option<CameraOrientation>,
    pub end_transform: Option<Matrix4>,
}

/// Bounding sphere used for camera distance calculations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingSphere {
    pub center: Cartesian3,
    pub radius: f64,
}

impl BoundingSphere {
    pub fn new(center: Cartesian3, radius: f64) -> Self {
        Self { center, radius }
    }
}

/// Core Scene Camera managing position, orientation, frustum, and coordinate transformations.
#[derive(Clone, Debug)]
pub struct Camera {
    pub position: Cartesian3,
    pub direction: Cartesian3,
    pub up: Cartesian3,
    pub right: Cartesian3,
    pub transform: Matrix4,
    pub inverse_transform: Matrix4,
    pub frustum: CameraFrustum,
    pub mode: SceneMode,
    pub map_mode_2d: MapMode2D,
    pub canvas: CameraCanvas,
    pub ellipsoid: Ellipsoid,
    pub minimum_zoom_distance: f64,
    pub maximum_zoom_distance: f64,
    pub default_move_amount: f64,
    pub default_look_amount: f64,
    pub default_rotate_amount: f64,
    pub default_zoom_amount: f64,
    pub constrained_axis: Option<Cartesian3>,
}

impl Camera {
    /// Creates a camera associated with a canvas.
    /// Throws DeveloperError if canvas is None or has non-positive dimensions.
    pub fn new(canvas: Option<CameraCanvas>) -> Result<Self, DeveloperError> {
        let canvas = canvas.ok_or_else(|| DeveloperError::new("canvas is required"))?;
        if canvas.client_width <= 0.0 || canvas.client_height <= 0.0 {
            return Err(DeveloperError::new("canvas must have positive dimensions"));
        }

        let position = Cartesian3::new(0.0, 0.0, 0.0);
        let direction = Cartesian3::new(0.0, 0.0, -1.0);
        let up = Cartesian3::new(0.0, 1.0, 0.0);
        let right = Cartesian3::new(1.0, 0.0, 0.0);

        Ok(Self {
            position,
            direction,
            up,
            right,
            transform: Matrix4::IDENTITY,
            inverse_transform: Matrix4::IDENTITY,
            frustum: CameraFrustum::Perspective(PerspectiveFrustum::default()),
            mode: SceneMode::Scene3D,
            map_mode_2d: MapMode2D::InfiniteScroll,
            canvas,
            ellipsoid: Ellipsoid::WGS84,
            minimum_zoom_distance: 1.0,
            maximum_zoom_distance: 5906376272000.0,
            default_move_amount: 100000.0,
            default_look_amount: PI / 60.0,
            default_rotate_amount: PI / 3600.0,
            default_zoom_amount: 100000.0,
            constrained_axis: None,
        })
    }

    /// Normalizes direction, up, and right vectors and ensures orthonormality.
    pub fn update_members(&mut self) {
        self.direction = self.direction.normalize();
        let right = self.direction.cross(&self.up);
        self.right = right.normalize();
        let up = self.right.cross(&self.direction);
        self.up = up.normalize();
    }

    /// Computes the 4x4 view matrix from camera right, up, direction, and position.
    pub fn view_matrix(&mut self) -> Matrix4 {
        self.update_members();
        self.compute_view_matrix()
    }

    /// Computes the 4x4 view matrix without mutating the camera.
    pub fn compute_view_matrix(&self) -> Matrix4 {
        let r = self.right;
        let u = self.up;
        let d = self.direction;
        let p = self.position;

        let rotation = Matrix4::new(
            r.x, r.y, r.z, 0.0,
            u.x, u.y, u.z, 0.0,
            -d.x, -d.y, -d.z, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        let translation = Matrix4::new(
            1.0, 0.0, 0.0, -p.x,
            0.0, 1.0, 0.0, -p.y,
            0.0, 0.0, 1.0, -p.z,
            0.0, 0.0, 0.0, 1.0,
        );
        rotation.multiply(&translation)
    }

    /// Computes the inverse view matrix.
    pub fn inverse_view_matrix(&self) -> Matrix4 {
        self.compute_view_matrix().inverse().unwrap_or(Matrix4::IDENTITY)
    }

    /// Sets the reference frame transform matrix and updates the inverse transform.
    pub fn set_transform(&mut self, transform: Matrix4) {
        self.transform = transform;
        self.inverse_transform = transform.inverse_transformation().unwrap_or(Matrix4::IDENTITY);
    }

    /// Position in world coordinates (transformed by `transform`).
    #[inline]
    pub fn position_wc(&self) -> Cartesian3 {
        self.transform.multiply_by_point(self.position)
    }

    /// Direction in world coordinates.
    #[inline]
    pub fn direction_wc(&self) -> Cartesian3 {
        self.transform.multiply_by_direction(self.direction)
    }

    /// Up vector in world coordinates.
    #[inline]
    pub fn up_wc(&self) -> Cartesian3 {
        self.transform.multiply_by_direction(self.up)
    }

    /// Right vector in world coordinates.
    #[inline]
    pub fn right_wc(&self) -> Cartesian3 {
        self.transform.multiply_by_direction(self.right)
    }

    /// Heading angle in radians. Returns None in morphing mode.
    pub fn heading(&self) -> Option<f64> {
        match self.mode {
            SceneMode::Morphing => None,
            SceneMode::Scene2D | SceneMode::ColumbusView => {
                let angle = zero_to_two_pi(self.right.y.atan2(self.right.x));
                Some(TWO_PI - angle)
            }
            SceneMode::Scene3D => {
                let to_fixed_frame = FixedFrameTransforms::east_north_up_to_fixed_frame(
                    self.position,
                    Some(self.ellipsoid),
                )
                .unwrap_or(Matrix4::IDENTITY);
                let transform = to_fixed_frame.get_matrix3();
                let transform_t = transform.transpose();
                let right_enu = transform_t.multiply_by_vector(self.right);
                let angle = zero_to_two_pi(right_enu.y.atan2(right_enu.x));
                Some(TWO_PI - angle)
            }
        }
    }

    /// Sets camera heading angle in radians.
    pub fn set_heading(&mut self, heading: f64) {
        match self.mode {
            SceneMode::Morphing => {}
            SceneMode::Scene2D => {
                if self.map_mode_2d == MapMode2D::Rotate {
                    let rot_angle = TWO_PI - heading;
                    self.right = Cartesian3::new(rot_angle.cos(), rot_angle.sin(), 0.0);
                    self.up = Cartesian3::new(-rot_angle.sin(), rot_angle.cos(), 0.0);
                    self.direction = -Cartesian3::UNIT_Z;
                }
            }
            SceneMode::ColumbusView => {
                let rot_angle = TWO_PI - heading;
                self.right = Cartesian3::new(rot_angle.cos(), rot_angle.sin(), 0.0);
                self.up = Cartesian3::new(-rot_angle.sin(), rot_angle.cos(), 0.0);
                self.direction = -Cartesian3::UNIT_Z;
            }
            SceneMode::Scene3D => {
                let current = self.heading().unwrap_or(0.0);
                let angle = heading - current;
                let normal = self
                    .ellipsoid
                    .geodetic_surface_normal(DVec3::from(self.position))
                    .map(Cartesian3::from)
                    .unwrap_or(Cartesian3::UNIT_Z);
                self.rotate(normal, -angle);
            }
        }
    }

    /// Pitch angle in radians. Undefined (None) if not 3D or Columbus View.
    pub fn pitch(&self) -> Option<f64> {
        match self.mode {
            SceneMode::Scene2D | SceneMode::Morphing => None,
            SceneMode::ColumbusView => {
                Some(self.direction.z.clamp(-1.0, 1.0).asin())
            }
            SceneMode::Scene3D => {
                let to_fixed_frame = FixedFrameTransforms::east_north_up_to_fixed_frame(
                    self.position,
                    Some(self.ellipsoid),
                )
                .unwrap_or(Matrix4::IDENTITY);
                let transform = to_fixed_frame.get_matrix3();
                let transform_t = transform.transpose();
                let dir_enu = transform_t.multiply_by_vector(self.direction);
                Some(dir_enu.z.clamp(-1.0, 1.0).asin())
            }
        }
    }

    /// Sets camera pitch angle in radians.
    pub fn set_pitch(&mut self, pitch: f64) {
        if let Some(current) = self.pitch() {
            let angle = pitch - current;
            self.rotate(self.right, angle);
        }
    }

    /// Roll angle in radians. Undefined (None) if not 3D or Columbus View.
    pub fn roll(&self) -> Option<f64> {
        match self.mode {
            SceneMode::Scene2D | SceneMode::Morphing => None,
            SceneMode::ColumbusView => {
                let roll = self.right.z.clamp(-1.0, 1.0).asin();
                Some(roll)
            }
            SceneMode::Scene3D => {
                let to_fixed_frame = FixedFrameTransforms::east_north_up_to_fixed_frame(
                    self.position,
                    Some(self.ellipsoid),
                )
                .unwrap_or(Matrix4::IDENTITY);
                let transform = to_fixed_frame.get_matrix3();
                let transform_t = transform.transpose();
                let right_enu = transform_t.multiply_by_vector(self.right);
                Some(right_enu.z.clamp(-1.0, 1.0).asin())
            }
        }
    }

    /// Sets camera roll angle in radians.
    pub fn set_roll(&mut self, roll: f64) {
        if let Some(current) = self.roll() {
            let angle = roll - current;
            self.rotate(self.direction, angle);
        }
    }

    /// Validates scene update mode.
    pub fn update(&mut self, mode: Option<SceneMode>) -> Result<(), DeveloperError> {
        let m = mode.ok_or_else(|| DeveloperError::new("mode is required"))?;
        self.mode = m;
        self.update_members();
        Ok(())
    }

    /// Translates camera position along arbitrary direction vector.
    pub fn move_direction(&mut self, direction: Cartesian3, amount: f64) {
        self.position = self.position + direction * amount;
    }

    /// Moves forward along camera direction vector.
    pub fn move_forward(&mut self, amount: f64) {
        self.move_direction(self.direction, amount);
    }

    /// Moves backward along camera direction vector.
    pub fn move_backward(&mut self, amount: f64) {
        self.move_direction(self.direction, -amount);
    }

    /// Moves up along camera up vector.
    pub fn move_up(&mut self, amount: f64) {
        self.move_direction(self.up, amount);
    }

    /// Moves down along camera up vector.
    pub fn move_down(&mut self, amount: f64) {
        self.move_direction(self.up, -amount);
    }

    /// Moves right along camera right vector.
    pub fn move_right(&mut self, amount: f64) {
        self.move_direction(self.right, amount);
    }

    /// Moves left along camera right vector.
    pub fn move_left(&mut self, amount: f64) {
        self.move_direction(self.right, -amount);
    }

    /// Rotates camera axes and position around an arbitrary axis vector.
    pub fn rotate(&mut self, axis: Cartesian3, angle: f64) {
        let q = Quaternion::from_axis_angle(axis, angle);
        let m = Matrix3::from_quaternion(&q);
        self.position = m.multiply_by_vector(self.position);
        self.direction = m.multiply_by_vector(self.direction);
        self.up = m.multiply_by_vector(self.up);
        self.right = m.multiply_by_vector(self.right);
        self.update_members();
    }

    /// Rotates up around right vector.
    pub fn rotate_up(&mut self, angle: f64) {
        self.rotate(self.right, -angle);
    }

    /// Rotates down around right vector.
    pub fn rotate_down(&mut self, angle: f64) {
        self.rotate(self.right, angle);
    }

    /// Rotates right around up vector.
    pub fn rotate_right(&mut self, angle: f64) {
        self.rotate(self.up, angle);
    }

    /// Rotates left around up vector.
    pub fn rotate_left(&mut self, angle: f64) {
        self.rotate(self.up, -angle);
    }

    /// Zooms in along camera direction.
    pub fn zoom_in(&mut self, amount: f64) {
        self.move_forward(amount);
    }

    /// Zooms out along camera direction.
    pub fn zoom_out(&mut self, amount: f64) {
        self.move_backward(amount);
    }

    /// Rotates camera axes (direction, up, right) around an arbitrary axis without moving position.
    pub fn look(&mut self, axis: Cartesian3, angle: f64) {
        let q = Quaternion::from_axis_angle(axis, angle);
        let m = Matrix3::from_quaternion(&q);
        self.direction = m.multiply_by_vector(self.direction);
        self.up = m.multiply_by_vector(self.up);
        self.right = m.multiply_by_vector(self.right);
        self.update_members();
    }

    /// Looks up around right vector.
    pub fn look_up(&mut self, amount: f64) {
        self.look(self.right, -amount);
    }

    /// Looks down around right vector.
    pub fn look_down(&mut self, amount: f64) {
        self.look(self.right, amount);
    }

    /// Looks right around up vector.
    pub fn look_right(&mut self, amount: f64) {
        self.look(self.up, amount);
    }

    /// Looks left around up vector.
    pub fn look_left(&mut self, amount: f64) {
        self.look(self.up, -amount);
    }

    /// Twists left around direction vector.
    pub fn twist_left(&mut self, amount: f64) {
        self.look(self.direction, amount);
    }

    /// Twists right around direction vector.
    pub fn twist_right(&mut self, amount: f64) {
        self.look(self.direction, -amount);
    }

    /// Sets camera view with destination and orientation options.
    pub fn set_view(&mut self, options: CameraViewOptions) {
        if let Some(dest) = options.destination {
            self.position = dest;
        }

        if let Some(orient) = options.orientation {
            match orient {
                CameraOrientation::HeadingPitchRoll(hpr) => {
                    self.set_heading(hpr.heading);
                    self.set_pitch(hpr.pitch);
                    self.set_roll(hpr.roll);
                }
                CameraOrientation::DirectionUp { direction, up } => {
                    self.direction = direction;
                    self.up = up;
                    self.update_members();
                }
                CameraOrientation::HeadingPitchRange(hpr) => {
                    let target = self.position;
                    self.look_at(target, hpr);
                }
            }
        }

        if let Some(end_transform) = options.end_transform {
            self.set_transform(end_transform);
        }
    }

    /// Focuses camera onto a target position with heading, pitch, and range offset.
    pub fn look_at(&mut self, target: Cartesian3, offset: HeadingPitchRange) {
        let heading = offset.heading;
        let pitch = offset.pitch;
        let range = offset.range;

        let to_fixed_frame = FixedFrameTransforms::east_north_up_to_fixed_frame(
            target,
            Some(self.ellipsoid),
        )
        .unwrap_or(Matrix4::IDENTITY);
        let rot = to_fixed_frame.get_matrix3();

        let cos_p = pitch.cos();
        let sin_p = pitch.sin();
        let cos_h = heading.cos();
        let sin_h = heading.sin();

        // Local ENU offset: x = east, y = north, z = up
        let local_offset = Cartesian3::new(
            range * cos_p * sin_h,
            -range * cos_p * cos_h,
            -range * sin_p,
        );

        let world_offset = rot.multiply_by_vector(local_offset);
        self.position = target + world_offset;
        let dir = target - self.position;
        self.direction = dir.normalize();
        let normal = self
            .ellipsoid
            .geodetic_surface_normal(DVec3::from(target))
            .map(Cartesian3::from)
            .unwrap_or(Cartesian3::UNIT_Z);
        self.up = normal;
        self.update_members();
    }

    /// Sets view with transform and offset.
    pub fn look_at_transform(&mut self, transform: Matrix4, offset: Option<HeadingPitchRange>) {
        self.set_transform(transform);
        if let Some(hpr) = offset {
            self.look_at(Cartesian3::ZERO, hpr);
        }
    }

    /// Computes a pick ray from 2D canvas pixel coordinates.
    pub fn get_pick_ray(&self, window_position: Cartesian2) -> Option<Ray> {
        let x = (2.0 * window_position.x / self.canvas.client_width) - 1.0;
        let y = 1.0 - (2.0 * window_position.y / self.canvas.client_height);

        let origin = self.position_wc();
        let ray_dir = (self.direction_wc() + self.right_wc() * x + self.up_wc() * y).normalize();
        Ray::new(DVec3::from(origin), DVec3::from(ray_dir)).ok()
    }

    /// Computes distance from camera position to the exterior of a bounding sphere.
    pub fn distance_to_bounding_sphere(&self, sphere: &BoundingSphere) -> f64 {
        let d = self.position.distance(&sphere.center);
        (d - sphere.radius).max(0.0)
    }
}
