//! Direct port and verification against CesiumJS `GridMaterialPropertySpec.js` (Apache-2.0).

use cesium_math::{Cartesian2, Color, ConstantProperty, GridMaterialProperty, JulianDate};

#[test]
fn constructor_provides_expected_defaults() {
    let property = GridMaterialProperty::default();
    assert_eq!(property.get_type(), "Grid");
    assert!(property.is_constant());
    assert_eq!(property.color, None);
    assert_eq!(property.cell_alpha, None);
    assert_eq!(property.line_count, None);
    assert_eq!(property.line_thickness, None);
    assert_eq!(property.line_offset, None);

    let result = property.get_value(None);
    assert_eq!(result.color, Color::WHITE);
    assert_eq!(result.cell_alpha, 0.1);
    assert_eq!(result.line_count, Cartesian2::new(8.0, 8.0));
    assert_eq!(result.line_thickness, Cartesian2::new(1.0, 1.0));
    assert_eq!(result.line_offset, Cartesian2::new(0.0, 0.0));
}

#[test]
fn works_with_custom_values() {
    let mut property = GridMaterialProperty::default();
    property.color = Some(ConstantProperty::from_value(Color::RED));
    property.cell_alpha = Some(ConstantProperty::from_value(1.0));
    property.line_count = Some(ConstantProperty::from_value(Cartesian2::new(3.4, 5.0)));
    property.line_thickness = Some(ConstantProperty::from_value(Cartesian2::new(2.0, 3.0)));
    property.line_offset = Some(ConstantProperty::from_value(Cartesian2::new(0.7, 0.8)));

    let result = property.get_value(Some(&JulianDate::now()));
    assert_eq!(result.color, Color::RED);
    assert_eq!(result.cell_alpha, 1.0);
    assert_eq!(result.line_count, Cartesian2::new(3.4, 5.0));
    assert_eq!(result.line_thickness, Cartesian2::new(2.0, 3.0));
    assert_eq!(result.line_offset, Cartesian2::new(0.7, 0.8));
}

#[test]
fn equals_works() {
    let left = GridMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(1.0)),
        Some(ConstantProperty::from_value(Cartesian2::new(3.4, 5.0))),
        Some(ConstantProperty::from_value(Cartesian2::new(2.0, 3.0))),
        Some(ConstantProperty::from_value(Cartesian2::new(0.7, 0.8))),
    );
    let right_equal = GridMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::RED)),
        Some(ConstantProperty::from_value(1.0)),
        Some(ConstantProperty::from_value(Cartesian2::new(3.4, 5.0))),
        Some(ConstantProperty::from_value(Cartesian2::new(2.0, 3.0))),
        Some(ConstantProperty::from_value(Cartesian2::new(0.7, 0.8))),
    );
    let right_diff = GridMaterialProperty::new(
        Some(ConstantProperty::from_value(Color::BLUE)),
        Some(ConstantProperty::from_value(1.0)),
        Some(ConstantProperty::from_value(Cartesian2::new(3.4, 5.0))),
        Some(ConstantProperty::from_value(Cartesian2::new(2.0, 3.0))),
        Some(ConstantProperty::from_value(Cartesian2::new(0.7, 0.8))),
    );

    assert!(left.equals(Some(&right_equal)));
    assert!(!left.equals(Some(&right_diff)));
    assert!(!left.equals(None));
}
