//! ScreenSpaceCameraController matching CesiumJS `Scene/ScreenSpaceCameraController.js` (Apache-2.0).

use cesium_math::{Camera, DeveloperError};
use glam::{DMat3, DQuat, DVec2, DVec3};

/// Modifies the camera position and orientation based on mouse, touch, or keyboard input.
pub struct ScreenSpaceCameraController {
    /// If true, inputs are allowed. If false, all inputs are disabled.
    pub enable_inputs: bool,
    /// If true, allows the user to pan around the map in 2D and Columbus view.
    pub enable_translate: bool,
    /// If true, allows the user to zoom in and out.
    pub enable_zoom: bool,
    /// If true, allows the user to rotate the world.
    pub enable_rotate: bool,
    /// If true, allows the user to tilt the camera in 3D and Columbus view.
    pub enable_tilt: bool,
    /// If true, allows the user to use free-look in 3D and Columbus view.
    pub enable_look: bool,
    /// Inertia parameter for spinning [0, 1).
    pub inertia_spin: f64,
    /// Inertia parameter for translating [0, 1).
    pub inertia_translate: f64,
    /// Inertia parameter for zooming [0, 1).
    pub inertia_zoom: f64,
    /// Maximum movement ratio per animation frame [0, 1).
    pub maximum_movement_ratio: f64,
    /// Duration in seconds of bounce back animations.
    pub bounce_animation_time: f64,
    /// The minimum distance in meters of camera position when zooming.
    pub minimum_zoom_distance: f64,
    /// The maximum distance in meters of camera position when zooming.
    pub maximum_zoom_distance: f64,
    /// Multiplier for zoom speed.
    pub zoom_factor: f64,
    is_destroyed: bool,
}

impl ScreenSpaceCameraController {
    /// Creates a new `ScreenSpaceCameraController`.
    /// Throws DeveloperError if scene is not provided.
    pub fn new(has_scene: bool) -> Result<Self, DeveloperError> {
        if !has_scene {
            return Err(DeveloperError::new("scene is required."));
        }

        Ok(Self {
            enable_inputs: true,
            enable_translate: true,
            enable_zoom: true,
            enable_rotate: true,
            enable_tilt: true,
            enable_look: true,
            inertia_spin: 0.9,
            inertia_translate: 0.9,
            inertia_zoom: 0.8,
            maximum_movement_ratio: 0.1,
            bounce_animation_time: 3.0,
            minimum_zoom_distance: 1.0,
            maximum_zoom_distance: f64::INFINITY,
            zoom_factor: 5.0,
            is_destroyed: false,
        })
    }

    /// Translates camera in 2D space.
    pub fn translate_2d(&self, camera: &mut Camera, delta: DVec2) {
        if !self.enable_inputs || !self.enable_translate {
            return;
        }

        let right = camera.camera_to_ecef.x_axis;
        let up = camera.camera_to_ecef.y_axis;

        let move_x = right * delta.x;
        let move_y = up * delta.y;
        camera.eye += move_x + move_y;
    }

    /// Zooms camera in 2D space.
    pub fn zoom_2d(&self, camera: &mut Camera, amount: f64) {
        if !self.enable_inputs || !self.enable_zoom {
            return;
        }

        let dir = camera.camera_to_ecef.z_axis;
        let dist = camera.eye.length();
        let scaled_amount = amount * self.zoom_factor;

        let new_dist = (dist - scaled_amount).clamp(self.minimum_zoom_distance, self.maximum_zoom_distance);
        if dist > 0.0 {
            camera.eye = camera.eye.normalize() * new_dist;
        } else {
            camera.eye -= dir * scaled_amount;
        }
    }

    /// Rotates camera in 2D space (twist around look axis).
    pub fn rotate_2d(&self, camera: &mut Camera, angle: f64) {
        if !self.enable_inputs || !self.enable_rotate {
            return;
        }

        let z_axis = camera.camera_to_ecef.z_axis;
        let rot = DQuat::from_axis_angle(z_axis, angle);
        let rot_mat = DMat3::from_quat(rot);

        camera.camera_to_ecef = rot_mat * camera.camera_to_ecef;
    }

