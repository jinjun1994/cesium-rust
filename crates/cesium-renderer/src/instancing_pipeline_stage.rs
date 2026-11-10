//! InstancingPipelineStage matching CesiumJS `Scene/Model/InstancingPipelineStage.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{
    AttributeCompression, AttributeType, Cartesian3, ComponentDatatype,
    Ellipsoid, FixedFrameTransforms, GeographicProjection, MapProjection, Matrix4, Quaternion,
    SceneMode,
};

use crate::buffer::{Buffer, BufferOptions, BufferUsage};
use crate::model_components::{Attribute, Instances, Node};
use crate::shader_builder::{ShaderBuilder, ShaderDestination};

pub const SHADERS_INSTANCING_STAGE_COMMON: &str = r#"
mat4 getInstancingTransform()
{
    mat4 instancingTransform;

    #ifdef HAS_INSTANCE_MATRICES
    instancingTransform = mat4(
        a_instancingTransformRow0.x, a_instancingTransformRow1.x, a_instancingTransformRow2.x, 0.0,
        a_instancingTransformRow0.y, a_instancingTransformRow1.y, a_instancingTransformRow2.y, 0.0,
        a_instancingTransformRow0.z, a_instancingTransformRow1.z, a_instancingTransformRow2.z, 0.0,
        a_instancingTransformRow0.w, a_instancingTransformRow1.w, a_instancingTransformRow2.w, 1.0
    );
    #else
    vec3 translation = vec3(0.0, 0.0, 0.0);
    vec3 scale = vec3(1.0, 1.0, 1.0);
    
        #ifdef HAS_INSTANCE_TRANSLATION
        translation = a_instanceTranslation;
        #endif
        #ifdef HAS_INSTANCE_SCALE
        scale = a_instanceScale;
        #endif

    instancingTransform = mat4(
        scale.x, 0.0, 0.0, 0.0,
        0.0, scale.y, 0.0, 0.0,
        0.0, 0.0, scale.z, 0.0,
        translation.x, translation.y, translation.z, 1.0
    ); 
    #endif

    return instancingTransform;
}

#ifdef USE_2D_INSTANCING
mat4 getInstancingTransform2D()
{
    mat4 instancingTransform2D;

    #ifdef HAS_INSTANCE_MATRICES
    instancingTransform2D = mat4(
        a_instancingTransform2DRow0.x, a_instancingTransform2DRow1.x, a_instancingTransform2DRow2.x, 0.0,
        a_instancingTransform2DRow0.y, a_instancingTransform2DRow1.y, a_instancingTransform2DRow2.y, 0.0,
        a_instancingTransform2DRow0.z, a_instancingTransform2DRow1.z, a_instancingTransform2DRow2.z, 0.0,
        a_instancingTransform2DRow0.w, a_instancingTransform2DRow1.w, a_instancingTransform2DRow2.w, 1.0
    );
    #else
    vec3 translation2D = vec3(0.0, 0.0, 0.0);
    vec3 scale = vec3(1.0, 1.0, 1.0);
    
        #ifdef HAS_INSTANCE_TRANSLATION
        translation2D = a_instanceTranslation2D;
        #endif
        #ifdef HAS_INSTANCE_SCALE
        scale = a_instanceScale;
        #endif

    instancingTransform2D = mat4(
        scale.x, 0.0, 0.0, 0.0,
        0.0, scale.y, 0.0, 0.0,
        0.0, 0.0, scale.z, 0.0,
        translation2D.x, translation2D.y, translation2D.z, 1.0
    ); 
    #endif

    return instancingTransform2D;
}
#endif
"#;

pub const SHADERS_INSTANCING_STAGE_VS: &str = r#"
void instancingStage(inout ProcessedAttributes attributes) 
{
    vec3 positionMC = attributes.positionMC;
    
    mat4 instancingTransform = getInstancingTransform();
    
    attributes.positionMC = (instancingTransform * vec4(positionMC, 1.0)).xyz;

    #ifdef HAS_NORMALS
    vec3 normalMC = attributes.normalMC;
    attributes.normalMC = (instancingTransform * vec4(normalMC, 0.0)).xyz;
    #endif

    #ifdef USE_2D_INSTANCING
    mat4 instancingTransform2D = getInstancingTransform2D();
    attributes.position2D = (instancingTransform2D * vec4(positionMC, 1.0)).xyz;
    #endif
}
"#;

