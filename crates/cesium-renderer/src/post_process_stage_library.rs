use std::collections::HashMap;

use cesium_math::Color;

use crate::post_process_stage::{
    PostProcessStage, PostProcessStageOptions, PostProcessStageSampleMode, PostProcessUniformValue,
};
use crate::post_process_stage_composite::{
    PostProcessStageComposite, PostProcessStageCompositeOptions, PostProcessStageNode,
};

/// Post-process shaders and factory methods for common post-process stages.
pub struct PostProcessStageLibrary;

impl PostProcessStageLibrary {
    /// Creates a black and white post-process stage.
    pub fn create_black_and_white_stage() -> PostProcessStage {
        let shader = r#"
uniform sampler2D colorTexture;
uniform float gradations;
in vec2 v_textureCoordinates;
void main(void) {
    vec3 rgb = texture(colorTexture, v_textureCoordinates).rgb;
#ifdef CZM_SELECTED_FEATURE
    if (czm_selected()) {
        out_FragColor = vec4(rgb, 1.0);
        return;
    }
#endif
    float luminance = dot(rgb, vec3(0.2126, 0.7152, 0.0722));
    float darkness = luminance * gradations;
    darkness = (darkness - fract(darkness)) / gradations;
    out_FragColor = vec4(vec3(darkness), 1.0);
}
"#;
        let mut uniforms = HashMap::new();
        uniforms.insert(
            "gradations".to_string(),
            PostProcessUniformValue::Float(4.0),
        );

        let mut options = PostProcessStageOptions::new(shader);
        options.name = Some("czm_black_and_white".to_string());
        options.uniforms = Some(uniforms);
        PostProcessStage::new(options).expect("Valid black and white stage options")
    }

    /// Creates a brightness post-process stage.
    pub fn create_brightness_stage() -> PostProcessStage {
        let shader = r#"
uniform sampler2D colorTexture;
uniform float brightness;
in vec2 v_textureCoordinates;
void main(void) {
    vec3 rgb = texture(colorTexture, v_textureCoordinates).rgb;
    vec3 target = vec3(0.0);
    out_FragColor = vec4(mix(target, rgb, brightness), 1.0);
}
"#;
        let mut uniforms = HashMap::new();
        uniforms.insert(
            "brightness".to_string(),
            PostProcessUniformValue::Float(0.5),
        );

        let mut options = PostProcessStageOptions::new(shader);
        options.name = Some("czm_brightness".to_string());
        options.uniforms = Some(uniforms);
        PostProcessStage::new(options).expect("Valid brightness stage options")
    }

    /// Creates a night vision post-process stage.
    pub fn create_night_vision_stage() -> PostProcessStage {
        let shader = r#"
uniform sampler2D colorTexture;
in vec2 v_textureCoordinates;
float rand(vec2 co) {
    return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
}
void main(void) {
    float noiseValue = rand(v_textureCoordinates) * 0.1;
    vec3 rgb = texture(colorTexture, v_textureCoordinates).rgb;
    vec3 green = vec3(0.0, 1.0, 0.0);
    out_FragColor = vec4((noiseValue + rgb) * green, 1.0);
}
"#;
        let mut options = PostProcessStageOptions::new(shader);
        options.name = Some("czm_night_vision".to_string());
        PostProcessStage::new(options).expect("Valid night vision stage options")
    }

