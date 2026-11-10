//! ImageryPipelineStage matching CesiumJS `Scene/Model/ImageryPipelineStage.js` (Apache-2.0).

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use cesium_math::{
    Cartesian4, ComponentDatatype, DeveloperError, Ellipsoid, GeoRectangle, NativeRectangle,
    WebMercatorTilingScheme,
};

use crate::buffer::Buffer;
use crate::imagery_layer::ImageryLayer;
use crate::imagery_layer_collection::ImageryLayerCollection;
use crate::mapped_positions::MappedPositions;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};

pub const DEFAULT_BRIGHTNESS: f32 = 1.0;
pub const DEFAULT_CONTRAST: f32 = 1.0;
pub const DEFAULT_HUE: f32 = 0.0;
pub const DEFAULT_SATURATION: f32 = 1.0;
pub const DEFAULT_GAMMA: f32 = 1.0;

/// Flags indicating which imagery features need to be processed in shaders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageryFlags {
    pub alpha: bool,
    pub brightness: bool,
    pub contrast: bool,
    pub hue: bool,
    pub saturation: bool,
    pub gamma: bool,
    pub color_to_alpha: bool,
}

/// The state of an imagery tile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageryState {
    #[default]
    Unloaded,
    Transitioning,
    Received,
    TextureLoaded,
    Ready,
    Failed,
    Invalid,
}

/// Representation of an imagery tile and its texture.
#[derive(Clone, Debug, PartialEq)]
pub struct Imagery {
    pub state: ImageryState,
    pub rectangle: GeoRectangle,
    pub texture: Option<String>,
    pub texture_web_mercator: Option<String>,
}

impl Imagery {
    pub fn new(rectangle: GeoRectangle) -> Self {
        Self {
            state: ImageryState::Ready,
            rectangle,
            texture: None,
            texture_web_mercator: None,
        }
    }
}

/// Describes one imagery tile covered by a primitive.
#[derive(Clone, Debug)]
pub struct ImageryCoverage {
    pub x: u32,
    pub y: u32,
    pub level: u32,
    pub imagery: Arc<RwLock<Imagery>>,
    pub texture_coordinate_rectangle: Cartesian4,
}

/// An assembled input for draping imagery over 3D Tiles in the shader.
#[derive(Clone, Debug)]
pub struct ImageryInput {
    pub imagery_layer: Arc<RwLock<ImageryLayer>>,
    pub texture: String,
    pub texture_translation_and_scale: Cartesian4,
    pub texture_coordinate_rectangle: Cartesian4,
    pub imagery_tex_coord_attribute_set_index: usize,
}

/// A vertex attribute associated with imagery texture coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageryStageVertexAttribute {
    pub index: usize,
    pub value: Option<Vec<f64>>,
    pub vertex_buffer: Option<Buffer>,
    pub count: usize,
    pub components_per_attribute: usize,
    pub component_datatype: ComponentDatatype,
    pub offset_in_bytes: usize,
    pub stride_in_bytes: Option<usize>,
    pub normalize: bool,
}

/// Holds imagery mapping and coverage information for a primitive.
#[derive(Clone, Debug, Default)]
pub struct ModelPrimitiveImagery {
    pub runtime_node_index: usize,
    pub runtime_primitive_index: usize,
    pub ready: bool,
    pub is_destroyed: bool,
    pub mapped_positions_by_layer_id: HashMap<u64, MappedPositions>,
    pub coverages_by_layer_id: HashMap<u64, Vec<ImageryCoverage>>,
    pub imagery_tex_coord_attributes: Vec<ImageryStageVertexAttribute>,
    pub projections: Vec<String>,
}

impl ModelPrimitiveImagery {
    pub fn new() -> Self {
        Self {
            runtime_node_index: 0,
            runtime_primitive_index: 0,
            ready: true,
            is_destroyed: false,
            mapped_positions_by_layer_id: HashMap::new(),
            coverages_by_layer_id: HashMap::new(),
            imagery_tex_coord_attributes: Vec::new(),
            projections: Vec::new(),
        }
    }

    pub fn with_indices(runtime_node_index: usize, runtime_primitive_index: usize) -> Self {
        Self {
            runtime_node_index,
            runtime_primitive_index,
            ready: true,
            is_destroyed: false,
            mapped_positions_by_layer_id: HashMap::new(),
            coverages_by_layer_id: HashMap::new(),
            imagery_tex_coord_attributes: Vec::new(),
            projections: Vec::new(),
        }
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.mapped_positions_by_layer_id.clear();
        self.coverages_by_layer_id.clear();
        self.imagery_tex_coord_attributes.clear();
        self.projections.clear();
    }