pub const SHADERS_LEGACY_INSTANCING_STAGE_VS: &str = r#"
void legacyInstancingStage(
    inout ProcessedAttributes attributes,
    out mat4 instanceModelView,
    out mat3 instanceModelViewInverseTranspose)
{
    vec3 positionMC = attributes.positionMC;

    mat4 instancingTransform = getInstancingTransform();
 
    mat4 instanceModel = instancingTransform * u_instance_nodeTransform;
    instanceModelView = u_instance_modifiedModelView;
    instanceModelViewInverseTranspose = mat3(u_instance_modifiedModelView * instanceModel);

    attributes.positionMC = (instanceModel * vec4(positionMC, 1.0)).xyz;
    
    #ifdef USE_2D_INSTANCING
    mat4 instancingTransform2D = getInstancingTransform2D();
    attributes.position2D = (instancingTransform2D * vec4(positionMC, 1.0)).xyz;
    #endif
}
"#;

/// Vertex attribute generated for GPU mesh instancing.
#[derive(Clone, Debug, PartialEq)]
pub struct InstancingVertexAttribute {
    pub index: usize,
    pub vertex_buffer: Option<Buffer>,
    pub components_per_attribute: usize,
    pub component_datatype: ComponentDatatype,
    pub normalize: bool,
    pub offset_in_bytes: usize,
    pub stride_in_bytes: Option<usize>,
    pub instance_divisor: usize,
}

/// Runtime node containing computed instancing buffers and bounds.
#[derive(Clone, Debug, Default)]
pub struct InstancingRuntimeNode {
    pub instancing_translation_min: Option<Cartesian3>,
    pub instancing_translation_max: Option<Cartesian3>,
    pub instancing_transforms_buffer: Option<Buffer>,
    pub instancing_transforms_buffer_2d: Option<Buffer>,
    pub instancing_translation_buffer_2d: Option<Buffer>,
    pub instancing_reference_point_2d: Option<Cartesian3>,
    pub transforms_typed_array: Option<Vec<f32>>,
    pub computed_transform: Matrix4,
}

/// Model properties relevant to InstancingPipelineStage.
#[derive(Clone, Debug, Default)]
pub struct InstancingModel {
    pub model_matrix: Matrix4,
    pub project_to_2d: bool,
    pub enable_pick: bool,
    pub scene_graph: InstancingSceneGraph,
    pub model_resources: Vec<Buffer>,
    pub pipeline_resources: Vec<Buffer>,
}

/// Scene graph transforms for instancing.
#[derive(Clone, Debug, Default)]
pub struct InstancingSceneGraph {
    pub computed_model_matrix: Matrix4,
    pub axis_correction_matrix: Matrix4,
    pub components_transform: Matrix4,
}

/// Frame state context for instancing.
#[derive(Clone, Debug)]
pub struct InstancingFrameState {
    pub mode: SceneMode,
    pub scene_3d_only: bool,
    pub webgl2: bool,
    pub view: Matrix4,
    pub view_3d: Matrix4,
    pub map_projection: GeographicProjection,
}

impl Default for InstancingFrameState {
    fn default() -> Self {
        Self {
            mode: SceneMode::Scene3D,
            scene_3d_only: false,
            webgl2: true,
            view: Matrix4::IDENTITY,
            view_3d: Matrix4::IDENTITY,
            map_projection: GeographicProjection::new(Ellipsoid::default()).unwrap_or_default(),
        }
    }
}

/// Render resources passed to InstancingPipelineStage.
pub struct InstancingRenderResources<'a> {
    pub attribute_index: &'a mut usize,
    pub attributes: &'a mut Vec<InstancingVertexAttribute>,
    pub shader_builder: &'a mut ShaderBuilder,
    pub uniform_map: &'a mut HashMap<String, Box<dyn Fn() -> serde_json::Value + Send + Sync>>,
    pub model: &'a mut InstancingModel,
    pub runtime_node: &'a mut InstancingRuntimeNode,
    pub instance_count: usize,
    pub feature_id_vertex_attribute_set_index: usize,
}

/// Pipeline stage that handles GPU mesh instancing at the node level.
pub struct InstancingPipelineStage;

impl InstancingPipelineStage {
    pub const NAME: &'static str = "InstancingPipelineStage";

