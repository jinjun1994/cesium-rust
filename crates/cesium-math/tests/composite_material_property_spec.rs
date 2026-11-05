//! Direct port and verification against CesiumJS `DataSources/CompositeMaterialPropertySpec.js` (Apache-2.0).

use cesium_math::{
    Color, ColorMaterialProperty, CompositeMaterialProperty, GridMaterialProperty, JulianDate,
    MaterialValue, TimeInterval, TimeStandard,
};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn default_constructor_has_expected_values() {
    let property = CompositeMaterialProperty::new();
    assert_eq!(property.intervals.len(), 0);
    assert!(property.intervals.is_empty());
    assert!(property.is_constant());
    let now = JulianDate::now();
    assert_eq!(property.get_type(Some(&now)), None);
    assert_eq!(property.get_value(Some(&now)), None);
}

#[test]
fn works_without_a_result_parameter() {
    let color_prop = Rc::new(ColorMaterialProperty::default());
    let grid_prop = Rc::new(GridMaterialProperty::default());

    let interval1 = TimeInterval {
        start: JulianDate::new(10.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        data: Some(color_prop.clone().into()),
        ..Default::default()
    };
    let interval2 = TimeInterval {
        start: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(14.0, 0.0, TimeStandard::TAI),
        is_start_included: false,
        data: Some(grid_prop.clone().into()),
        ..Default::default()
    };

    let mut property = CompositeMaterialProperty::new();
    property.intervals.add_interval(interval1);
    property.intervals.add_interval(interval2);
    assert!(!property.is_constant());

    let t1 = JulianDate::new(10.0, 0.0, TimeStandard::TAI);
    let result1 = property.get_value(Some(&t1));
    assert_eq!(property.get_type(Some(&t1)), Some("Color"));
    assert_eq!(result1, Some(MaterialValue::Color(color_prop.get_value(Some(&t1)))));

    let t2 = JulianDate::new(14.0, 0.0, TimeStandard::TAI);
    let result2 = property.get_value(Some(&t2));
    assert_eq!(property.get_type(Some(&t2)), Some("Grid"));
    assert_eq!(result2, Some(MaterialValue::Grid(grid_prop.get_value(Some(&t2)))));
}

#[test]
fn equals_works() {
    let interval1 = TimeInterval {
        start: JulianDate::new(10.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        data: Some(ColorMaterialProperty::default().into()),
        ..Default::default()
    };
    let interval2 = TimeInterval {
        start: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(14.0, 0.0, TimeStandard::TAI),
        is_start_included: false,
        data: Some(GridMaterialProperty::default().into()),
        ..Default::default()
    };

    let mut left = CompositeMaterialProperty::new();
    left.intervals.add_interval(interval1.clone());
    left.intervals.add_interval(interval2.clone());

    let mut right = CompositeMaterialProperty::new();
    right.intervals.add_interval(interval1);
    assert!(!left.equals(Some(&right)));

    right.intervals.add_interval(interval2);
    assert!(left.equals(Some(&right)));
}

#[test]
fn raises_definition_changed_event_in_all_cases() {
    let color_prop1 = Rc::new(ColorMaterialProperty::from_color(Color::RED));
    let color_prop2 = Rc::new(ColorMaterialProperty::from_color(Color::YELLOW));

    let interval1 = TimeInterval {
        start: JulianDate::new(10.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        data: Some(color_prop1.clone().into()),
        ..Default::default()
    };
    let interval2 = TimeInterval {
        start: JulianDate::new(12.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(14.0, 0.0, TimeStandard::TAI),
        is_start_included: false,
        data: Some(color_prop2.into()),
        ..Default::default()
    };

    let mut property = CompositeMaterialProperty::new();
    let count = Rc::new(Cell::new(0));
    let c = count.clone();
    let _sub = property.definition_changed().add_listener(move |_| {
        c.set(c.get() + 1);
    });

    property.intervals.add_interval(interval1);
    assert_eq!(count.get(), 1);

    property.intervals.add_interval(interval2.clone());
    assert_eq!(count.get(), 2);

    property.intervals.remove_interval(&interval2);
    assert_eq!(count.get(), 3);

    // Modifying child property's color inner value raises parent definitionChanged
    color_prop1.color.as_ref().unwrap().set_value(Some(Color::BLUE));
    assert_eq!(count.get(), 4);

    property.intervals.remove_all();
    assert_eq!(count.get(), 5);
}

#[test]
fn does_not_raise_definition_changed_for_an_overwritten_interval() {
    let color_prop1 = Rc::new(ColorMaterialProperty::from_color(Color::RED));
    let color_prop2 = Rc::new(ColorMaterialProperty::from_color(Color::YELLOW));

    let interval1 = TimeInterval {
        start: JulianDate::new(11.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(13.0, 0.0, TimeStandard::TAI),
        data: Some(color_prop1.clone().into()),
        ..Default::default()
    };
    let interval2 = TimeInterval {
        start: JulianDate::new(10.0, 0.0, TimeStandard::TAI),
        stop: JulianDate::new(14.0, 0.0, TimeStandard::TAI),
        is_start_included: false,
        data: Some(color_prop2.into()),
        ..Default::default()
    };

    let mut property = CompositeMaterialProperty::new();
    let count = Rc::new(Cell::new(0));
    let c = count.clone();
    let _sub = property.definition_changed().add_listener(move |_| {
        c.set(c.get() + 1);
    });

    property.intervals.add_interval(interval1);
    property.intervals.add_interval(interval2);
    assert_eq!(count.get(), 2);

    // interval2 overwrote interval1, so modifying color_prop1 does not raise definitionChanged
    color_prop1.color.as_ref().unwrap().set_value(Some(Color::BLUE));
    assert_eq!(count.get(), 2);
}

#[test]
fn get_value_uses_now_if_time_is_none() {
    let property = CompositeMaterialProperty::new();
    assert_eq!(property.get_value(None), None);
    assert_eq!(property.get_type(None), None);
}
