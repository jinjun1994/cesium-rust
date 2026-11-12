use cesium_math::{Cartesian2, KeyboardEventModifier};
use cesium_renderer::{CameraEventAggregator, CameraEventType};

#[test]
fn test_movement_and_last_movement_and_aggregation() {
    let mut agg = CameraEventAggregator::new();
    assert!(!agg.any_button_down());
    assert!(!agg.is_moving(CameraEventType::LeftDrag, None));
    assert!(agg.get_movement(CameraEventType::LeftDrag, None).is_none());
    assert!(agg
        .get_last_movement(CameraEventType::LeftDrag, None)
        .is_none());

    agg.on_mouse_down(
        CameraEventType::LeftDrag,
        None,
        Cartesian2::new(0.0, 0.0),
        10.0,
    );
    agg.on_mouse_move(
        CameraEventType::LeftDrag,
        None,
        Cartesian2::new(0.0, 0.0),
        Cartesian2::new(5.0, 5.0),
    );
    agg.on_mouse_move(
        CameraEventType::LeftDrag,
        None,
        Cartesian2::new(5.0, 5.0),
        Cartesian2::new(12.0, 15.0),
    );

    assert!(agg.is_moving(CameraEventType::LeftDrag, None));
    let mov = agg.get_movement(CameraEventType::LeftDrag, None).unwrap();
    assert_eq!(mov.start_position, Cartesian2::new(0.0, 0.0));
    assert_eq!(mov.end_position, Cartesian2::new(12.0, 15.0));

    agg.reset();
    assert!(!agg.is_moving(CameraEventType::LeftDrag, None));
    assert!(agg.get_movement(CameraEventType::LeftDrag, None).is_none());
    let last = agg
        .get_last_movement(CameraEventType::LeftDrag, None)
        .unwrap();
    assert_eq!(last.start_position, Cartesian2::new(0.0, 0.0));
    assert_eq!(last.end_position, Cartesian2::new(12.0, 15.0));
}

#[test]
fn test_button_down_any_button_down_and_press_release_times() {
    let mut agg = CameraEventAggregator::new();

    agg.on_mouse_down(
        CameraEventType::LeftDrag,
        Some(KeyboardEventModifier::Shift),
        Cartesian2::new(1.0, 2.0),
        100.0,
    );
    assert!(agg.is_button_down(CameraEventType::LeftDrag, Some(KeyboardEventModifier::Shift)));
    assert!(!agg.is_button_down(CameraEventType::LeftDrag, None));
    assert!(agg.any_button_down());
    assert_eq!(
        agg.get_button_press_time(CameraEventType::LeftDrag, Some(KeyboardEventModifier::Shift)),
        Some(100.0)
    );
    assert_eq!(
        agg.get_start_mouse_position(CameraEventType::LeftDrag, Some(KeyboardEventModifier::Shift)),
        Some(Cartesian2::new(1.0, 2.0))
    );

    agg.on_mouse_down(
        CameraEventType::RightDrag,
        None,
        Cartesian2::new(3.0, 4.0),
        110.0,
    );
    // Releasing any button cancels any_button_down
    agg.on_mouse_up(
        CameraEventType::RightDrag,
        None,
        Cartesian2::new(3.0, 4.0),
        150.0,
    );
    assert!(!agg.is_button_down(CameraEventType::RightDrag, None));
    assert!(!agg.any_button_down());
    assert_eq!(
        agg.get_button_release_time(CameraEventType::RightDrag, None),
        Some(150.0)
    );
}

#[test]
fn test_wheel_and_destroy() {
    let mut agg = CameraEventAggregator::new();
    agg.on_wheel(None, 120.0);
    agg.on_wheel(None, 60.0);
    let mov = agg.get_movement(CameraEventType::Wheel, None).unwrap();
    assert_eq!(mov.start_position, Cartesian2::ZERO);
    assert_eq!(mov.end_position, Cartesian2::new(0.0, 180.0));

    assert!(!agg.is_destroyed());
    agg.destroy();
    assert!(agg.is_destroyed());
}