    /// Transforms an array of Matrix4 to a Float32 column/row packed array.
    /// Each matrix writes 12 floats: row 0, row 1, row 2 across all 4 columns.
    pub fn transforms_to_typed_array(transforms: &[Matrix4]) -> Vec<f32> {
        let elements = 12;
        let count = transforms.len();
        let mut result = vec![0.0_f32; count * elements];

        for (i, transform) in transforms.iter().enumerate() {
            let offset = elements * i;
            let m = transform.values;

            result[offset] = m[0] as f32;
            result[offset + 1] = m[4] as f32;
            result[offset + 2] = m[8] as f32;
            result[offset + 3] = m[12] as f32;

            result[offset + 4] = m[1] as f32;
            result[offset + 5] = m[5] as f32;
            result[offset + 6] = m[9] as f32;
            result[offset + 7] = m[13] as f32;

            result[offset + 8] = m[2] as f32;
            result[offset + 9] = m[6] as f32;
            result[offset + 10] = m[10] as f32;
            result[offset + 11] = m[14] as f32;
        }

        result
    }

    /// Transforms an array of Cartesian3 translations to a Float32 array (3 floats per instance).
    pub fn translations_to_typed_array(translations: &[Cartesian3]) -> Vec<f32> {
        let mut result = Vec::with_capacity(translations.len() * 3);
        for t in translations {
            result.push(t.x as f32);
            result.push(t.y as f32);
            result.push(t.z as f32);
        }
        result
    }

    /// Computes instance transforms as matrices, updates translation min/max bounds,
    /// and clears typed arrays on the instances.
    pub fn get_instance_transforms_as_matrices(
        instances: &mut Instances,
        count: usize,
        runtime_node: &mut InstancingRuntimeNode,
    ) -> Vec<Matrix4> {
        let mut transforms = Vec::with_capacity(count);

        let mut translation_attr = None;
        let mut rotation_attr = None;
        let mut scale_attr = None;

        for attr in &mut instances.attributes {
            if let Some(sem) = &attr.semantic {
                match sem.as_str() {
                    "TRANSLATION" => translation_attr = Some(attr),
                    "ROTATION" => rotation_attr = Some(attr),
                    "SCALE" => scale_attr = Some(attr),
                    _ => {}
                }
            }
        }

        let mut instancing_translation_max = Cartesian3::new(f64::MIN, f64::MIN, f64::MIN);
        let mut instancing_translation_min = Cartesian3::new(f64::MAX, f64::MAX, f64::MAX);

        let translation_floats = if let Some(ref attr) = translation_attr {
            if let Some(ref bytes) = attr.typed_array {
                bytes_to_f32_vec(bytes)
            } else {
                vec![0.0_f32; count * 3]
            }
        } else {
            vec![0.0_f32; count * 3]
        };

        let rotation_floats = if let Some(ref attr) = rotation_attr {
            if let Some(ref bytes) = attr.typed_array {
                if attr.normalized {
                    let datatype = attr.component_datatype.unwrap_or(ComponentDatatype::Short);
                    let raw_f64 = bytes_to_f64_slice(bytes, datatype);
                    AttributeCompression::dequantize_f64(
                        &raw_f64,
                        datatype,
                        AttributeType::Vec4,
                        count,
                    )
                    .unwrap_or_else(|_| vec![0.0_f32; count * 4])
                } else {
                    bytes_to_f32_vec(bytes)
                }
            } else {
                vec![0.0_f32; count * 4]
            }
        } else {
            vec![0.0_f32; count * 4]
        };

        let scale_floats = if let Some(ref attr) = scale_attr {
            if let Some(ref bytes) = attr.typed_array {
                bytes_to_f32_vec(bytes)
            } else {
                vec![1.0_f32; count * 3]
            }
        } else {
            vec![1.0_f32; count * 3]
        };

        let has_rotation = rotation_attr.is_some();

        for i in 0..count {
            let tx = translation_floats.get(i * 3).copied().unwrap_or(0.0) as f64;
            let ty = translation_floats.get(i * 3 + 1).copied().unwrap_or(0.0) as f64;
            let tz = translation_floats.get(i * 3 + 2).copied().unwrap_or(0.0) as f64;
            let translation = Cartesian3::new(tx, ty, tz);

            instancing_translation_min = Cartesian3::new(
                instancing_translation_min.x.min(translation.x),
                instancing_translation_min.y.min(translation.y),
                instancing_translation_min.z.min(translation.z),
            );
            instancing_translation_max = Cartesian3::new(
                instancing_translation_max.x.max(translation.x),
                instancing_translation_max.y.max(translation.y),
                instancing_translation_max.z.max(translation.z),
            );

            let rx = rotation_floats.get(i * 4).copied().unwrap_or(0.0) as f64;
            let ry = rotation_floats.get(i * 4 + 1).copied().unwrap_or(0.0) as f64;
            let rz = rotation_floats.get(i * 4 + 2).copied().unwrap_or(0.0) as f64;
            let rw = if has_rotation {
                rotation_floats.get(i * 4 + 3).copied().unwrap_or(1.0) as f64
            } else {
                1.0
            };
            let rotation = Quaternion::new(rx, ry, rz, rw);

            let sx = scale_floats.get(i * 3).copied().unwrap_or(1.0) as f64;
            let sy = scale_floats.get(i * 3 + 1).copied().unwrap_or(1.0) as f64;
            let sz = scale_floats.get(i * 3 + 2).copied().unwrap_or(1.0) as f64;
            let scale = Cartesian3::new(sx, sy, sz);

            let transform =
                Matrix4::from_translation_quaternion_rotation_scale(translation, rotation, scale);
            transforms.push(transform);
        }

        runtime_node.instancing_translation_min = Some(instancing_translation_min);
        runtime_node.instancing_translation_max = Some(instancing_translation_max);

        // Unload typed arrays
        for attr in &mut instances.attributes {
            if let Some(sem) = &attr.semantic {
                if sem == "TRANSLATION" || sem == "ROTATION" || sem == "SCALE" {
                    attr.typed_array = None;
                }
            }
        }

        transforms
    }

