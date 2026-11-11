use crate::context::Context;

/// Options for point cloud shading matching CesiumJS `Scene/PointCloudShading.js`.
#[derive(Debug, Clone, PartialEq)]
pub struct PointCloudShading {
    pub attenuation: bool,
    pub geometric_error_scale: f64,
    pub maximum_attenuation: Option<f64>,
    pub base_resolution: Option<f64>,
    pub eye_dome_lighting: bool,
    pub eye_dome_lighting_strength: f64,
    pub eye_dome_lighting_radius: f64,
    pub back_face_culling: bool,
    pub normal_shading: bool,
}

impl Default for PointCloudShading {
    fn default() -> Self {
        Self {
            attenuation: false,
            geometric_error_scale: 1.0,
            maximum_attenuation: None,
            base_resolution: None,
            eye_dome_lighting: true,
            eye_dome_lighting_strength: 1.0,
            eye_dome_lighting_radius: 1.0,
            back_face_culling: false,
            normal_shading: true,
        }
    }
}

impl PointCloudShading {
    /// Creates a new `PointCloudShading` with default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Determines whether point cloud shading (EDL and floating-point rendering) is supported by the context.
    pub fn is_supported(context: &Context) -> bool {
        context.floating_point_texture && context.draw_buffers && context.fragment_depth
    }
}
