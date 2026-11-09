use cesium_math::{Cartesian3, DeveloperError};
use glam::{DMat3, DQuat, DVec3};

/// Represents orientation angles (in degrees) from device orientation sensors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceOrientationEvent {
    /// Motion around Z axis [0, 360).
    pub alpha: f64,
    /// Motion around X axis [-180, 180).
    pub beta: f64,
    /// Motion around Y axis [-90, 90).
    pub gamma: f64,
}

/// A minimal camera orientation state compatible with `DeviceOrientationCameraController`.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraOrientationState {
    /// Camera world position.
    pub position: Cartesian3,
    /// Camera view direction (normalized).
    pub direction: Cartesian3,
    /// Camera up direction (normalized).
    pub up: Cartesian3,
    /// Camera right direction (normalized).
    pub right: Cartesian3,
}

impl CameraOrientationState {
    /// Creates a new `CameraOrientationState`.
    pub fn new(position: Cartesian3, direction: Cartesian3, up: Cartesian3) -> Self {
        let right = direction.cross(&up).normalize();
        Self {
            position,
            direction,
            up,
            right,
        }
    }
}

/// Rotates camera axes using alpha, beta, gamma device orientation changes.
pub fn rotate_camera(
    camera: &mut CameraOrientationState,
    alpha: f64,
    beta: f64,
    gamma: f64,
) {
    let dir = DVec3::new(camera.direction.x, camera.direction.y, camera.direction.z);
    let right = DVec3::new(camera.right.x, camera.right.y, camera.right.z);
    let up = DVec3::new(camera.up.x, camera.up.y, camera.up.z);

    let b_quat = DQuat::from_axis_angle(dir, beta);
    let g_quat = DQuat::from_axis_angle(right, gamma);
    let rot_quat = g_quat * b_quat;

    let a_quat = DQuat::from_axis_angle(up, alpha);
    let rot_quat = a_quat * rot_quat;

    let matrix = DMat3::from_quat(rot_quat);
    let new_right = matrix * right;
    let new_up = matrix * up;
    let new_dir = matrix * dir;

    camera.right = Cartesian3::new(new_right.x, new_right.y, new_right.z);
    camera.up = Cartesian3::new(new_up.x, new_up.y, new_up.z);
    camera.direction = Cartesian3::new(new_dir.x, new_dir.y, new_dir.z);
}

/// Controls the camera orientation based on device orientation sensor inputs.
#[derive(Debug, Clone, Default)]
pub struct DeviceOrientationCameraController {
    last_alpha: Option<f64>,
    last_beta: Option<f64>,
    last_gamma: Option<f64>,
    alpha: Option<f64>,
    beta: Option<f64>,
    gamma: Option<f64>,
    is_destroyed: bool,
}

impl DeviceOrientationCameraController {
    /// Creates a new `DeviceOrientationCameraController`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Receives orientation angles in degrees from the device.
    pub fn on_device_orientation(
        &mut self,
        event: Option<DeviceOrientationEvent>,
    ) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        if let Some(e) = event {
            self.alpha = Some(e.alpha.to_radians());
            self.beta = Some(e.beta.to_radians());
            self.gamma = Some(e.gamma.to_radians());
        } else {
            self.alpha = None;
            self.beta = None;
            self.gamma = None;
        }

        Ok(())
    }

    /// Updates the camera orientation based on accumulated delta rotation.
    pub fn update(
        &mut self,
        camera: &mut CameraOrientationState,
    ) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        let alpha = match self.alpha {
            Some(a) => a,
            None => return Ok(()),
        };
        let beta = self.beta.unwrap_or(0.0);
        let gamma = self.gamma.unwrap_or(0.0);

        if self.last_alpha.is_none() {
            self.last_alpha = Some(alpha);
            self.last_beta = Some(beta);
            self.last_gamma = Some(gamma);
            return Ok(());
        }

        let last_a = self.last_alpha.unwrap();
        let last_b = self.last_beta.unwrap();
        let last_g = self.last_gamma.unwrap();

        let a = last_a - alpha;
        let b = last_b - beta;
        let g = last_g - gamma;

        rotate_camera(camera, -a, b, g);

        self.last_alpha = Some(alpha);
        self.last_beta = Some(beta);
        self.last_gamma = Some(gamma);

        Ok(())
    }

    /// Returns true if destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys the controller.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}
