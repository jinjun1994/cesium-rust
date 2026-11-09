use cesium_math::{Cartesian3, EPSILON14};
use cesium_renderer::{
    CameraOrientationState, DeviceOrientationCameraController, DeviceOrientationEvent,
};

fn make_camera() -> CameraOrientationState {
    // position = (-1, 0, 0), direction = (1, 0, 0), up = (0, 0, 1)
    CameraOrientationState::new(
        Cartesian3::new(-1.0, 0.0, 0.0),
        Cartesian3::UNIT_X,
        Cartesian3::UNIT_Z,
    )
}

fn fire_event(
    controller: &mut DeviceOrientationCameraController,
    camera: &mut CameraOrientationState,
    event: DeviceOrientationEvent,
) {
    // set default orientation (alpha = 0, beta = 0, gamma = 0)
    controller
        .on_device_orientation(Some(DeviceOrientationEvent {
            alpha: 0.0,
            beta: 0.0,
            gamma: 0.0,
        }))
        .unwrap();
    controller.update(camera).unwrap();

    // update delta orientation
    controller.on_device_orientation(Some(event)).unwrap();
    controller.update(camera).unwrap();
}

fn assert_cartesian_epsilon(actual: Cartesian3, expected: Cartesian3, epsilon: f64) {
    assert!(
        (actual.x - expected.x).abs() <= epsilon,
        "x: actual {}, expected {}",
        actual.x,
        expected.x
    );
    assert!(
        (actual.y - expected.y).abs() <= epsilon,
        "y: actual {}, expected {}",
        actual.y,
        expected.y
    );
    assert!(
        (actual.z - expected.z).abs() <= epsilon,
        "z: actual {}, expected {}",
        actual.z,
        expected.z
    );
}

#[test]
fn test_rotates_for_alpha() {
    let mut controller = DeviceOrientationCameraController::new();
    let mut camera = make_camera();

    let initial_pos = camera.position;
    let initial_up = camera.up;

    fire_event(
        &mut controller,
        &mut camera,
        DeviceOrientationEvent {
            alpha: 90.0,
            beta: 0.0,
            gamma: 0.0,
        },
    );

    assert_eq!(camera.position, initial_pos);
    assert_cartesian_epsilon(camera.direction, Cartesian3::UNIT_Y, EPSILON14);
    assert_cartesian_epsilon(camera.up, initial_up, EPSILON14);
    assert_cartesian_epsilon(camera.right, Cartesian3::UNIT_X, EPSILON14);
}

#[test]
fn test_rotates_for_beta() {
    let mut controller = DeviceOrientationCameraController::new();
    let mut camera = make_camera();

    let initial_pos = camera.position;
    let initial_dir = camera.direction;

    fire_event(
        &mut controller,
        &mut camera,
        DeviceOrientationEvent {
            alpha: 0.0,
            beta: 90.0,
            gamma: 0.0,
        },
    );

    assert_eq!(camera.position, initial_pos);
    assert_cartesian_epsilon(camera.direction, initial_dir, EPSILON14);
    assert_cartesian_epsilon(camera.up, Cartesian3::UNIT_Y, EPSILON14);
    assert_cartesian_epsilon(camera.right, Cartesian3::UNIT_Z, EPSILON14);
}

#[test]
fn test_rotates_for_gamma() {
    let mut controller = DeviceOrientationCameraController::new();
    let mut camera = make_camera();

    let initial_pos = camera.position;
    let initial_right = camera.right;

    fire_event(
        &mut controller,
        &mut camera,
        DeviceOrientationEvent {
            alpha: 0.0,
            beta: 0.0,
            gamma: 90.0,
        },
    );

    assert_eq!(camera.position, initial_pos);
    assert_cartesian_epsilon(camera.direction, -Cartesian3::UNIT_Z, EPSILON14);
    assert_cartesian_epsilon(camera.up, Cartesian3::UNIT_X, EPSILON14);
    assert_cartesian_epsilon(camera.right, initial_right, EPSILON14);
}

#[test]
fn test_is_destroyed() {
    let mut controller = DeviceOrientationCameraController::new();
    assert!(!controller.is_destroyed());
    controller.destroy();
    assert!(controller.is_destroyed());
}