    /// Computes Cartesian3 translations from translation attribute, sets min/max bounds,
    /// and unloads the typed array.
    pub fn get_instance_translations_as_cartesian3s(
        translation_attr: &mut Attribute,
        count: usize,
        runtime_node: &mut InstancingRuntimeNode,
    ) -> Vec<Cartesian3> {
        let mut translations = Vec::with_capacity(count);
        let floats = if let Some(ref bytes) = translation_attr.typed_array {
            bytes_to_f32_vec(bytes)
        } else {
            vec![0.0_f32; count * 3]
        };

        let mut instancing_translation_min = Cartesian3::new(f64::MAX, f64::MAX, f64::MAX);
        let mut instancing_translation_max = Cartesian3::new(f64::MIN, f64::MIN, f64::MIN);

        for i in 0..count {
            let tx = floats.get(i * 3).copied().unwrap_or(0.0) as f64;
            let ty = floats.get(i * 3 + 1).copied().unwrap_or(0.0) as f64;
            let tz = floats.get(i * 3 + 2).copied().unwrap_or(0.0) as f64;
            let translation = Cartesian3::new(tx, ty, tz);

            instancing_translation_min = Cartesian3::new(
                instancing_translation_min.x.min(translation.x),
                instancing_translation_min.y.min(translation.y),
                instancing_translation_min.z.min(translation.z),
            );
            instancing_translation_max = Cartesian3::new(
                instancing_translation_max.x.max(translation.x),
                instancing_translation_max.y.max(translation.y),
                instancing_translation_max.z.max(translation.z),
            );

            translations.push(translation);
        }

        runtime_node.instancing_translation_min = Some(instancing_translation_min);
        runtime_node.instancing_translation_max = Some(instancing_translation_max);
        translation_attr.typed_array = None;

        translations
    }

