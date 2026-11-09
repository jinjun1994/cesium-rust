//! Direct port and verification against CesiumJS `Renderer/DrawSpec.js` (Apache-2.0).

use cesium_math::{
    BoundingRectangle, Color, ComponentDatatype, IndexDatatype, PrimitiveType, WindingOrder,
};
use cesium_renderer::shader_cache::ShaderProgram;
use cesium_renderer::{
    BlendEquation, BlendFunction, BlendingState, Buffer, BufferOptions, BufferUsage, ClearCommand,
    ClearCommandOptions, ColorMaskState, Context, ContextLimits, ContextOptions, CullFace,
    CullState, DepthFunction, DepthRangeState, DepthTestState, DrawCommand, DrawCommandOptions,
    IndexBufferOptions, PolygonOffsetState, ReadPixelsOptions, RenderState, SampleCoverageState,
    ScissorTestState, StencilFunction, StencilOperation, StencilOperationState, StencilTestState,
    VertexArray, VertexArrayOptions, VertexAttributeOptions,
};
use std::collections::BTreeMap;
use std::rc::Rc;

fn create_test_context() -> Context {
    create_test_context_with_alpha(false)
}

fn create_test_context_with_alpha(alpha: bool) -> Context {
    Context::new(ContextOptions {
        width: Some(1),
        height: Some(1),
        antialias: Some(true),
        alpha: Some(alpha),
        request_webgl1: Some(false),
    })
    .expect("Valid test context creation")
}

fn create_shader_program(
    _context: &Context,
    vs: &str,
    fs: &str,
    attributes: &[(&str, u32)],
) -> Rc<ShaderProgram> {
    let mut attr_locations = BTreeMap::new();
    for (name, loc) in attributes {
        attr_locations.insert(name.to_string(), *loc);
    }
    Rc::new(ShaderProgram::new(
        cesium_renderer::shader_source::ShaderSource::new(
            cesium_renderer::shader_source::ShaderSourceOptions {
                sources: vec![vs.to_string()],
                ..Default::default()
            },
        )
        .unwrap(),
        vs.to_string(),
        cesium_renderer::shader_source::ShaderSource::new(
            cesium_renderer::shader_source::ShaderSourceOptions {
                sources: vec![fs.to_string()],
                ..Default::default()
            },
        )
        .unwrap(),
        fs.to_string(),
        attr_locations,
    ))
}

fn read_pixel_color(context: &Context) -> [u8; 4] {
    let pixels = context
        .read_pixels(Some(ReadPixelsOptions {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            framebuffer: None,
        }))
        .expect("Read pixels");
    [pixels[0], pixels[1], pixels[2], pixels[3]]
}

fn create_vertex_buffer(bytes: Vec<u8>) -> Buffer {
    Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 0,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: Some(bytes),
    })
    .unwrap()
}

fn create_index_buffer(bytes: Vec<u8>, datatype: IndexDatatype) -> Buffer {
    Buffer::create_index_buffer(IndexBufferOptions {
        size_in_bytes: 0,
        usage: Some(BufferUsage::StaticDraw),
        index_datatype: datatype,
        typed_array: Some(bytes),
    })
    .unwrap()
}

fn bytemuck_cast_bytes(floats: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(floats.len() * 4);
    for f in floats {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

#[test]
fn test_draws_a_white_point() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_PointSize = 1.0; gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let vb = create_vertex_buffer(bytemuck_cast_bytes(&[0.0f32, 0.0, 0.0, 1.0]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(vb),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 255, 255, 255]);
}

#[test]
fn test_draws_a_white_point_with_an_index_buffer() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_PointSize = 1.0; gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let index_buffer = create_index_buffer(vec![0, 0, 0, 0], IndexDatatype::UnsignedShort);
    let vb = create_vertex_buffer(bytemuck_cast_bytes(&[0.0f32, 0.0, 0.0, 1.0]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(vb),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: Some(index_buffer),
    });

    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 255, 255, 255]);
}

