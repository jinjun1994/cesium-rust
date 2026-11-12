use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{Clock, ClockOptions, ClockRange, ClockStep, JulianDate};
use cesium_viewer::widgets::{AnimationViewModel, ClockViewModel};

fn verify_paused_state(vm: &AnimationViewModel) {
    assert!(vm.pause_view_model().toggled());
    assert!(!vm.play_reverse_view_model().toggled());
    assert!(!vm.play_forward_view_model().toggled());
    assert!(!vm.play_realtime_view_model().toggled());
}

fn verify_forward_state(vm: &AnimationViewModel) {
    assert!(!vm.pause_view_model().toggled());
    assert!(!vm.play_reverse_view_model().toggled());
    assert!(vm.play_forward_view_model().toggled());
    assert!(!vm.play_realtime_view_model().toggled());
}

fn verify_reverse_state(vm: &AnimationViewModel) {
    assert!(!vm.pause_view_model().toggled());
    assert!(vm.play_reverse_view_model().toggled());
    assert!(!vm.play_forward_view_model().toggled());
    assert!(!vm.play_realtime_view_model().toggled());
}

#[test]
fn constructor_sets_expected_properties() {
    let clock_vm = ClockViewModel::default();
    let vm = AnimationViewModel::new(clock_vm);
    assert_eq!(vm.clock_view_model().multiplier(), 1.0);
    verify_paused_state(&vm);
}

#[test]
fn default_time_formatter_produces_expected_result() {
    let clock_vm = ClockViewModel::default();
    let vm = AnimationViewModel::new(clock_vm.clone());
    let date = JulianDate::from_iso8601("2012-03-05T06:07:08.89Z").unwrap();

    clock_vm.set_multiplier(1.0);
    assert_eq!(AnimationViewModel::default_time_formatter(date, &vm), "06:07:08 UTC");

    clock_vm.set_multiplier(-1.0);
    assert_eq!(AnimationViewModel::default_time_formatter(date, &vm), "06:07:08 UTC");

    clock_vm.set_multiplier(-0.5);
    assert_eq!(AnimationViewModel::default_time_formatter(date, &vm), "06:07:08.890");

    clock_vm.set_multiplier(0.5);
    assert_eq!(AnimationViewModel::default_time_formatter(date, &vm), "06:07:08.890");
}

#[test]
fn set_time_formatter_overrides_the_default_formatter() {
    let clock_vm = ClockViewModel::default();
    let vm = AnimationViewModel::new(clock_vm);

    vm.set_time_formatter(|_date, _vm| "My Time".to_string());
    assert_eq!(vm.time_label(), "My Time");
}

#[test]
fn default_date_formatter_produces_expected_result() {
    let clock_vm = ClockViewModel::default();
    let vm = AnimationViewModel::new(clock_vm);
    let date = JulianDate::from_iso8601("2012-03-05T06:07:08.89Z").unwrap();
    assert_eq!(AnimationViewModel::default_date_formatter(date, &vm), "Mar 5 2012");
}

#[test]
fn set_date_formatter_overrides_the_default_formatter() {
    let clock_vm = ClockViewModel::default();
    let vm = AnimationViewModel::new(clock_vm);

    vm.set_date_formatter(|_date, _vm| "My Date".to_string());
    assert_eq!(vm.date_label(), "My Date");
}

#[test]
fn correctly_formats_speed_label() {
    let clock_vm = ClockViewModel::default();
    let vm = AnimationViewModel::new(clock_vm.clone());

    clock_vm.set_multiplier(1.0);
    assert_eq!(vm.multiplier_label(), "1x");

    clock_vm.set_multiplier(0.005);
    assert_eq!(vm.multiplier_label(), "0.005x");

    clock_vm.set_multiplier(0.5);
    assert_eq!(vm.multiplier_label(), "0.5x");

    clock_vm.set_multiplier(0.125);
    assert_eq!(vm.multiplier_label(), "0.125x");

    clock_vm.set_multiplier(64.0);
    assert_eq!(vm.multiplier_label(), "64x");

    clock_vm.set_clock_step(ClockStep::SystemClock);
    assert_eq!(vm.multiplier_label(), "Today");
}

