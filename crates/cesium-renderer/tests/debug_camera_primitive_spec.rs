use cesium_math::{Camera, Color, DeveloperError};
use cesium_renderer::{DebugCameraPrimitive, DebugCameraPrimitiveOptions};

fn make_camera() -> Camera {
    Camera::look_at(
        glam::DVec3::new(0.0, 0.0, 10.0),
        glam::DVec3::ZERO,
        glam::DVec3::Y,
        800,
        600,
    )
    .unwrap()
}

#[test]
fn test_throws_if_options_camera_is_undefined() {
    let err = DebugCameraPrimitive::new(DebugCameraPrimitiveOptions::default())
        .expect_err("should throw without camera");
    assert_eq!(err.name(), DeveloperError::NAME);
}

#[test]
fn test_gets_default_properties() {
    let mut p = DebugCameraPrimitive::new(DebugCameraPrimitiveOptions {
        camera: Some(make_camera()),
        ..Default::default()
    })
    .expect("should construct");

    assert!(p.show);
    assert!(p.id.is_none());
    p.destroy();
    assert!(p.is_destroyed());
}

#[test]
fn test_constructs_with_options() {
    let mut p = DebugCameraPrimitive::new(DebugCameraPrimitiveOptions {
        camera: Some(make_camera()),
        frustum_splits: Some(vec![0.1, 1000.0]),
        color: Some(Color::YELLOW),
        update_on_change: Some(false),
        show: Some(false),
        id: Some("id".to_string()),
    })
    .expect("should construct with options");

    assert!(!p.show);
    assert_eq!(p.id.as_deref(), Some("id"));
    assert_eq!(p.color, Color::YELLOW);
    assert!(!p.update_on_change);
    p.destroy();
    assert!(p.is_destroyed());
}

#[test]
fn test_updates_when_underlying_camera_changes() {
    let mut p = DebugCameraPrimitive::new(DebugCameraPrimitiveOptions {
        camera: Some(make_camera()),
        update_on_change: Some(true),
        ..Default::default()
    })
    .expect("should construct");

    assert_eq!(p.generation, 0);
    p.update().unwrap();
    assert_eq!(p.generation, 1);
    p.update().unwrap();
    assert_eq!(p.generation, 2);
}

#[test]
fn test_does_not_update_when_update_on_change_is_false() {
    let mut p = DebugCameraPrimitive::new(DebugCameraPrimitiveOptions {
        camera: Some(make_camera()),
        update_on_change: Some(false),
        ..Default::default()
    })
    .expect("should construct");

    assert_eq!(p.generation, 0);
    p.update().unwrap();
    assert_eq!(p.generation, 0);
}

#[test]
fn test_is_destroyed() {
    let mut p = DebugCameraPrimitive::new(DebugCameraPrimitiveOptions {
        camera: Some(make_camera()),
        ..Default::default()
    })
    .expect("should construct");

    assert!(!p.is_destroyed());
    p.destroy();
    assert!(p.is_destroyed());
}