#[test]
fn test_draws_a_red_point_with_two_vertex_buffers() {
    let mut context = create_test_context();
    let vs = "in vec4 position; in mediump float intensity; out mediump float fs_intensity; void main() { gl_PointSize = 1.0; gl_Position = position; fs_intensity = intensity; }";
    let fs = "in mediump float fs_intensity; void main() { out_FragColor = vec4(fs_intensity, 0.0, 0.0, 1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0), ("intensity", 1)]);

    let vb0 = create_vertex_buffer(bytemuck_cast_bytes(&[0.0f32, 0.0, 0.0, 1.0]));
    let vb1 = create_vertex_buffer(bytemuck_cast_bytes(&[1.0f32]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![
            VertexAttributeOptions {
                index: Some(0),
                enabled: Some(true),
                vertex_buffer: Some(vb0),
                components_per_attribute: 4,
                component_datatype: Some(ComponentDatatype::Float),
                normalize: Some(false),
                offset_in_bytes: Some(0),
                stride_in_bytes: Some(0),
                instance_divisor: Some(0),
            },
            VertexAttributeOptions {
                index: Some(1),
                enabled: Some(true),
                vertex_buffer: Some(vb1),
                components_per_attribute: 1,
                component_datatype: Some(ComponentDatatype::Float),
                normalize: Some(false),
                offset_in_bytes: Some(0),
                stride_in_bytes: Some(0),
                instance_divisor: Some(0),
            },
        ],
        index_buffer: None,
    });

    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 0, 0, 255]);
}

#[test]
fn test_draws_a_red_point_with_one_interleaved_vertex_buffers() {
    let mut context = create_test_context();
    let vs = "in vec4 position; in mediump float intensity; out mediump float fs_intensity; void main() { gl_PointSize = 1.0; gl_Position = position; fs_intensity = intensity; }";
    let fs = "in mediump float fs_intensity; void main() { out_FragColor = vec4(fs_intensity, 0.0, 0.0, 1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0), ("intensity", 1)]);

    let interleaved = create_vertex_buffer(bytemuck_cast_bytes(&[0.0f32, 0.0, 0.0, 1.0, 1.0]));

    let stride = 5 * 4;
    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![
            VertexAttributeOptions {
                index: Some(0),
                enabled: Some(true),
                vertex_buffer: Some(interleaved.clone()),
                components_per_attribute: 4,
                component_datatype: Some(ComponentDatatype::Float),
                normalize: Some(false),
                offset_in_bytes: Some(0),
                stride_in_bytes: Some(stride),
                instance_divisor: Some(0),
            },
            VertexAttributeOptions {
                index: Some(1),
                enabled: Some(true),
                vertex_buffer: Some(interleaved),
                components_per_attribute: 1,
                component_datatype: Some(ComponentDatatype::Float),
                normalize: Some(false),
                offset_in_bytes: Some(16),
                stride_in_bytes: Some(stride),
                instance_divisor: Some(0),
            },
        ],
        index_buffer: None,
    });

    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 0, 0, 255]);
}

#[test]
fn test_draws_with_scissor_test() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_PointSize = 1.0; gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let vb = create_vertex_buffer(bytemuck_cast_bytes(&[0.0f32, 0.0, 0.0, 1.0]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(vb),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    // 1 of 3: Clear to black
    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // 2 of 3: Render point - fails scissor test
    let mut command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.clone().into()),
        vertex_array: Some(va.clone().into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                scissor_test: ScissorTestState {
                    enabled: true,
                    rectangle: BoundingRectangle::new(1.0, 1.0, 0.0, 0.0),
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // 3 of 3: Render point - passes scissor test
    command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                scissor_test: ScissorTestState {
                    enabled: true,
                    rectangle: BoundingRectangle::new(0.0, 0.0, 1.0, 1.0),
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 255, 255, 255]);
}

#[test]
fn test_draws_with_color_mask() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_PointSize = 1.0; gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let vb = create_vertex_buffer(bytemuck_cast_bytes(&[0.0f32, 0.0, 0.0, 1.0]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(vb),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // 2 of 3: Render point - red color mask only
    let mut command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.clone().into()),
        vertex_array: Some(va.clone().into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                color_mask: ColorMaskState {
                    red: true,
                    green: false,
                    blue: false,
                    alpha: false,
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 0, 0, 255]);

    // 3 of 3: Render point - blue color mask (blue channel added, red untouched)
    command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                color_mask: ColorMaskState {
                    red: false,
                    green: false,
                    blue: true,
                    alpha: false,
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 0, 255, 255]);
}

