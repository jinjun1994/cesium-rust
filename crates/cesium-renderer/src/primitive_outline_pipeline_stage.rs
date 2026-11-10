//! PrimitiveOutlinePipelineStage matching CesiumJS `Scene/Model/PrimitiveOutlinePipelineStage.js` (Apache-2.0).

use std::collections::HashMap;
use cesium_math::{AttributeType, Cartesian4, ComponentDatatype};

use crate::model_components::Primitive;
use crate::model_render_resources::ModelUniformCallback;
use crate::primitive_outline_generator::PrimitiveOutlineGenerator;
use crate::primitive_render_resources::PrimitiveRenderResources;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};
use crate::texture::Texture;
use crate::vertex_array::VertexAttribute;

/// GLSL vertex shader code for primitive outline stage.
pub const PRIMITIVE_OUTLINE_STAGE_VS: &str = r#"void primitiveOutlineStage() {
    v_outlineCoordinates = a_outlineCoordinates;
}"#;

/// GLSL fragment shader code for primitive outline stage.
pub const PRIMITIVE_OUTLINE_STAGE_FS: &str = r#"void primitiveOutlineStage(inout czm_modelMaterial material) {
    if (!model_showOutline) {
        return;
    }

    float outlineX = 
        texture(model_outlineTexture, vec2(v_outlineCoordinates.x, 0.5)).r;
    float outlineY = 
        texture(model_outlineTexture, vec2(v_outlineCoordinates.y, 0.5)).r;
    float outlineZ = 
        texture(model_outlineTexture, vec2(v_outlineCoordinates.z, 0.5)).r;
    float outlineness = max(outlineX, max(outlineY, outlineZ));

    material.diffuse = mix(material.diffuse, model_outlineColor.rgb, model_outlineColor.a * outlineness);
}"#;

/// Trait defining the render resources required by [`PrimitiveOutlinePipelineStage`].
pub trait PrimitiveOutlineRenderResources {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder;
    fn attributes_mut(&mut self) -> &mut Vec<VertexAttribute>;
    fn attribute_index_mut(&mut self) -> &mut u32;
    fn outline_color(&self) -> Cartesian4;
    fn show_outline(&self) -> bool;
    fn set_outline_texture_uniform(&mut self, texture: Texture);
    fn set_outline_color_uniform(&mut self, color: Cartesian4);
    fn set_show_outline_uniform(&mut self, show: bool);
}

/// Standalone test render resources container for PrimitiveOutlinePipelineStage.
#[derive(Debug, Default)]
pub struct MockPrimitiveOutlineModel {
    pub outline_color: Cartesian4,
    pub show_outline: bool,
}

#[derive(Default)]
pub struct MockPrimitiveOutlineRenderResources {
    pub model: MockPrimitiveOutlineModel,
    pub shader_builder: ShaderBuilder,
    pub attributes: Vec<VertexAttribute>,
    pub attribute_index: u32,
    pub uniform_map: HashMap<String, ModelUniformCallback>,
    pub outline_texture: Option<Texture>,
    pub bound_outline_color: Option<Cartesian4>,
    pub bound_show_outline: Option<bool>,
}

impl std::fmt::Debug for MockPrimitiveOutlineRenderResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockPrimitiveOutlineRenderResources")
            .field("model", &self.model)
            .field("shader_builder", &self.shader_builder)
            .field("attributes", &self.attributes)
            .field("attribute_index", &self.attribute_index)
            .field("outline_texture", &self.outline_texture)
            .field("bound_outline_color", &self.bound_outline_color)
            .field("bound_show_outline", &self.bound_show_outline)
            .finish()
    }
}

impl PrimitiveOutlineRenderResources for MockPrimitiveOutlineRenderResources {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder {
        &mut self.shader_builder
    }

    fn attributes_mut(&mut self) -> &mut Vec<VertexAttribute> {
        &mut self.attributes
    }

    fn attribute_index_mut(&mut self) -> &mut u32 {
        &mut self.attribute_index
    }

    fn outline_color(&self) -> Cartesian4 {
        self.model.outline_color
    }

    fn show_outline(&self) -> bool {
        self.model.show_outline
    }

    fn set_outline_texture_uniform(&mut self, texture: Texture) {
        self.outline_texture = Some(texture);
    }

