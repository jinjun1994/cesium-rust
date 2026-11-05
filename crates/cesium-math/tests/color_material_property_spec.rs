//! Direct port and verification against CesiumJS `ColorMaterialPropertySpec.js` (Apache-2.0).

use cesium_math::{Color, ColorMaterialProperty, ConstantProperty, JulianDate};

#[test]
fn constructor_provides_expected_defaults() {
    let property = ColorMaterialProperty::default();
    assert_eq!(property.color, None);
    assert_eq!(property.get_type(), "Color");
    assert!(property.is_constant());
    assert_eq!(property.get_value(None), Color::WHITE);

    let color_prop = ConstantProperty::from_value(Color::BLUE);
    let property = ColorMaterialProperty::new(Some(color_prop));
    assert_eq!(property.get_value(None), Color::BLUE);

    let property = ColorMaterialProperty::from_color(Color::BLUE);
    assert_eq!(property.get_value(None), Color::BLUE);
}

#[test]
fn works_with_constant_values() {
    let mut property = ColorMaterialProperty::default();
    property.color = Some(ConstantProperty::from_value(Color::RED));

    let result = property.get_value(Some(&JulianDate::now()));
    assert_eq!(result, Color::RED);
}

#[test]
fn equals_works() {
    let left = ColorMaterialProperty::from_color(Color::WHITE);
    let right_equal = ColorMaterialProperty::from_color(Color::WHITE);
    let right_diff = ColorMaterialProperty::from_color(Color::BLACK);

    assert!(left.equals(Some(&right_equal)));
    assert!(!left.equals(Some(&right_diff)));
    assert!(!left.equals(None));
}