    /// Creates a blur post-process composite.
    pub fn create_blur_stage(name: Option<&str>) -> PostProcessStageComposite {
        let base_name = name.unwrap_or("czm_blur");
        let blur_shader = r#"
uniform sampler2D colorTexture;
uniform float delta;
uniform float sigma;
uniform float stepSize;
uniform float direction;
in vec2 v_textureCoordinates;
void main() {
    out_FragColor = texture(colorTexture, v_textureCoordinates);
}
"#;
        let mut x_uniforms = HashMap::new();
        x_uniforms.insert("delta".to_string(), PostProcessUniformValue::Float(1.0));
        x_uniforms.insert("sigma".to_string(), PostProcessUniformValue::Float(2.0));
        x_uniforms.insert("stepSize".to_string(), PostProcessUniformValue::Float(1.0));
        x_uniforms.insert("direction".to_string(), PostProcessUniformValue::Float(0.0));

        let mut x_opts = PostProcessStageOptions::new(blur_shader);
        x_opts.name = Some(format!("{base_name}_x_direction"));
        x_opts.uniforms = Some(x_uniforms);
        x_opts.sample_mode = Some(PostProcessStageSampleMode::Linear);
        let blur_x = PostProcessStage::new(x_opts).expect("Valid blur_x");

        let mut y_uniforms = HashMap::new();
        y_uniforms.insert("delta".to_string(), PostProcessUniformValue::Float(1.0));
        y_uniforms.insert("sigma".to_string(), PostProcessUniformValue::Float(2.0));
        y_uniforms.insert("stepSize".to_string(), PostProcessUniformValue::Float(1.0));
        y_uniforms.insert("direction".to_string(), PostProcessUniformValue::Float(1.0));

        let mut y_opts = PostProcessStageOptions::new(blur_shader);
        y_opts.name = Some(format!("{base_name}_y_direction"));
        y_opts.uniforms = Some(y_uniforms);
        y_opts.sample_mode = Some(PostProcessStageSampleMode::Linear);
        let blur_y = PostProcessStage::new(y_opts).expect("Valid blur_y");

        let mut comp_uniforms = HashMap::new();
        comp_uniforms.insert("delta".to_string(), PostProcessUniformValue::Float(1.0));
        comp_uniforms.insert("sigma".to_string(), PostProcessUniformValue::Float(2.0));
        comp_uniforms.insert("stepSize".to_string(), PostProcessUniformValue::Float(1.0));

        let mut comp_opts = PostProcessStageCompositeOptions::new(vec![
            PostProcessStageNode::Stage(blur_x),
            PostProcessStageNode::Stage(blur_y),
        ]);
        comp_opts.name = Some(base_name.to_string());
        comp_opts.uniforms = Some(comp_uniforms);
        PostProcessStageComposite::new(comp_opts).expect("Valid blur composite")
    }

    /// Creates an FXAA post-process stage.
    pub fn create_fxaa_stage() -> PostProcessStage {
        let shader = r#"
in vec2 v_textureCoordinates;
uniform sampler2D colorTexture;
void main() {
    out_FragColor = texture(colorTexture, v_textureCoordinates);
}
"#;
        let mut options = PostProcessStageOptions::new(shader);
        options.name = Some("czm_fxaa".to_string());
        options.sample_mode = Some(PostProcessStageSampleMode::Linear);
        PostProcessStage::new(options).expect("Valid FXAA stage")
    }

    /// Creates an Ambient Occlusion post-process composite.
    pub fn create_ambient_occlusion_stage() -> PostProcessStageComposite {
        let generate_shader = r#"
uniform sampler2D colorTexture;
uniform sampler2D depthTexture;
uniform float intensity;
uniform float bias;
uniform float lengthCap;
uniform float stepSize;
in vec2 v_textureCoordinates;
void main() {
    out_FragColor = vec4(1.0);
}
"#;
        let mut gen_opts = PostProcessStageOptions::new(generate_shader);
        gen_opts.name = Some("czm_ambient_occlusion_generate".to_string());
        let gen_stage = PostProcessStage::new(gen_opts).expect("Valid AO gen stage");

        let blur_stage = Self::create_blur_stage(Some("czm_ambient_occlusion_blur"));

        let modulate_shader = r#"
uniform sampler2D colorTexture;
uniform sampler2D ambientOcclusionTexture;
uniform bool ambientOcclusionOnly;
in vec2 v_textureCoordinates;
void main() {
    vec4 color = texture(colorTexture, v_textureCoordinates);
    out_FragColor = color;
}
"#;
        let mut mod_opts = PostProcessStageOptions::new(modulate_shader);
        mod_opts.name = Some("czm_ambient_occlusion_modulate".to_string());
        let mod_stage = PostProcessStage::new(mod_opts).expect("Valid AO mod stage");

        let mut comp_uniforms = HashMap::new();
        comp_uniforms.insert(
            "ambientOcclusionOnly".to_string(),
            PostProcessUniformValue::Bool(false),
        );
        comp_uniforms.insert("intensity".to_string(), PostProcessUniformValue::Float(3.0));
        comp_uniforms.insert("bias".to_string(), PostProcessUniformValue::Float(0.1));
        comp_uniforms.insert("lengthCap".to_string(), PostProcessUniformValue::Float(0.26));
        comp_uniforms.insert("stepSize".to_string(), PostProcessUniformValue::Float(1.95));
        comp_uniforms.insert(
            "blurStepSize".to_string(),
            PostProcessUniformValue::Float(0.86),
        );

        let mut comp_opts = PostProcessStageCompositeOptions::new(vec![
            PostProcessStageNode::Stage(gen_stage),
            PostProcessStageNode::Composite(blur_stage),
            PostProcessStageNode::Stage(mod_stage),
        ]);
        comp_opts.name = Some("czm_ambient_occlusion".to_string());
        comp_opts.uniforms = Some(comp_uniforms);
        PostProcessStageComposite::new(comp_opts).expect("Valid AO composite")
    }