    pub fn update(&mut self) {
        self.ready = true;
    }

    pub fn compute_unique_ellipsoids(
        imagery_layers: Option<&ImageryLayerCollection>,
    ) -> Result<Vec<Ellipsoid>, DeveloperError> {
        let layers = imagery_layers.ok_or_else(|| DeveloperError::new("imageryLayers is required"))?;
        let mut unique = Vec::new();
        for i in 0..layers.length() {
            if let Ok(layer_arc) = layers.get(i) {
                let layer = layer_arc.read().unwrap();
                let ell = layer.ellipsoid();
                if !unique.contains(&ell) {
                    unique.push(ell);
                }
            }
        }
        Ok(unique)
    }

    pub fn compute_unique_ellipsoids_from_slice(
        ellipsoids: Option<&[Ellipsoid]>,
    ) -> Result<Vec<Ellipsoid>, DeveloperError> {
        let list = ellipsoids.ok_or_else(|| DeveloperError::new("ellipsoids is required"))?;
        let mut unique = Vec::new();
        for ell in list {
            if !unique.contains(ell) {
                unique.push(*ell);
            }
        }
        Ok(unique)
    }

    pub fn extract_projections_checked(
        imagery_layers: Option<&ImageryLayerCollection>,
    ) -> Result<Vec<String>, DeveloperError> {
        let layers = imagery_layers.ok_or_else(|| DeveloperError::new("imageryLayers is required"))?;
        Ok(Self::extract_projections(layers))
    }

    pub fn mapped_positions_for_imagery_layer(
        &self,
        layer: &ImageryLayer,
    ) -> Option<&MappedPositions> {
        self.mapped_positions_by_layer_id.get(&layer.id())
    }

    pub fn coverages_for_imagery_layer(&self, layer: &ImageryLayer) -> &[ImageryCoverage] {
        self.coverages_by_layer_id
            .get(&layer.id())
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn imagery_tex_coord_attributes_per_projection(&self) -> &[ImageryStageVertexAttribute] {
        &self.imagery_tex_coord_attributes
    }

    pub fn extract_projections(imagery_layers: &ImageryLayerCollection) -> Vec<String> {
        let mut projections = Vec::with_capacity(imagery_layers.length());
        for i in 0..imagery_layers.length() {
            if let Ok(layer_arc) = imagery_layers.get(i) {
                let _guard = layer_arc.read().unwrap();
                projections.push("WebMercator".to_string());
            }
        }
        projections
    }
}

/// Model options for imagery pipeline stage.
#[derive(Clone, Debug, Default)]
pub struct ImageryModelOptions {
    pub imagery_layers: Option<Arc<RwLock<ImageryLayerCollection>>>,
}

/// Runtime primitive for imagery draping.
#[derive(Clone, Debug, Default)]
pub struct ImageryRuntimePrimitive {
    pub model_primitive_imagery: Option<ModelPrimitiveImagery>,
    pub indices_count: Option<usize>,
    pub attribute_count: usize,
}

/// FrameState context for imagery draping.
#[derive(Clone, Debug, Default)]
pub struct ImageryFrameState {
    pub default_texture: Option<String>,
}

/// Render resources for ImageryPipelineStage.
pub struct ImageryRenderResources<'a> {
    pub model: &'a ImageryModelOptions,
    pub shader_builder: &'a mut ShaderBuilder,
    pub uniform_map: &'a mut HashMap<String, Box<dyn Fn() -> serde_json::Value + Send + Sync>>,
    pub attributes: &'a mut Vec<ImageryStageVertexAttribute>,
    pub attribute_index: &'a mut usize,
    pub count: usize,
}

/// Uniform values calculated from imagery inputs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImageryUniforms {
    pub textures: Vec<String>,
    pub coordinate_rectangles: Vec<Cartesian4>,
    pub translation_and_scales: Vec<Cartesian4>,
    pub alphas: Vec<f32>,
    pub brightnesses: Vec<f32>,
    pub contrasts: Vec<f32>,
    pub hues: Vec<f32>,
    pub saturations: Vec<f32>,
    pub one_over_gammas: Vec<f32>,
    pub color_to_alphas: Vec<Cartesian4>,
}

/// A pipeline stage that modifies the model shader to take into account
/// imagery textures that are draped over a primitive of the model.
pub struct ImageryPipelineStage;

