//! Range behavior adapted from CesiumJS packages/core/Specs/ClockSpec.js.
//! The Rust API takes elapsed seconds and does not yet implement JulianDate,
//! ClockStep, clock events, or the full Cesium Clock constructor contract.
use cesium_viewer::clock::{Clock, Range};

fn clock(range: Range, current_time: f64, multiplier: f64) -> Clock {
    Clock {
        start_time: 0.0,
        stop_time: 60.0,
        current_time,
        multiplier,
        should_animate: true,
        range,
    }
}

#[test]
fn unbounded_clock_advances_forward_and_backward_past_range() {
    for (start, speed, expected) in [(60.0, 1.5, 61.5), (0.0, -1.5, -1.5)] {
        let mut c = clock(Range::Unbounded, start, speed);
        c.tick(1.0);
        assert_eq!(c.current_time, expected);
        c.tick(1.0);
        assert_eq!(c.current_time, expected + speed);
    }
}

#[test]
fn clamped_clock_stops_at_both_endpoints() {
    for (start, speed, expected) in [(60.0, 100.0, 60.0), (0.0, -100.0, 0.0)] {
        let mut c = clock(Range::Clamped, start, speed);
        c.tick(1.0);
        assert_eq!(c.current_time, expected);
        c.tick(1.0);
        assert_eq!(c.current_time, expected);
    }
}

#[test]
fn looping_clock_wraps_forward_and_clamps_backward() {
    let mut forward = clock(Range::LoopStop, 60.0, 1.5);
    forward.tick(1.0);
    assert_eq!(forward.current_time, 1.5);
    forward.tick(1.0);
    assert_eq!(forward.current_time, 3.0);

    let mut backward = clock(Range::LoopStop, 0.0, -100.0);
    backward.tick(1.0);
    assert_eq!(backward.current_time, 0.0);
}

#[test]
fn animation_can_be_paused_and_resumed() {
    let mut c = clock(Range::Unbounded, 30.0, 1.5);
    c.should_animate = false;
    c.tick(1.0);
    assert_eq!(c.current_time, 30.0);
    c.should_animate = true;
    c.tick(1.0);
    assert_eq!(c.current_time, 31.5);
}
