use std::collections::HashMap;

use cesium_math::{BoundingRectangle, Color};
use cesium_renderer::framebuffer::{PixelDatatype, PixelFormat};
use cesium_renderer::post_process_stage::{
    PostProcessStage, PostProcessStageOptions, PostProcessStageSampleMode, PostProcessUniformValue,
    SelectedFeature,
};

#[test]
fn test_post_process_stage_constructs() {
    let fragment_shader = "uniform vec4 color; void main() { out_FragColor = color; }";
    let mut uniforms = HashMap::new();
    uniforms.insert("color".to_string(), PostProcessUniformValue::Color(Color::RED));
    let texture_scale = 0.5;
    let force_power_of_two = true;
    let sample_mode = PostProcessStageSampleMode::Linear;
    let pixel_format = PixelFormat::Rgb;
    let pixel_datatype = PixelDatatype::UnsignedInt;
    let clear_color = Color::BLUE;
    let scissor_rectangle = BoundingRectangle::new(0.0, 0.0, 5.0, 5.0);
    let name = "wonka vision".to_string();

    let mut options = PostProcessStageOptions::new(fragment_shader);
    options.uniforms = Some(uniforms.clone());
    options.texture_scale = Some(texture_scale);
    options.force_power_of_two = Some(force_power_of_two);
    options.sample_mode = Some(sample_mode);
    options.pixel_format = Some(pixel_format);
    options.pixel_datatype = Some(pixel_datatype);
    options.clear_color = Some(clear_color);
    options.scissor_rectangle = Some(scissor_rectangle);
    options.name = Some(name.clone());

    let stage = PostProcessStage::new(options).unwrap();
    assert_eq!(stage.fragment_shader, fragment_shader);
    assert_eq!(stage.uniforms, Some(uniforms));
    assert_eq!(stage.texture_scale, texture_scale);
    assert_eq!(stage.force_power_of_two, force_power_of_two);
    assert_eq!(stage.sample_mode, sample_mode);
    assert_eq!(stage.pixel_format, pixel_format);
    assert_eq!(stage.pixel_datatype, pixel_datatype);
    assert_eq!(stage.clear_color, clear_color);
    assert_eq!(stage.scissor_rectangle, scissor_rectangle);
    assert_eq!(stage.name, name);
    assert!(stage.output_texture.is_none());
}

#[test]
fn test_post_process_stage_default_constructs() {
    let fragment_shader = "void main() { out_FragColor = vec4(1.0); }";
    let options = PostProcessStageOptions::new(fragment_shader);
    let stage = PostProcessStage::new(options).unwrap();

    assert_eq!(stage.fragment_shader, fragment_shader);
    assert!(stage.uniforms.is_none());
    assert_eq!(stage.texture_scale, 1.0);
    assert!(!stage.force_power_of_two);
    assert_eq!(stage.sample_mode, PostProcessStageSampleMode::Nearest);
    assert_eq!(stage.pixel_format, PixelFormat::Rgba);
    assert_eq!(stage.pixel_datatype, PixelDatatype::UnsignedByte);
    assert_eq!(stage.clear_color, Color::BLACK);
    assert_eq!(
        stage.scissor_rectangle,
        BoundingRectangle::new(0.0, 0.0, 0.0, 0.0)
    );
    assert!(!stage.name.is_empty());
    assert!(stage.output_texture.is_none());
}

#[test]
fn test_post_process_stage_throws_without_fragment_shader() {
    let options = PostProcessStageOptions::new("");
    assert!(PostProcessStage::new(options).is_err());
}

#[test]
fn test_post_process_stage_throws_with_invalid_texture_scale() {
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let mut opt1 = PostProcessStageOptions::new(fs);
    opt1.texture_scale = Some(-1.0);
    assert!(PostProcessStage::new(opt1).is_err());

    let mut opt2 = PostProcessStageOptions::new(fs);
    opt2.texture_scale = Some(2.0);
    assert!(PostProcessStage::new(opt2).is_err());

    let mut opt3 = PostProcessStageOptions::new(fs);
    opt3.texture_scale = Some(0.0);
    assert!(PostProcessStage::new(opt3).is_err());
}

#[test]
fn test_post_process_stage_throws_if_pixel_format_is_not_a_color_format() {
    let mut options = PostProcessStageOptions::new("void main() { out_FragColor = vec4(1.0); }");
    options.pixel_format = Some(PixelFormat::DepthStencil);
    assert!(PostProcessStage::new(options).is_err());
}

#[test]
fn test_post_process_stage_clamps_selected_id_texture_to_maximum_texture_size() {
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let mut stage = PostProcessStage::new(PostProcessStageOptions::new(fs)).unwrap();

    PostProcessStage::set_maximum_texture_size_override(Some(2));
    PostProcessStage::clear_last_warning();

    let feature = SelectedFeature::new(Color::from_bytes(1, 2, 3, 255));
    stage.set_selected(Some(vec![
        feature.clone(),
        feature.clone(),
        feature,
    ]));

    assert_eq!(stage.selected_id_texture.as_ref().unwrap().width, 2);
    let warning = PostProcessStage::last_warning().unwrap();
    assert_eq!(warning.0, "postProcessStageSelectedTextureSize");
    assert!(warning.1.contains("exceeds the maximum texture size (2)"));

    PostProcessStage::set_maximum_texture_size_override(None);
}

#[test]
fn test_post_process_stage_destroys() {
    let mut stage = PostProcessStage::new(PostProcessStageOptions::new(
        "void main() { out_FragColor = vec4(1.0); }",
    ))
    .unwrap();
    assert!(!stage.is_destroyed());
    stage.destroy();
    assert!(stage.is_destroyed());
}
