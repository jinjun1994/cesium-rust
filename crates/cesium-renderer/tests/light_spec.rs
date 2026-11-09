use cesium_math::{Cartesian3, Color};
use cesium_renderer::{DirectionalLight, DirectionalLightOptions, Light, LightBase};

#[test]
fn base_light_throws_instantiation_error_on_property_access() {
    let light = LightBase::new();

    let color_res = light.color();
    assert!(color_res.is_err());
    assert!(color_res
        .unwrap_err()
        .message
        .contains("cannot be instantiated directly"));

    let intensity_res = light.intensity();
    assert!(intensity_res.is_err());
    assert!(intensity_res
        .unwrap_err()
        .message
        .contains("cannot be instantiated directly"));
}

#[test]
fn directional_light_implements_light_interface() {
    let dir_light = DirectionalLight::new(DirectionalLightOptions {
        direction: Cartesian3::new(1.0, 0.0, 0.0),
        color: Some(Color::YELLOW),
        intensity: Some(2.5),
    })
    .unwrap();

    let light_ref: &dyn Light = &dir_light;
    assert_eq!(light_ref.color(), Color::YELLOW);
    assert_eq!(light_ref.intensity(), 2.5);
}
