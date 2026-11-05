use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use cesium_math::{
    get_input_event_key, Cartesian2, InputAction, KeyboardEventModifier, MotionEvent, MouseButton,
    PinchMovementEvent, PositionedEvent, ScreenSpaceEventHandler, ScreenSpaceEventType,
};

#[test]
fn input_event_key_generation() {
    // Without modifiers
    assert_eq!(
        get_input_event_key(ScreenSpaceEventType::LeftDown, None),
        "0"
    );
    assert_eq!(
        get_input_event_key(ScreenSpaceEventType::MouseMove, None),
        "15"
    );

    // With single modifier
    assert_eq!(
        get_input_event_key(
            ScreenSpaceEventType::LeftClick,
            Some(&[KeyboardEventModifier::Shift])
        ),
        "2+0"
    );

    // With multiple modifiers (must be sorted regardless of order passed)
    assert_eq!(
        get_input_event_key(
            ScreenSpaceEventType::RightClick,
            Some(&[KeyboardEventModifier::Alt, KeyboardEventModifier::Ctrl])
        ),
        "7+1+2"
    );
    assert_eq!(
        get_input_event_key(
            ScreenSpaceEventType::RightClick,
            Some(&[KeyboardEventModifier::Ctrl, KeyboardEventModifier::Alt])
        ),
        "7+1+2"
    );
}

#[test]
fn register_and_retrieve_and_remove_actions() {
    let mut handler = ScreenSpaceEventHandler::new();
    let action_called = Arc::new(AtomicBool::new(false));
    let action_called_clone = action_called.clone();

    let action = InputAction::Positioned(Arc::new(move |_| {
        action_called_clone.store(true, Ordering::SeqCst);
    }));

    handler.set_input_action(
        action,
        ScreenSpaceEventType::LeftClick,
        Some(&[KeyboardEventModifier::Shift]),
    );

    // Action is retrievable
    assert!(handler
        .get_input_action(
            ScreenSpaceEventType::LeftClick,
            Some(&[KeyboardEventModifier::Shift])
        )
        .is_some());

    // Action with different modifier is not found
    assert!(handler
        .get_input_action(ScreenSpaceEventType::LeftClick, None)
        .is_none());

    // Remove action
    handler.remove_input_action(
        ScreenSpaceEventType::LeftClick,
        Some(&[KeyboardEventModifier::Shift]),
    );
    assert!(handler
        .get_input_action(
            ScreenSpaceEventType::LeftClick,
            Some(&[KeyboardEventModifier::Shift])
        )
        .is_none());
}

#[test]
fn mouse_down_up_and_click_within_tolerance() {
    let mut handler = ScreenSpaceEventHandler::new();

    let down_count = Arc::new(AtomicU32::new(0));
    let up_count = Arc::new(AtomicU32::new(0));
    let click_count = Arc::new(AtomicU32::new(0));

    let d_c = down_count.clone();
    handler.set_input_action(
        InputAction::Positioned(Arc::new(move |ev: &PositionedEvent| {
            assert_eq!(ev.position, Cartesian2::new(10.0, 20.0));
            d_c.fetch_add(1, Ordering::SeqCst);
        })),
        ScreenSpaceEventType::LeftDown,
        None,
    );

    let u_c = up_count.clone();
    handler.set_input_action(
        InputAction::Positioned(Arc::new(move |ev: &PositionedEvent| {
            assert_eq!(ev.position, Cartesian2::new(12.0, 21.0));
            u_c.fetch_add(1, Ordering::SeqCst);
        })),
        ScreenSpaceEventType::LeftUp,
        None,
    );

    let c_c = click_count.clone();
    handler.set_input_action(
        InputAction::Positioned(Arc::new(move |ev: &PositionedEvent| {
            assert_eq!(ev.position, Cartesian2::new(12.0, 21.0));
            c_c.fetch_add(1, Ordering::SeqCst);
        })),
        ScreenSpaceEventType::LeftClick,
        None,
    );

    // Down at (10, 20)
    handler
        .handle_mouse_down(MouseButton::Left, Cartesian2::new(10.0, 20.0), None)
        .unwrap();
    assert_eq!(down_count.load(Ordering::SeqCst), 1);
    assert_eq!(up_count.load(Ordering::SeqCst), 0);
    assert_eq!(click_count.load(Ordering::SeqCst), 0);

    // Up at (12, 21) -> distance is sqrt(2^2 + 1^2) = sqrt(5) ~= 2.23px <= 5px tolerance
    handler
        .handle_mouse_up(MouseButton::Left, Cartesian2::new(12.0, 21.0), None)
        .unwrap();
    assert_eq!(down_count.load(Ordering::SeqCst), 1);
    assert_eq!(up_count.load(Ordering::SeqCst), 1);
    assert_eq!(click_count.load(Ordering::SeqCst), 1);
}

