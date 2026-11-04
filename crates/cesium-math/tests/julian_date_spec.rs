use cesium_math::{
    GregorianDate, JulianDate, LeapSecond, TimeConstants, TimeStandard, LEAP_SECONDS,
};
use std::cmp::Ordering;

#[test]
fn test_time_constants_upstream() {
    assert_eq!(TimeConstants::SECONDS_PER_DAY, 86400.0);
    assert_eq!(TimeConstants::SECONDS_PER_HOUR, 3600.0);
    assert_eq!(TimeConstants::SECONDS_PER_MINUTE, 60.0);
    assert_eq!(TimeConstants::HOURS_PER_DAY, 24.0);
    assert_eq!(TimeConstants::MINUTES_PER_DAY, 1440.0);
    assert_eq!(TimeConstants::DAYS_PER_JULIAN_CENTURY, 36525.0);
    assert_eq!(TimeConstants::MODIFIED_JULIAN_DATE_DIFFERENCE, 2400000.5);
}

#[test]
fn test_leap_seconds_table() {
    assert_eq!(LEAP_SECONDS.len(), 28);
    let ls = LeapSecond::new(JulianDate::new_tai(2441317, 43210.0), 10.0);
    assert_eq!(ls.julian_date.day_number, 2441317);
    assert_eq!(ls.offset, 10.0);
}

#[test]
fn test_julian_date_constructors_upstream() {
    // Construct default date
    let default_date = JulianDate::default();
    assert_eq!(default_date.day_number, 0);
    assert_eq!(default_date.seconds_of_day, 10.0);

    // Construct fractional day matching upstream test:
    // new JulianDate(2448257.75, 0, TimeStandard.UTC) === new JulianDate(2448257, 64826, TimeStandard.TAI)
    let jd_frac = JulianDate::new(2448257.75, 0.0, TimeStandard::UTC);
    let expected = JulianDate::new(2448257.0, 64826.0, TimeStandard::TAI);
    assert_eq!(jd_frac.day_number, expected.day_number);
    assert_eq!(jd_frac.seconds_of_day, expected.seconds_of_day);

    // Construct basic TAI
    let tai_date = JulianDate::new(12.0, 12.5, TimeStandard::TAI);
    assert_eq!(tai_date.day_number, 12);
    assert_eq!(tai_date.seconds_of_day, 12.5);
}

#[test]
fn test_julian_date_arithmetic_and_difference() {
    let date = JulianDate::new(2450000.0, 100.0, TimeStandard::TAI);
    let plus_100 = date.add_seconds(100.0);
    assert_eq!(plus_100.seconds_of_day, 200.0);
    assert_eq!(date.seconds_difference(&plus_100), -100.0);
    assert_eq!(plus_100.seconds_difference(&date), 100.0);

    let plus_one_day = date.add_days(1.0);
    assert_eq!(plus_one_day.day_number, 2450001);
    assert_eq!(plus_one_day.days_difference(&date), 1.0);

    assert_eq!(date.compare(&plus_100), Ordering::Less);
    assert_eq!(plus_100.compare(&date), Ordering::Greater);
    assert_eq!(date.compare(&date), Ordering::Equal);
    assert!(date.less_than(&plus_100));
    assert!(plus_100.greater_than(&date));
    assert!(date.equals(&date));
    assert!(date.equals_epsilon(&plus_100, 150.0));
    assert!(!date.equals_epsilon(&plus_100, 50.0));
}

#[test]
fn test_julian_date_iso8601_and_gregorian() {
    let iso = "2000-01-01T12:00:00Z";
    let jd = JulianDate::from_iso8601(iso).unwrap();

    let g = jd.to_gregorian_date();
    assert_eq!(g.year, 2000);
    assert_eq!(g.month, 1);
    assert_eq!(g.day, 1);
    assert_eq!(g.hour, 12);
    assert_eq!(g.minute, 0);
    assert_eq!(g.second, 0);

    let iso_out = jd.to_iso8601();
    assert_eq!(iso_out, iso);
}

#[test]
fn test_gregorian_date_default() {
    let g = GregorianDate::default();
    assert_eq!(g.year, 1);
    assert_eq!(g.month, 1);
    assert_eq!(g.day, 1);
    assert_eq!(g.hour, 0);
    assert_eq!(g.minute, 0);
    assert_eq!(g.second, 0);
    assert_eq!(g.millisecond, 0.0);
    assert!(!g.is_leap_second);
}
