//! ScreenSpaceCameraControllerSpec matching CesiumJS `Scene/ScreenSpaceCameraControllerSpec.js` (Apache-2.0).

use cesium_math::Camera;
use cesium_renderer::ScreenSpaceCameraController;
use glam::{DVec2, DVec3};

fn create_test_camera() -> Camera {
    let eye = DVec3::new(0.0, 0.0, 10_000_000.0);
    let target = DVec3::ZERO;
    let up = DVec3::Y;
    Camera::look_at(eye, target, up, 1024, 768).unwrap()
}

#[test]
fn test_constructor_throws_without_scene() {
    assert!(ScreenSpaceCameraController::new(false).is_err());
    assert!(ScreenSpaceCameraController::new(true).is_ok());
}

#[test]
fn test_translate_in_2d() {
    let controller = ScreenSpaceCameraController::new(true).unwrap();
    let mut camera = create_test_camera();
    let initial_eye = camera.eye;

    controller.translate_2d(&mut camera, DVec2::new(100.0, 0.0));
    assert!(camera.eye.x > initial_eye.x);

    controller.translate_2d(&mut camera, DVec2::new(-200.0, 0.0));
    assert!(camera.eye.x < initial_eye.x);

    controller.translate_2d(&mut camera, DVec2::new(0.0, 50.0));
    assert!(camera.eye.y > initial_eye.y);
}

#[test]
fn test_zoom_in_2d() {
    let controller = ScreenSpaceCameraController::new(true).unwrap();
    let mut camera = create_test_camera();
    let initial_dist = camera.eye.length();

    // Zoom in (positive amount moves camera closer)
    controller.zoom_2d(&mut camera, 1000.0);
    assert!(camera.eye.length() < initial_dist);

    // Zoom out (negative amount moves camera further)
    controller.zoom_2d(&mut camera, -2000.0);
    assert!(camera.eye.length() > initial_dist);
}

#[test]
fn test_rotate_in_2d() {
    let controller = ScreenSpaceCameraController::new(true).unwrap();
    let mut camera = create_test_camera();
    let initial_up = camera.camera_to_ecef.y_axis;

    controller.rotate_2d(&mut camera, 0.5);
    let new_up = camera.camera_to_ecef.y_axis;
    assert_ne!(initial_up, new_up);
}

#[test]
fn test_translate_in_columbus_view() {
    let controller = ScreenSpaceCameraController::new(true).unwrap();
    let mut camera = create_test_camera();
    let initial_eye = camera.eye;

    controller.translate_columbus_view(&mut camera, DVec2::new(500.0, 300.0));
    assert_ne!(camera.eye, initial_eye);
}

#[test]
fn test_zoom_in_columbus_view() {
    let controller = ScreenSpaceCameraController::new(true).unwrap();
    let mut camera = create_test_camera();
    let initial_eye = camera.eye;

    controller.zoom_columbus_view(&mut camera, 500.0);
    assert_ne!(camera.eye, initial_eye);
}

#[test]
fn test_rotate_in_columbus_view() {
    let controller = ScreenSpaceCameraController::new(true).unwrap();
    let mut camera = create_test_camera();
    let initial_basis = camera.camera_to_ecef;

    controller.rotate_columbus_view(&mut camera, 0.2);
    assert_ne!(camera.camera_to_ecef, initial_basis);
}

#[test]
fn test_pan_rotate_zoom_in_3d() {
    let controller = ScreenSpaceCameraController::new(true).unwrap();
    let mut camera = create_test_camera();

    // 1. Pan 3D
    let initial_eye = camera.eye;
    controller.pan_3d(&mut camera, DVec2::new(10.0, 10.0));
    assert_ne!(camera.eye, initial_eye);

    // 2. Rotate 3D
    let initial_basis = camera.camera_to_ecef;
    controller.rotate_3d(&mut camera, DVec2::new(20.0, -10.0));
    assert_ne!(camera.camera_to_ecef, initial_basis);

    // 3. Zoom 3D
    let pre_zoom_dist = camera.eye.length();
    controller.zoom_3d(&mut camera, 2000.0);
    assert!(camera.eye.length() < pre_zoom_dist);

    // 4. Tilt 3D
    let pre_tilt_basis = camera.camera_to_ecef;
    controller.tilt_3d(&mut camera, 0.1);
    assert_ne!(camera.camera_to_ecef, pre_tilt_basis);

    // 5. Look 3D
    let pre_look_basis = camera.camera_to_ecef;
    controller.look_3d(&mut camera, DVec2::new(0.05, 0.05));
    assert_ne!(camera.camera_to_ecef, pre_look_basis);
}

#[test]
fn test_disable_inputs_prevents_camera_modification() {
    let mut controller = ScreenSpaceCameraController::new(true).unwrap();
    let mut camera = create_test_camera();

    controller.enable_inputs = false;
    let initial_eye = camera.eye;
    let initial_basis = camera.camera_to_ecef;

    controller.translate_2d(&mut camera, DVec2::new(100.0, 100.0));
    controller.zoom_2d(&mut camera, 1000.0);
    controller.rotate_2d(&mut camera, 0.5);
    controller.pan_3d(&mut camera, DVec2::new(50.0, 50.0));
    controller.rotate_3d(&mut camera, DVec2::new(50.0, 50.0));
    controller.zoom_3d(&mut camera, 500.0);
    controller.tilt_3d(&mut camera, 0.2);
    controller.look_3d(&mut camera, DVec2::new(0.1, 0.1));

    assert_eq!(camera.eye, initial_eye);
    assert_eq!(camera.camera_to_ecef, initial_basis);
}

#[test]
fn test_destroys() {
    let mut controller = ScreenSpaceCameraController::new(true).unwrap();
    assert!(!controller.is_destroyed());
    controller.destroy();
    assert!(controller.is_destroyed());
}