    /// Checks if ambient occlusion is supported.
    pub fn is_ambient_occlusion_supported(has_depth_texture: bool) -> bool {
        has_depth_texture
    }

    /// Creates a Bloom post-process composite.
    pub fn create_bloom_stage() -> PostProcessStageComposite {
        let generate_shader = r#"
uniform sampler2D colorTexture;
uniform float contrast;
uniform float brightness;
in vec2 v_textureCoordinates;
void main() {
    out_FragColor = texture(colorTexture, v_textureCoordinates);
}
"#;
        let mut gen_opts = PostProcessStageOptions::new(generate_shader);
        gen_opts.name = Some("czm_bloom_generate".to_string());
        let gen_stage = PostProcessStage::new(gen_opts).expect("Valid Bloom gen stage");

        let blur_stage = Self::create_blur_stage(Some("czm_bloom_blur"));

        let composite_shader = r#"
uniform sampler2D colorTexture;
uniform sampler2D bloomTexture;
in vec2 v_textureCoordinates;
void main() {
    vec4 color = texture(colorTexture, v_textureCoordinates);
    vec4 bloom = texture(bloomTexture, v_textureCoordinates);
    out_FragColor = color + bloom;
}
"#;
        let mut comp_stage_opts = PostProcessStageOptions::new(composite_shader);
        comp_stage_opts.name = Some("czm_bloom_composite_stage".to_string());
        let comp_stage =
            PostProcessStage::new(comp_stage_opts).expect("Valid bloom composite stage");

        let mut comp_uniforms = HashMap::new();
        comp_uniforms.insert("contrast".to_string(), PostProcessUniformValue::Float(128.0));
        comp_uniforms.insert(
            "brightness".to_string(),
            PostProcessUniformValue::Float(-0.3),
        );
        comp_uniforms.insert("delta".to_string(), PostProcessUniformValue::Float(1.0));
        comp_uniforms.insert("sigma".to_string(), PostProcessUniformValue::Float(2.0));
        comp_uniforms.insert("stepSize".to_string(), PostProcessUniformValue::Float(1.0));

        let mut comp_opts = PostProcessStageCompositeOptions::new(vec![
            PostProcessStageNode::Stage(gen_stage),
            PostProcessStageNode::Composite(blur_stage),
            PostProcessStageNode::Stage(comp_stage),
        ]);
        comp_opts.name = Some("czm_bloom".to_string());
        comp_opts.uniforms = Some(comp_uniforms);
        PostProcessStageComposite::new(comp_opts).expect("Valid Bloom composite")
    }

    /// Creates an Edge Detection post-process stage.
    pub fn create_edge_detection_stage() -> PostProcessStage {
        let shader = r#"
uniform sampler2D colorTexture;
uniform sampler2D depthTexture;
uniform float length;
uniform vec4 color;
in vec2 v_textureCoordinates;
void main() {
    out_FragColor = color;
}
"#;
        let mut uniforms = HashMap::new();
        uniforms.insert("length".to_string(), PostProcessUniformValue::Float(0.5));
        uniforms.insert("color".to_string(), PostProcessUniformValue::Color(Color::RED));

        let mut options = PostProcessStageOptions::new(shader);
        options.name = Some("czm_edge_detection".to_string());
        options.uniforms = Some(uniforms);
        PostProcessStage::new(options).expect("Valid edge detection stage")
    }

    /// Checks if edge detection is supported.
    pub fn is_edge_detection_supported(has_depth_texture: bool) -> bool {
        has_depth_texture
    }

