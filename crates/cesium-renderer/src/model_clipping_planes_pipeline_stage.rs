//! ModelClippingPlanesPipelineStage matching CesiumJS `Scene/Model/ModelClippingPlanesPipelineStage.js` (Apache-2.0).

use std::sync::Arc;

use cesium_math::{Color, Matrix4};
use crate::clipping_plane_collection::ClippingPlaneCollection;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};

pub const SHADERS_MODEL_CLIPPING_PLANES_STAGE_FS: &str = r#"
#ifdef USE_CLIPPING_PLANES_FLOAT_TEXTURE
vec4 getClippingPlane(
    highp sampler2D packedClippingPlanes,
    int clippingPlaneNumber,
    mat4 transform
) {
    int pixY = clippingPlaneNumber / CLIPPING_PLANES_TEXTURE_WIDTH;
    int pixX = clippingPlaneNumber - (pixY * CLIPPING_PLANES_TEXTURE_WIDTH);
    float pixelWidth = 1.0 / float(CLIPPING_PLANES_TEXTURE_WIDTH);
    float pixelHeight = 1.0 / float(CLIPPING_PLANES_TEXTURE_HEIGHT);
    float u = (float(pixX) + 0.5) * pixelWidth;
    float v = (float(pixY) + 0.5) * pixelHeight;
    vec4 plane = texture(packedClippingPlanes, vec2(u, v));
    return czm_transformPlane(plane, transform);
}
#else
vec4 getClippingPlane(
    highp sampler2D packedClippingPlanes,
    int clippingPlaneNumber,
    mat4 transform
) {
    int clippingPlaneStartIndex = clippingPlaneNumber * 2;
    int pixY = clippingPlaneStartIndex / CLIPPING_PLANES_TEXTURE_WIDTH;
    int pixX = clippingPlaneStartIndex - (pixY * CLIPPING_PLANES_TEXTURE_WIDTH);
    float pixelWidth = 1.0 / float(CLIPPING_PLANES_TEXTURE_WIDTH);
    float pixelHeight = 1.0 / float(CLIPPING_PLANES_TEXTURE_HEIGHT);
    float u = (float(pixX) + 0.5) * pixelWidth;
    float v = (float(pixY) + 0.5) * pixelHeight;
    vec4 oct32 = texture(packedClippingPlanes, vec2(u, v)) * 255.0;
    vec2 oct = vec2(oct32.x * 256.0 + oct32.y, oct32.z * 256.0 + oct32.w);
    vec4 plane;
    plane.xyz = czm_octDecode(oct, 65535.0);
    plane.w = czm_unpackFloat(texture(packedClippingPlanes, vec2(u + pixelWidth, v)));
    return czm_transformPlane(plane, transform);
}
#endif

float clip(vec4 fragCoord, sampler2D clippingPlanes, mat4 clippingPlanesMatrix) {
    vec4 position = czm_windowToEyeCoordinates(fragCoord);
    vec3 clipNormal = vec3(0.0);
    vec3 clipPosition = vec3(0.0);
    float pixelWidth = czm_metersPerPixel(position);
    
    #ifdef UNION_CLIPPING_REGIONS
    float clipAmount;
    #else
    float clipAmount = 0.0;
    bool clipped = true;
    #endif

    for (int i = 0; i < CLIPPING_PLANES_LENGTH; ++i) {
        vec4 clippingPlane = getClippingPlane(clippingPlanes, i, clippingPlanesMatrix);
        clipNormal = clippingPlane.xyz;
        clipPosition = -clippingPlane.w * clipNormal;
        float amount = dot(clipNormal, (position.xyz - clipPosition)) / pixelWidth;
        
        #ifdef UNION_CLIPPING_REGIONS
        clipAmount = czm_branchFreeTernary(i == 0, amount, min(amount, clipAmount));
        if (amount <= 0.0) {
            discard;
        }
        #else
        clipAmount = max(amount, clipAmount);
        clipped = clipped && (amount <= 0.0);
        #endif
    }

    #ifndef UNION_CLIPPING_REGIONS
    if (clipped) {
        discard;
    }
    #endif
    
    return clipAmount;
}

void modelClippingPlanesStage(inout vec4 color)
{
    float clipDistance = clip(gl_FragCoord, model_clippingPlanes, model_clippingPlanesMatrix);
    vec4 clippingPlanesEdgeColor = vec4(1.0);
    clippingPlanesEdgeColor.rgb = model_clippingPlanesEdgeStyle.rgb;
    float clippingPlanesEdgeWidth = model_clippingPlanesEdgeStyle.a;
    
    if (clipDistance > 0.0 && clipDistance < clippingPlanesEdgeWidth) {
        color = clippingPlanesEdgeColor;
    }
}
"#;

/// Model descriptor for clipping planes processing.
#[derive(Clone, Debug)]
pub struct ModelClippingPlanesModel {
    pub clipping_planes: ClippingPlaneCollection,
    pub model_matrix: Matrix4,
    pub clipping_planes_matrix: Matrix4,
}

impl ModelClippingPlanesModel {
    pub fn new(
        clipping_planes: ClippingPlaneCollection,
        model_matrix: Matrix4,
        clipping_planes_matrix: Matrix4,
    ) -> Self {
        Self {
            clipping_planes,
            model_matrix,
            clipping_planes_matrix,
        }
    }
}

/// Uniform state for camera inverse view.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelClippingPlanesUniformState {
    pub inverse_view_3d: Matrix4,
}

impl Default for ModelClippingPlanesUniformState {
    fn default() -> Self {
        Self {
            inverse_view_3d: Matrix4::IDENTITY,
        }
    }
}