impl ImageryPipelineStage {
    pub const NAME: &'static str = "ImageryPipelineStage";

    /// Process a primitive render resources.
    pub fn process(
        primitive_render_resources: &mut ImageryRenderResources<'_>,
        primitive: &ImageryRuntimePrimitive,
        _frame_state: &ImageryFrameState,
    ) {
        let model_primitive_imagery = match &primitive.model_primitive_imagery {
            Some(mpi) if mpi.ready => mpi,
            _ => return,
        };

        let imagery_layers_arc = match &primitive_render_resources.model.imagery_layers {
            Some(arc) => Arc::clone(arc),
            None => return,
        };

        let imagery_layers = imagery_layers_arc.read().unwrap();
        let all_projections = if !model_primitive_imagery.projections.is_empty() {
            model_primitive_imagery.projections.clone()
        } else {
            ModelPrimitiveImagery::extract_projections(&imagery_layers)
        };

        let mut unique_projections: Vec<String> = Vec::new();
        for proj in &all_projections {
            if !unique_projections.contains(proj) {
                unique_projections.push(proj.clone());
            }
        }

        let imagery_tex_coord_attribute_set_indices =
            Self::compute_index_mapping(&all_projections, &unique_projections);

        let mut imagery_inputs = Self::create_imagery_inputs(
            &imagery_layers,
            model_primitive_imagery,
            &imagery_tex_coord_attribute_set_indices,
        );

        if imagery_inputs.is_empty() {
            return;
        }

        if imagery_inputs.len() > 10 {
            imagery_inputs.truncate(10);
        }

        Self::add_imagery_tex_coord_attributes_to_render_resources(
            model_primitive_imagery,
            primitive_render_resources,
        );

        let mut layers_vec = Vec::with_capacity(imagery_layers.length());
        for i in 0..imagery_layers.length() {
            if let Ok(l) = imagery_layers.get(i) {
                layers_vec.push(l.read().unwrap().clone());
            }
        }

        Self::process_imagery_inputs(
            &layers_vec,
            primitive_render_resources,
            &imagery_inputs,
            unique_projections.len(),
        );
    }

    /// Computes index mapping from source elements to target elements.
    pub fn compute_index_mapping<T: PartialEq>(source: &[T], target: &[T]) -> Vec<usize> {
        source
            .iter()
            .map(|elem| target.iter().position(|t| t == elem).unwrap_or(0))
            .collect()
    }

    /// Creates imagery inputs for all visible layers and coverages.
    pub fn create_imagery_inputs(
        imagery_layers: &ImageryLayerCollection,
        model_primitive_imagery: &ModelPrimitiveImagery,
        imagery_tex_coord_attribute_set_indices: &[usize],
    ) -> Vec<ImageryInput> {
        let mut inputs = Vec::new();

        for i in 0..imagery_layers.length() {
            let layer_arc = match imagery_layers.get(i) {
                Ok(l) => l,
                Err(_) => continue,
            };

            let layer_guard = layer_arc.read().unwrap();
            if !layer_guard.show() {
                continue;
            }

            let tex_coord_idx = imagery_tex_coord_attribute_set_indices
                .get(i)
                .copied()
                .unwrap_or(0);

            let mapped_positions = match model_primitive_imagery
                .mapped_positions_for_imagery_layer(&layer_guard)
            {
                Some(mp) => mp,
                None => continue,
            };

            let cartographic_bounding_rectangle = mapped_positions.cartographic_bounding_rectangle();
            let coverages = model_primitive_imagery.coverages_for_imagery_layer(&layer_guard);

            drop(layer_guard);

            for coverage in coverages {
                if let Some(input) = Self::create_imagery_input(
                    &layer_arc,
                    coverage,
                    &cartographic_bounding_rectangle,
                    tex_coord_idx,
                ) {
                    inputs.push(input);
                }
            }
        }

        inputs
    }