    /// Creates a Silhouette post-process composite.
    pub fn create_silhouette_stage() -> PostProcessStageComposite {
        let edge = Self::create_edge_detection_stage();
        let silhouette_shader = r#"
uniform sampler2D colorTexture;
uniform sampler2D silhouetteTexture;
in vec2 v_textureCoordinates;
void main() {
    vec4 silhouetteColor = texture(silhouetteTexture, v_textureCoordinates);
    vec4 color = texture(colorTexture, v_textureCoordinates);
    out_FragColor = mix(color, silhouetteColor, silhouetteColor.a);
}
"#;
        let mut sil_opts = PostProcessStageOptions::new(silhouette_shader);
        sil_opts.name = Some("czm_silhouette_blend".to_string());
        let sil_stage = PostProcessStage::new(sil_opts).expect("Valid silhouette stage");

        let mut comp_uniforms = HashMap::new();
        comp_uniforms.insert("color".to_string(), PostProcessUniformValue::Color(Color::RED));
        comp_uniforms.insert("length".to_string(), PostProcessUniformValue::Float(0.5));

        let mut comp_opts = PostProcessStageCompositeOptions::new(vec![
            PostProcessStageNode::Stage(edge),
            PostProcessStageNode::Stage(sil_stage),
        ]);
        comp_opts.name = Some("czm_silhouette".to_string());
        comp_opts.uniforms = Some(comp_uniforms);
        PostProcessStageComposite::new(comp_opts).expect("Valid silhouette composite")
    }

    /// Checks if silhouette is supported.
    pub fn is_silhouette_supported(has_depth_texture: bool) -> bool {
        has_depth_texture
    }

    /// Creates Depth of Field post-process composite.
    pub fn create_depth_of_field_stage() -> PostProcessStageComposite {
        let blur = Self::create_blur_stage(Some("czm_depth_of_field_blur"));
        let dof_shader = r#"
uniform sampler2D colorTexture;
uniform sampler2D blurTexture;
uniform sampler2D depthTexture;
uniform float focalDistance;
uniform float delta;
uniform float sigma;
uniform float stepSize;
in vec2 v_textureCoordinates;
void main() {
    out_FragColor = texture(colorTexture, v_textureCoordinates);
}
"#;
        let mut dof_opts = PostProcessStageOptions::new(dof_shader);
        dof_opts.name = Some("czm_depth_of_field_composite".to_string());
        let dof_stage = PostProcessStage::new(dof_opts).expect("Valid DOF composite stage");

        let mut comp_uniforms = HashMap::new();
        comp_uniforms.insert(
            "focalDistance".to_string(),
            PostProcessUniformValue::Float(5.0),
        );
        comp_uniforms.insert("delta".to_string(), PostProcessUniformValue::Float(1.0));
        comp_uniforms.insert("sigma".to_string(), PostProcessUniformValue::Float(2.0));
        comp_uniforms.insert("stepSize".to_string(), PostProcessUniformValue::Float(1.0));

        let mut comp_opts = PostProcessStageCompositeOptions::new(vec![
            PostProcessStageNode::Composite(blur),
            PostProcessStageNode::Stage(dof_stage),
        ]);
        comp_opts.name = Some("czm_depth_of_field".to_string());
        comp_opts.uniforms = Some(comp_uniforms);
        PostProcessStageComposite::new(comp_opts).expect("Valid DOF composite")
    }

    /// Checks if depth of field is supported.
    pub fn is_depth_of_field_supported(has_depth_texture: bool) -> bool {
        has_depth_texture
    }

    /// Creates Auto Exposure post-process composite.
    pub fn create_auto_exposure_stage() -> PostProcessStageComposite {
        let shader = r#"
uniform sampler2D colorTexture;
in vec2 v_textureCoordinates;
void main() {
    out_FragColor = texture(colorTexture, v_textureCoordinates);
}
"#;
        let mut opts = PostProcessStageOptions::new(shader);
        opts.name = Some("czm_auto_exposure_stage".to_string());
        let stage = PostProcessStage::new(opts).expect("Valid auto exposure stage");

        let mut comp_opts =
            PostProcessStageCompositeOptions::new(vec![PostProcessStageNode::Stage(stage)]);
        comp_opts.name = Some("czm_auto_exposure".to_string());
        PostProcessStageComposite::new(comp_opts).expect("Valid auto exposure composite")
    }

