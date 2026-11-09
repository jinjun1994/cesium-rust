use cesium_math::Color;
use cesium_renderer::{DynamicEnvironmentMapManager, DynamicEnvironmentMapManagerOptions};

#[test]
fn test_constructs_with_default_values() {
    let manager = DynamicEnvironmentMapManager::new(None);

    assert!(manager.enabled);
    assert!(manager.should_update);
    assert_eq!(manager.maximum_seconds_difference, 3600.0);
    assert_eq!(manager.maximum_position_epsilon, 1000.0);
    assert_eq!(manager.atmosphere_scattering_intensity, 2.0);
    assert_eq!(manager.gamma, 1.0);
    assert_eq!(manager.brightness, 1.0);
    assert_eq!(manager.saturation, 1.0);
    assert_eq!(
        manager.ground_color,
        DynamicEnvironmentMapManager::AVERAGE_EARTH_GROUND_COLOR
    );
    assert_eq!(manager.ground_albedo, 0.31);
}

#[test]
fn test_constructs_with_options() {
    let manager = DynamicEnvironmentMapManager::new(Some(DynamicEnvironmentMapManagerOptions {
        enabled: Some(false),
        maximum_seconds_difference: Some(0.0),
        maximum_position_epsilon: Some(0.0),
        atmosphere_scattering_intensity: Some(0.0),
        gamma: Some(0.0),
        brightness: Some(0.0),
        saturation: Some(0.0),
        ground_color: Some(Color::BLUE),
        ground_albedo: Some(0.0),
        ..Default::default()
    }));

    assert!(!manager.enabled);
    assert!(manager.should_update);
    assert_eq!(manager.maximum_seconds_difference, 0.0);
    assert_eq!(manager.maximum_position_epsilon, 0.0);
    assert_eq!(manager.atmosphere_scattering_intensity, 0.0);
    assert_eq!(manager.gamma, 0.0);
    assert_eq!(manager.brightness, 0.0);
    assert_eq!(manager.saturation, 0.0);
    assert_eq!(manager.ground_color, Color::BLUE);
    assert_eq!(manager.ground_albedo, 0.0);
}

#[test]
fn test_uses_default_spherical_harmonic_coefficients() {
    let manager = DynamicEnvironmentMapManager::new(None);

    assert_eq!(manager.spherical_harmonic_coefficients.len(), 9);
    assert_eq!(
        manager.spherical_harmonic_coefficients,
        DynamicEnvironmentMapManager::DEFAULT_SPHERICAL_HARMONIC_COEFFICIENTS
    );
}

#[test]
fn test_uses_minimum_of_zero_mipmap_levels() {
    let manager = DynamicEnvironmentMapManager::new(Some(DynamicEnvironmentMapManagerOptions {
        mipmap_levels: Some(-9999),
        ..Default::default()
    }));

    assert_eq!(manager.mipmap_levels, 0);
}

#[test]
fn test_destroys() {
    let mut manager = DynamicEnvironmentMapManager::new(None);
    assert!(!manager.is_destroyed());
    manager.destroy();
    assert!(manager.is_destroyed());
}