    /// Creates a single imagery input for a coverage.
    pub fn create_imagery_input(
        imagery_layer_arc: &Arc<RwLock<ImageryLayer>>,
        coverage: &ImageryCoverage,
        cartographic_bounding_rectangle: &GeoRectangle,
        imagery_tex_coord_attribute_set_index: usize,
    ) -> Option<ImageryInput> {
        let imagery_guard = coverage.imagery.read().unwrap();
        if imagery_guard.state == ImageryState::Failed || imagery_guard.state == ImageryState::Invalid
        {
            return None;
        }

        let texture = imagery_guard
            .texture_web_mercator
            .as_ref()
            .or(imagery_guard.texture.as_ref())
            .cloned()?;

        let imagery_rectangle = imagery_guard.rectangle;
        drop(imagery_guard);

        let layer_guard = imagery_layer_arc.read().unwrap();
        let texture_translation_and_scale = Self::compute_texture_translation_and_scale(
            &layer_guard,
            cartographic_bounding_rectangle,
            &imagery_rectangle,
        );
        drop(layer_guard);

        Some(ImageryInput {
            imagery_layer: Arc::clone(imagery_layer_arc),
            texture,
            texture_translation_and_scale,
            texture_coordinate_rectangle: coverage.texture_coordinate_rectangle,
            imagery_tex_coord_attribute_set_index,
        })
    }

    /// Computes translation and scale from native bounding rectangles.
    pub fn compute_texture_translation_and_scale_from_native(
        native_bounding_rectangle: &NativeRectangle,
        native_imagery_rectangle: &NativeRectangle,
    ) -> Cartesian4 {
        let width = native_imagery_rectangle.east - native_imagery_rectangle.west;
        let height = native_imagery_rectangle.north - native_imagery_rectangle.south;
        let inv_imagery_width = if width.abs() > 0.0 { 1.0 / width } else { 1.0 };
        let inv_imagery_height = if height.abs() > 0.0 { 1.0 / height } else { 1.0 };

        let delta_west = native_bounding_rectangle.west - native_imagery_rectangle.west;
        let delta_south = native_bounding_rectangle.south - native_imagery_rectangle.south;

        let bound_width = native_bounding_rectangle.east - native_bounding_rectangle.west;
        let bound_height = native_bounding_rectangle.north - native_bounding_rectangle.south;

        let offset_x = delta_west * inv_imagery_width;
        let offset_y = delta_south * inv_imagery_height;
        let scale_x = bound_width * inv_imagery_width;
        let scale_y = bound_height * inv_imagery_height;

        Cartesian4::new(offset_x, offset_y, scale_x, scale_y)
    }

    /// Computes translation and scale for an imagery layer using its tiling scheme.
    pub fn compute_texture_translation_and_scale(
        _imagery_layer: &ImageryLayer,
        non_native_bounding_rectangle: &GeoRectangle,
        non_native_imagery_rectangle: &GeoRectangle,
    ) -> Cartesian4 {
        let tiling_scheme = WebMercatorTilingScheme::new(Ellipsoid::default(), None, 1, 1).unwrap_or_default();
        let native_bounding_rectangle = tiling_scheme
            .rectangle_to_native(*non_native_bounding_rectangle)
            .unwrap_or(NativeRectangle {
                west: non_native_bounding_rectangle.west,
                south: non_native_bounding_rectangle.south,
                east: non_native_bounding_rectangle.east,
                north: non_native_bounding_rectangle.north,
            });
        let native_imagery_rectangle = tiling_scheme
            .rectangle_to_native(*non_native_imagery_rectangle)
            .unwrap_or(NativeRectangle {
                west: non_native_imagery_rectangle.west,
                south: non_native_imagery_rectangle.south,
                east: non_native_imagery_rectangle.east,
                north: non_native_imagery_rectangle.north,
            });

        Self::compute_texture_translation_and_scale_from_native(
            &native_bounding_rectangle,
            &native_imagery_rectangle,
        )
    }

    /// Computes the imagery flags across an array of imagery layers.
    pub fn compute_imagery_flags(imagery_layers: &[ImageryLayer]) -> ImageryFlags {
        let mut flags = ImageryFlags::default();
        for layer in imagery_layers {
            flags.alpha |= (layer.alpha() - 1.0).abs() > 1e-6;
            flags.brightness |= (layer.brightness() - DEFAULT_BRIGHTNESS).abs() > 1e-6;
            flags.contrast |= (layer.contrast() - DEFAULT_CONTRAST).abs() > 1e-6;
            flags.hue |= (layer.hue() - DEFAULT_HUE).abs() > 1e-6;
            flags.saturation |= (layer.saturation() - DEFAULT_SATURATION).abs() > 1e-6;
            flags.gamma |= (layer.gamma() - DEFAULT_GAMMA).abs() > 1e-6;
            flags.color_to_alpha |=
                layer.color_to_alpha().is_some() && layer.color_to_alpha_threshold() > 0.0;
        }
        flags
    }