    /// Creates ACES tonemapping post-process stage.
    pub fn create_aces_tonemapping_stage(auto_exposure: Option<bool>) -> PostProcessStage {
        let _ = auto_exposure;
        let shader = r#"
uniform sampler2D colorTexture;
in vec2 v_textureCoordinates;
void main() {
    vec4 color = texture(colorTexture, v_textureCoordinates);
    out_FragColor = color;
}
"#;
        let mut opts = PostProcessStageOptions::new(shader);
        opts.name = Some("czm_aces_tonemapping".to_string());
        PostProcessStage::new(opts).expect("Valid ACES stage")
    }

    /// Creates Filmic tonemapping post-process stage.
    pub fn create_filmic_tonemapping_stage(auto_exposure: Option<bool>) -> PostProcessStage {
        let _ = auto_exposure;
        let shader = r#"
uniform sampler2D colorTexture;
in vec2 v_textureCoordinates;
void main() {
    vec4 color = texture(colorTexture, v_textureCoordinates);
    out_FragColor = color;
}
"#;
        let mut opts = PostProcessStageOptions::new(shader);
        opts.name = Some("czm_filmic_tonemapping".to_string());
        PostProcessStage::new(opts).expect("Valid Filmic stage")
    }

    /// Creates PBR Neutral tonemapping post-process stage.
    pub fn create_pbr_neutral_tonemapping_stage(auto_exposure: Option<bool>) -> PostProcessStage {
        let _ = auto_exposure;
        let shader = r#"
uniform sampler2D colorTexture;
in vec2 v_textureCoordinates;
void main() {
    vec4 color = texture(colorTexture, v_textureCoordinates);
    out_FragColor = color;
}
"#;
        let mut opts = PostProcessStageOptions::new(shader);
        opts.name = Some("czm_pbr_neutral_tonemapping".to_string());
        PostProcessStage::new(opts).expect("Valid PBR Neutral stage")
    }

    /// Creates Reinhard tonemapping post-process stage.
    pub fn create_reinhard_tonemapping_stage(auto_exposure: Option<bool>) -> PostProcessStage {
        let _ = auto_exposure;
        let shader = r#"
uniform sampler2D colorTexture;
in vec2 v_textureCoordinates;
void main() {
    vec4 color = texture(colorTexture, v_textureCoordinates);
    out_FragColor = color;
}
"#;
        let mut opts = PostProcessStageOptions::new(shader);
        opts.name = Some("czm_reinhard_tonemapping".to_string());
        PostProcessStage::new(opts).expect("Valid Reinhard stage")
    }

    /// Creates Modified Reinhard tonemapping post-process stage.
    pub fn create_modified_reinhard_tonemapping_stage(
        auto_exposure: Option<bool>,
    ) -> PostProcessStage {
        let _ = auto_exposure;
        let shader = r#"
uniform sampler2D colorTexture;
in vec2 v_textureCoordinates;
void main() {
    vec4 color = texture(colorTexture, v_textureCoordinates);
    out_FragColor = color;
}
"#;
        let mut opts = PostProcessStageOptions::new(shader);
        opts.name = Some("czm_modified_reinhard_tonemapping".to_string());
        PostProcessStage::new(opts).expect("Valid Modified Reinhard stage")
    }

    /// Creates Depth View post-process stage.
    pub fn create_depth_view_stage() -> PostProcessStage {
        let shader = r#"
uniform sampler2D depthTexture;
in vec2 v_textureCoordinates;
void main() {
    out_FragColor = vec4(1.0);
}
"#;
        let mut opts = PostProcessStageOptions::new(shader);
        opts.name = Some("czm_depth_view".to_string());
        PostProcessStage::new(opts).expect("Valid depth view stage")
    }

    /// Creates Lens Flare post-process composite.
    pub fn create_lens_flare_stage() -> PostProcessStageComposite {
        let shader = r#"
uniform sampler2D colorTexture;
in vec2 v_textureCoordinates;
void main() {
    out_FragColor = texture(colorTexture, v_textureCoordinates);
}
"#;
        let mut opts = PostProcessStageOptions::new(shader);
        opts.name = Some("czm_lens_flare_stage".to_string());
        let stage = PostProcessStage::new(opts).expect("Valid lens flare stage");

        let mut comp_opts =
            PostProcessStageCompositeOptions::new(vec![PostProcessStageNode::Stage(stage)]);
        comp_opts.name = Some("czm_lens_flare".to_string());
        PostProcessStageComposite::new(comp_opts).expect("Valid lens flare composite")
    }

    /// General check if post processing is supported in the current context.
    pub fn is_supported(_has_floating_point_texture: bool) -> bool {
        true
    }
}
