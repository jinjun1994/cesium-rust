use cesium_math::Color;
use cesium_renderer::light::Light;
use cesium_renderer::sun_light::{SunLight, SunLightOptions};

#[test]
fn test_sun_light_constructs_with_default_options() {
    let light = SunLight::new(None);

    assert_eq!(light.color, Color::WHITE);
    assert_eq!(light.intensity, 2.0);
    assert_eq!(light.color(), Color::WHITE);
    assert_eq!(light.intensity(), 2.0);

    let default_light = SunLight::default();
    assert_eq!(light, default_light);
}

#[test]
fn test_sun_light_constructs_with_all_options() {
    let light = SunLight::new(Some(SunLightOptions {
        color: Some(Color::RED),
        intensity: Some(3.5),
    }));

    assert_eq!(light.color, Color::RED);
    assert_eq!(light.intensity, 3.5);
    assert_eq!(light.color(), Color::RED);
    assert_eq!(light.intensity(), 3.5);
}