#[test]
fn test_draws_with_additive_blending() {
    let mut context = create_test_context_with_alpha(true);
    let vs = "in vec4 position; void main() { gl_PointSize = 1.0; gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(0.5); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let vb = create_vertex_buffer(bytemuck_cast_bytes(&[0.0f32, 0.0, 0.0, 1.0]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(vb),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    ClearCommand::new(ClearCommandOptions {
        color: Some(Color::new(0.0, 0.0, 0.0, 0.0)),
        ..Default::default()
    })
    .execute(&mut context)
    .unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 0]);

    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                blending: BlendingState {
                    enabled: true,
                    equation_rgb: BlendEquation::Add,
                    equation_alpha: BlendEquation::Add,
                    function_source_rgb: BlendFunction::One,
                    function_source_alpha: BlendFunction::One,
                    function_destination_rgb: BlendFunction::One,
                    function_destination_alpha: BlendFunction::One,
                    ..Default::default()
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });

    // 2 of 3: Blend: 0 + 0.5
    command.execute(&mut context).unwrap();
    let p1 = read_pixel_color(&context);
    assert!((p1[0] as i32 - 127).abs() <= 1);
    assert!((p1[1] as i32 - 127).abs() <= 1);
    assert!((p1[2] as i32 - 127).abs() <= 1);
    assert!((p1[3] as i32 - 127).abs() <= 1);

    // 3 of 3: Blend: 0.5 + 0.5
    command.execute(&mut context).unwrap();
    let p2 = read_pixel_color(&context);
    assert!((p2[0] as i32 - 254).abs() <= 1);
    assert!((p2[1] as i32 - 254).abs() <= 1);
    assert!((p2[2] as i32 - 254).abs() <= 1);
    assert!((p2[3] as i32 - 254).abs() <= 1);
}

#[test]
fn test_draws_with_alpha_blending() {
    let mut context = create_test_context_with_alpha(true);
    let vs = "in vec4 position; void main() { gl_PointSize = 1.0; gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0, 1.0, 1.0, 0.5); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let vb = create_vertex_buffer(bytemuck_cast_bytes(&[0.0f32, 0.0, 0.0, 1.0]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(vb),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    ClearCommand::new(ClearCommandOptions {
        color: Some(Color::new(0.0, 0.0, 0.0, 0.0)),
        ..Default::default()
    })
    .execute(&mut context)
    .unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 0]);

    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                blending: BlendingState {
                    enabled: true,
                    equation_rgb: BlendEquation::Add,
                    equation_alpha: BlendEquation::Subtract,
                    function_source_rgb: BlendFunction::SrcAlpha,
                    function_source_alpha: BlendFunction::One,
                    function_destination_rgb: BlendFunction::OneMinusSrcAlpha,
                    function_destination_alpha: BlendFunction::Zero,
                    ..Default::default()
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });

    // 2 of 3: Blend: RGB: (255 * 0.5) + (0 * 0.5), Alpha: 0.5 + 0
    command.execute(&mut context).unwrap();
    let p1 = read_pixel_color(&context);
    assert!((p1[0] as i32 - 127).abs() <= 1);
    assert!((p1[1] as i32 - 127).abs() <= 1);
    assert!((p1[2] as i32 - 127).abs() <= 1);
    assert!((p1[3] as i32 - 127).abs() <= 1);

    // 3 of 3: Blend: RGB: (255 * 0.5) + (127 * 0.5), Alpha: 0.5 + 0
    command.execute(&mut context).unwrap();
    let p2 = read_pixel_color(&context);
    assert!((p2[0] as i32 - 191).abs() <= 2);
    assert!((p2[1] as i32 - 191).abs() <= 2);
    assert!((p2[2] as i32 - 191).abs() <= 2);
    assert!((p2[3] as i32 - 127).abs() <= 1);
}

#[test]
fn test_draws_with_blend_color() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_PointSize = 1.0; gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0, 1.0, 1.0, 1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let vb = create_vertex_buffer(bytemuck_cast_bytes(&[0.0f32, 0.0, 0.0, 1.0]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(vb),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                blending: BlendingState {
                    enabled: true,
                    color: Color::new(0.5, 0.5, 0.5, 0.5),
                    equation_rgb: BlendEquation::Subtract,
                    equation_alpha: BlendEquation::Subtract,
                    function_source_rgb: BlendFunction::ConstantColor,
                    function_source_alpha: BlendFunction::One,
                    function_destination_rgb: BlendFunction::Zero,
                    function_destination_alpha: BlendFunction::Zero,
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });

    command.execute(&mut context).unwrap();
    let p = read_pixel_color(&context);
    assert!((p[0] as i32 - 128).abs() <= 1);
    assert!((p[1] as i32 - 128).abs() <= 1);
    assert!((p[2] as i32 - 128).abs() <= 1);
    assert_eq!(p[3], 255);
}

