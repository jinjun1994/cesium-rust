use std::cell::Cell;
use std::rc::Rc;

use cesium_math::{Clock, ClockOptions, ClockRange, ClockStep, JulianDate, TimeStandard};

fn jd(day: f64) -> JulianDate {
    JulianDate::new(day, 0.0, TimeStandard::UTC)
}

fn jd_sec(day: f64, sec: f64) -> JulianDate {
    JulianDate::new(day, sec, TimeStandard::UTC)
}

#[test]
fn sets_default_parameters_when_constructed() {
    let clock = Clock::default();
    assert_eq!(clock.stop_time(), clock.start_time().add_days(1.0));
    assert_eq!(clock.start_time(), clock.current_time());
    assert_eq!(clock.multiplier(), 1.0);
    assert_eq!(clock.clock_step(), ClockStep::SystemClockMultiplier);
    assert_eq!(clock.clock_range(), ClockRange::Unbounded);
    assert!(clock.can_animate);
    assert!(!clock.should_animate());
}

#[test]
fn sets_provided_constructor_parameters_correctly() {
    let start = jd(12.0);
    let stop = jd(112.0);
    let current_time = jd(13.0);
    let step = ClockStep::TickDependent;
    let range = ClockRange::LoopStop;
    let multiplier = 1.5;

    let mut clock = Clock::new(ClockOptions {
        start_time: Some(start),
        stop_time: Some(stop),
        current_time: Some(current_time),
        clock_step: Some(step),
        multiplier: Some(multiplier),
        clock_range: Some(range),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(clock.start_time(), start);
    assert_eq!(clock.stop_time(), stop);
    assert_eq!(clock.current_time(), current_time);
    assert_eq!(clock.clock_step(), step);
    assert_eq!(clock.clock_range(), range);
    assert_eq!(clock.multiplier(), multiplier);
    assert!(clock.can_animate);
    assert!(!clock.should_animate());

    clock = Clock::new(ClockOptions {
        can_animate: Some(false),
        ..Default::default()
    })
    .unwrap();
    assert!(!clock.can_animate);

    clock = Clock::new(ClockOptions {
        should_animate: Some(true),
        ..Default::default()
    })
    .unwrap();
    assert!(clock.should_animate());
}

#[test]
fn works_when_constructed_with_no_current_time_parameter() {
    let start = jd(12.0);
    let stop = jd(112.0);
    let step = ClockStep::TickDependent;
    let range = ClockRange::LoopStop;
    let multiplier = 1.5;

    let clock = Clock::new(ClockOptions {
        start_time: Some(start),
        stop_time: Some(stop),
        clock_step: Some(step),
        multiplier: Some(multiplier),
        clock_range: Some(range),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(clock.start_time(), start);
    assert_eq!(clock.stop_time(), stop);
    assert_eq!(clock.current_time(), start);
    assert_eq!(clock.clock_step(), step);
    assert_eq!(clock.clock_range(), range);
    assert_eq!(clock.multiplier(), multiplier);
    assert!(clock.can_animate);
    assert!(!clock.should_animate());
}

#[test]
fn works_when_constructed_with_no_start_time_parameter() {
    let stop = jd(112.0);
    let current_time = jd(13.0);
    let step = ClockStep::TickDependent;
    let range = ClockRange::LoopStop;
    let multiplier = 1.5;

    let clock = Clock::new(ClockOptions {
        stop_time: Some(stop),
        current_time: Some(current_time),
        clock_step: Some(step),
        multiplier: Some(multiplier),
        clock_range: Some(range),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(clock.start_time(), current_time);
    assert_eq!(clock.stop_time(), stop);
    assert_eq!(clock.current_time(), current_time);
    assert_eq!(clock.clock_step(), step);
    assert_eq!(clock.clock_range(), range);
    assert_eq!(clock.multiplier(), multiplier);
    assert!(clock.can_animate);
    assert!(!clock.should_animate());
}

#[test]
fn works_when_constructed_with_no_start_time_or_stop_time() {
    let current_time = jd(12.0);
    let step = ClockStep::TickDependent;
    let range = ClockRange::LoopStop;
    let multiplier = 1.5;

    let clock = Clock::new(ClockOptions {
        current_time: Some(current_time),
        clock_step: Some(step),
        multiplier: Some(multiplier),
        clock_range: Some(range),
        ..Default::default()
    })
    .unwrap();

    let expected_stop = current_time.add_days(1.0);
    assert_eq!(clock.start_time(), current_time);
    assert_eq!(clock.stop_time(), expected_stop);
    assert_eq!(clock.current_time(), current_time);
    assert_eq!(clock.clock_step(), step);
    assert_eq!(clock.clock_range(), range);
    assert_eq!(clock.multiplier(), multiplier);
    assert!(clock.can_animate);
    assert!(!clock.should_animate());
}

#[test]
fn works_when_constructed_with_no_start_time_or_current_time() {
    let stop = jd(13.0);
    let step = ClockStep::TickDependent;
    let range = ClockRange::LoopStop;
    let multiplier = 1.5;

    let clock = Clock::new(ClockOptions {
        stop_time: Some(stop),
        clock_step: Some(step),
        multiplier: Some(multiplier),
        clock_range: Some(range),
        ..Default::default()
    })
    .unwrap();

    let expected_start = stop.add_days(-1.0);
    assert_eq!(clock.start_time(), expected_start);
    assert_eq!(clock.stop_time(), stop);
    assert_eq!(clock.current_time(), expected_start);
    assert_eq!(clock.clock_step(), step);
    assert_eq!(clock.clock_range(), range);
    assert_eq!(clock.multiplier(), multiplier);
    assert!(clock.can_animate);
    assert!(!clock.should_animate());
}

#[test]
fn works_when_constructed_with_no_current_time_or_stop_time() {
    let start = jd(12.0);
    let step = ClockStep::TickDependent;
    let range = ClockRange::LoopStop;
    let multiplier = 1.5;

    let clock = Clock::new(ClockOptions {
        start_time: Some(start),
        clock_step: Some(step),
        multiplier: Some(multiplier),
        clock_range: Some(range),
        ..Default::default()
    })
    .unwrap();

    let expected_stop = start.add_days(1.0);
    assert_eq!(clock.start_time(), start);
    assert_eq!(clock.stop_time(), expected_stop);
    assert_eq!(clock.current_time(), start);
    assert_eq!(clock.clock_step(), step);
    assert_eq!(clock.clock_range(), range);
    assert_eq!(clock.multiplier(), multiplier);
    assert!(clock.can_animate);
    assert!(!clock.should_animate());
}

#[test]
fn works_when_constructed_with_no_stop_time_parameter() {
    let start = jd(12.0);
    let current_time = jd(12.0);
    let step = ClockStep::TickDependent;
    let range = ClockRange::LoopStop;
    let multiplier = 1.5;

    let clock = Clock::new(ClockOptions {
        start_time: Some(start),
        current_time: Some(current_time),
        clock_step: Some(step),
        multiplier: Some(multiplier),
        clock_range: Some(range),
        ..Default::default()
    })
    .unwrap();

    let expected_stop = start.add_days(1.0);
    assert_eq!(clock.start_time(), start);
    assert_eq!(clock.stop_time(), expected_stop);
    assert_eq!(clock.current_time(), current_time);
    assert_eq!(clock.clock_step(), step);
    assert_eq!(clock.clock_range(), range);
    assert_eq!(clock.multiplier(), multiplier);
    assert!(clock.can_animate);
    assert!(!clock.should_animate());
}

#[test]
fn throws_when_constructed_if_start_time_is_after_stop_time() {
    let start = jd(1.0);
    let stop = jd(0.0);
    let result = Clock::new(ClockOptions {
        start_time: Some(start),
        stop_time: Some(stop),
        ..Default::default()
    });
    assert!(result.is_err());
}

#[test]
fn animates_forward_in_tick_dependent_mode() {
    let start = jd(0.0);
    let stop = jd(1.0);
    let mut current_time = jd_sec(0.0, 43200.0);
    let multiplier = 1.5;

    let mut clock = Clock::new(ClockOptions {
        start_time: Some(start),
        stop_time: Some(stop),
        current_time: Some(current_time),
        clock_step: Some(ClockStep::TickDependent),
        multiplier: Some(multiplier),
        clock_range: Some(ClockRange::LoopStop),
        should_animate: Some(true),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(clock.current_time(), current_time);

    current_time = current_time.add_seconds(multiplier);
    assert_eq!(current_time, clock.tick());
    assert_eq!(clock.current_time(), current_time);

    current_time = current_time.add_seconds(multiplier);
    assert_eq!(current_time, clock.tick());
    assert_eq!(clock.current_time(), current_time);
}

#[test]
fn animates_backwards_in_tick_dependent_mode() {
    let start = jd(0.0);
    let stop = jd(1.0);
    let mut current_time = jd_sec(0.0, 43200.0);
    let multiplier = -1.5;

    let mut clock = Clock::new(ClockOptions {
        start_time: Some(start),
        stop_time: Some(stop),
        current_time: Some(current_time),
        clock_step: Some(ClockStep::TickDependent),
        multiplier: Some(multiplier),
        clock_range: Some(ClockRange::LoopStop),
        should_animate: Some(true),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(clock.current_time(), current_time);

    current_time = current_time.add_seconds(multiplier);
    assert_eq!(current_time, clock.tick());
    assert_eq!(clock.current_time(), current_time);

    current_time = current_time.add_seconds(multiplier);
    assert_eq!(current_time, clock.tick());
    assert_eq!(clock.current_time(), current_time);
}

#[test]
fn animates_forwards_past_stop_time_in_unbounded_tick_dependent_mode() {
    let start = jd(0.0);
    let stop = jd(1.0);
    let mut current_time = stop;
    let multiplier = 1.5;

    let mut clock = Clock::new(ClockOptions {
        start_time: Some(start),
        stop_time: Some(stop),
        current_time: Some(current_time),
        clock_step: Some(ClockStep::TickDependent),
        multiplier: Some(multiplier),
        clock_range: Some(ClockRange::Unbounded),
        should_animate: Some(true),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(clock.current_time(), current_time);

    current_time = current_time.add_seconds(multiplier);
    assert_eq!(current_time, clock.tick());
    assert_eq!(clock.current_time(), current_time);

    current_time = current_time.add_seconds(multiplier);
    assert_eq!(current_time, clock.tick());
    assert_eq!(clock.current_time(), current_time);
}

#[test]
fn animates_backwards_past_start_time_in_unbounded_tick_dependent_mode() {
    let start = jd(0.0);
    let stop = jd(1.0);
    let mut current_time = start;
    let multiplier = -1.5;

    let mut clock = Clock::new(ClockOptions {
        start_time: Some(start),
        stop_time: Some(stop),
        current_time: Some(current_time),
        clock_step: Some(ClockStep::TickDependent),
        multiplier: Some(multiplier),
        clock_range: Some(ClockRange::Unbounded),
        should_animate: Some(true),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(clock.current_time(), current_time);

    current_time = current_time.add_seconds(multiplier);
    assert_eq!(current_time, clock.tick());
    assert_eq!(clock.current_time(), current_time);

    current_time = current_time.add_seconds(multiplier);
    assert_eq!(current_time, clock.tick());
    assert_eq!(clock.current_time(), current_time);
}

#[test]
fn loops_back_to_start_time_when_animating_forward_past_stop_time_in_loop_stop_tick_dependent_mode() {
    let start = jd(0.0);
    let stop = jd(1.0);
    let current_time = stop;
    let multiplier = 1.5;

    let mut clock = Clock::new(ClockOptions {
        start_time: Some(start),
        stop_time: Some(stop),
        current_time: Some(current_time),
        clock_step: Some(ClockStep::TickDependent),
        multiplier: Some(multiplier),
        clock_range: Some(ClockRange::LoopStop),
        should_animate: Some(true),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(clock.current_time(), current_time);

    let mut expected = start.add_seconds(multiplier);
    assert_eq!(expected, clock.tick());
    assert_eq!(clock.current_time(), expected);

    expected = expected.add_seconds(multiplier);
    assert_eq!(expected, clock.tick());
    assert_eq!(clock.current_time(), expected);
}

#[test]
fn stops_at_start_when_animating_backwards_past_start_time_in_loop_stop_tick_dependent_mode() {
    let start = jd(0.0);
    let stop = jd(1.0);
    let current_time = start;
    let multiplier = -100.0;

    let mut clock = Clock::new(ClockOptions {
        start_time: Some(start),
        stop_time: Some(stop),
        current_time: Some(current_time),
        clock_step: Some(ClockStep::TickDependent),
        multiplier: Some(multiplier),
        clock_range: Some(ClockRange::LoopStop),
        should_animate: Some(true),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(clock.current_time(), current_time);
    assert_eq!(start, clock.tick());
    assert_eq!(start, clock.current_time());
}

#[test]
fn stops_at_stop_time_when_animating_forwards_past_stop_time_in_clamped_tick_dependent_mode() {
    let start = jd(0.0);
    let stop = jd(1.0);
    let current_time = stop;
    let multiplier = 100.0;

    let mut clock = Clock::new(ClockOptions {
        start_time: Some(start),
        stop_time: Some(stop),
        current_time: Some(current_time),
        clock_step: Some(ClockStep::TickDependent),
        multiplier: Some(multiplier),
        clock_range: Some(ClockRange::Clamped),
        should_animate: Some(true),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(clock.current_time(), current_time);
    assert_eq!(stop, clock.tick());
    assert_eq!(stop, clock.current_time());
}

#[test]
fn stops_at_start_time_when_animating_backwards_past_start_time_in_clamped_tick_dependent_mode() {
    let start = jd(0.0);
    let stop = jd(1.0);
    let current_time = start;
    let multiplier = -100.0;

    let mut clock = Clock::new(ClockOptions {
        start_time: Some(start),
        stop_time: Some(stop),
        current_time: Some(current_time),
        clock_step: Some(ClockStep::TickDependent),
        multiplier: Some(multiplier),
        clock_range: Some(ClockRange::Clamped),
        should_animate: Some(true),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(clock.current_time(), current_time);
    assert_eq!(start, clock.tick());
    assert_eq!(start, clock.current_time());
}

#[test]
fn fires_on_stop_event_when_end_time_is_reached_and_clock_range_is_clamped() {
    let start = jd(0.0);
    let stop = jd(1.0);
    let current_time = stop;
    let multiplier = 100.0;

    let mut clock = Clock::new(ClockOptions {
        start_time: Some(start),
        stop_time: Some(stop),
        current_time: Some(current_time),
        clock_step: Some(ClockStep::TickDependent),
        multiplier: Some(multiplier),
        clock_range: Some(ClockRange::Clamped),
        should_animate: Some(true),
        ..Default::default()
    })
    .unwrap();

    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();
    clock.on_stop.add_listener(move |_| {
        called_clone.set(true);
    });

    clock.tick();
    assert!(called.get());
}

#[test]
fn fires_on_stop_event_when_end_time_is_reached_and_clock_range_is_loop_stop() {
    let start = jd(0.0);
    let stop = jd(1.0);
    let current_time = stop;
    let multiplier = 100.0;

    let mut clock = Clock::new(ClockOptions {
        start_time: Some(start),
        stop_time: Some(stop),
        current_time: Some(current_time),
        clock_step: Some(ClockStep::TickDependent),
        multiplier: Some(multiplier),
        clock_range: Some(ClockRange::LoopStop),
        should_animate: Some(true),
        ..Default::default()
    })
    .unwrap();

    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();
    clock.on_stop.add_listener(move |_| {
        called_clone.set(true);
    });

    clock.tick();
    assert!(called.get());
}

#[test]
fn uses_current_time_in_system_clock_mode() {
    let mut clock = Clock::new(ClockOptions {
        clock_step: Some(ClockStep::SystemClock),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(clock.multiplier(), 1.0);
    assert!(clock.should_animate());

    let time1 = clock.tick_with_delta(Some(0.0));
    let time2 = clock.tick_with_delta(Some(1.0));
    assert_eq!(time2, time1.add_seconds(1.0));
}

#[test]
fn switches_out_of_system_clock_mode_when_changing_current_time() {
    let mut clock = Clock::new(ClockOptions {
        clock_step: Some(ClockStep::SystemClock),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(clock.clock_step(), ClockStep::SystemClock);

    let cur = clock.current_time();
    clock.set_current_time(cur);
    assert_eq!(clock.clock_step(), ClockStep::SystemClock);

    clock.set_current_time(jd(1.0));
    assert_eq!(clock.clock_step(), ClockStep::SystemClockMultiplier);
}

#[test]
fn switches_out_of_system_clock_mode_when_changing_multiplier() {
    let mut clock = Clock::new(ClockOptions {
        clock_step: Some(ClockStep::SystemClock),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(clock.clock_step(), ClockStep::SystemClock);

    clock.set_multiplier(1.0);
    assert_eq!(clock.clock_step(), ClockStep::SystemClock);

    clock.set_multiplier(1.5);
    assert_eq!(clock.clock_step(), ClockStep::SystemClockMultiplier);
}

#[test]
fn switches_out_of_system_clock_mode_when_changing_should_animate() {
    let mut clock = Clock::new(ClockOptions {
        clock_step: Some(ClockStep::SystemClock),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(clock.clock_step(), ClockStep::SystemClock);

    clock.set_should_animate(true);
    assert_eq!(clock.clock_step(), ClockStep::SystemClock);

    clock.set_should_animate(false);
    assert_eq!(clock.clock_step(), ClockStep::SystemClockMultiplier);
}

#[test]
fn sets_current_time_multiplier_and_should_animate_when_switching_to_system_clock_mode() {
    let mut clock = Clock::new(ClockOptions {
        current_time: Some(jd(1.0)),
        clock_step: Some(ClockStep::SystemClockMultiplier),
        multiplier: Some(1.5),
        should_animate: Some(false),
        ..Default::default()
    })
    .unwrap();

    clock.set_clock_step(ClockStep::SystemClock);
    assert_eq!(clock.clock_step(), ClockStep::SystemClock);
    assert_eq!(clock.multiplier(), 1.0);
    assert!(clock.should_animate());
}

#[test]
fn stays_in_system_clock_mode_when_changing_other_unrelated_properties() {
    let mut clock = Clock::new(ClockOptions {
        clock_step: Some(ClockStep::SystemClock),
        ..Default::default()
    })
    .unwrap();

    clock.set_start_time(jd(1.0));
    assert_eq!(clock.clock_step(), ClockStep::SystemClock);

    clock.set_stop_time(jd(2.0));
    assert_eq!(clock.clock_step(), ClockStep::SystemClock);

    clock.set_clock_range(ClockRange::Clamped);
    assert_eq!(clock.clock_step(), ClockStep::SystemClock);

    clock.can_animate = false;
    assert_eq!(clock.clock_step(), ClockStep::SystemClock);
}

#[test]
fn uses_multiplier_in_system_clock_multiplier_mode() {
    let start = jd(2457577.0);
    let mut clock = Clock::new(ClockOptions {
        current_time: Some(start),
        clock_step: Some(ClockStep::SystemClockMultiplier),
        multiplier: Some(2.0),
        should_animate: Some(true),
        ..Default::default()
    })
    .unwrap();

    let time1 = clock.tick_with_delta(Some(0.0));
    assert_eq!(time1, start);

    let time2 = clock.tick_with_delta(Some(1.0));
    assert_eq!(time2, start.add_seconds(2.0));
}

#[test]
fn does_not_advance_if_should_animate_is_false_and_does_advance_if_true() {
    let start = jd(2457577.0);
    let mut clock = Clock::new(ClockOptions {
        start_time: Some(start),
        current_time: Some(start),
        clock_step: Some(ClockStep::SystemClockMultiplier),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(clock.current_time(), start);
    clock.set_should_animate(false);

    let mut time1 = clock.tick_with_delta(Some(1.0));
    assert_eq!(time1, start);
    assert_eq!(clock.current_time(), start);

    clock.set_should_animate(true);
    time1 = clock.tick_with_delta(Some(1.0));
    assert_eq!(time1, start.add_seconds(1.0));

    let time2 = clock.tick_with_delta(Some(1.0));
    assert_eq!(time2, start.add_seconds(2.0));

    clock.set_current_time(start);
    clock.set_clock_step(ClockStep::TickDependent);

    clock.set_should_animate(false);
    time1 = clock.tick_with_delta(Some(1.0));
    assert_eq!(time1, start);
    assert_eq!(clock.current_time(), start);

    clock.set_should_animate(true);
    time1 = clock.tick_with_delta(Some(1.0));
    assert_eq!(time1, start.add_seconds(1.0));
}