    /// Adds imagery texture coordinate attributes to render resources.
    pub fn add_imagery_tex_coord_attributes_to_render_resources(
        model_primitive_imagery: &ModelPrimitiveImagery,
        primitive_render_resources: &mut ImageryRenderResources<'_>,
    ) {
        for attr in model_primitive_imagery.imagery_tex_coord_attributes_per_projection() {
            let mut clone_attr = attr.clone();
            clone_attr.index = *primitive_render_resources.attribute_index;
            *primitive_render_resources.attribute_index += 1;
            primitive_render_resources.attributes.push(clone_attr);
        }
    }

    /// Adds attributes and varyings to the shader builder.
    pub fn add_attributes(shader_builder: &mut ShaderBuilder, num_tex_coords: usize) {
        for i in 0..num_tex_coords {
            let attr_name = format!("a_imagery_texCoord_{i}");
            let varying_name = format!("v_imagery_texCoord_{i}");
            shader_builder.add_attribute("vec2", &attr_name);
            shader_builder.add_varying("vec2", &varying_name, None);
        }

        let function_id = "initializeImageryAttributes";
        let signature = format!("void {function_id}()");
        shader_builder.add_function(function_id, &signature, ShaderDestination::VERTEX);

        let lines: Vec<String> = (0..num_tex_coords)
            .map(|i| format!("v_imagery_texCoord_{i} = a_imagery_texCoord_{i};"))
            .collect();
        let str_lines: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        shader_builder.add_function_lines(function_id, &str_lines);
    }

    /// Defines uniforms required for imagery draping in the fragment shader.
    pub fn define_uniforms(shader_builder: &mut ShaderBuilder, imagery_flags: &ImageryFlags) {
        shader_builder.add_uniform(
            "sampler2D",
            "u_imageryTextures[IMAGERY_TEXTURE_UNITS]",
            Some(ShaderDestination::FRAGMENT),
        );
        shader_builder.add_uniform(
            "vec4",
            "u_imageryTextureCoordinateRectangle[IMAGERY_TEXTURE_UNITS]",
            Some(ShaderDestination::FRAGMENT),
        );
        shader_builder.add_uniform(
            "vec4",
            "u_imageryTextureTranslationAndScale[IMAGERY_TEXTURE_UNITS]",
            Some(ShaderDestination::FRAGMENT),
        );

        if imagery_flags.alpha {
            shader_builder.add_uniform(
                "float",
                "u_imageryTextureAlpha[IMAGERY_TEXTURE_UNITS]",
                Some(ShaderDestination::FRAGMENT),
            );
        }
        if imagery_flags.brightness {
            shader_builder.add_uniform(
                "float",
                "u_imageryTextureBrightness[IMAGERY_TEXTURE_UNITS]",
                Some(ShaderDestination::FRAGMENT),
            );
        }
        if imagery_flags.contrast {
            shader_builder.add_uniform(
                "float",
                "u_imageryTextureContrast[IMAGERY_TEXTURE_UNITS]",
                Some(ShaderDestination::FRAGMENT),
            );
        }
        if imagery_flags.hue {
            shader_builder.add_uniform(
                "float",
                "u_imageryTextureHue[IMAGERY_TEXTURE_UNITS]",
                Some(ShaderDestination::FRAGMENT),
            );
        }
        if imagery_flags.saturation {
            shader_builder.add_uniform(
                "float",
                "u_imageryTextureSaturation[IMAGERY_TEXTURE_UNITS]",
                Some(ShaderDestination::FRAGMENT),
            );
        }
        if imagery_flags.gamma {
            shader_builder.add_uniform(
                "float",
                "u_imageryTextureOneOverGamma[IMAGERY_TEXTURE_UNITS]",
                Some(ShaderDestination::FRAGMENT),
            );
        }
        if imagery_flags.color_to_alpha {
            shader_builder.add_uniform(
                "vec4",
                "u_imageryTextureColorToAlpha[IMAGERY_TEXTURE_UNITS]",
                Some(ShaderDestination::FRAGMENT),
            );
        }
    }