#[test]
fn pause_button_restores_current_state() {
    let clock_vm = ClockViewModel::default();
    let vm = AnimationViewModel::new(clock_vm.clone());

    verify_paused_state(&vm);

    // Unpause -> forward
    vm.pause_view_model().command.call().unwrap();
    verify_forward_state(&vm);

    // Pause again
    vm.pause_view_model().command.call().unwrap();
    verify_paused_state(&vm);
}

#[test]
fn animating_forwards_negates_multiplier_if_negative() {
    let clock_vm = ClockViewModel::default();
    clock_vm.set_multiplier(-2.0);
    let vm = AnimationViewModel::new(clock_vm.clone());

    vm.play_forward_view_model().command.call().unwrap();
    assert_eq!(clock_vm.multiplier(), 2.0);
    verify_forward_state(&vm);
}

#[test]
fn animating_backwards_negates_multiplier_if_positive() {
    let clock_vm = ClockViewModel::default();
    clock_vm.set_multiplier(2.0);
    let vm = AnimationViewModel::new(clock_vm.clone());

    vm.play_reverse_view_model().command.call().unwrap();
    assert_eq!(clock_vm.multiplier(), -2.0);
    verify_reverse_state(&vm);
}

#[test]
fn slower_and_faster_cycle_through_defined_multipliers() {
    let clock_vm = ClockViewModel::default();
    clock_vm.set_multiplier(1.0);
    let vm = AnimationViewModel::new(clock_vm.clone());

    vm.slower().call().unwrap();
    assert_eq!(clock_vm.multiplier(), 0.5);

    vm.slower().call().unwrap();
    assert_eq!(clock_vm.multiplier(), 0.25);

    vm.faster().call().unwrap();
    assert_eq!(clock_vm.multiplier(), 0.5);

    vm.faster().call().unwrap();
    assert_eq!(clock_vm.multiplier(), 1.0);
}

#[test]
fn slower_has_no_effect_at_slowest_speed() {
    let clock_vm = ClockViewModel::default();
    let vm = AnimationViewModel::new(clock_vm.clone());
    let slowest = -vm.get_shuttle_ring_ticks().last().copied().unwrap();
    clock_vm.set_multiplier(slowest);

    vm.slower().call().unwrap();
    assert_eq!(clock_vm.multiplier(), slowest);
}

#[test]
fn faster_has_no_effect_at_fastest_speed() {
    let clock_vm = ClockViewModel::default();
    let vm = AnimationViewModel::new(clock_vm.clone());
    let fastest = vm.get_shuttle_ring_ticks().last().copied().unwrap();
    clock_vm.set_multiplier(fastest);

    vm.faster().call().unwrap();
    assert_eq!(clock_vm.multiplier(), fastest);
}

#[test]
fn shuttle_ring_angles_set_expected_multipliers() {
    let clock_vm = ClockViewModel::default();
    let vm = AnimationViewModel::new(clock_vm.clone());

    vm.set_shuttle_ring_angle(0.0);
    assert_eq!(clock_vm.multiplier(), 0.0);

    vm.set_shuttle_ring_angle(15.0);
    assert_eq!(clock_vm.multiplier(), 1.0);

    vm.set_shuttle_ring_angle(-15.0);
    assert_eq!(clock_vm.multiplier(), -1.0);

    vm.set_shuttle_ring_angle(105.0);
    assert_eq!(clock_vm.multiplier(), 604800.0);

    vm.set_shuttle_ring_angle(-105.0);
    assert_eq!(clock_vm.multiplier(), -604800.0);
}

