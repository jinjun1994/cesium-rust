use cesium_math::{JulianDate, TimeInterval, TimeStandard};

#[test]
fn test_time_interval_contains() {
    let t0 = JulianDate::new(2450000.0, 0.0, TimeStandard::TAI);
    let t1 = JulianDate::new(2450000.0, 100.0, TimeStandard::TAI);
    let t_mid = JulianDate::new(2450000.0, 50.0, TimeStandard::TAI);
    let t_outside = JulianDate::new(2450000.0, 150.0, TimeStandard::TAI);

    // [t0, t1]
    let interval = TimeInterval::new(t0, t1, true, true);
    assert!(interval.contains(&t_mid));
    assert!(interval.contains(&t0));
    assert!(interval.contains(&t1));
    assert!(!interval.contains(&t_outside));

    // (t0, t1)
    let open_interval = TimeInterval::new(t0, t1, false, false);
    assert!(open_interval.contains(&t_mid));
    assert!(!open_interval.contains(&t0));
    assert!(!open_interval.contains(&t1));
}

#[test]
fn test_time_interval_intersect() {
    let t0 = JulianDate::new(2450000.0, 0.0, TimeStandard::TAI);
    let t1 = JulianDate::new(2450000.0, 100.0, TimeStandard::TAI);
    let t2 = JulianDate::new(2450000.0, 50.0, TimeStandard::TAI);
    let t3 = JulianDate::new(2450000.0, 150.0, TimeStandard::TAI);

    let int1 = TimeInterval::new(t0, t1, true, true); // [0, 100]
    let int2 = TimeInterval::new(t2, t3, true, true); // [50, 150]

    let inter = int1.intersect(&int2).unwrap();
    assert_eq!(inter.start, t2);
    assert_eq!(inter.stop, t1);
    assert!(inter.is_start_included);
    assert!(inter.is_stop_included);

    // Disjoint
    let t4 = JulianDate::new(2450000.0, 200.0, TimeStandard::TAI);
    let t5 = JulianDate::new(2450000.0, 300.0, TimeStandard::TAI);
    let int3 = TimeInterval::new(t4, t5, true, true);
    assert!(int1.intersect(&int3).is_none());
}

#[test]
fn test_time_interval_empty_and_equality() {
    let t0 = JulianDate::new(2450000.0, 0.0, TimeStandard::TAI);
    let t1 = JulianDate::new(2450000.0, 100.0, TimeStandard::TAI);

    let valid = TimeInterval::new(t0, t1, true, true);
    assert!(!valid.is_empty());

    let inverted = TimeInterval::new(t1, t0, true, true);
    assert!(inverted.is_empty());

    let cloned = valid;
    assert!(valid.equals(&cloned));
    assert!(valid.equals_epsilon(&cloned, 1e-12));
}
