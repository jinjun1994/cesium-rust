//! Direct port and verification against CesiumJS `TimeIntervalCollectionPositionPropertySpec.js` (Apache-2.0).
#![allow(clippy::field_reassign_with_default)]

use cesium_math::{
    Cartesian3, JulianDate, PositionProperty, ReferenceFrame, TimeInterval,
    TimeIntervalCollectionPositionProperty, TimeStandard,
};

#[test]
fn default_constructor_has_expected_values() {
    let property = TimeIntervalCollectionPositionProperty::new(None);
    assert_eq!(property.intervals.len(), 0);
    assert_eq!(property.get_value(Some(&JulianDate::now())), None);
    assert_eq!(property.reference_frame, ReferenceFrame::Fixed);
}

#[test]
fn works_with_values_and_reference_frames() {
    let mut interval1 = TimeInterval::default();
    interval1.start = JulianDate::new(2450000.0, 10.0, TimeStandard::TAI);
    interval1.stop = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval1.data = Some(Cartesian3::new(1.0, 2.0, 3.0));

    let mut interval2 = TimeInterval::default();
    interval2.start = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval2.stop = JulianDate::new(2450000.0, 14.0, TimeStandard::TAI);
    interval2.is_start_included = false;
    interval2.data = Some(Cartesian3::new(4.0, 5.0, 6.0));

    let mut property = TimeIntervalCollectionPositionProperty::new(None);
    property.intervals.add_interval(interval1);
    property.intervals.add_interval(interval2);

    let result1 = property.get_value(Some(&JulianDate::new(2450000.0, 10.0, TimeStandard::TAI)));
    assert_eq!(result1, Some(Cartesian3::new(1.0, 2.0, 3.0)));

    let result2 = property.get_value(Some(&JulianDate::new(2450000.0, 14.0, TimeStandard::TAI)));
    assert_eq!(result2, Some(Cartesian3::new(4.0, 5.0, 6.0)));

    // Test with Inertial frame
    let mut property_inertial = TimeIntervalCollectionPositionProperty::new(Some(ReferenceFrame::Inertial));
    property_inertial.intervals.add_interval(interval1);

    let expected_fixed = PositionProperty::convert_to_reference_frame(
        Some(&JulianDate::new(2450000.0, 10.0, TimeStandard::TAI)),
        Some(Cartesian3::new(1.0, 2.0, 3.0)),
        ReferenceFrame::Inertial,
        ReferenceFrame::Fixed,
    );

    let result_fixed = property_inertial.get_value(Some(&JulianDate::new(2450000.0, 10.0, TimeStandard::TAI)));
    assert_eq!(result_fixed, expected_fixed);

    let result_inertial = property_inertial.get_value_in_reference_frame(
        Some(&JulianDate::new(2450000.0, 10.0, TimeStandard::TAI)),
        ReferenceFrame::Inertial,
    );
    assert_eq!(result_inertial, Some(Cartesian3::new(1.0, 2.0, 3.0)));
}

#[test]
fn equals_works() {
    let mut interval1 = TimeInterval::default();
    interval1.start = JulianDate::new(2450000.0, 10.0, TimeStandard::TAI);
    interval1.stop = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval1.data = Some(Cartesian3::new(1.0, 2.0, 3.0));

    let mut left = TimeIntervalCollectionPositionProperty::new(None);
    left.intervals.add_interval(interval1);

    let mut right = TimeIntervalCollectionPositionProperty::new(None);
    right.intervals.add_interval(interval1);

    assert!(left.equals(Some(&right)));

    right.reference_frame = ReferenceFrame::Inertial;
    assert!(!left.equals(Some(&right)));
    assert!(!left.equals(None));
}
