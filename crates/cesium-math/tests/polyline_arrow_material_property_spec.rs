//! Direct port and verification against CesiumJS `PolylineArrowMaterialPropertySpec.js` (Apache-2.0).

use cesium_math::{Color, ConstantProperty, JulianDate, PolylineArrowMaterialProperty};

#[test]
fn constructor_provides_expected_defaults() {
    let property = PolylineArrowMaterialProperty::default();
    assert_eq!(property.get_type(), "PolylineArrow");
    assert!(property.is_constant());
    assert_eq!(property.color, None);

    let result = property.get_value(None);
    assert_eq!(result, Color::WHITE);

    let color_prop = ConstantProperty::from_value(Color::BLUE);
    let property = PolylineArrowMaterialProperty::new(Some(color_prop));
    assert_eq!(property.get_value(None), Color::BLUE);

    let property = PolylineArrowMaterialProperty::from_color(Color::BLUE);
    assert_eq!(property.get_value(None), Color::BLUE);
}

#[test]
fn works_with_constant_values() {
    let mut property = PolylineArrowMaterialProperty::default();
    property.color = Some(ConstantProperty::from_value(Color::RED));

    let result = property.get_value(Some(&JulianDate::now()));
    assert_eq!(result, Color::RED);
}

#[test]
fn equals_works() {
    let left = PolylineArrowMaterialProperty::from_color(Color::WHITE);
    let right_equal = PolylineArrowMaterialProperty::from_color(Color::WHITE);
    let right_diff = PolylineArrowMaterialProperty::from_color(Color::BLACK);

    assert!(left.equals(Some(&right_equal)));
    assert!(!left.equals(Some(&right_diff)));
    assert!(!left.equals(None));
}
