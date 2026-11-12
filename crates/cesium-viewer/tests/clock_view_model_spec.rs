use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{Clock, ClockOptions, ClockRange, ClockStep, JulianDate};
use cesium_viewer::widgets::ClockViewModel;

#[test]
fn default_constructor_creates_a_clock() {
    let clock_view_model = ClockViewModel::default();
    let clock = clock_view_model.clock();
    let c = clock.borrow();
    assert_eq!(c.clock_range(), ClockRange::Unbounded);
}

#[test]
fn constructor_sets_expected_properties() {
    let start_time = JulianDate::from_iso8601("2012-01-01T00:00:00Z").unwrap();
    let stop_time = JulianDate::from_iso8601("2012-01-02T00:00:00Z").unwrap();
    let current_time = JulianDate::from_iso8601("2012-01-01T12:00:00Z").unwrap();

    let clock = Clock::new(ClockOptions {
        start_time: Some(start_time),
        stop_time: Some(stop_time),
        current_time: Some(current_time),
        multiplier: Some(1.0),
        clock_step: Some(ClockStep::TickDependent),
        clock_range: Some(ClockRange::Unbounded),
        can_animate: Some(false),
        should_animate: Some(false),
    })
    .unwrap();

    let clock_rc = Rc::new(RefCell::new(clock));
    let clock_view_model = ClockViewModel::new(Some(Rc::clone(&clock_rc)));

    assert!(Rc::ptr_eq(&clock_view_model.clock(), &clock_rc));
    assert_eq!(clock_view_model.start_time(), start_time);
    assert_eq!(clock_view_model.stop_time(), stop_time);
    assert_eq!(clock_view_model.current_time(), current_time);
    assert_eq!(clock_view_model.multiplier(), 1.0);
    assert_eq!(clock_view_model.clock_step(), ClockStep::TickDependent);
    assert_eq!(clock_view_model.clock_range(), ClockRange::Unbounded);
    assert!(!clock_view_model.can_animate());
    assert!(!clock_view_model.should_animate());
    assert!(clock_view_model.system_time().day_number > 0);
}

#[test]
fn observables_are_updated_from_the_clock() {
    let start_time = JulianDate::from_iso8601("2012-01-01T00:00:00Z").unwrap();
    let stop_time = JulianDate::from_iso8601("2012-01-02T00:00:00Z").unwrap();
    let current_time = JulianDate::from_iso8601("2012-01-01T12:00:00Z").unwrap();

    let clock = Clock::new(ClockOptions {
        start_time: Some(start_time),
        stop_time: Some(stop_time),
        current_time: Some(current_time),
        multiplier: Some(1.0),
        clock_step: Some(ClockStep::TickDependent),
        clock_range: Some(ClockRange::Unbounded),
        can_animate: Some(true),
        should_animate: Some(false),
    })
    .unwrap();

    let clock_rc = Rc::new(RefCell::new(clock));
    let clock_view_model = ClockViewModel::new(Some(Rc::clone(&clock_rc)));

    let new_start_time = JulianDate::from_iso8601("2013-01-01T00:00:00Z").unwrap();
    let new_stop_time = JulianDate::from_iso8601("2013-01-02T00:00:00Z").unwrap();
    let new_current_time = JulianDate::from_iso8601("2013-01-01T12:00:00Z").unwrap();

    {
        let mut c = clock_rc.borrow_mut();
        c.set_start_time(new_start_time);
        c.set_stop_time(new_stop_time);
        c.set_current_time(new_current_time);
        c.set_multiplier(2.0);
        c.set_clock_step(ClockStep::SystemClockMultiplier);
        c.set_clock_range(ClockRange::Clamped);
        c.set_should_animate(true);
    }

    // Before tick or synchronize, view model retains old values
    assert_ne!(clock_view_model.start_time(), new_start_time);
    assert_ne!(clock_view_model.stop_time(), new_stop_time);
    assert_ne!(clock_view_model.current_time(), new_current_time);
    assert_ne!(clock_view_model.multiplier(), 2.0);
    assert_ne!(
        clock_view_model.clock_step(),
        ClockStep::SystemClockMultiplier
    );
    assert_ne!(clock_view_model.clock_range(), ClockRange::Clamped);
    assert!(!clock_view_model.should_animate());

    // Advance clock tick, triggering on_tick subscriber
    clock_rc.borrow_mut().tick();

    assert_eq!(clock_view_model.start_time(), new_start_time);
    assert_eq!(clock_view_model.stop_time(), new_stop_time);
    // tick advanced current time, clock_view_model current_time reflects clock's tick value
    assert_eq!(
        clock_view_model.current_time(),
        clock_rc.borrow().current_time()
    );
    assert_eq!(clock_view_model.multiplier(), 2.0);
    assert_eq!(
        clock_view_model.clock_step(),
        ClockStep::SystemClockMultiplier
    );
    assert_eq!(clock_view_model.clock_range(), ClockRange::Clamped);
    assert!(clock_view_model.should_animate());
}

#[test]
fn modifying_view_model_updates_underlying_clock() {
    let clock_view_model = ClockViewModel::default();
    let test_time = JulianDate::from_iso8601("2025-06-15T08:30:00Z").unwrap();

    clock_view_model.set_multiplier(4.0);
    assert_eq!(clock_view_model.multiplier(), 4.0);
    assert_eq!(clock_view_model.clock().borrow().multiplier(), 4.0);

    clock_view_model.set_current_time(test_time);
    assert_eq!(clock_view_model.current_time(), test_time);
    assert_eq!(clock_view_model.clock().borrow().current_time(), test_time);

    clock_view_model.set_clock_range(ClockRange::LoopStop);
    assert_eq!(clock_view_model.clock_range(), ClockRange::LoopStop);
    assert_eq!(
        clock_view_model.clock().borrow().clock_range(),
        ClockRange::LoopStop
    );

    assert!(!clock_view_model.is_destroyed());
    clock_view_model.destroy();
    assert!(clock_view_model.is_destroyed());
}
