use cesium_math::{
    Cartesian2, Cartesian3, Ellipsoid, FixedFrameTransforms,
    HeadingPitchRange, Matrix4, SceneMode, EPSILON15, EPSILON8,
    PI, PI_OVER_TWO, TWO_PI,
};
use cesium_renderer::cesium_widget::MapMode2D;
use cesium_renderer::{BoundingSphere, Camera, CameraCanvas, CameraOrientation, CameraViewOptions};

fn create_test_camera() -> Camera {
    let canvas = CameraCanvas {
        client_width: 512.0,
        client_height: 384.0,
    };
    let mut camera = Camera::new(Some(canvas)).expect("camera creation should succeed");
    camera.position = Cartesian3::UNIT_Z;
    camera.up = Cartesian3::UNIT_Y;
    camera.direction = -Cartesian3::UNIT_Z;
    camera.right = camera.direction.cross(&camera.up);
    camera.minimum_zoom_distance = 0.0;
    camera
}

#[test]
fn constructor_throws_when_no_canvas() {
    assert!(Camera::new(None).is_err());
    assert!(Camera::new(Some(CameraCanvas {
        client_width: 0.0,
        client_height: 384.0
    }))
    .is_err());
}

#[test]
fn get_view_matrix() {
    let mut camera = create_test_camera();
    let view_matrix = camera.view_matrix();
    let position = camera.position;
    let up = camera.up;
    let dir = camera.direction;
    let right = camera.right;

    let rotation = Matrix4::new(
        right.x, right.y, right.z, 0.0,
        up.x, up.y, up.z, 0.0,
        -dir.x, -dir.y, -dir.z, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );
    let translation = Matrix4::new(
        1.0, 0.0, 0.0, -position.x,
        0.0, 1.0, 0.0, -position.y,
        0.0, 0.0, 1.0, -position.z,
        0.0, 0.0, 0.0, 1.0,
    );
    let expected = rotation.multiply(&translation);
    assert!(view_matrix.equals_epsilon(&expected, EPSILON8));
}

#[test]
fn get_inverse_view_matrix() {
    let mut camera = create_test_camera();
    let view_matrix = camera.view_matrix();
    let expected = view_matrix.inverse().unwrap();
    let inv = camera.inverse_view_matrix();
    assert!(inv.equals_epsilon(&expected, EPSILON15));
}

#[test]
fn get_inverse_transform() {
    let mut camera = create_test_camera();
    let transform = Matrix4::new(
        5.0, 0.0, 0.0, 1.0,
        0.0, 5.0, 0.0, 2.0,
        0.0, 0.0, 5.0, 3.0,
        0.0, 0.0, 0.0, 1.0,
    );
    camera.set_transform(transform);
    let expected = transform.inverse_transformation().unwrap();
    assert!(camera.inverse_transform.equals_epsilon(&expected, EPSILON8));
}

#[test]
fn computes_orthonormal_direction_up_and_right_vectors() {
    let mut camera = create_test_camera();
    camera.direction = Cartesian3::new(-0.32297853365047874, 0.9461560708446421, 0.021761351171635013);
    camera.up = Cartesian3::new(0.9327219113001013, 0.31839266745173644, -2.9874778345595487e-10);
    camera.right = Cartesian3::new(0.0069286549295528715, -0.020297288960790985, 0.9853344956450351);

    assert!((camera.right.magnitude() - 1.0).abs() > EPSILON8);
    assert!((camera.up.magnitude() - 1.0).abs() > EPSILON8);

    let view_matrix = camera.view_matrix();
    assert!((camera.right.magnitude() - 1.0).abs() < EPSILON8);
    assert!((camera.up.magnitude() - 1.0).abs() < EPSILON8);

    let inverse_affine = view_matrix.inverse_transformation().unwrap();
    let inverse = view_matrix.inverse().unwrap();
    assert!(inverse_affine.equals_epsilon(&inverse, EPSILON8));
}