#[test]
fn test_draws_with_culling() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let quad = create_vertex_buffer(bytemuck_cast_bytes(&[
        -1000.0f32, -1000.0, 0.0, 1.0, 1000.0, -1000.0, 0.0, 1.0, -1000.0, 1000.0, 0.0, 1.0,
        1000.0, 1000.0, 0.0, 1.0,
    ]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(quad),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // Cull front faces - nothing is drawn
    let mut command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::TriangleStrip),
        shader_program: Some(sp.clone().into()),
        vertex_array: Some(va.clone().into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                cull: CullState {
                    enabled: true,
                    face: CullFace::Front,
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // Cull back faces - nothing is culled
    command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::TriangleStrip),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                cull: CullState {
                    enabled: true,
                    face: CullFace::Back,
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 255, 255, 255]);
}

#[test]
fn test_draws_with_front_face_winding_order() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let quad = create_vertex_buffer(bytemuck_cast_bytes(&[
        -1000.0f32, -1000.0, 0.0, 1.0, 1000.0, -1000.0, 0.0, 1.0, -1000.0, 1000.0, 0.0, 1.0,
        1000.0, 1000.0, 0.0, 1.0,
    ]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(quad),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // Cull back faces with Clockwise winding order - quad treated as back face and culled
    let mut command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::TriangleStrip),
        shader_program: Some(sp.clone().into()),
        vertex_array: Some(va.clone().into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                front_face: WindingOrder::Clockwise,
                cull: CullState {
                    enabled: true,
                    face: CullFace::Back,
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // Cull back faces with CounterClockwise winding order - front face rendered
    command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::TriangleStrip),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                front_face: WindingOrder::CounterClockwise,
                cull: CullState {
                    enabled: true,
                    face: CullFace::Back,
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 255, 255, 255]);
}

#[test]
fn test_draws_with_the_depth_test() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let quad = create_vertex_buffer(bytemuck_cast_bytes(&[
        -1000.0f32, -1000.0, 0.0, 1.0, 1000.0, -1000.0, 0.0, 1.0, -1000.0, 1000.0, 0.0, 1.0,
        1000.0, 1000.0, 0.0, 1.0,
    ]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(quad),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::TriangleStrip),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                depth_test: DepthTestState {
                    enabled: true,
                    func: DepthFunction::LEqual,
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });

    // 1 of 2. Passes depth test (cleared to depth 1.0)
    let clear_command = ClearCommand::new(ClearCommandOptions {
        color: Some(Color::new(0.0, 0.0, 0.0, 0.0)),
        depth: Some(1.0),
        ..Default::default()
    });
    clear_command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 255, 255, 255]);

    // 2 of 2. Fails depth test (cleared to depth 0.0)
    let clear_fail = ClearCommand::new(ClearCommandOptions {
        color: Some(Color::new(0.0, 0.0, 0.0, 0.0)),
        depth: Some(0.0),
        ..Default::default()
    });
    clear_fail.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);
}