    /// Process a node's instancing attributes.
    pub fn process(
        render_resources: &mut InstancingRenderResources<'_>,
        node: &mut Node,
        frame_state: &InstancingFrameState,
    ) {
        let instances = match &mut node.instances {
            Some(inst) if !inst.attributes.is_empty() => inst,
            _ => return,
        };

        let count = instances.attributes[0].count;
        let transform_in_world_space = instances.transform_in_world_space;

        render_resources.shader_builder.add_define("HAS_INSTANCING", None, None);
        render_resources
            .shader_builder
            .add_vertex_lines(&[SHADERS_INSTANCING_STAGE_COMMON]);

        let use_2d = frame_state.mode != SceneMode::Scene3D
            && !frame_state.scene_3d_only
            && render_resources.model.project_to_2d;
        let keep_typed_array = render_resources.model.enable_pick && !frame_state.webgl2;

        let mut instancing_vertex_attributes = Vec::new();

        Self::process_transform_attributes(
            render_resources,
            instances,
            &mut instancing_vertex_attributes,
            frame_state,
            use_2d,
            keep_typed_array,
        );

        Self::process_feature_id_attributes(
            render_resources,
            instances,
            &mut instancing_vertex_attributes,
        );

        if transform_in_world_space {
            render_resources.shader_builder.add_define(
                "USE_LEGACY_INSTANCING",
                None,
                Some(ShaderDestination::VERTEX),
            );
            render_resources.shader_builder.add_uniform(
                "mat4",
                "u_instance_modifiedModelView",
                Some(ShaderDestination::VERTEX),
            );
            render_resources.shader_builder.add_uniform(
                "mat4",
                "u_instance_nodeTransform",
                Some(ShaderDestination::VERTEX),
            );

            let model_matrix = render_resources.model.model_matrix;
            let components_transform = render_resources.model.scene_graph.components_transform;
            let view = if use_2d {
                frame_state.view_3d
            } else {
                frame_state.view
            };

            let mut modified_model_matrix = model_matrix.multiply(&components_transform);
            if !use_2d && frame_state.mode != SceneMode::Scene3D {
                modified_model_matrix = FixedFrameTransforms::basis_to_2d(
                    &frame_state.map_projection,
                    &modified_model_matrix,
                )
                .unwrap_or(modified_model_matrix);
            }
            let modified_model_view = view.multiply(&modified_model_matrix);
            let axis_correction = render_resources.model.scene_graph.axis_correction_matrix;
            let computed_transform = render_resources.runtime_node.computed_transform;
            let node_transform = axis_correction.multiply(&computed_transform);

            render_resources.uniform_map.insert(
                "u_instance_modifiedModelView".to_string(),
                Box::new(move || {
                    let vals = modified_model_view.values;
                    serde_json::to_value(vals).unwrap_or_default()
                }),
            );

            render_resources.uniform_map.insert(
                "u_instance_nodeTransform".to_string(),
                Box::new(move || {
                    let vals = node_transform.values;
                    serde_json::to_value(vals).unwrap_or_default()
                }),
            );

            render_resources
                .shader_builder
                .add_vertex_lines(&[SHADERS_LEGACY_INSTANCING_STAGE_VS]);
        } else {
            render_resources
                .shader_builder
                .add_vertex_lines(&[SHADERS_INSTANCING_STAGE_VS]);
        }

        if use_2d {
            render_resources.shader_builder.add_define(
                "USE_2D_INSTANCING",
                None,
                Some(ShaderDestination::VERTEX),
            );
            render_resources.shader_builder.add_uniform(
                "mat4",
                "u_modelView2D",
                Some(ShaderDestination::VERTEX),
            );

            let ref_point = render_resources
                .runtime_node
                .instancing_reference_point_2d
                .unwrap_or_default();
            let model_matrix_2d = Matrix4::from_translation(ref_point);
            let model_view_2d = frame_state.view.multiply(&model_matrix_2d);

            render_resources.uniform_map.insert(
                "u_modelView2D".to_string(),
                Box::new(move || {
                    let vals = model_view_2d.values;
                    serde_json::to_value(vals).unwrap_or_default()
                }),
            );
        }

        render_resources.instance_count = count;
        render_resources
            .attributes
            .extend(instancing_vertex_attributes);
    }

    fn process_transform_attributes(
        render_resources: &mut InstancingRenderResources<'_>,
        instances: &mut Instances,
        instancing_vertex_attributes: &mut Vec<InstancingVertexAttribute>,
        frame_state: &InstancingFrameState,
        use_2d: bool,
        keep_typed_array: bool,
    ) {
        let has_rotation = instances
            .attributes
            .iter()
            .any(|a| a.semantic.as_deref() == Some("ROTATION"));

        if has_rotation || keep_typed_array {
            Self::process_transform_matrix_attributes(
                render_resources,
                instances,
                instancing_vertex_attributes,
                frame_state,
                use_2d,
                keep_typed_array,
            );
        } else {
            Self::process_transform_vec3_attributes(
                render_resources,
                instances,
                instancing_vertex_attributes,
                frame_state,
                use_2d,
                keep_typed_array,
            );
        }
    }

