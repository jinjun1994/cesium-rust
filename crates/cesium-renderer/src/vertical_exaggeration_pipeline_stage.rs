//! VerticalExaggerationPipelineStage matching CesiumJS `Scene/Model/VerticalExaggerationPipelineStage.js` (Apache-2.0).

use std::collections::HashMap;
use cesium_math::Cartesian2;

use crate::model_render_resources::ModelUniformCallback;
use crate::shader_builder::{ShaderBuilder, ShaderDestination};

/// Vertex shader code for vertical exaggeration calculations.
pub const VERTICAL_EXAGGERATION_STAGE_VS: &str = r#"void verticalExaggerationStage(
  inout ProcessedAttributes attributes
) {
  // Compute the distance from the camera to the local center of curvature.
  vec4 vertexPositionENU = czm_modelToEnu * vec4(attributes.positionMC, 1.0);
  vec2 vertexAzimuth = normalize(vertexPositionENU.xy);
  // Curvature = 1 / radius of curvature.
  float azimuthalCurvature = dot(vertexAzimuth * vertexAzimuth, czm_eyeEllipsoidCurvature);
  float eyeToCenter = 1.0 / azimuthalCurvature + czm_eyeHeight;

  // Compute the approximate ellipsoid normal at the vertex position.
  // Uses a circular approximation for the Earth curvature along the geodesic.
  vec3 vertexPositionEC = (czm_modelView * vec4(attributes.positionMC, 1.0)).xyz;
  vec3 centerToVertex = eyeToCenter * czm_eyeEllipsoidNormalEC + vertexPositionEC;
  vec3 vertexEllipsoidNormalEC = normalize(centerToVertex);

  // Estimate the (sine of the) angle between the camera direction and the ellipsoid normal.
  float verticalDistance = dot(vertexPositionEC, czm_eyeEllipsoidNormalEC);
  float horizontalDistance = length(vertexPositionEC - verticalDistance * czm_eyeEllipsoidNormalEC);
  float sinTheta = horizontalDistance / (eyeToCenter + verticalDistance);
  bool isSmallAngle = clamp(sinTheta, 0.0, 0.05) == sinTheta;

  // Approximate the change in height above the ellipsoid, from camera to vertex position.
  float exactVersine = 1.0 - dot(czm_eyeEllipsoidNormalEC, vertexEllipsoidNormalEC);
  float smallAngleVersine = 0.5 * sinTheta * sinTheta;
  float versine = isSmallAngle ? smallAngleVersine : exactVersine;
  float dHeight = dot(vertexPositionEC, vertexEllipsoidNormalEC) - eyeToCenter * versine;
  float vertexHeight = czm_eyeHeight + dHeight;

  // Transform the approximate ellipsoid normal to model coordinates.
  vec3 exaggerationDirectionScaledMC = (czm_inverseModelView * vec4(vertexEllipsoidNormalEC, 0.0)).xyz;

  // Compute the exaggeration and apply it along the exaggeration direction in model coordinates.
  float stretch = u_verticalExaggerationAndRelativeHeight.x;
  float shift = u_verticalExaggerationAndRelativeHeight.y;
  float exaggeration = (vertexHeight - shift) * (stretch - 1.0);
  attributes.positionMC += exaggeration * exaggerationDirectionScaledMC;
}"#;

/// Frame state configuration for vertical exaggeration.
#[derive(Clone, Copy, Debug)]
pub struct VerticalExaggerationFrameState {
    pub vertical_exaggeration: f64,
    pub vertical_exaggeration_relative_height: f64,
}

impl Default for VerticalExaggerationFrameState {
    fn default() -> Self {
        Self {
            vertical_exaggeration: 1.0,
            vertical_exaggeration_relative_height: 0.0,
        }
    }
}

/// Standalone render resources for VerticalExaggerationPipelineStage.
#[derive(Default)]
pub struct MockVerticalExaggerationRenderResources {
    pub shader_builder: ShaderBuilder,
    pub uniform_map: HashMap<String, ModelUniformCallback>,
    pub bound_exaggeration: Option<Cartesian2>,
}

impl std::fmt::Debug for MockVerticalExaggerationRenderResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockVerticalExaggerationRenderResources")
            .field("shader_builder", &self.shader_builder)
            .field("bound_exaggeration", &self.bound_exaggeration)
            .finish()
    }
}

/// Trait defining the render resources required by [`VerticalExaggerationPipelineStage`].
pub trait VerticalExaggerationRenderResources {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder;
    fn set_vertical_exaggeration_uniform(&mut self, exaggeration: Cartesian2);
}

impl VerticalExaggerationRenderResources for MockVerticalExaggerationRenderResources {
    fn shader_builder_mut(&mut self) -> &mut ShaderBuilder {
        &mut self.shader_builder
    }

    fn set_vertical_exaggeration_uniform(&mut self, exaggeration: Cartesian2) {
        self.bound_exaggeration = Some(exaggeration);
        self.uniform_map.insert(
            "u_verticalExaggerationAndRelativeHeight".to_string(),
            Box::new(move || exaggeration.x),
        );
    }
}

/// The vertical exaggeration pipeline stage transforms the vertex positions based on
/// scene vertical exaggeration settings.
pub struct VerticalExaggerationPipelineStage;

impl VerticalExaggerationPipelineStage {
    pub const NAME: &'static str = "VerticalExaggerationPipelineStage";

    /// Process a primitive for vertical exaggeration.
    pub fn process<R: VerticalExaggerationRenderResources>(
        render_resources: &mut R,
        frame_state: &VerticalExaggerationFrameState,
    ) {
        let shader_builder = render_resources.shader_builder_mut();

        shader_builder.add_vertex_lines(&[VERTICAL_EXAGGERATION_STAGE_VS]);

        shader_builder.add_define(
            "HAS_VERTICAL_EXAGGERATION",
            None,
            Some(ShaderDestination::VERTEX),
        );

        shader_builder.add_uniform(
            "vec2",
            "u_verticalExaggerationAndRelativeHeight",
            Some(ShaderDestination::VERTEX),
        );

        let exaggeration_uniform = Cartesian2::new(
            frame_state.vertical_exaggeration,
            frame_state.vertical_exaggeration_relative_height,
        );
        render_resources.set_vertical_exaggeration_uniform(exaggeration_uniform);
    }
}