#[test]
fn get_heading_in_modes() {
    let mut camera = create_test_camera();

    // Morphing: undefined
    camera.mode = SceneMode::Morphing;
    assert!(camera.heading().is_none());

    // Scene2D
    camera.mode = SceneMode::Scene2D;
    let expected_2d = TWO_PI - camera.right.y.atan2(camera.right.x);
    assert!((camera.heading().unwrap() - expected_2d).abs() < EPSILON8);

    // Columbus View
    camera.mode = SceneMode::ColumbusView;
    let expected_cv = TWO_PI - camera.right.y.atan2(camera.right.x);
    assert!((camera.heading().unwrap() - expected_cv).abs() < EPSILON8);

    // Scene3D
    camera.mode = SceneMode::Scene3D;
    let to_fixed = FixedFrameTransforms::east_north_up_to_fixed_frame(camera.position, Some(Ellipsoid::WGS84)).unwrap();
    let transform_t = to_fixed.get_matrix3().transpose();
    let right_enu = transform_t.multiply_by_vector(camera.right);
    let expected_3d = TWO_PI - cesium_math::zero_to_two_pi(right_enu.y.atan2(right_enu.x));
    assert!((camera.heading().unwrap() - expected_3d).abs() < EPSILON8);
}

#[test]
fn heading_2d_rotation_mode() {
    let mut camera = create_test_camera();
    camera.mode = SceneMode::Scene2D;
    camera.map_mode_2d = MapMode2D::Rotate;

    let target_heading = PI / 3.0;
    camera.set_heading(target_heading);
    let current_heading = camera.heading().unwrap();
    assert!((current_heading - target_heading).abs() < 1e-6);

    // In InfiniteScroll mode, heading rotation is suppressed
    camera.map_mode_2d = MapMode2D::InfiniteScroll;
    let old_heading = camera.heading().unwrap();
    camera.set_heading(PI / 2.0);
    assert!((camera.heading().unwrap() - old_heading).abs() < 1e-6);
}

#[test]
fn pitch_and_roll_in_modes() {
    let mut camera = create_test_camera();

    // Morphing / 2D has no pitch/roll
    camera.mode = SceneMode::Scene2D;
    assert!(camera.pitch().is_none());
    assert!(camera.roll().is_none());

    camera.mode = SceneMode::Morphing;
    assert!(camera.pitch().is_none());
    assert!(camera.roll().is_none());

    // Columbus view pitch & roll
    camera.mode = SceneMode::ColumbusView;
    assert!(camera.pitch().is_some());
    assert!(camera.roll().is_some());

    // 3D pitch & roll
    camera.mode = SceneMode::Scene3D;
    assert!(camera.pitch().is_some());
    assert!(camera.roll().is_some());
}

#[test]
fn update_validates_mode() {
    let mut camera = create_test_camera();
    assert!(camera.update(None).is_err());
    assert!(camera.update(Some(SceneMode::Scene3D)).is_ok());
}

#[test]
fn camera_movement_and_orientation_operations() {
    let mut camera = create_test_camera();
    let initial_pos = camera.position;

    camera.move_forward(10.0);
    assert!((camera.position - (initial_pos + camera.direction * 10.0)).magnitude() < EPSILON8);

    camera.move_backward(5.0);
    camera.move_up(2.0);
    camera.move_down(2.0);
    camera.move_right(3.0);
    camera.move_left(3.0);

    camera.zoom_in(10.0);
    camera.zoom_out(5.0);

    // Rotation & Looking
    camera.look_up(0.1);
    camera.look_down(0.1);
    camera.twist_left(0.1);
    camera.twist_right(0.1);
}

#[test]
fn camera_set_view_and_look_at() {
    let mut camera = create_test_camera();
    let target_dest = Cartesian3::new(100.0, 200.0, 300.0);

    camera.set_view(CameraViewOptions {
        destination: Some(target_dest),
        orientation: Some(CameraOrientation::DirectionUp {
            direction: Cartesian3::new(0.0, 1.0, 0.0),
            up: Cartesian3::new(0.0, 0.0, 1.0),
        }),
        end_transform: None,
    });
    assert!((camera.position - target_dest).magnitude() < EPSILON8);

    let target = Cartesian3::new(1000.0, 2000.0, 3000.0);
    let hpr = HeadingPitchRange {
        heading: 0.0,
        pitch: -PI_OVER_TWO,
        range: 500.0,
    };
    camera.look_at(target, hpr);
    assert!(camera.position.distance(&target) > 0.0);
}

#[test]
fn pick_ray_and_bounding_sphere_distance() {
    let camera = create_test_camera();
    let window_pos = Cartesian2::new(256.0, 192.0);
    let ray = camera.get_pick_ray(window_pos);
    assert!(ray.is_some());

    let sphere = BoundingSphere::new(Cartesian3::new(0.0, 0.0, 10.0), 2.0);
    let dist = camera.distance_to_bounding_sphere(&sphere);
    assert!(dist >= 0.0);
}