    /// Creates the function signature for `sampleAndBlend`.
    pub fn create_sample_and_blend_function_signature(imagery_flags: &ImageryFlags) -> String {
        let function_id = "sampleAndBlend";
        let mut parameters = vec![
            "vec4 previousColor".to_string(),
            "sampler2D textureToSample".to_string(),
            "vec2 textureCoordinates".to_string(),
            "vec4 textureCoordinateRectangle".to_string(),
            "vec4 textureCoordinateTranslationAndScale".to_string(),
        ];

        if imagery_flags.alpha {
            parameters.push("float textureAlpha".to_string());
        }
        if imagery_flags.brightness {
            parameters.push("float textureBrightness".to_string());
        }
        if imagery_flags.contrast {
            parameters.push("float textureContrast".to_string());
        }
        if imagery_flags.hue {
            parameters.push("float textureHue".to_string());
        }
        if imagery_flags.saturation {
            parameters.push("float textureSaturation".to_string());
        }
        if imagery_flags.gamma {
            parameters.push("float textureOneOverGamma".to_string());
        }
        if imagery_flags.color_to_alpha {
            parameters.push("vec4 colorToAlpha".to_string());
        }

        let parameters_string = parameters.join(", ");
        format!("vec4 {function_id}({parameters_string})")
    }

    /// Builds the `sampleAndBlend` function in the fragment shader.
    pub fn build_sample_and_blend_function(
        shader_builder: &mut ShaderBuilder,
        imagery_flags: &ImageryFlags,
    ) {
        let function_id = "sampleAndBlend";
        let signature = Self::create_sample_and_blend_function_signature(imagery_flags);
        shader_builder.add_function(function_id, &signature, ShaderDestination::FRAGMENT);

        shader_builder.add_function_lines(function_id, &["float effectiveAlpha = 1.0;"]);
        if imagery_flags.alpha {
            shader_builder.add_function_lines(function_id, &["effectiveAlpha = textureAlpha;"]);
        }

        let mut lines = vec![
            "if (textureCoordinates.x < textureCoordinateRectangle.x) effectiveAlpha = 0.0;",
            "if (textureCoordinates.x > textureCoordinateRectangle.z) effectiveAlpha = 0.0;",
            "if (textureCoordinates.y < textureCoordinateRectangle.y) effectiveAlpha = 0.0;",
            "if (textureCoordinates.y > textureCoordinateRectangle.w) effectiveAlpha = 0.0;",
            "vec2 translation = textureCoordinateTranslationAndScale.xy;",
            "vec2 scale = textureCoordinateTranslationAndScale.zw;",
            "vec2 effectiveTextureCoordinates = textureCoordinates * scale + translation;",
            "vec4 value = texture(textureToSample, effectiveTextureCoordinates);",
            "value = czm_srgbToLinear(value);",
            "vec3 color = value.rgb;",
            "float alpha = value.a;",
        ];

        if imagery_flags.color_to_alpha {
            lines.extend(vec![
                "vec3 colorDiff = abs(color.rgb - colorToAlpha.rgb);",
                "colorDiff.r = czm_maximumComponent(colorDiff);",
                "alpha = czm_branchFreeTernary(colorDiff.r < colorToAlpha.a, 0.0, alpha);",
            ]);
        }

        if imagery_flags.gamma {
            lines.push("color = pow(color, vec3(textureOneOverGamma));");
        } else {
            lines.extend(vec![
                "vec4 tempColor = czm_gammaCorrect(vec4(color, alpha));",
                "color = tempColor.rgb;",
                "alpha = tempColor.a;",
            ]);
        }

        if imagery_flags.brightness {
            lines.push("color = mix(vec3(0.0), color, textureBrightness);");
        }
        if imagery_flags.contrast {
            lines.push("color = mix(vec3(0.5), color, textureContrast);");
        }
        if imagery_flags.hue {
            lines.push("color = czm_hue(color, textureHue);");
        }
        if imagery_flags.saturation {
            lines.push("color = czm_saturation(color, textureSaturation);");
        }

        lines.extend(vec![
            "float sourceAlpha = alpha * effectiveAlpha;",
            "float outAlpha = mix(previousColor.a, 1.0, sourceAlpha);",
            "outAlpha += sign(outAlpha) - 1.0;",
            "vec3 outColor = mix(previousColor.rgb * previousColor.a, color, sourceAlpha) / outAlpha;",
            "vec4 result = vec4(outColor, max(outAlpha, 0.0));",
            "return result;",
        ]);

        shader_builder.add_function_lines(function_id, &lines);
    }