#[test]
fn mouse_up_exceeding_tolerance_does_not_click() {
    let mut handler = ScreenSpaceEventHandler::new();

    let click_count = Arc::new(AtomicU32::new(0));
    let c_c = click_count.clone();
    handler.set_input_action(
        InputAction::Positioned(Arc::new(move |_| {
            c_c.fetch_add(1, Ordering::SeqCst);
        })),
        ScreenSpaceEventType::LeftClick,
        None,
    );

    // Down at (10, 20)
    handler
        .handle_mouse_down(MouseButton::Left, Cartesian2::new(10.0, 20.0), None)
        .unwrap();

    // Up at (20, 30) -> distance sqrt(100+100) = 14.14px > 5.0px tolerance
    handler
        .handle_mouse_up(MouseButton::Left, Cartesian2::new(20.0, 30.0), None)
        .unwrap();

    assert_eq!(click_count.load(Ordering::SeqCst), 0);
}

#[test]
fn right_and_middle_mouse_buttons() {
    let mut handler = ScreenSpaceEventHandler::new();

    let right_click = Arc::new(AtomicBool::new(false));
    let r_c = right_click.clone();
    handler.set_input_action(
        InputAction::Positioned(Arc::new(move |_| {
            r_c.store(true, Ordering::SeqCst);
        })),
        ScreenSpaceEventType::RightClick,
        None,
    );

    let middle_click = Arc::new(AtomicBool::new(false));
    let m_c = middle_click.clone();
    handler.set_input_action(
        InputAction::Positioned(Arc::new(move |_| {
            m_c.store(true, Ordering::SeqCst);
        })),
        ScreenSpaceEventType::MiddleClick,
        None,
    );

    // Right click
    handler
        .handle_mouse_down(MouseButton::Right, Cartesian2::new(50.0, 50.0), None)
        .unwrap();
    handler
        .handle_mouse_up(MouseButton::Right, Cartesian2::new(50.0, 50.0), None)
        .unwrap();
    assert!(right_click.load(Ordering::SeqCst));
    assert!(!middle_click.load(Ordering::SeqCst));

    // Middle click
    handler
        .handle_mouse_down(MouseButton::Middle, Cartesian2::new(100.0, 100.0), None)
        .unwrap();
    handler
        .handle_mouse_up(MouseButton::Middle, Cartesian2::new(100.0, 100.0), None)
        .unwrap();
    assert!(middle_click.load(Ordering::SeqCst));
}

