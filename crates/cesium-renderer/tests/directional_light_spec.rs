use cesium_math::{Cartesian3, Color};
use cesium_renderer::{DirectionalLight, DirectionalLightError, DirectionalLightOptions};

#[test]
fn test_constructs_with_default_options() {
    let light = DirectionalLight::new(DirectionalLightOptions {
        direction: Cartesian3::UNIT_X,
        color: None,
        intensity: None,
    })
    .expect("should construct DirectionalLight");

    assert_eq!(light.direction, Cartesian3::UNIT_X);
    assert_eq!(light.color, Color::WHITE);
    assert_eq!(light.intensity, 1.0);
}

#[test]
fn test_constructs_with_all_options() {
    let light = DirectionalLight::new(DirectionalLightOptions {
        direction: Cartesian3::UNIT_X,
        color: Some(Color::RED),
        intensity: Some(2.0),
    })
    .expect("should construct DirectionalLight");

    assert_eq!(light.direction, Cartesian3::UNIT_X);
    assert_eq!(light.color, Color::RED);
    assert_eq!(light.intensity, 2.0);
}

#[test]
fn test_throws_if_direction_is_zero_length() {
    let err = DirectionalLight::new(DirectionalLightOptions {
        direction: Cartesian3::ZERO,
        color: None,
        intensity: None,
    })
    .expect_err("should reject zero-length direction");

    assert_eq!(err, DirectionalLightError::ZeroLengthDirection);
}