    /// Creates call arguments for `sampleAndBlend` for the specified imagery input.
    pub fn create_sample_and_blend_call_arguments(
        imagery_flags: &ImageryFlags,
        imagery_tex_coord_attribute_set_index: usize,
        i: usize,
    ) -> String {
        let texture_coordinates = format!("v_imagery_texCoord_{imagery_tex_coord_attribute_set_index}");
        let mut args = vec![
            "blendedBaseColor".to_string(),
            format!("u_imageryTextures[{i}]"),
            texture_coordinates,
            format!("u_imageryTextureCoordinateRectangle[{i}]"),
            format!("u_imageryTextureTranslationAndScale[{i}]"),
        ];

        if imagery_flags.alpha {
            args.push(format!("u_imageryTextureAlpha[{i}]"));
        }
        if imagery_flags.brightness {
            args.push(format!("u_imageryTextureBrightness[{i}]"));
        }
        if imagery_flags.contrast {
            args.push(format!("u_imageryTextureContrast[{i}]"));
        }
        if imagery_flags.hue {
            args.push(format!("u_imageryTextureHue[{i}]"));
        }
        if imagery_flags.saturation {
            args.push(format!("u_imageryTextureSaturation[{i}]"));
        }
        if imagery_flags.gamma {
            args.push(format!("u_imageryTextureOneOverGamma[{i}]"));
        }
        if imagery_flags.color_to_alpha {
            args.push(format!("u_imageryTextureColorToAlpha[{i}]"));
        }

        args.join(", ")
    }

    /// Creates the main `blendBaseColorWithImagery` shader function.
    pub fn create_main_imagery_shader(
        shader_builder: &mut ShaderBuilder,
        imagery_inputs: &[ImageryInput],
        imagery_flags: &ImageryFlags,
    ) {
        let function_id = "blendBaseColorWithImagery";
        shader_builder.add_function(
            function_id,
            "vec4 blendBaseColorWithImagery(vec4 baseColorWithAlpha)",
            ShaderDestination::FRAGMENT,
        );
        shader_builder.add_function_lines(function_id, &["vec4 blendedBaseColor = baseColorWithAlpha;"]);

        for (i, imagery_input) in imagery_inputs.iter().enumerate() {
            let args_string = Self::create_sample_and_blend_call_arguments(
                imagery_flags,
                imagery_input.imagery_tex_coord_attribute_set_index,
                i,
            );
            let call_line = format!("blendedBaseColor = sampleAndBlend({args_string});");
            shader_builder.add_function_lines(function_id, &[&call_line]);
        }

        shader_builder.add_function_lines(function_id, &["return blendedBaseColor;"]);
    }

    /// Creates an `ImageryUniforms` struct holding the uniforms for all inputs.
    pub fn create_imagery_uniforms(imagery_inputs: &[ImageryInput]) -> ImageryUniforms {
        let length = imagery_inputs.len();
        let mut uniforms = ImageryUniforms {
            textures: Vec::with_capacity(length),
            coordinate_rectangles: Vec::with_capacity(length),
            translation_and_scales: Vec::with_capacity(length),
            alphas: Vec::with_capacity(length),
            brightnesses: Vec::with_capacity(length),
            contrasts: Vec::with_capacity(length),
            hues: Vec::with_capacity(length),
            saturations: Vec::with_capacity(length),
            one_over_gammas: Vec::with_capacity(length),
            color_to_alphas: Vec::with_capacity(length),
        };

        for input in imagery_inputs {
            let layer_guard = input.imagery_layer.read().unwrap();
            uniforms.textures.push(input.texture.clone());
            uniforms
                .translation_and_scales
                .push(input.texture_translation_and_scale);
            uniforms
                .coordinate_rectangles
                .push(input.texture_coordinate_rectangle);

            uniforms.alphas.push(layer_guard.alpha());
            uniforms.brightnesses.push(layer_guard.brightness());
            uniforms.contrasts.push(layer_guard.contrast());
            uniforms.hues.push(layer_guard.hue());
            uniforms.saturations.push(layer_guard.saturation());
            let gamma = layer_guard.gamma();
            let one_over_gamma = if gamma.abs() > 1e-6 { 1.0 / gamma } else { 1.0 };
            uniforms.one_over_gammas.push(one_over_gamma);

            if let Some(color) = layer_guard.color_to_alpha() {
                if layer_guard.color_to_alpha_threshold() > 0.0 {
                    uniforms.color_to_alphas.push(Cartesian4::new(
                        color.red as f64,
                        color.green as f64,
                        color.blue as f64,
                        layer_guard.color_to_alpha_threshold() as f64,
                    ));
                } else {
                    uniforms
                        .color_to_alphas
                        .push(Cartesian4::new(0.0, 0.0, 0.0, -1.0));
                }
            } else {
                uniforms
                    .color_to_alphas
                    .push(Cartesian4::new(0.0, 0.0, 0.0, -1.0));
            }
        }

        uniforms
    }

