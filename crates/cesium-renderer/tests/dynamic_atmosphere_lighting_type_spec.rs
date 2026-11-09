use cesium_renderer::DynamicAtmosphereLightingType;

#[derive(Default)]
struct MockGlobe {
    enable_lighting: bool,
    dynamic_atmosphere_lighting: bool,
    dynamic_atmosphere_lighting_from_sun: bool,
}

#[test]
fn test_returns_none_when_lighting_is_disabled() {
    let mut globe = MockGlobe::default();

    assert_eq!(
        DynamicAtmosphereLightingType::from_globe_flags(
            globe.enable_lighting,
            globe.dynamic_atmosphere_lighting,
            globe.dynamic_atmosphere_lighting_from_sun,
        ),
        DynamicAtmosphereLightingType::None
    );

    globe.enable_lighting = true;
    assert_eq!(
        DynamicAtmosphereLightingType::from_globe_flags(
            globe.enable_lighting,
            globe.dynamic_atmosphere_lighting,
            globe.dynamic_atmosphere_lighting_from_sun,
        ),
        DynamicAtmosphereLightingType::None
    );

    globe.enable_lighting = false;
    globe.dynamic_atmosphere_lighting = true;
    assert_eq!(
        DynamicAtmosphereLightingType::from_globe_flags(
            globe.enable_lighting,
            globe.dynamic_atmosphere_lighting,
            globe.dynamic_atmosphere_lighting_from_sun,
        ),
        DynamicAtmosphereLightingType::None
    );
}

#[test]
fn test_selects_light_type_depending_on_globe_dynamic_atmosphere_lighting_from_sun() {
    let mut globe = MockGlobe {
        enable_lighting: true,
        dynamic_atmosphere_lighting: true,
        dynamic_atmosphere_lighting_from_sun: true,
    };

    assert_eq!(
        DynamicAtmosphereLightingType::from_globe_flags(
            globe.enable_lighting,
            globe.dynamic_atmosphere_lighting,
            globe.dynamic_atmosphere_lighting_from_sun,
        ),
        DynamicAtmosphereLightingType::Sunlight
    );

    globe.dynamic_atmosphere_lighting_from_sun = false;
    assert_eq!(
        DynamicAtmosphereLightingType::from_globe_flags(
            globe.enable_lighting,
            globe.dynamic_atmosphere_lighting,
            globe.dynamic_atmosphere_lighting_from_sun,
        ),
        DynamicAtmosphereLightingType::SceneLight
    );
}
