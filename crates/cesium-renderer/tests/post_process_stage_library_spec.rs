use cesium_math::Color;
use cesium_renderer::post_process_stage::PostProcessUniformValue;
use cesium_renderer::post_process_stage_library::PostProcessStageLibrary;

#[test]
fn test_post_process_stage_library_black_and_white() {
    let stage = PostProcessStageLibrary::create_black_and_white_stage();
    assert_eq!(stage.name, "czm_black_and_white");
    assert!(stage.fragment_shader.contains("gradations"));
    let uniforms = stage.uniforms.unwrap();
    assert_eq!(
        uniforms.get("gradations"),
        Some(&PostProcessUniformValue::Float(4.0))
    );
}

#[test]
fn test_post_process_stage_library_brightness() {
    let stage = PostProcessStageLibrary::create_brightness_stage();
    assert_eq!(stage.name, "czm_brightness");
    assert!(stage.fragment_shader.contains("brightness"));
    let uniforms = stage.uniforms.unwrap();
    assert_eq!(
        uniforms.get("brightness"),
        Some(&PostProcessUniformValue::Float(0.5))
    );
}

#[test]
fn test_post_process_stage_library_night_vision() {
    let stage = PostProcessStageLibrary::create_night_vision_stage();
    assert_eq!(stage.name, "czm_night_vision");
    assert!(stage.fragment_shader.contains("noiseValue"));
}

#[test]
fn test_post_process_stage_library_depth_view() {
    let stage = PostProcessStageLibrary::create_depth_view_stage();
    assert_eq!(stage.name, "czm_depth_view");
    assert!(stage.fragment_shader.contains("depthTexture"));
}

#[test]
fn test_post_process_stage_library_blur() {
    let composite = PostProcessStageLibrary::create_blur_stage(None);
    assert_eq!(composite.name, "czm_blur");
    assert_eq!(composite.len(), 2);
    let uniforms = composite.uniforms.unwrap();
    assert_eq!(
        uniforms.get("delta"),
        Some(&PostProcessUniformValue::Float(1.0))
    );
    assert_eq!(
        uniforms.get("sigma"),
        Some(&PostProcessUniformValue::Float(2.0))
    );
    assert_eq!(
        uniforms.get("stepSize"),
        Some(&PostProcessUniformValue::Float(1.0))
    );
}

#[test]
fn test_post_process_stage_library_depth_of_field() {
    let composite = PostProcessStageLibrary::create_depth_of_field_stage();
    assert_eq!(composite.name, "czm_depth_of_field");
    assert_eq!(composite.len(), 2);
    assert!(PostProcessStageLibrary::is_depth_of_field_supported(true));
    assert!(!PostProcessStageLibrary::is_depth_of_field_supported(false));
}

#[test]
fn test_post_process_stage_library_edge_detection() {
    let stage = PostProcessStageLibrary::create_edge_detection_stage();
    assert_eq!(stage.name, "czm_edge_detection");
    assert!(PostProcessStageLibrary::is_edge_detection_supported(true));
    assert!(!PostProcessStageLibrary::is_edge_detection_supported(false));
}

#[test]
fn test_post_process_stage_library_silhouette() {
    let composite = PostProcessStageLibrary::create_silhouette_stage();
    assert_eq!(composite.name, "czm_silhouette");
    assert_eq!(composite.len(), 2);
    assert!(PostProcessStageLibrary::is_silhouette_supported(true));
    assert!(!PostProcessStageLibrary::is_silhouette_supported(false));

    let uniforms = composite.uniforms.unwrap();
    assert_eq!(
        uniforms.get("color"),
        Some(&PostProcessUniformValue::Color(Color::RED))
    );
}

#[test]
fn test_post_process_stage_library_bloom() {
    let composite = PostProcessStageLibrary::create_bloom_stage();
    assert_eq!(composite.name, "czm_bloom");
    assert_eq!(composite.len(), 3);
    let uniforms = composite.uniforms.unwrap();
    assert_eq!(
        uniforms.get("contrast"),
        Some(&PostProcessUniformValue::Float(128.0))
    );
    assert_eq!(
        uniforms.get("brightness"),
        Some(&PostProcessUniformValue::Float(-0.3))
    );
}

#[test]
fn test_post_process_stage_library_ambient_occlusion() {
    let composite = PostProcessStageLibrary::create_ambient_occlusion_stage();
    assert_eq!(composite.name, "czm_ambient_occlusion");
    assert_eq!(composite.len(), 3);
    assert!(PostProcessStageLibrary::is_ambient_occlusion_supported(true));
    assert!(!PostProcessStageLibrary::is_ambient_occlusion_supported(false));

    let uniforms = composite.uniforms.unwrap();
    assert_eq!(
        uniforms.get("intensity"),
        Some(&PostProcessUniformValue::Float(3.0))
    );
}

#[test]
fn test_post_process_stage_library_fxaa() {
    let stage = PostProcessStageLibrary::create_fxaa_stage();
    assert_eq!(stage.name, "czm_fxaa");
    assert!(stage.fragment_shader.contains("colorTexture"));
}

#[test]
fn test_post_process_stage_library_tonemapping_stages() {
    let aces = PostProcessStageLibrary::create_aces_tonemapping_stage(None);
    assert_eq!(aces.name, "czm_aces_tonemapping");

    let filmic = PostProcessStageLibrary::create_filmic_tonemapping_stage(None);
    assert_eq!(filmic.name, "czm_filmic_tonemapping");

    let pbr_neutral = PostProcessStageLibrary::create_pbr_neutral_tonemapping_stage(None);
    assert_eq!(pbr_neutral.name, "czm_pbr_neutral_tonemapping");

    let reinhard = PostProcessStageLibrary::create_reinhard_tonemapping_stage(None);
    assert_eq!(reinhard.name, "czm_reinhard_tonemapping");

    let mod_reinhard = PostProcessStageLibrary::create_modified_reinhard_tonemapping_stage(None);
    assert_eq!(mod_reinhard.name, "czm_modified_reinhard_tonemapping");
}

#[test]
fn test_post_process_stage_library_auto_exposure_and_lens_flare() {
    let auto_exp = PostProcessStageLibrary::create_auto_exposure_stage();
    assert_eq!(auto_exp.name, "czm_auto_exposure");

    let lens_flare = PostProcessStageLibrary::create_lens_flare_stage();
    assert_eq!(lens_flare.name, "czm_lens_flare");
}
