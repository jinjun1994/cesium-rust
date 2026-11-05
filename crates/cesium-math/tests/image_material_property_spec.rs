//! Direct port and verification against CesiumJS `ImageMaterialPropertySpec.js` (Apache-2.0).

use cesium_math::{Cartesian2, Color, ConstantProperty, ImageMaterialProperty, JulianDate};

#[test]
fn constructor_provides_expected_defaults() {
    let property = ImageMaterialProperty::default();
    assert_eq!(property.get_type(), "Image");
    assert!(property.is_constant());
    assert_eq!(property.image, None);
    assert_eq!(property.repeat, None);
    assert_eq!(property.color, None);
    assert_eq!(property.transparent, None);

    let result = property.get_value(None);
    assert_eq!(result.image, None);
    assert_eq!(result.repeat, Cartesian2::new(1.0, 1.0));
    assert_eq!(result.color, Color::WHITE);
    assert!(!result.transparent);
}

#[test]
fn works_with_custom_values() {
    let mut property = ImageMaterialProperty::default();
    property.image = Some(ConstantProperty::from_value("http://test.invalid/image.png".to_string()));
    property.repeat = Some(ConstantProperty::from_value(Cartesian2::new(2.0, 3.0)));
    property.color = Some(ConstantProperty::from_value(Color::RED));
    property.transparent = Some(ConstantProperty::from_value(true));

    let result = property.get_value(Some(&JulianDate::now()));
    assert_eq!(result.image, Some("http://test.invalid/image.png".to_string()));
    assert_eq!(result.repeat, Cartesian2::new(2.0, 3.0));
    assert_eq!(result.color, Color::RED);
    assert!(result.transparent);
}

#[test]
fn equals_works() {
    let left = ImageMaterialProperty::new(
        Some(ConstantProperty::from_value("test.png".to_string())),
        Some(ConstantProperty::from_value(Cartesian2::new(2.0, 3.0))),
        None,
        None,
    );
    let right_equal = ImageMaterialProperty::new(
        Some(ConstantProperty::from_value("test.png".to_string())),
        Some(ConstantProperty::from_value(Cartesian2::new(2.0, 3.0))),
        None,
        None,
    );
    let right_diff = ImageMaterialProperty::new(
        Some(ConstantProperty::from_value("diff.png".to_string())),
        Some(ConstantProperty::from_value(Cartesian2::new(2.0, 3.0))),
        None,
        None,
    );

    assert!(left.equals(Some(&right_equal)));
    assert!(!left.equals(Some(&right_diff)));
    assert!(!left.equals(None));
}
