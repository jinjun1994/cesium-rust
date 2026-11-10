use cesium_math::{AttributeType, Cartesian4, ComponentDatatype};
use cesium_renderer::buffer::{Buffer, BufferOptions};
use cesium_renderer::model_components::{Attribute, Primitive};
use cesium_renderer::primitive_outline_pipeline_stage::{
    MockPrimitiveOutlineModel, MockPrimitiveOutlineRenderResources, PrimitiveOutlinePipelineStage,
    PRIMITIVE_OUTLINE_STAGE_FS, PRIMITIVE_OUTLINE_STAGE_VS,
};

#[test]
fn test_primitive_outline_pipeline_stage_name() {
    assert_eq!(
        PrimitiveOutlinePipelineStage::NAME,
        "PrimitiveOutlinePipelineStage"
    );
}

#[test]
fn test_processes_model_with_cesium_primitive_outline_extension() {
    let mut render_resources = MockPrimitiveOutlineRenderResources {
        model: MockPrimitiveOutlineModel {
            outline_color: Cartesian4::new(1.0, 0.0, 0.0, 1.0), // RED
            show_outline: false,
        },
        attribute_index: 1,
        ..Default::default()
    };

    let buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 60,
        ..Default::default()
    })
    .unwrap();

    let outline_coordinates = Attribute {
        name: Some("_OUTLINE_COORDINATES".to_string()),
        buffer: Some(buffer.clone()),
        component_datatype: Some(ComponentDatatype::Float),
        attribute_type: Some(AttributeType::Vec3),
        normalized: false,
        count: 5,
        byte_offset: 0,
        byte_stride: None,
        ..Default::default()
    };

    let primitive = Primitive {
        outline_coordinates: Some(outline_coordinates),
        ..Default::default()
    };

    PrimitiveOutlinePipelineStage::process(&mut render_resources, &primitive);

    // Uniforms
    assert!(render_resources.outline_texture.is_some());
    assert_eq!(
        render_resources.bound_outline_color,
        Some(Cartesian4::new(1.0, 0.0, 0.0, 1.0))
    );
    assert_eq!(render_resources.bound_show_outline, Some(false));

    // ShaderBuilder checks
    let sb = &render_resources.shader_builder;

    assert!(sb.vertex_define_lines().contains(&"HAS_PRIMITIVE_OUTLINE".to_string()));
    assert!(sb.fragment_define_lines().contains(&"HAS_PRIMITIVE_OUTLINE".to_string()));

    assert!(sb
        .attribute_lines()
        .contains(&"in vec3 a_outlineCoordinates;".to_string()));

    assert!(sb
        .vertex_varying_lines()
        .contains(&"out vec3 v_outlineCoordinates;".to_string()));
    assert!(sb
        .fragment_varying_lines()
        .contains(&"in vec3 v_outlineCoordinates;".to_string()));

    assert!(sb
        .fragment_uniform_lines()
        .contains(&"uniform sampler2D model_outlineTexture;".to_string()));
    assert!(sb
        .fragment_uniform_lines()
        .contains(&"uniform vec4 model_outlineColor;".to_string()));
    assert!(sb
        .fragment_uniform_lines()
        .contains(&"uniform bool model_showOutline;".to_string()));

    assert!(sb.vertex_lines().contains(&PRIMITIVE_OUTLINE_STAGE_VS.to_string()));
    assert!(sb.fragment_lines().contains(&PRIMITIVE_OUTLINE_STAGE_FS.to_string()));

    // Attributes
    assert_eq!(render_resources.attributes.len(), 1);
    let attr = &render_resources.attributes[0];
    assert_eq!(attr.index, 1);
    assert_eq!(render_resources.attribute_index, 2);
    assert_eq!(attr.vertex_buffer.as_ref().unwrap().id, buffer.id);
    assert_eq!(attr.components_per_attribute, 3);
    assert_eq!(attr.component_datatype, ComponentDatatype::Float);
    assert_eq!(attr.offset_in_bytes, 0);
    assert_eq!(attr.stride_in_bytes, 0);
    assert!(!attr.normalize);
}