    /// Translates camera in Columbus View.
    pub fn translate_columbus_view(&self, camera: &mut Camera, delta: DVec2) {
        if !self.enable_inputs || !self.enable_translate {
            return;
        }

        let right = camera.camera_to_ecef.x_axis;
        let up = camera.camera_to_ecef.y_axis;
        camera.eye += right * delta.x + up * delta.y;
    }

    /// Zooms camera in Columbus View.
    pub fn zoom_columbus_view(&self, camera: &mut Camera, amount: f64) {
        if !self.enable_inputs || !self.enable_zoom {
            return;
        }

        let look_dir = -camera.camera_to_ecef.z_axis;
        let move_vec = look_dir * (amount * self.zoom_factor);
        camera.eye += move_vec;
    }

    /// Rotates camera in Columbus View.
    pub fn rotate_columbus_view(&self, camera: &mut Camera, angle: f64) {
        if !self.enable_inputs || !self.enable_rotate {
            return;
        }

        let z_axis = DVec3::Z;
        let rot = DQuat::from_axis_angle(z_axis, angle);
        let rot_mat = DMat3::from_quat(rot);

        camera.camera_to_ecef = rot_mat * camera.camera_to_ecef;
    }

    /// Pans camera in 3D around globe.
    pub fn pan_3d(&self, camera: &mut Camera, delta: DVec2) {
        if !self.enable_inputs || !self.enable_translate {
            return;
        }

        let right = camera.camera_to_ecef.x_axis;
        let up = camera.camera_to_ecef.y_axis;

        let pan_scale = camera.eye.length() * 0.001;
        camera.eye -= (right * delta.x + up * delta.y) * pan_scale;
    }

    /// Rotates camera in 3D around look target or globe.
    pub fn rotate_3d(&self, camera: &mut Camera, delta: DVec2) {
        if !self.enable_inputs || !self.enable_rotate {
            return;
        }

        let angle_x = -delta.x * 0.005;
        let angle_y = -delta.y * 0.005;

        let up = camera.camera_to_ecef.y_axis;
        let right = camera.camera_to_ecef.x_axis;

        let q_yaw = DQuat::from_axis_angle(up, angle_x);
        let q_pitch = DQuat::from_axis_angle(right, angle_y);
        let rot = q_pitch * q_yaw;
        let rot_mat = DMat3::from_quat(rot);

        camera.camera_to_ecef = rot_mat * camera.camera_to_ecef;
    }

    /// Zooms camera in 3D toward target.
    pub fn zoom_3d(&self, camera: &mut Camera, amount: f64) {
        if !self.enable_inputs || !self.enable_zoom {
            return;
        }

        let look_dir = -camera.camera_to_ecef.z_axis;
        let dist = camera.eye.length();
        let scaled_amount = amount * self.zoom_factor;

        let new_dist = (dist - scaled_amount).clamp(self.minimum_zoom_distance, self.maximum_zoom_distance);
        camera.eye = if dist > 0.0 {
            camera.eye.normalize() * new_dist
        } else {
            camera.eye + look_dir * scaled_amount
        };
    }

    /// Tilts camera in 3D.
    pub fn tilt_3d(&self, camera: &mut Camera, delta: f64) {
        if !self.enable_inputs || !self.enable_tilt {
            return;
        }

        let right = camera.camera_to_ecef.x_axis;
        let rot = DQuat::from_axis_angle(right, delta);
        let rot_mat = DMat3::from_quat(rot);

        camera.camera_to_ecef = rot_mat * camera.camera_to_ecef;
    }

    /// Looks camera freely in 3D.
    pub fn look_3d(&self, camera: &mut Camera, delta: DVec2) {
        if !self.enable_inputs || !self.enable_look {
            return;
        }

        let right = camera.camera_to_ecef.x_axis;
        let up = camera.camera_to_ecef.y_axis;

        let q_yaw = DQuat::from_axis_angle(up, -delta.x);
        let q_pitch = DQuat::from_axis_angle(right, -delta.y);
        let rot = q_pitch * q_yaw;
        let rot_mat = DMat3::from_quat(rot);

        camera.camera_to_ecef = rot_mat * camera.camera_to_ecef;
    }

    /// Whether this controller is destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys this controller.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}