    fn process_transform_matrix_attributes(
        render_resources: &mut InstancingRenderResources<'_>,
        instances: &mut Instances,
        instancing_vertex_attributes: &mut Vec<InstancingVertexAttribute>,
        frame_state: &InstancingFrameState,
        use_2d: bool,
        keep_typed_array: bool,
    ) {
        render_resources.shader_builder.add_define("HAS_INSTANCE_MATRICES", None, None);
        let count = instances.attributes[0].count;

        let (buffer, transforms) = if let Some(buf) =
            &render_resources.runtime_node.instancing_transforms_buffer
        {
            (buf.clone(), Vec::new())
        } else {
            let transforms = Self::get_instance_transforms_as_matrices(
                instances,
                count,
                render_resources.runtime_node,
            );

            let transforms_typed_array = Self::transforms_to_typed_array(&transforms);
            let bytes: Vec<u8> = transforms_typed_array
                .iter()
                .flat_map(|f| f.to_ne_bytes())
                .collect();

            let buffer = Buffer::create_vertex_buffer(BufferOptions {
                usage: Some(BufferUsage::StaticDraw),
                typed_array: Some(bytes),
                size_in_bytes: 0,
            })
            .unwrap();

            render_resources.model.model_resources.push(buffer.clone());
            if keep_typed_array {
                render_resources.runtime_node.transforms_typed_array = Some(transforms_typed_array);
            }
            render_resources.runtime_node.instancing_transforms_buffer = Some(buffer.clone());
            (buffer, transforms)
        };

        Self::process_matrix_attributes(
            render_resources,
            buffer,
            instancing_vertex_attributes,
            "Transform",
        );

        if !use_2d {
            return;
        }

        Self::compute_reference_point_2d(render_resources, frame_state);

        let buffer_2d = if let Some(buf_2d) =
            &render_resources.runtime_node.instancing_transforms_buffer_2d
        {
            buf_2d.clone()
        } else {
            let projected_transforms =
                Self::project_transforms_to_2d(&transforms, render_resources, frame_state);
            let projected_typed_array = Self::transforms_to_typed_array(&projected_transforms);
            let bytes_2d: Vec<u8> = projected_typed_array
                .iter()
                .flat_map(|f| f.to_ne_bytes())
                .collect();

            let buffer_2d = Buffer::create_vertex_buffer(BufferOptions {
                usage: Some(BufferUsage::StaticDraw),
                typed_array: Some(bytes_2d),
                size_in_bytes: 0,
            })
            .unwrap();

            render_resources.model.model_resources.push(buffer_2d.clone());
            render_resources.runtime_node.instancing_transforms_buffer_2d = Some(buffer_2d.clone());
            buffer_2d
        };

        Self::process_matrix_attributes(
            render_resources,
            buffer_2d,
            instancing_vertex_attributes,
            "Transform2D",
        );
    }

    fn process_transform_vec3_attributes(
        render_resources: &mut InstancingRenderResources<'_>,
        instances: &mut Instances,
        instancing_vertex_attributes: &mut Vec<InstancingVertexAttribute>,
        frame_state: &InstancingFrameState,
        use_2d: bool,
        keep_typed_array: bool,
    ) {
        let mut scale_attr_idx = None;
        let mut translation_attr_idx = None;

        for (i, attr) in instances.attributes.iter().enumerate() {
            if let Some(sem) = &attr.semantic {
                if sem == "SCALE" {
                    scale_attr_idx = Some(i);
                } else if sem == "TRANSLATION" {
                    translation_attr_idx = Some(i);
                }
            }
        }

        if let Some(idx) = scale_attr_idx {
            render_resources.shader_builder.add_define("HAS_INSTANCE_SCALE", None, None);
            let attr = &instances.attributes[idx];
            Self::process_vec3_attribute(
                render_resources,
                attr.buffer.clone(),
                attr.byte_offset,
                attr.byte_stride,
                instancing_vertex_attributes,
                "Scale",
            );
        }

        let trans_idx = match translation_attr_idx {
            Some(idx) => idx,
            None => return,
        };

        let mut instancing_translations = None;
        let attr = &mut instances.attributes[trans_idx];
        if attr.typed_array.is_some() {
            instancing_translations = Some(Self::get_instance_translations_as_cartesian3s(
                attr,
                attr.count,
                render_resources.runtime_node,
            ));
        } else if render_resources
            .runtime_node
            .instancing_translation_min
            .is_none()
        {
            if let (Some(min), Some(max)) = (&attr.min, &attr.max) {
                if min.len() >= 3 && max.len() >= 3 {
                    render_resources.runtime_node.instancing_translation_min =
                        Some(Cartesian3::new(min[0], min[1], min[2]));
                    render_resources.runtime_node.instancing_translation_max =
                        Some(Cartesian3::new(max[0], max[1], max[2]));
                }
            }
        }

        render_resources
            .shader_builder
            .add_define("HAS_INSTANCE_TRANSLATION", None, None);
        Self::process_vec3_attribute(
            render_resources,
            attr.buffer.clone(),
            attr.byte_offset,
            attr.byte_stride,
            instancing_vertex_attributes,
            "Translation",
        );

        if !use_2d && !keep_typed_array {
            return;
        }

        Self::compute_reference_point_2d(render_resources, frame_state);

        let translations = instancing_translations.unwrap_or_default();
        let projected_translations =
            Self::project_translations_to_2d(&translations, render_resources, frame_state);
        let projected_typed_array = Self::translations_to_typed_array(&projected_translations);
        let bytes_2d: Vec<u8> = projected_typed_array
            .iter()
            .flat_map(|f| f.to_ne_bytes())
            .collect();

        if keep_typed_array {
            render_resources.runtime_node.transforms_typed_array = Some(projected_typed_array);
        }

        let buffer_2d = Buffer::create_vertex_buffer(BufferOptions {
            usage: Some(BufferUsage::StaticDraw),
            typed_array: Some(bytes_2d),
            size_in_bytes: 0,
        })
        .unwrap();

        render_resources.model.model_resources.push(buffer_2d.clone());
        render_resources.runtime_node.instancing_translation_buffer_2d = Some(buffer_2d.clone());

        if use_2d {
            Self::process_vec3_attribute(
                render_resources,
                Some(buffer_2d),
                0,
                None,
                instancing_vertex_attributes,
                "Translation2D",
            );
        }
    }