#[test]
fn mouse_move_and_wheel() {
    let mut handler = ScreenSpaceEventHandler::new();

    let move_recorded = Arc::new(AtomicBool::new(false));
    let m_r = move_recorded.clone();
    handler.set_input_action(
        InputAction::Motion(Arc::new(move |ev: &MotionEvent| {
            assert_eq!(ev.start_position, Cartesian2::new(10.0, 10.0));
            assert_eq!(ev.end_position, Cartesian2::new(30.0, 40.0));
            m_r.store(true, Ordering::SeqCst);
        })),
        ScreenSpaceEventType::MouseMove,
        None,
    );

    let wheel_delta = Arc::new(AtomicU32::new(0));
    let w_d = wheel_delta.clone();
    handler.set_input_action(
        InputAction::Wheel(Arc::new(move |delta: f64| {
            w_d.store(delta as u32, Ordering::SeqCst);
        })),
        ScreenSpaceEventType::Wheel,
        None,
    );

    handler
        .handle_mouse_down(MouseButton::Left, Cartesian2::new(10.0, 10.0), None)
        .unwrap();
    handler
        .handle_mouse_move(Cartesian2::new(30.0, 40.0), None)
        .unwrap();
    assert!(move_recorded.load(Ordering::SeqCst));

    handler.handle_wheel(120.0, None).unwrap();
    assert_eq!(wheel_delta.load(Ordering::SeqCst), 120);
}

#[test]
fn pinch_gesture_lifecycle_and_motion() {
    let mut handler = ScreenSpaceEventHandler::new();

    let start_fired = Arc::new(AtomicBool::new(false));
    let s_f = start_fired.clone();
    handler.set_input_action(
        InputAction::Empty(Arc::new(move || {
            s_f.store(true, Ordering::SeqCst);
        })),
        ScreenSpaceEventType::PinchStart,
        None,
    );

    let move_fired = Arc::new(AtomicBool::new(false));
    let m_f = move_fired.clone();
    handler.set_input_action(
        InputAction::PinchMovement(Arc::new(move |ev: &PinchMovementEvent| {
            // Distance start and end should have y > 0
            assert!(ev.distance.start_position.y > 0.0);
            assert!(ev.distance.end_position.y > 0.0);
            m_f.store(true, Ordering::SeqCst);
        })),
        ScreenSpaceEventType::PinchMove,
        None,
    );

    let end_fired = Arc::new(AtomicBool::new(false));
    let e_f = end_fired.clone();
    handler.set_input_action(
        InputAction::Empty(Arc::new(move || {
            e_f.store(true, Ordering::SeqCst);
        })),
        ScreenSpaceEventType::PinchEnd,
        None,
    );

    // 1. Pinch start with 2 touches at (0, 0) and (100, 0)
    handler
        .handle_pinch_start(Cartesian2::new(0.0, 0.0), Cartesian2::new(100.0, 0.0), None)
        .unwrap();
    assert!(start_fired.load(Ordering::SeqCst));

    // 2. Pinch move to (0, 0) and (200, 0)
    handler
        .handle_pinch_move(Cartesian2::new(0.0, 0.0), Cartesian2::new(200.0, 0.0), None)
        .unwrap();
    assert!(move_fired.load(Ordering::SeqCst));

    // 3. Pinch end
    handler.handle_pinch_end(None).unwrap();
    assert!(end_fired.load(Ordering::SeqCst));
}

#[test]
fn destroy_behavior() {
    let mut handler = ScreenSpaceEventHandler::new();
    assert!(!handler.is_destroyed());

    handler.set_input_action(
        InputAction::Empty(Arc::new(|| {})),
        ScreenSpaceEventType::LeftClick,
        None,
    );
    assert!(handler
        .get_input_action(ScreenSpaceEventType::LeftClick, None)
        .is_some());

    handler.destroy();
    assert!(handler.is_destroyed());
    assert!(handler
        .get_input_action(ScreenSpaceEventType::LeftClick, None)
        .is_none());

    // Operations after destroy return DeveloperError
    assert!(handler
        .handle_mouse_down(MouseButton::Left, Cartesian2::ZERO, None)
        .is_err());
    assert!(handler
        .handle_mouse_up(MouseButton::Left, Cartesian2::ZERO, None)
        .is_err());
    assert!(handler.handle_mouse_move(Cartesian2::ZERO, None).is_err());
    assert!(handler.handle_wheel(1.0, None).is_err());
}