#[test]
fn shuttle_ring_angles_snap_to_ticks() {
    let clock_vm = ClockViewModel::default();
    let vm = AnimationViewModel::new(clock_vm.clone());
    vm.set_snap_to_ticks(true);

    let max_multiplier = *vm.get_shuttle_ring_ticks().last().unwrap();
    let min_multiplier = -max_multiplier;

    // Max angle should produce max speed
    vm.set_shuttle_ring_angle(105.0);
    assert_eq!(clock_vm.multiplier(), max_multiplier);

    // Min angle should produce min speed
    vm.set_shuttle_ring_angle(-105.0);
    assert_eq!(clock_vm.multiplier(), min_multiplier);

    // 15 degrees is 1x
    vm.set_shuttle_ring_angle(15.0);
    assert_eq!(clock_vm.multiplier(), 1.0);

    vm.set_shuttle_ring_angle(-15.0);
    assert_eq!(clock_vm.multiplier(), -1.0);

    vm.set_shuttle_ring_angle(45.0);
    assert_eq!(clock_vm.multiplier(), 120.0);

    vm.set_shuttle_ring_angle(-90.0);
    assert_eq!(clock_vm.multiplier(), -43200.0);

    vm.set_shuttle_ring_angle(0.0);
    assert_eq!(clock_vm.multiplier(), 0.001);
}

#[test]
fn animating_backwards_pauses_with_bounded_start_time() {
    let center_time = JulianDate::from_iso8601("2012-01-01T12:00:00Z").unwrap();
    let start_time = JulianDate::from_iso8601("2012-01-01T00:00:00Z").unwrap();
    let stop_time = JulianDate::from_iso8601("2012-01-02T00:00:00Z").unwrap();

    let clock_vm = ClockViewModel::default();
    clock_vm.set_start_time(start_time);
    clock_vm.set_stop_time(stop_time);
    clock_vm.set_clock_step(ClockStep::TickDependent);
    clock_vm.set_current_time(center_time);
    clock_vm.set_should_animate(false);

    let vm = AnimationViewModel::new(clock_vm.clone());
    verify_paused_state(&vm);

    // Play in reverse while clamped
    clock_vm.set_multiplier(-1.0);
    clock_vm.set_clock_range(ClockRange::Clamped);
    vm.play_reverse_view_model().command.call().unwrap();
    verify_reverse_state(&vm);

    // Set current time to start time -> paused
    clock_vm.set_current_time(start_time);
    verify_paused_state(&vm);

    // Animate in reverse again with LoopStop
    clock_vm.set_current_time(center_time);
    clock_vm.set_clock_range(ClockRange::LoopStop);
    vm.play_reverse_view_model().command.call().unwrap();
    verify_reverse_state(&vm);

    clock_vm.set_current_time(start_time);
    verify_paused_state(&vm);

    // Reversing at start while bounded has no effect
    vm.play_reverse_view_model().command.call().unwrap();
    verify_paused_state(&vm);

    // Set to unbounded -> reverse works
    clock_vm.set_clock_range(ClockRange::Unbounded);
    vm.play_reverse_view_model().command.call().unwrap();
    verify_reverse_state(&vm);
}