    fn process_matrix_attributes(
        render_resources: &mut InstancingRenderResources<'_>,
        buffer: Buffer,
        instancing_vertex_attributes: &mut Vec<InstancingVertexAttribute>,
        attribute_string: &str,
    ) {
        let vertex_size_in_floats = 12;
        let component_byte_size = 4; // float = 4 bytes
        let stride_in_bytes = component_byte_size * vertex_size_in_floats;

        for row in 0..3 {
            let offset = component_byte_size * 4 * row;
            let attr = InstancingVertexAttribute {
                index: *render_resources.attribute_index,
                vertex_buffer: Some(buffer.clone()),
                components_per_attribute: 4,
                component_datatype: ComponentDatatype::Float,
                normalize: false,
                offset_in_bytes: offset,
                stride_in_bytes: Some(stride_in_bytes),
                instance_divisor: 1,
            };
            *render_resources.attribute_index += 1;
            instancing_vertex_attributes.push(attr);

            let row_name = format!("a_instancing{attribute_string}Row{row}");
            render_resources.shader_builder.add_attribute("vec4", &row_name);
        }
    }

    fn process_vec3_attribute(
        render_resources: &mut InstancingRenderResources<'_>,
        buffer: Option<Buffer>,
        byte_offset: usize,
        byte_stride: Option<usize>,
        instancing_vertex_attributes: &mut Vec<InstancingVertexAttribute>,
        attribute_string: &str,
    ) {
        let attr = InstancingVertexAttribute {
            index: *render_resources.attribute_index,
            vertex_buffer: buffer,
            components_per_attribute: 3,
            component_datatype: ComponentDatatype::Float,
            normalize: false,
            offset_in_bytes: byte_offset,
            stride_in_bytes: byte_stride,
            instance_divisor: 1,
        };
        *render_resources.attribute_index += 1;
        instancing_vertex_attributes.push(attr);

        let name = format!("a_instance{attribute_string}");
        render_resources.shader_builder.add_attribute("vec3", &name);
    }

    fn process_feature_id_attributes(
        render_resources: &mut InstancingRenderResources<'_>,
        instances: &Instances,
        instancing_vertex_attributes: &mut Vec<InstancingVertexAttribute>,
    ) {
        for attr in &instances.attributes {
            if attr.semantic.as_deref() != Some("_FEATURE_ID") {
                continue;
            }

            let set_index = attr.set_index.unwrap_or(0);
            if set_index >= render_resources.feature_id_vertex_attribute_set_index {
                render_resources.feature_id_vertex_attribute_set_index = set_index + 1;
            }

            let components = attr
                .attribute_type
                .map(|t| t.get_number_of_components())
                .unwrap_or(1);
            let datatype = attr.component_datatype.unwrap_or(ComponentDatatype::Float);

            instancing_vertex_attributes.push(InstancingVertexAttribute {
                index: *render_resources.attribute_index,
                vertex_buffer: attr.buffer.clone(),
                components_per_attribute: components,
                component_datatype: datatype,
                normalize: false,
                offset_in_bytes: attr.byte_offset,
                stride_in_bytes: attr.byte_stride,
                instance_divisor: 1,
            });
            *render_resources.attribute_index += 1;

            let attr_name = format!("a_instanceFeatureId_{set_index}");
            render_resources.shader_builder.add_attribute("float", &attr_name);
        }
    }