    /// Sets uniforms in the uniform map.
    pub fn set_imagery_uniforms(
        uniform_map: &mut HashMap<String, Box<dyn Fn() -> serde_json::Value + Send + Sync>>,
        uniforms: ImageryUniforms,
    ) {
        let u_textures = uniforms.textures.clone();
        uniform_map.insert(
            "u_imageryTextures".to_string(),
            Box::new(move || serde_json::to_value(&u_textures).unwrap_or_default()),
        );

        let u_rects = uniforms.coordinate_rectangles.clone();
        uniform_map.insert(
            "u_imageryTextureCoordinateRectangle".to_string(),
            Box::new(move || {
                let list: Vec<[f64; 4]> = u_rects
                    .iter()
                    .map(|r| [r.x, r.y, r.z, r.w])
                    .collect();
                serde_json::to_value(&list).unwrap_or_default()
            }),
        );

        let u_trans = uniforms.translation_and_scales.clone();
        uniform_map.insert(
            "u_imageryTextureTranslationAndScale".to_string(),
            Box::new(move || {
                let list: Vec<[f64; 4]> = u_trans
                    .iter()
                    .map(|t| [t.x, t.y, t.z, t.w])
                    .collect();
                serde_json::to_value(&list).unwrap_or_default()
            }),
        );

        let u_alpha = uniforms.alphas.clone();
        uniform_map.insert(
            "u_imageryTextureAlpha".to_string(),
            Box::new(move || serde_json::to_value(&u_alpha).unwrap_or_default()),
        );

        let u_bright = uniforms.brightnesses.clone();
        uniform_map.insert(
            "u_imageryTextureBrightness".to_string(),
            Box::new(move || serde_json::to_value(&u_bright).unwrap_or_default()),
        );

        let u_contrast = uniforms.contrasts.clone();
        uniform_map.insert(
            "u_imageryTextureContrast".to_string(),
            Box::new(move || serde_json::to_value(&u_contrast).unwrap_or_default()),
        );

        let u_hue = uniforms.hues.clone();
        uniform_map.insert(
            "u_imageryTextureHue".to_string(),
            Box::new(move || serde_json::to_value(&u_hue).unwrap_or_default()),
        );

        let u_sat = uniforms.saturations.clone();
        uniform_map.insert(
            "u_imageryTextureSaturation".to_string(),
            Box::new(move || serde_json::to_value(&u_sat).unwrap_or_default()),
        );

        let u_gamma = uniforms.one_over_gammas.clone();
        uniform_map.insert(
            "u_imageryTextureOneOverGamma".to_string(),
            Box::new(move || serde_json::to_value(&u_gamma).unwrap_or_default()),
        );

        let u_cta = uniforms.color_to_alphas.clone();
        uniform_map.insert(
            "u_imageryTextureColorToAlpha".to_string(),
            Box::new(move || {
                let list: Vec<[f64; 4]> = u_cta
                    .iter()
                    .map(|c| [c.x, c.y, c.z, c.w])
                    .collect();
                serde_json::to_value(&list).unwrap_or_default()
            }),
        );
    }

    /// Internal process function for imagery inputs.
    pub fn process_imagery_inputs(
        imagery_layers_array: &[ImageryLayer],
        render_resources: &mut ImageryRenderResources<'_>,
        imagery_inputs: &[ImageryInput],
        num_imagery_tex_coord_attributes: usize,
    ) {
        let shader_builder = &mut render_resources.shader_builder;
        let imagery_flags = Self::compute_imagery_flags(imagery_layers_array);
        let num_textures = imagery_inputs.len();

        shader_builder.add_define("HAS_IMAGERY", None, None);
        let num_tex_str = num_textures.to_string();
        shader_builder.add_define("IMAGERY_TEXTURE_UNITS", Some(&num_tex_str), None);

        Self::add_attributes(shader_builder, num_imagery_tex_coord_attributes);
        Self::define_uniforms(shader_builder, &imagery_flags);
        Self::build_sample_and_blend_function(shader_builder, &imagery_flags);
        Self::create_main_imagery_shader(shader_builder, imagery_inputs, &imagery_flags);

        let uniforms = Self::create_imagery_uniforms(imagery_inputs);
        Self::set_imagery_uniforms(render_resources.uniform_map, uniforms);
    }
}
