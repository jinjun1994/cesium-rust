use std::collections::BTreeMap;
use cesium_renderer::{
    Buffer, BufferOptions, BufferUsage, ComputeCommand, ComputeCommandOptions, PixelFormat,
    ShaderCache, ShaderContextInfo, ShaderProgram, ShaderProgramOptions, Texture, TextureOptions,
    VertexArray, VertexArrayOptions, VertexAttributeOptions,
};

#[test]
fn test_throws_if_no_shader_is_provided() {
    let output_texture = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        pixel_format: Some(PixelFormat::Rgba),
        ..Default::default()
    });

    let mut cmd = ComputeCommand::new(ComputeCommandOptions {
        output_texture: Some(output_texture),
        ..Default::default()
    });

    assert!(cmd.execute().is_err());
}

#[test]
fn test_throws_if_no_output_texture_is_provided() {
    let mut cmd = ComputeCommand::new(ComputeCommandOptions {
        fragment_shader_source: Some("void main() { out_FragColor = vec4(1.0); }".into()),
        ..Default::default()
    });

    assert!(cmd.execute().is_err());
}

#[test]
fn test_renderer_resources_preserved_or_destroyed_based_on_persists_flag() {
    let cache = ShaderCache::new(ShaderContextInfo::default());
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";

    let shader_program = ShaderProgram::from_cache(
        &cache,
        ShaderProgramOptions {
            vertex_shader_source: vs.into(),
            fragment_shader_source: fs.into(),
            attribute_locations: BTreeMap::new(),
        },
    );

    let vertex_buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 16,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: None,
    }).unwrap();

    let vertex_array = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(vertex_buffer),
            components_per_attribute: 4,
            component_datatype: None,
            normalize: None,
            offset_in_bytes: None,
            stride_in_bytes: None,
            instance_divisor: None,
        }],
        index_buffer: None,
    });

    let output_texture = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        pixel_format: Some(PixelFormat::Rgba),
        ..Default::default()
    });

    // 1. Check resources preserved when persists is true
    let mut cmd_persists = ComputeCommand::new(ComputeCommandOptions {
        vertex_array: Some(vertex_array.clone()),
        shader_program: Some(shader_program.clone()),
        output_texture: Some(output_texture.clone()),
        persists: Some(true),
        ..Default::default()
    });

    assert!(cmd_persists.execute().is_ok());
    assert!(!shader_program.borrow().is_destroyed());
    assert!(!cmd_persists.vertex_array.as_ref().unwrap().is_destroyed());
    assert!(!cmd_persists.output_texture.as_ref().unwrap().is_destroyed());

    // 2. Check resources destroyed when persists is false, except output_texture
    let mut cmd_transient = ComputeCommand::new(ComputeCommandOptions {
        vertex_array: Some(vertex_array),
        shader_program: Some(shader_program.clone()),
        output_texture: Some(output_texture),
        persists: Some(false),
        ..Default::default()
    });

    assert!(cmd_transient.execute().is_ok());
    assert!(cmd_transient.vertex_array.as_ref().unwrap().is_destroyed());
    assert!(!cmd_transient.output_texture.as_ref().unwrap().is_destroyed());
}
