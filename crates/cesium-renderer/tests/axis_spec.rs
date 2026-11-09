use cesium_math::{Cartesian4, Matrix4, EPSILON1};
use cesium_renderer::Axis;

fn convert_up_axis(up_axis: Cartesian4, transformation: Matrix4, expected: Cartesian4) {
    let transformed = transformation.multiply_by_vector(up_axis);
    let normalized = transformed.normalize();
    assert!(
        normalized.equals_epsilon(&expected, Some(EPSILON1), None),
        "Expected {:?}, got {:?}",
        expected,
        normalized
    );
}

#[test]
fn test_convert_y_up_to_z_up() {
    convert_up_axis(Cartesian4::UNIT_Y, Axis::Y_UP_TO_Z_UP, Cartesian4::UNIT_Z);
}

#[test]
fn test_convert_y_up_to_x_up() {
    convert_up_axis(Cartesian4::UNIT_Y, Axis::Y_UP_TO_X_UP, Cartesian4::UNIT_X);
}

#[test]
fn test_convert_z_up_to_x_up() {
    convert_up_axis(Cartesian4::UNIT_Z, Axis::Z_UP_TO_X_UP, Cartesian4::UNIT_X);
}

#[test]
fn test_convert_z_up_to_y_up() {
    convert_up_axis(Cartesian4::UNIT_Z, Axis::Z_UP_TO_Y_UP, Cartesian4::UNIT_Y);
}

#[test]
fn test_convert_x_up_to_y_up() {
    convert_up_axis(Cartesian4::UNIT_X, Axis::X_UP_TO_Y_UP, Cartesian4::UNIT_Y);
}

#[test]
fn test_convert_x_up_to_z_up() {
    convert_up_axis(Cartesian4::UNIT_X, Axis::X_UP_TO_Z_UP, Cartesian4::UNIT_Z);
}

#[test]
fn test_axis_enum_values_and_names() {
    assert_eq!(Axis::X as i32, 0);
    assert_eq!(Axis::Y as i32, 1);
    assert_eq!(Axis::Z as i32, 2);

    assert_eq!(i32::from(Axis::X), 0);
    assert_eq!(i32::from(Axis::Y), 1);
    assert_eq!(i32::from(Axis::Z), 2);

    assert_eq!(Axis::try_from(0).unwrap(), Axis::X);
    assert_eq!(Axis::try_from(1).unwrap(), Axis::Y);
    assert_eq!(Axis::try_from(2).unwrap(), Axis::Z);
    assert!(Axis::try_from(3).is_err());

    assert_eq!(Axis::from_name("X"), Some(Axis::X));
    assert_eq!(Axis::from_name("Y"), Some(Axis::Y));
    assert_eq!(Axis::from_name("Z"), Some(Axis::Z));
    assert_eq!(Axis::from_name("W"), None);
    assert_eq!(Axis::from_name(""), None);

    assert_eq!(Axis::X.name(), "X");
    assert_eq!(Axis::Y.name(), "Y");
    assert_eq!(Axis::Z.name(), "Z");
}
