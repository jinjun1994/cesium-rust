//! Direct port and verification against CesiumJS `DataSourceClockSpec.js` (Apache-2.0).

use cesium_math::{ClockRange, ClockStep, DataSourceClock, JulianDate};

#[test]
fn test_merge_assigns_unassigned_properties() {
    let mut source = DataSourceClock::new();
    source.start_time = Some(JulianDate::now());
    source.stop_time = Some(JulianDate::now());
    source.current_time = Some(JulianDate::now());
    source.clock_range = Some(ClockRange::Clamped);
    source.clock_step = Some(ClockStep::TickDependent);
    source.multiplier = Some(1.0);

    let mut target = DataSourceClock::new();
    target.merge(&source);

    assert_eq!(target.start_time, source.start_time);
    assert_eq!(target.stop_time, source.stop_time);
    assert_eq!(target.current_time, source.current_time);
    assert_eq!(target.clock_range, source.clock_range);
    assert_eq!(target.clock_step, source.clock_step);
    assert_eq!(target.multiplier, source.multiplier);
}

#[test]
fn test_merge_does_not_assign_assigned_properties() {
    let mut source = DataSourceClock::new();
    source.start_time = Some(JulianDate::now());
    source.stop_time = Some(JulianDate::now());
    source.current_time = Some(JulianDate::now());
    source.clock_range = Some(ClockRange::LoopStop);
    source.clock_step = Some(ClockStep::SystemClock);
    source.multiplier = Some(10.0);

    let start_time = JulianDate::from_iso8601("2020-01-01T00:00:00Z").unwrap();
    let stop_time = JulianDate::from_iso8601("2020-01-02T00:00:00Z").unwrap();
    let current_time = start_time;
    let clock_range = ClockRange::Clamped;
    let clock_step = ClockStep::TickDependent;
    let multiplier = 1.0;

    let mut target = DataSourceClock::new();
    target.start_time = Some(start_time);
    target.stop_time = Some(stop_time);
    target.current_time = Some(current_time);
    target.clock_range = Some(clock_range);
    target.clock_step = Some(clock_step);
    target.multiplier = Some(multiplier);

    target.merge(&source);

    assert_eq!(target.start_time, Some(start_time));
    assert_eq!(target.stop_time, Some(stop_time));
    assert_eq!(target.current_time, Some(current_time));
    assert_eq!(target.clock_range, Some(clock_range));
    assert_eq!(target.clock_step, Some(clock_step));
    assert_eq!(target.multiplier, Some(multiplier));
}

#[test]
fn test_clone_works() {
    let mut source = DataSourceClock::new();
    source.start_time = Some(JulianDate::now());
    source.stop_time = Some(JulianDate::now());
    source.current_time = Some(JulianDate::now());
    source.clock_range = Some(ClockRange::Clamped);
    source.clock_step = Some(ClockStep::TickDependent);
    source.multiplier = Some(1.0);

    let result = source.clone();
    assert_eq!(result.start_time, source.start_time);
    assert_eq!(result.stop_time, source.stop_time);
    assert_eq!(result.current_time, source.current_time);
    assert_eq!(result.clock_range, source.clock_range);
    assert_eq!(result.clock_step, source.clock_step);
    assert_eq!(result.multiplier, source.multiplier);

    let mut existing = DataSourceClock::new();
    source.clone_to(&mut existing);
    assert_eq!(existing, source);
}

#[test]
fn test_merge_throws_if_source_undefined() {
    let mut target = DataSourceClock::new();
    assert!(target.try_merge(None).is_err());
}

#[test]
fn test_gets_value_as_clock_instance() {
    let start = JulianDate::from_iso8601("2020-01-01T00:00:00Z").unwrap();
    let stop = JulianDate::from_iso8601("2020-01-02T00:00:00Z").unwrap();
    let current = start;

    let mut source = DataSourceClock::new();
    source.start_time = Some(start);
    source.stop_time = Some(stop);
    source.current_time = Some(current);
    source.clock_range = Some(ClockRange::Clamped);
    source.clock_step = Some(ClockStep::TickDependent);
    source.multiplier = Some(2.0);

    let clock = source.get_value();
    assert_eq!(clock.start_time(), start);
    assert_eq!(clock.stop_time(), stop);
    assert_eq!(clock.current_time(), current);
    assert_eq!(clock.clock_range(), ClockRange::Clamped);
    assert_eq!(clock.clock_step(), ClockStep::TickDependent);
    assert_eq!(clock.multiplier(), 2.0);

    // Default fallbacks when properties are None
    let default_source = DataSourceClock::new();
    let default_clock = default_source.get_value();
    assert_eq!(default_clock.clock_range(), ClockRange::Unbounded);
    assert_eq!(default_clock.clock_step(), ClockStep::SystemClockMultiplier);
    assert_eq!(default_clock.multiplier(), 1.0);
}
