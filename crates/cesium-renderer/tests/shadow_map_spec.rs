use cesium_math::{Camera, Matrix4};
use glam::{DMat3, DVec3};
use cesium_renderer::shadow_map::{ShadowMap, ShadowMapOptions};

fn create_camera() -> Camera {
    Camera {
        eye: DVec3::new(0.0, 0.0, 10.0),
        camera_to_ecef: DMat3::IDENTITY,
        fovy: 1.0,
        near: 0.1,
        width: 800,
        height: 600,
    }
}

#[test]
fn test_default_shadow_map_properties() {
    let camera = create_camera();
    let shadow_map = ShadowMap::new(ShadowMapOptions {
        light_camera: Some(camera),
        ..Default::default()
    })
    .expect("should construct default shadow map");

    assert!(shadow_map.enabled());
    assert!(!shadow_map.soft_shadows());
    assert!(!shadow_map.is_point_light());
    assert!(!shadow_map.is_spot_light());
    assert!(shadow_map.cascades_enabled());
    assert_eq!(shadow_map.number_of_cascades(), 4);
    assert!(shadow_map.normal_offset());
    assert!((shadow_map.darkness() - 0.3).abs() < 1e-6);
    assert_eq!(shadow_map.size(), 2048);
    assert_eq!(shadow_map.passes().len(), 4);
    assert_eq!(shadow_map.texture_size().x, 4096.0);
    assert_eq!(shadow_map.texture_size().y, 4096.0);
}

#[test]
fn test_throws_without_light_camera() {
    let result = ShadowMap::new(ShadowMapOptions {
        light_camera: None,
        ..Default::default()
    });
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.message.contains("lightCamera is required."));
}

#[test]
fn test_throws_when_number_of_cascades_invalid() {
    let camera = create_camera();
    let result = ShadowMap::new(ShadowMapOptions {
        light_camera: Some(camera),
        number_of_cascades: Some(3),
        ..Default::default()
    });
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.message.contains("Only one or four cascades are supported."));
}

#[test]
fn test_single_cascade_shadow_map() {
    let camera = create_camera();
    let shadow_map = ShadowMap::new(ShadowMapOptions {
        light_camera: Some(camera),
        number_of_cascades: Some(1),
        ..Default::default()
    })
    .expect("should create single cascade shadow map");

    assert_eq!(shadow_map.number_of_cascades(), 1);
    assert_eq!(shadow_map.passes().len(), 1);
    assert_eq!(shadow_map.texture_size().x, 2048.0);
    assert_eq!(shadow_map.texture_size().y, 2048.0);
    assert_eq!(shadow_map.passes()[0].viewport, (0, 0, 2048, 2048));
}

#[test]
fn test_point_light_shadow_map() {
    let camera = create_camera();
    let shadow_map = ShadowMap::new(ShadowMapOptions {
        light_camera: Some(camera),
        is_point_light: Some(true),
        point_light_radius: Some(150.0),
        ..Default::default()
    })
    .expect("should create point light shadow map");

    assert!(shadow_map.is_point_light());
    assert!(!shadow_map.cascades_enabled());
    assert_eq!(shadow_map.number_of_cascades(), 0);
    assert_eq!(shadow_map.passes().len(), 6);
    assert_eq!(shadow_map.texture_size().x, 2048.0);
    assert_eq!(shadow_map.texture_size().y, 2048.0);
    for pass in shadow_map.passes() {
        assert_eq!(pass.viewport, (0, 0, 2048, 2048));
    }
}

#[test]
fn test_directional_shadow_map_without_cascades() {
    let camera = create_camera();
    let shadow_map = ShadowMap::new(ShadowMapOptions {
        light_camera: Some(camera),
        cascades_enabled: Some(false),
        ..Default::default()
    })
    .expect("should create non-cascaded shadow map");

    assert!(!shadow_map.cascades_enabled());
    assert_eq!(shadow_map.passes().len(), 1);
    assert_eq!(shadow_map.texture_size().x, 2048.0);
}

#[test]
fn test_changes_size_and_texture_offsets() {
    let camera = create_camera();
    let mut shadow_map = ShadowMap::new(ShadowMapOptions {
        light_camera: Some(camera),
        number_of_cascades: Some(4),
        size: Some(2048),
        ..Default::default()
    })
    .expect("should create shadow map");

    shadow_map.set_size(256);
    assert_eq!(shadow_map.size(), 256);
    assert_eq!(shadow_map.texture_size().x, 512.0);
    assert_eq!(shadow_map.texture_size().y, 512.0);

    let passes = shadow_map.passes();
    assert_eq!(passes[0].viewport, (0, 0, 256, 256));
    assert_eq!(passes[1].viewport, (256, 0, 256, 256));
    assert_eq!(passes[2].viewport, (0, 256, 256, 256));
    assert_eq!(passes[3].viewport, (256, 256, 256, 256));

    // Verify texture offset matrix for pass 1 (biasX = 256/512 = 0.5, biasY = 0.0, scale = 0.5)
    let p1_matrix = passes[1].texture_offsets;
    let expected = Matrix4::new(
        0.5, 0.0, 0.0, 0.5,
        0.0, 0.5, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );
    assert_eq!(p1_matrix, expected);
}

#[test]
fn test_bias_synchronization() {
    let camera = create_camera();
    let mut shadow_map = ShadowMap::new(ShadowMapOptions {
        light_camera: Some(camera),
        ..Default::default()
    })
    .expect("should create shadow map");

    assert!(shadow_map.terrain_bias().normal_offset);
    assert!(shadow_map.primitive_bias().normal_offset);
    assert!(shadow_map.point_bias().normal_offset);

    shadow_map.set_normal_offset(false);
    assert!(!shadow_map.normal_offset());
    assert!(!shadow_map.terrain_bias().normal_offset);
    assert!(!shadow_map.primitive_bias().normal_offset);
    assert!(!shadow_map.point_bias().normal_offset);
}

#[test]
fn test_compute_cascade_splits() {
    let splits = ShadowMap::compute_cascade_splits(1.0, 1000.0, 0.5);
    assert!(splits[0] > 1.0);
    assert!(splits[1] > splits[0]);
    assert!(splits[2] > splits[1]);
    assert!((splits[3] - 1000.0).abs() < 1e-4);
}