    fn compute_reference_point_2d(
        render_resources: &mut InstancingRenderResources<'_>,
        frame_state: &InstancingFrameState,
    ) {
        let runtime_node = &mut *render_resources.runtime_node;
        let min_pos = runtime_node
            .instancing_translation_min
            .unwrap_or(Cartesian3::ZERO);
        let max_pos = runtime_node
            .instancing_translation_max
            .unwrap_or(Cartesian3::ZERO);

        let model_matrix = render_resources.model.scene_graph.computed_model_matrix;
        let trans_min = model_matrix.multiply_by_point(min_pos);
        let trans_max = model_matrix.multiply_by_point(max_pos);

        let proj_min = compute_actual_ellipsoid_position(frame_state, trans_min);
        let proj_max = compute_actual_ellipsoid_position(frame_state, trans_max);

        let ref_pt = Cartesian3::new(
            (proj_min.x + proj_max.x) * 0.5,
            (proj_min.y + proj_max.y) * 0.5,
            (proj_min.z + proj_max.z) * 0.5,
        );
        runtime_node.instancing_reference_point_2d = Some(ref_pt);
    }

    fn project_transforms_to_2d(
        transforms: &[Matrix4],
        render_resources: &InstancingRenderResources<'_>,
        frame_state: &InstancingFrameState,
    ) -> Vec<Matrix4> {
        let ref_pt = render_resources
            .runtime_node
            .instancing_reference_point_2d
            .unwrap_or(Cartesian3::ZERO);

        transforms
            .iter()
            .map(|t| {
                let mut projected = FixedFrameTransforms::basis_to_2d(&frame_state.map_projection, t)
                    .unwrap_or(*t);
                let pos = projected.get_translation();
                let final_trans = Cartesian3::new(pos.x - ref_pt.x, pos.y - ref_pt.y, pos.z - ref_pt.z);
                projected.set_translation(final_trans);
                projected
            })
            .collect()
    }

    fn project_translations_to_2d(
        translations: &[Cartesian3],
        render_resources: &InstancingRenderResources<'_>,
        frame_state: &InstancingFrameState,
    ) -> Vec<Cartesian3> {
        let ref_pt = render_resources
            .runtime_node
            .instancing_reference_point_2d
            .unwrap_or(Cartesian3::ZERO);

        translations
            .iter()
            .map(|t| {
                let proj = compute_actual_ellipsoid_position(frame_state, *t);
                Cartesian3::new(proj.x - ref_pt.x, proj.y - ref_pt.y, proj.z - ref_pt.z)
            })
            .collect()
    }
}

fn compute_actual_ellipsoid_position(
    frame_state: &InstancingFrameState,
    position: Cartesian3,
) -> Cartesian3 {
    let mode = frame_state.mode;
    if mode == SceneMode::Scene3D {
        return position;
    }

    let projection = &frame_state.map_projection;
    let dvec = glam::DVec3::new(position.x, position.y, position.z);
    let carto = match projection.ellipsoid().cartesian_to_cartographic(dvec) {
        Some(c) => c,
        None => return position,
    };
    let projected = match projection.project(carto) {
        Some(p) => p,
        None => return position,
    };

    match mode {
        SceneMode::ColumbusView => Cartesian3::new(projected.z, projected.x, projected.y),
        SceneMode::Scene2D => Cartesian3::new(0.0, projected.x, projected.y),
        _ => position,
    }
}

fn bytes_to_f32_vec(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_ne_bytes(*chunk))
        .collect()
}

fn bytes_to_f64_slice(bytes: &[u8], datatype: ComponentDatatype) -> Vec<f64> {
    match datatype {
        ComponentDatatype::Byte => bytes.iter().map(|&b| (b as i8) as f64).collect(),
        ComponentDatatype::UnsignedByte => bytes.iter().map(|&b| b as f64).collect(),
        ComponentDatatype::Short => bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| i16::from_ne_bytes(*c) as f64)
            .collect(),
        ComponentDatatype::UnsignedShort => bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_ne_bytes(*c) as f64)
            .collect(),
        ComponentDatatype::Int => bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| i32::from_ne_bytes(*c) as f64)
            .collect(),
        ComponentDatatype::UnsignedInt => bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| u32::from_ne_bytes(*c) as f64)
            .collect(),
        ComponentDatatype::Float => bytes_to_f32_vec(bytes).iter().map(|&f| f as f64).collect(),
        ComponentDatatype::Double => bytes
            .as_chunks::<8>()
            .0
            .iter()
            .map(|c| f64::from_ne_bytes(*c))
            .collect(),
    }
}