/// Context descriptor for clipping planes processing.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ModelClippingPlanesContext {
    pub floating_point_texture: bool,
    pub uniform_state: ModelClippingPlanesUniformState,
}

/// Frame state containing context for clipping planes processing.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ModelClippingPlanesFrameState {
    pub context: ModelClippingPlanesContext,
}

pub type TextureSamplerGetter = Arc<dyn Fn() -> Option<u32> + Send + Sync>;
pub type EdgeStyleGetter = Arc<dyn Fn() -> Color + Send + Sync>;
pub type MatrixGetter = Arc<dyn Fn() -> Matrix4 + Send + Sync>;

/// Uniform map populated by `ModelClippingPlanesPipelineStage`.
#[derive(Clone, Default)]
pub struct ModelClippingPlanesUniformMap {
    pub model_clipping_planes: Option<TextureSamplerGetter>,
    pub model_clipping_planes_edge_style: Option<EdgeStyleGetter>,
    pub model_clipping_planes_matrix: Option<MatrixGetter>,
}

impl ModelClippingPlanesUniformMap {
    pub fn model_clipping_planes(&self) -> Option<u32> {
        self.model_clipping_planes.as_ref().and_then(|cb| cb())
    }

    pub fn model_clipping_planes_edge_style(&self) -> Option<Color> {
        self.model_clipping_planes_edge_style.as_ref().map(|cb| cb())
    }

    pub fn model_clipping_planes_matrix(&self) -> Option<Matrix4> {
        self.model_clipping_planes_matrix.as_ref().map(|cb| cb())
    }
}

/// Render resources configured by `ModelClippingPlanesPipelineStage`.
pub struct ModelClippingPlanesRenderResources {
    pub shader_builder: ShaderBuilder,
    pub uniform_map: ModelClippingPlanesUniformMap,
    pub model: ModelClippingPlanesModel,
}

impl ModelClippingPlanesRenderResources {
    pub fn new(model: ModelClippingPlanesModel) -> Self {
        Self {
            shader_builder: ShaderBuilder::new(),
            uniform_map: ModelClippingPlanesUniformMap::default(),
            model,
        }
    }
}

/// Pipeline stage for applying clipping planes to a 3D model.
pub struct ModelClippingPlanesPipelineStage;

impl ModelClippingPlanesPipelineStage {
    pub const NAME: &'static str = "ModelClippingPlanesPipelineStage";

    /// Modifies the shader builder and uniform map to configure clipping planes rendering.
    pub fn process(
        render_resources: &mut ModelClippingPlanesRenderResources,
        model: &ModelClippingPlanesModel,
        frame_state: &ModelClippingPlanesFrameState,
    ) {
        let clipping_planes = &model.clipping_planes;
        let context = &frame_state.context;
        let shader_builder = &mut render_resources.shader_builder;

        shader_builder.add_define(
            "HAS_CLIPPING_PLANES",
            None,
            Some(ShaderDestination::FRAGMENT),
        );

        let length_str = clipping_planes.len().to_string();
        shader_builder.add_define(
            "CLIPPING_PLANES_LENGTH",
            Some(&length_str),
            Some(ShaderDestination::FRAGMENT),
        );

        if clipping_planes.union_clipping_regions {
            shader_builder.add_define(
                "UNION_CLIPPING_REGIONS",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }

        if context.floating_point_texture {
            shader_builder.add_define(
                "USE_CLIPPING_PLANES_FLOAT_TEXTURE",
                None,
                Some(ShaderDestination::FRAGMENT),
            );
        }

        let texture_resolution =
            clipping_planes.get_texture_resolution(context.floating_point_texture, None);

        let width_str = (texture_resolution.x as u32).to_string();
        let height_str = (texture_resolution.y as u32).to_string();

        shader_builder.add_define(
            "CLIPPING_PLANES_TEXTURE_WIDTH",
            Some(&width_str),
            Some(ShaderDestination::FRAGMENT),
        );

        shader_builder.add_define(
            "CLIPPING_PLANES_TEXTURE_HEIGHT",
            Some(&height_str),
            Some(ShaderDestination::FRAGMENT),
        );

        shader_builder.add_uniform(
            "sampler2D",
            "model_clippingPlanes",
            Some(ShaderDestination::FRAGMENT),
        );
        shader_builder.add_uniform(
            "vec4",
            "model_clippingPlanesEdgeStyle",
            Some(ShaderDestination::FRAGMENT),
        );
        shader_builder.add_uniform(
            "mat4",
            "model_clippingPlanesMatrix",
            Some(ShaderDestination::FRAGMENT),
        );

        shader_builder.add_fragment_lines(&[SHADERS_MODEL_CLIPPING_PLANES_STAGE_FS]);

        let edge_color = clipping_planes.edge_color;
        let edge_width = clipping_planes.edge_width;
        let edge_style = Color::new(
            edge_color.red,
            edge_color.green,
            edge_color.blue,
            edge_width as f32,
        );

        let inverse_view_3d = context.uniform_state.inverse_view_3d;
        let clipping_planes_matrix = model.clipping_planes_matrix;

        render_resources.uniform_map.model_clipping_planes = Some(Arc::new(|| Some(1)));
        render_resources.uniform_map.model_clipping_planes_edge_style =
            Some(Arc::new(move || edge_style));
        render_resources.uniform_map.model_clipping_planes_matrix = Some(Arc::new(move || {
            let inverse_view_transpose = inverse_view_3d.transpose();
            inverse_view_transpose.multiply(&clipping_planes_matrix)
        }));
    }
}
