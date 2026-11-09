use cesium_math::{Cartesian2, Cartesian3};
use cesium_renderer::image_based_lighting::{ImageBasedLighting, ImageBasedLightingOptions};

fn test_coefficients() -> Vec<Cartesian3> {
    vec![
        Cartesian3::new(1.0, 1.0, 1.0),
        Cartesian3::new(2.0, 2.0, 2.0),
        Cartesian3::new(3.0, 3.0, 3.0),
        Cartesian3::new(4.0, 4.0, 4.0),
        Cartesian3::new(5.0, 5.0, 5.0),
        Cartesian3::new(6.0, 6.0, 6.0),
        Cartesian3::new(7.0, 7.0, 7.0),
        Cartesian3::new(8.0, 8.0, 8.0),
        Cartesian3::new(9.0, 9.0, 9.0),
    ]
}

#[test]
fn constructs_with_default_values() {
    let ibl = ImageBasedLighting::new(None).expect("valid creation");
    assert_eq!(ibl.image_based_lighting_factor(), Cartesian2::new(1.0, 1.0));
    assert!(ibl.spherical_harmonic_coefficients().is_none());
    assert!(ibl.specular_environment_maps().is_none());
}

#[test]
fn throws_when_constructing_with_an_invalid_factor() {
    let res1 = ImageBasedLighting::new(Some(ImageBasedLightingOptions {
        image_based_lighting_factor: Some(Cartesian2::new(-1.0, 0.0)),
        ..Default::default()
    }));
    assert!(res1.is_err());

    let res2 = ImageBasedLighting::new(Some(ImageBasedLightingOptions {
        image_based_lighting_factor: Some(Cartesian2::new(0.5, 1.5)),
        ..Default::default()
    }));
    assert!(res2.is_err());
}

#[test]
fn throws_when_constructing_with_invalid_spherical_harmonic_coefficients() {
    let res = ImageBasedLighting::new(Some(ImageBasedLightingOptions {
        spherical_harmonic_coefficients: Some(vec![]),
        ..Default::default()
    }));
    assert!(res.is_err());
    assert!(res.unwrap_err().message.contains("must be an array of 9 Cartesian3 values"));

    let res_eight = ImageBasedLighting::new(Some(ImageBasedLightingOptions {
        spherical_harmonic_coefficients: Some(vec![Cartesian3::ZERO; 8]),
        ..Default::default()
    }));
    assert!(res_eight.is_err());
}

#[test]
fn throws_when_setting_invalid_values() {
    let mut ibl = ImageBasedLighting::default();

    let res_factor = ibl.set_image_based_lighting_factor(Cartesian2::new(-1.0, 0.0));
    assert!(res_factor.is_err());

    let res_sh = ibl.set_spherical_harmonic_coefficients(Some(vec![]));
    assert!(res_sh.is_err());
}

#[test]
fn image_based_lighting_factor_saves_previous_value() {
    let mut ibl = ImageBasedLighting::new(Some(ImageBasedLightingOptions {
        image_based_lighting_factor: Some(Cartesian2::ZERO),
        ..Default::default()
    }))
    .unwrap();

    ibl.set_image_based_lighting_factor(Cartesian2::new(1.0, 1.0)).unwrap();
    assert_eq!(ibl.image_based_lighting_factor(), Cartesian2::new(1.0, 1.0));
    assert_eq!(ibl.previous_image_based_lighting_factor(), Cartesian2::ZERO);
}

#[test]
fn spherical_harmonic_coefficients_saves_previous_value() {
    let coeffs = test_coefficients();
    let mut ibl = ImageBasedLighting::new(Some(ImageBasedLightingOptions {
        spherical_harmonic_coefficients: Some(coeffs.clone()),
        ..Default::default()
    }))
    .unwrap();

    ibl.set_spherical_harmonic_coefficients(None).unwrap();
    assert!(ibl.spherical_harmonic_coefficients().is_none());
    assert_eq!(
        ibl.previous_spherical_harmonic_coefficients(),
        Some(coeffs.as_slice())
    );
}

#[test]
fn enabled_returns_correct_values() {
    let mut ibl = ImageBasedLighting::default();
    assert!(ibl.enabled());

    ibl.set_image_based_lighting_factor(Cartesian2::ZERO).unwrap();
    assert!(!ibl.enabled());

    ibl.set_image_based_lighting_factor(Cartesian2::new(0.5, 0.0)).unwrap();
    assert!(ibl.enabled());

    ibl.set_image_based_lighting_factor(Cartesian2::new(0.0, 0.2)).unwrap();
    assert!(ibl.enabled());
}

#[test]
fn update_logic_and_shader_regeneration() {
    let mut ibl = ImageBasedLighting::new(Some(ImageBasedLightingOptions {
        image_based_lighting_factor: Some(Cartesian2::ZERO),
        ..Default::default()
    }))
    .unwrap();

    // First frame: factor toggled from 0 to 1
    ibl.set_image_based_lighting_factor(Cartesian2::new(1.0, 1.0)).unwrap();
    ibl.update(1, None, None, false).unwrap();
    assert!(ibl.should_regenerate_shaders());

    // Same frame: redundant call is no-op
    ibl.update(1, None, None, false).unwrap();
    assert!(ibl.should_regenerate_shaders());

    // Next frame: no changes
    ibl.update(2, None, None, false).unwrap();
    assert!(!ibl.should_regenerate_shaders());

    // Enabling default SH coefficients via frame state
    let dummy_sh = test_coefficients();
    ibl.update(3, None, Some(&dummy_sh), false).unwrap();
    assert!(ibl.should_regenerate_shaders());
    assert!(ibl.use_default_spherical_harmonics());
}

#[test]
fn destroy_state_lifecycle() {
    let mut ibl = ImageBasedLighting::default();
    assert!(!ibl.is_destroyed());

    ibl.destroy().unwrap();
    assert!(ibl.is_destroyed());

    assert!(ibl.destroy().is_err());
    assert!(ibl.set_image_based_lighting_factor(Cartesian2::new(1.0, 1.0)).is_err());
}
