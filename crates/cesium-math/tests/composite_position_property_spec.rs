//! Direct port and verification against CesiumJS `CompositePositionPropertySpec.js` (Apache-2.0).
#![allow(clippy::field_reassign_with_default)]

use cesium_math::{
    Cartesian3, CompositePositionProperty, ConstantPositionProperty, JulianDate, PositionProperty,
    ReferenceFrame, TimeInterval, TimeStandard,
};

#[test]
fn default_constructor_has_expected_values() {
    let property = CompositePositionProperty::new(None);
    assert_eq!(property.intervals.len(), 0);
    assert_eq!(property.get_value(Some(&JulianDate::now())), None);
    assert_eq!(property.reference_frame, ReferenceFrame::Fixed);
    assert!(property.is_constant());

    let property_inertial = CompositePositionProperty::new(Some(ReferenceFrame::Inertial));
    assert_eq!(property_inertial.reference_frame, ReferenceFrame::Inertial);
}

#[test]
fn works_with_intervals_and_reference_frames() {
    let mut interval1 = TimeInterval::default();
    interval1.start = JulianDate::new(2450000.0, 10.0, TimeStandard::TAI);
    interval1.stop = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval1.data = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1.0, 2.0, 3.0)),
        Some(ReferenceFrame::Inertial),
    ));

    let mut interval2 = TimeInterval::default();
    interval2.start = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval2.stop = JulianDate::new(2450000.0, 14.0, TimeStandard::TAI);
    interval2.is_start_included = false;
    interval2.data = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(4.0, 5.0, 6.0)),
        Some(ReferenceFrame::Fixed),
    ));

    let mut property = CompositePositionProperty::new(None);
    property.intervals.add_interval(interval1);
    property.intervals.add_interval(interval2);

    assert!(!property.is_constant());

    let value_inertial = Cartesian3::new(1.0, 2.0, 3.0);
    let expected_fixed = PositionProperty::convert_to_reference_frame(
        Some(&JulianDate::new(2450000.0, 10.0, TimeStandard::TAI)),
        Some(value_inertial),
        ReferenceFrame::Inertial,
        ReferenceFrame::Fixed,
    );

    let result1 = property.get_value(Some(&JulianDate::new(2450000.0, 10.0, TimeStandard::TAI)));
    assert_eq!(result1, expected_fixed);

    let result2 = property.get_value(Some(&JulianDate::new(2450000.0, 14.0, TimeStandard::TAI)));
    assert_eq!(result2, Some(Cartesian3::new(4.0, 5.0, 6.0)));

    let result_inertial = property.get_value_in_reference_frame(
        Some(&JulianDate::new(2450000.0, 10.0, TimeStandard::TAI)),
        ReferenceFrame::Inertial,
    );
    assert_eq!(result_inertial, Some(value_inertial));
}

#[test]
fn equals_works() {
    let mut interval1 = TimeInterval::default();
    interval1.start = JulianDate::new(2450000.0, 10.0, TimeStandard::TAI);
    interval1.stop = JulianDate::new(2450000.0, 12.0, TimeStandard::TAI);
    interval1.data = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1.0, 2.0, 3.0)),
        None,
    ));

    let mut left = CompositePositionProperty::new(None);
    left.intervals.add_interval(interval1.clone());

    let mut right = CompositePositionProperty::new(None);
    right.intervals.add_interval(interval1);

    assert!(left.equals(Some(&right)));

    right.reference_frame = ReferenceFrame::Inertial;
    assert!(!left.equals(Some(&right)));
    assert!(!left.equals(None));
}
