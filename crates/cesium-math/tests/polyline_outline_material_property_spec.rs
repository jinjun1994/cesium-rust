//! Direct port and verification against CesiumJS `PolylineOutlineMaterialPropertySpec.js` (Apache-2.0).

use cesium_math::{Color, ConstantProperty, JulianDate, PolylineOutlineMaterialProperty};

#[test]
fn constructor_provides_expected_defaults() {
    let property = PolylineOutlineMaterialProperty::default();
    assert_eq!(property.get_type(), "PolylineOutline");
    assert!(property.is_constant());
    assert_eq!(property.color, None);
    assert_eq!(property.outline_color, None);
    assert_eq!(property.outline_width, None);

    let result = property.get_value(None);
    assert_eq!(result.color, Color::WHITE);
    assert_eq!(result.outline_color, Color::BLACK);
    assert_eq!(result.outline_width, 1.0);
}

#[test]
fn works_with_custom_values() {
    let mut property = PolylineOutlineMaterialProperty::default();
    property.color = Some(ConstantProperty::from_value(Color::RED));
    property.outline_color = Some(ConstantProperty::from_value(Color::BLUE));
    property.outline_width = Some(ConstantProperty::from_value(5.0));

    let result = property.get_value(Some(&JulianDate::now()));
    assert_eq!(result.color, Color::RED);
    assert_eq!(result.outline_color, Color::BLUE);
    assert_eq!(result.outline_width, 5.0);
}

#[test]
fn equals_works() {
    let left = PolylineOutlineMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(Color::BLUE)),
        Some(ConstantProperty::from_value(5.0)),
    );
    let right_equal = PolylineOutlineMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(Color::BLUE)),
        Some(ConstantProperty::from_value(5.0)),
    );
    let right_diff = PolylineOutlineMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(Color::BLUE)),
        Some(ConstantProperty::from_value(1.0)),
    );

    assert!(left.equals(Some(&right_equal)));
    assert!(!left.equals(Some(&right_diff)));
    assert!(!left.equals(None));
}