#[test]
fn test_draws_with_depth_range() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_PointSize = 1.0; gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(gl_DepthRange.near, gl_DepthRange.far, 0.0, 1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let vb = create_vertex_buffer(bytemuck_cast_bytes(&[0.0f32, 0.0, 0.0, 1.0]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(vb),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                depth_range: DepthRangeState {
                    near: 0.25,
                    far: 0.75,
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [64, 191, 0, 255]);
}

#[test]
fn test_draws_with_line_width() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let line = create_vertex_buffer(bytemuck_cast_bytes(&[
        -1000.0f32, -1000.0, 0.0, 1.0, 1000.0, 1000.0, 0.0, 1.0,
    ]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(line),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Lines),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                line_width: ContextLimits::MAXIMUM_ALIASED_LINE_WIDTH as u32,
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_ne!(read_pixel_color(&context), [0, 0, 0, 255]);
}

#[test]
fn test_draws_with_polygon_offset() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_PointSize = 1.0; gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let vb = create_vertex_buffer(bytemuck_cast_bytes(&[0.0f32, 0.0, 0.0, 1.0]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(vb),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                polygon_offset: PolygonOffsetState {
                    enabled: true,
                    factor: 1.0,
                    units: 1.0,
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 255, 255, 255]);
}

#[test]
fn test_draws_with_sample_coverage() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_PointSize = 1.0; gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let vb = create_vertex_buffer(bytemuck_cast_bytes(&[0.0f32, 0.0, 0.0, 1.0]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(vb),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // Sample coverage value = 0: not drawn
    let mut command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.clone().into()),
        vertex_array: Some(va.clone().into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                sample_coverage: SampleCoverageState {
                    enabled: true,
                    value: 0.0,
                    invert: false,
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // Sample coverage disabled: drawn
    command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                sample_coverage: SampleCoverageState {
                    enabled: false,
                    ..Default::default()
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 255, 255, 255]);
}

#[test]
fn test_draws_with_stencil_test_front() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_PointSize = 1.0; gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let quad = create_vertex_buffer(bytemuck_cast_bytes(&[
        -1000.0f32, -1000.0, 0.0, 1.0, 1000.0, -1000.0, 0.0, 1.0, -1000.0, 1000.0, 0.0, 1.0,
        1000.0, 1000.0, 0.0, 1.0,
    ]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(quad),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    let rs = RenderState::from_cache(Some(RenderState {
        stencil_test: StencilTestState {
            enabled: true,
            front_function: StencilFunction::Equal,
            reference: 1,
            mask: 1,
            ..Default::default()
        },
        ..Default::default()
    }));

    // 1 of 4. Clear, including stencil (0)
    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // 2 of 4. Render where stencil is set (current is 0 != 1) - nothing is drawn
    let mut command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::TriangleStrip),
        shader_program: Some(sp.clone().into()),
        vertex_array: Some(va.clone().into()),
        render_state: Some(rs.clone().into()),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // 3 of 4. Render to stencil only, increment to 1
    command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::TriangleStrip),
        shader_program: Some(sp.clone().into()),
        vertex_array: Some(va.clone().into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                color_mask: ColorMaskState {
                    red: false,
                    green: false,
                    blue: false,
                    alpha: false,
                },
                stencil_test: StencilTestState {
                    enabled: true,
                    front_operation: StencilOperationState {
                        z_pass: StencilOperation::Incr,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // 4 of 4. Render where stencil is set (current is 1 == 1) - drawn!
    command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::TriangleStrip),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        render_state: Some(rs.into()),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 255, 255, 255]);
}

#[test]
fn test_draws_with_stencil_test_back() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let quad = create_vertex_buffer(bytemuck_cast_bytes(&[
        -1000.0f32, -1000.0, 0.0, 1.0, 1000.0, -1000.0, 0.0, 1.0, -1000.0, 1000.0, 0.0, 1.0,
        1000.0, 1000.0, 0.0, 1.0,
    ]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(quad),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    let rs = RenderState::from_cache(Some(RenderState {
        front_face: WindingOrder::Clockwise,
        stencil_test: StencilTestState {
            enabled: true,
            back_function: StencilFunction::NotEqual,
            reference: 0,
            ..Default::default()
        },
        ..Default::default()
    }));

    // 1 of 4. Clear, including stencil (0)
    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // 2 of 4. Render where stencil != 0 (current is 0) - nothing drawn
    let mut command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::TriangleStrip),
        shader_program: Some(sp.clone().into()),
        vertex_array: Some(va.clone().into()),
        render_state: Some(rs.clone().into()),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // 3 of 4. Render to stencil only, invert (0 -> 255)
    command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::TriangleStrip),
        shader_program: Some(sp.clone().into()),
        vertex_array: Some(va.clone().into()),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                front_face: WindingOrder::Clockwise,
                color_mask: ColorMaskState {
                    red: false,
                    green: false,
                    blue: false,
                    alpha: false,
                },
                stencil_test: StencilTestState {
                    enabled: true,
                    back_operation: StencilOperationState {
                        z_pass: StencilOperation::Invert,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // 4 of 4. Render where stencil != 0 (current is 255 != 0) - drawn!
    command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::TriangleStrip),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        render_state: Some(rs.into()),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 255, 255, 255]);
}

