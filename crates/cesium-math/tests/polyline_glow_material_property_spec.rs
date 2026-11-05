//! Direct port and verification against CesiumJS `PolylineGlowMaterialPropertySpec.js` (Apache-2.0).

use cesium_math::{Color, ConstantProperty, JulianDate, PolylineGlowMaterialProperty};

#[test]
fn constructor_provides_expected_defaults() {
    let property = PolylineGlowMaterialProperty::default();
    assert_eq!(property.get_type(), "PolylineGlow");
    assert!(property.is_constant());
    assert_eq!(property.color, None);
    assert_eq!(property.glow_power, None);
    assert_eq!(property.taper_power, None);

    let result = property.get_value(None);
    assert_eq!(result.color, Color::WHITE);
    assert_eq!(result.glow_power, 0.25);
    assert_eq!(result.taper_power, 1.0);
}

#[test]
fn works_with_custom_values() {
    let mut property = PolylineGlowMaterialProperty::default();
    property.color = Some(ConstantProperty::from_value(Color::RED));
    property.glow_power = Some(ConstantProperty::from_value(0.75));
    property.taper_power = Some(ConstantProperty::from_value(0.85));

    let result = property.get_value(Some(&JulianDate::now()));
    assert_eq!(result.color, Color::RED);
    assert_eq!(result.glow_power, 0.75);
    assert_eq!(result.taper_power, 0.85);
}

#[test]
fn equals_works() {
    let left = PolylineGlowMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(0.75)),
        Some(ConstantProperty::from_value(0.85)),
    );
    let right_equal = PolylineGlowMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(0.75)),
        Some(ConstantProperty::from_value(0.85)),
    );
    let right_diff = PolylineGlowMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::BLUE)),
        Some(ConstantProperty::from_value(0.75)),
        Some(ConstantProperty::from_value(0.85)),
    );

    assert!(left.equals(Some(&right_equal)));
    assert!(!left.equals(Some(&right_diff)));
    assert!(!left.equals(None));
}
