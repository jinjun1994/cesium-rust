//! Direct port and verification against CesiumJS `CheckerboardMaterialPropertySpec.js` (Apache-2.0).

use cesium_math::{
    Cartesian2, CheckerboardMaterialProperty, Color, ConstantProperty, JulianDate,
};

#[test]
fn constructor_provides_expected_defaults() {
    let property = CheckerboardMaterialProperty::default();
    assert_eq!(property.get_type(), "Checkerboard");
    assert!(property.is_constant());
    assert_eq!(property.even_color, None);
    assert_eq!(property.odd_color, None);
    assert_eq!(property.repeat, None);

    let result = property.get_value(None);
    assert_eq!(result.light_color, Color::WHITE);
    assert_eq!(result.dark_color, Color::BLACK);
    assert_eq!(result.repeat, Cartesian2::new(2.0, 2.0));
}

#[test]
fn works_with_custom_values() {
    let mut property = CheckerboardMaterialProperty::default();
    property.even_color = Some(ConstantProperty::from_value(Color::RED));
    property.odd_color = Some(ConstantProperty::from_value(Color::BLUE));
    property.repeat = Some(ConstantProperty::from_value(Cartesian2::new(5.0, 5.0)));

    let result = property.get_value(Some(&JulianDate::now()));
    assert_eq!(result.light_color, Color::RED);
    assert_eq!(result.dark_color, Color::BLUE);
    assert_eq!(result.repeat, Cartesian2::new(5.0, 5.0));
}

#[test]
fn equals_works() {
    let prop1 = CheckerboardMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(Color::BLUE)),
        Some(ConstantProperty::from_value(Cartesian2::new(1.0, 1.0))),
    );
    let prop2 = CheckerboardMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(Color::BLUE)),
        Some(ConstantProperty::from_value(Cartesian2::new(1.0, 1.0))),
    );
    let prop3 = CheckerboardMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::WHITE)),
        Some(ConstantProperty::from_value(Color::BLUE)),
        Some(ConstantProperty::from_value(Cartesian2::new(1.0, 1.0))),
    );

    assert!(prop1.equals(Some(&prop2)));
    assert!(!prop1.equals(Some(&prop3)));
    assert!(!prop1.equals(None));
}