    fn set_outline_color_uniform(&mut self, color: Cartesian4) {
        self.bound_outline_color = Some(color);
    }

    fn set_show_outline_uniform(&mut self, show: bool) {
        self.bound_show_outline = Some(show);
    }
}

/// Provider trait for models that support primitive outlines.
pub trait PrimitiveOutlineModelProvider {
    fn outline_color(&self) -> Cartesian4;
    fn show_outline(&self) -> bool;
}

impl<'a, M: PrimitiveOutlineModelProvider> PrimitiveOutlineRenderResources
    for PrimitiveRenderResources<'a, M>
{
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder {
        &mut self.shader_builder
    }

    fn attributes_mut(&mut self) -> &mut Vec<VertexAttribute> {
        &mut self.attributes
    }

    fn attribute_index_mut(&mut self) -> &mut u32 {
        &mut self.attribute_index
    }

    fn outline_color(&self) -> Cartesian4 {
        self.model.outline_color()
    }

    fn show_outline(&self) -> bool {
        self.model.show_outline()
    }

    fn set_outline_texture_uniform(&mut self, _texture: Texture) {}

    fn set_outline_color_uniform(&mut self, _color: Cartesian4) {}

    fn set_show_outline_uniform(&mut self, _show: bool) {}
}

/// The primitive outline pipeline stage configures the shader to render outlines
/// from the CESIUM_primitive_outline extension.
pub struct PrimitiveOutlinePipelineStage;

impl PrimitiveOutlinePipelineStage {
    pub const NAME: &'static str = "PrimitiveOutlinePipelineStage";

    /// Process a primitive to configure outlines.
    pub fn process<R: PrimitiveOutlineRenderResources>(
        render_resources: &mut R,
        primitive: &Primitive,
    ) {
        let shader_builder = render_resources.shader_builder_mut();

        shader_builder.add_define(
            "HAS_PRIMITIVE_OUTLINE",
            None,
            Some(ShaderDestination::BOTH),
        );

        shader_builder.add_attribute("vec3", "a_outlineCoordinates");
        shader_builder.add_varying("vec3", "v_outlineCoordinates", None);

        if let Some(ref outline_coords) = primitive.outline_coordinates {
            let attr_idx = *render_resources.attribute_index_mut();
            *render_resources.attribute_index_mut() += 1;

            let num_components = match outline_coords.attribute_type {
                Some(AttributeType::Vec3) => 3,
                _ => 3,
            };

            let vertex_attribute = VertexAttribute {
                index: attr_idx,
                enabled: true,
                vertex_buffer: outline_coords.buffer.clone(),
                components_per_attribute: num_components,
                component_datatype: outline_coords
                    .component_datatype
                    .unwrap_or(ComponentDatatype::Float),
                normalize: outline_coords.normalized,
                offset_in_bytes: outline_coords.byte_offset,
                stride_in_bytes: outline_coords.byte_stride.unwrap_or(0),
                instance_divisor: 0,
            };

            render_resources.attributes_mut().push(vertex_attribute);
        }

        let shader_builder = render_resources.shader_builder_mut();
        shader_builder.add_uniform(
            "sampler2D",
            "model_outlineTexture",
            Some(ShaderDestination::FRAGMENT),
        );

        let outline_texture = PrimitiveOutlineGenerator::create_texture();
        render_resources.set_outline_texture_uniform(outline_texture);

        let color = render_resources.outline_color();
        let show = render_resources.show_outline();

        let shader_builder = render_resources.shader_builder_mut();
        shader_builder.add_uniform(
            "vec4",
            "model_outlineColor",
            Some(ShaderDestination::FRAGMENT),
        );
        render_resources.set_outline_color_uniform(color);

        let shader_builder = render_resources.shader_builder_mut();
        shader_builder.add_uniform(
            "bool",
            "model_showOutline",
            Some(ShaderDestination::FRAGMENT),
        );
        render_resources.set_show_outline_uniform(show);

        let shader_builder = render_resources.shader_builder_mut();
        shader_builder.add_vertex_lines(&[PRIMITIVE_OUTLINE_STAGE_VS]);
        shader_builder.add_fragment_lines(&[PRIMITIVE_OUTLINE_STAGE_FS]);
    }
}
