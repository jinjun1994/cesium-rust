//! Direct port and verification against CesiumJS `PolylineDashMaterialPropertySpec.js` (Apache-2.0).

use cesium_math::{Color, ConstantProperty, JulianDate, PolylineDashMaterialProperty};

#[test]
fn constructor_provides_expected_defaults() {
    let property = PolylineDashMaterialProperty::default();
    assert_eq!(property.get_type(), "PolylineDash");
    assert!(property.is_constant());
    assert_eq!(property.color, None);
    assert_eq!(property.gap_color, None);
    assert_eq!(property.dash_length, None);
    assert_eq!(property.dash_pattern, None);

    let result = property.get_value(None);
    assert_eq!(result.color, Color::WHITE);
    assert_eq!(result.gap_color, Color::TRANSPARENT);
    assert_eq!(result.dash_length, 16.0);
    assert_eq!(result.dash_pattern, 255.0);
}

#[test]
fn works_with_custom_values() {
    let mut property = PolylineDashMaterialProperty::default();
    property.color = Some(ConstantProperty::from_value(Color::RED));
    property.gap_color = Some(ConstantProperty::from_value(Color::YELLOW));
    property.dash_length = Some(ConstantProperty::from_value(10.0));
    property.dash_pattern = Some(ConstantProperty::from_value(7.0));

    let result = property.get_value(Some(&JulianDate::now()));
    assert_eq!(result.color, Color::RED);
    assert_eq!(result.gap_color, Color::YELLOW);
    assert_eq!(result.dash_length, 10.0);
    assert_eq!(result.dash_pattern, 7.0);
}

#[test]
fn equals_works() {
    let left = PolylineDashMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(Color::YELLOW)),
        Some(ConstantProperty::from_value(10.0)),
        Some(ConstantProperty::from_value(7.0)),
    );
    let right_equal = PolylineDashMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(Color::YELLOW)),
        Some(ConstantProperty::from_value(10.0)),
        Some(ConstantProperty::from_value(7.0)),
    );
    let right_diff = PolylineDashMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(Color::BLACK)),
        Some(ConstantProperty::from_value(10.0)),
        Some(ConstantProperty::from_value(7.0)),
    );

    assert!(left.equals(Some(&right_equal)));
    assert!(!left.equals(Some(&right_diff)));
    assert!(!left.equals(None));
}