#[test]
fn test_draws_with_an_offset_and_count() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_PointSize = 1.0; gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let vb = create_vertex_buffer(bytemuck_cast_bytes(&[
        0.0f32, 0.0, 0.0, -1.0, // Point 0: clipped (w <= 0)
        0.0, 0.0, 0.0, 1.0, // Point 1: visible (w = 1)
    ]));

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![VertexAttributeOptions {
            index: Some(0),
            enabled: Some(true),
            vertex_buffer: Some(vb),
            components_per_attribute: 4,
            component_datatype: Some(ComponentDatatype::Float),
            normalize: Some(false),
            offset_in_bytes: Some(0),
            stride_in_bytes: Some(0),
            instance_divisor: Some(0),
        }],
        index_buffer: None,
    });

    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // The first point does not generate any pixels
    let mut command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        offset: Some(0),
        count: Some(1),
        shader_program: Some(sp.clone().into()),
        vertex_array: Some(va.clone().into()),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    // The second point generates pixels
    command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        offset: Some(1),
        count: Some(1),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [255, 255, 255, 255]);
}

#[test]
fn test_draws_two_instances_of_a_point_with_different_per_instance_colors() {
    let mut context = create_test_context();
    let vs = "in vec4 position; in vec4 color; out vec4 v_color; void main() { gl_PointSize = 1.0; gl_Position = position; v_color = color; }";
    let fs = "in vec4 v_color; void main() { out_FragColor = v_color; }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0), ("color", 1)]);

    let pos_buf = create_vertex_buffer(bytemuck_cast_bytes(&[0.0f32, 0.0, 0.0, 1.0]));
    let color_buf = create_vertex_buffer(vec![255, 0, 0, 255, 0, 255, 0, 255]);

    let va = VertexArray::new(VertexArrayOptions {
        attributes: vec![
            VertexAttributeOptions {
                index: Some(0),
                enabled: Some(true),
                vertex_buffer: Some(pos_buf),
                components_per_attribute: 4,
                component_datatype: Some(ComponentDatatype::Float),
                normalize: Some(false),
                offset_in_bytes: Some(0),
                stride_in_bytes: Some(0),
                instance_divisor: Some(0),
            },
            VertexAttributeOptions {
                index: Some(1),
                enabled: Some(true),
                vertex_buffer: Some(color_buf),
                components_per_attribute: 4,
                component_datatype: Some(ComponentDatatype::UnsignedByte),
                normalize: Some(true),
                offset_in_bytes: Some(0),
                stride_in_bytes: Some(0),
                instance_divisor: Some(1),
            },
        ],
        index_buffer: None,
    });

    ClearCommand::ALL.execute(&mut context).unwrap();
    assert_eq!(read_pixel_color(&context), [0, 0, 0, 255]);

    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        instance_count: Some(2),
        render_state: Some(
            RenderState::from_cache(Some(RenderState {
                blending: BlendingState::ADDITIVE_BLEND,
                ..Default::default()
            }))
            .into(),
        ),
        ..Default::default()
    });
    command.execute(&mut context).unwrap();
    // Red + Green = Yellow
    assert_eq!(read_pixel_color(&context), [255, 255, 0, 255]);
}

#[test]
fn test_fails_to_draw_missing_command() {
    let mut context = create_test_context();
    assert!(context.draw_command(None).is_err());
}

#[test]
fn test_fails_to_draw_missing_shader_program() {
    let mut context = create_test_context();
    let va = VertexArray::default();
    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        vertex_array: Some(va.into()),
        shader_program: None,
        ..Default::default()
    });
    assert!(command.execute(&mut context).is_err());
}

#[test]
fn test_fails_to_draw_missing_vertex_array() {
    let mut context = create_test_context();
    let vs = "in vec4 position; void main() { gl_PointSize = 1.0; gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);

    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: None,
        ..Default::default()
    });
    assert!(command.execute(&mut context).is_err());
}

#[test]
fn test_throws_when_instance_count_greater_than_one_and_instancing_is_disabled() {
    let mut context = create_test_context();
    context.instanced_arrays = false; // Simulate WebGL 1 without instanced arrays extension

    let vs = "in vec4 position; void main() { gl_PointSize = 1.0; gl_Position = position; }";
    let fs = "void main() { out_FragColor = vec4(1.0); }";
    let sp = create_shader_program(&context, vs, fs, &[("position", 0)]);
    let va = VertexArray::default();

    let command = DrawCommand::new(DrawCommandOptions {
        primitive_type: Some(PrimitiveType::Points),
        shader_program: Some(sp.into()),
        vertex_array: Some(va.into()),
        instance_count: Some(2),
        ..Default::default()
    });
    let err = command.execute(&mut context).unwrap_err();
    assert!(err.message.contains("Instanced arrays"));
}
