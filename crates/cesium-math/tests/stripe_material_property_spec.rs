//! Direct port and verification against CesiumJS `StripeMaterialPropertySpec.js` (Apache-2.0).

use cesium_math::{Color, ConstantProperty, JulianDate, StripeMaterialProperty, StripeOrientation};

#[test]
fn constructor_provides_expected_defaults() {
    let property = StripeMaterialProperty::default();
    assert_eq!(property.get_type(), "Stripe");
    assert!(property.is_constant());
    assert_eq!(property.orientation, None);
    assert_eq!(property.even_color, None);
    assert_eq!(property.odd_color, None);
    assert_eq!(property.offset, None);
    assert_eq!(property.repeat, None);

    let result = property.get_value(None);
    assert!(result.horizontal);
    assert_eq!(result.even_color, Color::WHITE);
    assert_eq!(result.odd_color, Color::BLACK);
    assert_eq!(result.offset, 0.0);
    assert_eq!(result.repeat, 1.0);
}

#[test]
fn works_with_custom_values() {
    let mut property = StripeMaterialProperty::default();
    property.orientation = Some(ConstantProperty::from_value(StripeOrientation::Vertical));
    property.even_color = Some(ConstantProperty::from_value(Color::RED));
    property.odd_color = Some(ConstantProperty::from_value(Color::BLUE));
    property.offset = Some(ConstantProperty::from_value(10.0));
    property.repeat = Some(ConstantProperty::from_value(20.0));

    let result = property.get_value(Some(&JulianDate::now()));
    assert!(!result.horizontal);
    assert_eq!(result.even_color, Color::RED);
    assert_eq!(result.odd_color, Color::BLUE);
    assert_eq!(result.offset, 10.0);
    assert_eq!(result.repeat, 20.0);
}

#[test]
fn equals_works() {
    let left = StripeMaterialProperty::new(
        Some(ConstantProperty::from_value(StripeOrientation::Vertical)),
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(Color::BLUE)),
        Some(ConstantProperty::from_value(1.0)),
        Some(ConstantProperty::from_value(2.0)),
    );
    let right_equal = StripeMaterialProperty::new(
        Some(ConstantProperty::from_value(StripeOrientation::Vertical)),
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(Color::BLUE)),
        Some(ConstantProperty::from_value(1.0)),
        Some(ConstantProperty::from_value(2.0)),
    );
    let right_diff = StripeMaterialProperty::new(
        Some(ConstantProperty::from_value(StripeOrientation::Horizontal)),
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(Color::BLUE)),
        Some(ConstantProperty::from_value(1.0)),
        Some(ConstantProperty::from_value(2.0)),
    );

    assert!(left.equals(Some(&right_equal)));
    assert!(!left.equals(Some(&right_diff)));
    assert!(!left.equals(None));
}
