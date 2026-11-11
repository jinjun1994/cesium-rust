use cesium_renderer::context::{Context, ContextOptions};
use cesium_renderer::point_cloud_shading::PointCloudShading;

#[test]
fn test_creates_expected_instance_from_raw_assignment_and_construction() {
    let point_cloud_shading = PointCloudShading::new();
    assert!(!point_cloud_shading.attenuation);
    assert!((point_cloud_shading.geometric_error_scale - 1.0).abs() < 1e-6);
    assert!(point_cloud_shading.maximum_attenuation.is_none());
    assert!(point_cloud_shading.base_resolution.is_none());
    assert!(point_cloud_shading.eye_dome_lighting);
    assert!((point_cloud_shading.eye_dome_lighting_strength - 1.0).abs() < 1e-6);
    assert!((point_cloud_shading.eye_dome_lighting_radius - 1.0).abs() < 1e-6);
    assert!(!point_cloud_shading.back_face_culling);
    assert!(point_cloud_shading.normal_shading);

    let custom_shading = PointCloudShading {
        attenuation: false,
        geometric_error_scale: 2.0,
        maximum_attenuation: Some(16.0),
        base_resolution: Some(0.1),
        eye_dome_lighting: true,
        eye_dome_lighting_strength: 0.1,
        eye_dome_lighting_radius: 2.0,
        back_face_culling: true,
        normal_shading: false,
    };
    assert!(!custom_shading.attenuation);
    assert!((custom_shading.geometric_error_scale - 2.0).abs() < 1e-6);
    assert_eq!(custom_shading.maximum_attenuation, Some(16.0));
    assert_eq!(custom_shading.base_resolution, Some(0.1));
    assert!(custom_shading.eye_dome_lighting);
    assert!((custom_shading.eye_dome_lighting_strength - 0.1).abs() < 1e-6);
    assert!((custom_shading.eye_dome_lighting_radius - 2.0).abs() < 1e-6);
    assert!(custom_shading.back_face_culling);
    assert!(!custom_shading.normal_shading);
}

#[test]
fn test_provides_a_method_for_checking_if_point_cloud_shading_is_supported() {
    let mut ctx = Context::new(ContextOptions {
        width: Some(256),
        height: Some(256),
        ..Default::default()
    })
    .unwrap();

    // Default context may vary in mock settings
    ctx.floating_point_texture = true;
    ctx.draw_buffers = true;
    ctx.fragment_depth = true;
    let expected = ctx.floating_point_texture && ctx.draw_buffers && ctx.fragment_depth;
    assert_eq!(PointCloudShading::is_supported(&ctx), expected);
    assert!(PointCloudShading::is_supported(&ctx));

    ctx.floating_point_texture = false;
    assert!(!PointCloudShading::is_supported(&ctx));
}
