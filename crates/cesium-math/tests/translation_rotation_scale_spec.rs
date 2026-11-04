use cesium_math::{Cartesian3, Quaternion, TranslationRotationScale};

#[test]
fn test_sets_correct_values_when_constructed_default() {
    let transformation = TranslationRotationScale::default();

    assert_eq!(transformation.translation, Cartesian3::ZERO);
    assert_eq!(transformation.rotation, Quaternion::IDENTITY);
    assert_eq!(transformation.scale, Cartesian3::new(1.0, 1.0, 1.0));
}

#[test]
fn test_sets_correct_values_when_constructed_with_arguments() {
    let translation = Cartesian3::UNIT_Y;
    let rotation = Quaternion::new(0.5, 0.5, 0.5, 0.5);
    let scale = Cartesian3::UNIT_X;

    let transformation = TranslationRotationScale::new(translation, rotation, scale);

    assert_eq!(transformation.translation, translation);
    assert_eq!(transformation.rotation, rotation);
    assert_eq!(transformation.scale, scale);
}

#[test]
#[allow(clippy::field_reassign_with_default)]
fn test_has_a_working_equals_function() {
    let mut left = TranslationRotationScale::default();
    left.translation = Cartesian3::UNIT_Y;
    left.rotation = Quaternion::new(0.5, 0.5, 0.5, 0.5);
    left.scale = Cartesian3::UNIT_X;

    let mut right = TranslationRotationScale::default();
    right.translation = Cartesian3::UNIT_Y;
    right.rotation = Quaternion::new(0.5, 0.5, 0.5, 0.5);
    right.scale = Cartesian3::UNIT_X;
    assert!(left.equals(&right));

    right.scale = Cartesian3::ZERO;
    assert!(!left.equals(&right));

    right.scale = Cartesian3::UNIT_X;
    right.translation = Cartesian3::ZERO;
    assert!(!left.equals(&right));

    right.translation = Cartesian3::UNIT_Y;
    right.rotation = Quaternion::ZERO;
    assert!(!left.equals(&right));
}