#[test]
fn dragging_shuttle_ring_does_not_pause_with_bounded_start_or_stop_time() {
    let center_time = JulianDate::from_iso8601("2012-01-01T12:00:00Z").unwrap();
    let start_time = JulianDate::from_iso8601("2012-01-01T00:00:00Z").unwrap();
    let stop_time = JulianDate::from_iso8601("2012-01-02T00:00:00Z").unwrap();

    let clock_vm = ClockViewModel::default();
    clock_vm.set_start_time(start_time);
    clock_vm.set_stop_time(stop_time);
    clock_vm.set_clock_step(ClockStep::TickDependent);
    clock_vm.set_clock_range(ClockRange::Clamped);
    clock_vm.set_multiplier(1.0);

    let vm = AnimationViewModel::new(clock_vm.clone());
    verify_paused_state(&vm);

    clock_vm.set_current_time(center_time);
    vm.play_forward_view_model().command.call().unwrap();
    verify_forward_state(&vm);

    // Dragging won't stop at stop_time
    vm.set_shuttle_ring_dragging(true);
    clock_vm.set_current_time(stop_time);
    verify_forward_state(&vm);

    // Drag complete stops
    vm.set_shuttle_ring_dragging(false);
    verify_paused_state(&vm);

    // Reverse
    clock_vm.set_current_time(center_time);
    vm.play_reverse_view_model().command.call().unwrap();
    verify_reverse_state(&vm);

    vm.set_shuttle_ring_dragging(true);
    clock_vm.set_current_time(start_time);
    verify_reverse_state(&vm);

    vm.set_shuttle_ring_dragging(false);
    verify_paused_state(&vm);
}

#[test]
fn animating_forward_pauses_with_bounded_stop_time() {
    let center_time = JulianDate::from_iso8601("2012-01-01T12:00:00Z").unwrap();
    let start_time = JulianDate::from_iso8601("2012-01-01T00:00:00Z").unwrap();
    let stop_time = JulianDate::from_iso8601("2012-01-02T00:00:00Z").unwrap();

    let clock_vm = ClockViewModel::default();
    clock_vm.set_start_time(start_time);
    clock_vm.set_stop_time(stop_time);
    clock_vm.set_clock_step(ClockStep::TickDependent);
    clock_vm.set_current_time(center_time);
    clock_vm.set_should_animate(false);

    let vm = AnimationViewModel::new(clock_vm.clone());
    verify_paused_state(&vm);

    clock_vm.set_multiplier(1.0);
    clock_vm.set_clock_range(ClockRange::Clamped);
    vm.play_forward_view_model().command.call().unwrap();
    verify_forward_state(&vm);

    clock_vm.set_current_time(stop_time);
    verify_paused_state(&vm);

    vm.play_forward_view_model().command.call().unwrap();
    verify_paused_state(&vm);

    clock_vm.set_clock_range(ClockRange::Unbounded);
    vm.play_forward_view_model().command.call().unwrap();
    verify_forward_state(&vm);
}

#[test]
fn shuttle_ring_ticks_sorting_and_getting() {
    let clock_vm = ClockViewModel::default();
    let vm = AnimationViewModel::new(clock_vm);

    let initial_ticks = vm.get_shuttle_ring_ticks();
    assert!(!initial_ticks.is_empty());

    let custom_ticks = vec![10.0, 2.0, 50.0, 2.0, 1.0];
    vm.set_shuttle_ring_ticks(&custom_ticks).unwrap();
    let new_ticks = vm.get_shuttle_ring_ticks();
    assert_eq!(new_ticks, vec![1.0, 2.0, 10.0, 50.0]);
}

#[test]
fn realtime_can_execute_and_tooltip() {
    let start_time = JulianDate::from_iso8601("2000-01-01T00:00:00Z").unwrap();
    let stop_time = JulianDate::from_iso8601("2000-01-02T00:00:00Z").unwrap();
    let clock = Clock::new(ClockOptions {
        start_time: Some(start_time),
        stop_time: Some(stop_time),
        clock_range: Some(ClockRange::Clamped),
        ..Default::default()
    })
    .unwrap();

    let clock_vm = ClockViewModel::new(Some(Rc::new(RefCell::new(clock))));
    let vm = AnimationViewModel::new(clock_vm.clone());

    // System time is > 2000, so not in range
    assert_eq!(vm.play_realtime_view_model().tooltip(), "Current time not in range");

    // Change to unbounded
    clock_vm.set_clock_range(ClockRange::Unbounded);
    assert_eq!(vm.play_realtime_view_model().tooltip(), "Today (real-time)");
}
