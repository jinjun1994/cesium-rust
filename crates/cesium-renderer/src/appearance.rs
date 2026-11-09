//! Appearance base interface matching CesiumJS `Scene/Appearance.js` (Apache-2.0).

use crate::material::Material;
use crate::render_state::{BlendingState, CullFace, CullState, DepthFunction, DepthTestState, RenderState};

/// Options for configuring an Appearance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AppearanceOptions {
    pub material: Option<Material>,
    pub translucent: Option<bool>,
    pub closed: Option<bool>,
    pub vertex_shader_source: Option<String>,
    pub fragment_shader_source: Option<String>,
    pub render_state: Option<RenderState>,
    pub flat: Option<bool>,
    pub face_forward: Option<bool>,
}

/// Defines the GLSL vertex and fragment shaders and render state for drawing geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct Appearance {
    pub material: Option<Material>,
    pub translucent: bool,
    pub closed: bool,
    pub flat: bool,
    pub face_forward: bool,
    pub vertex_shader_source: Option<String>,
    pub fragment_shader_source: Option<String>,
    pub render_state: Option<RenderState>,
}

impl Default for Appearance {
    fn default() -> Self {
        Self::new(AppearanceOptions::default())
    }
}

impl Appearance {
    /// Creates a new Appearance instance.
    pub fn new(options: AppearanceOptions) -> Self {
        Self {
            material: options.material,
            translucent: options.translucent.unwrap_or(true),
            closed: options.closed.unwrap_or(false),
            flat: options.flat.unwrap_or(false),
            face_forward: options.face_forward.unwrap_or(false),
            vertex_shader_source: options.vertex_shader_source,
            fragment_shader_source: options.fragment_shader_source,
            render_state: options.render_state,
        }
    }

    /// Procedurally creates the full GLSL fragment shader source for this appearance.
    pub fn get_fragment_shader_source(&self) -> String {
        let mut parts = Vec::new();
        if self.flat {
            parts.push("#define FLAT".to_string());
        }
        if self.face_forward {
            parts.push("#define FACE_FORWARD".to_string());
        }
        if let Some(ref m) = self.material {
            parts.push(m.shader_source.clone());
        }
        if let Some(ref fs) = self.fragment_shader_source {
            parts.push(fs.clone());
        }
        parts.join("\n")
    }

    /// Determines if the geometry is translucent based on translucent flag and material.
    pub fn is_translucent(&self) -> bool {
        if let Some(ref m) = self.material {
            m.is_translucent()
        } else {
            self.translucent
        }
    }

    /// Computes the active render state for this appearance based on translucency.
    pub fn get_render_state(&self) -> RenderState {
        let translucent = self.is_translucent();
        let mut rs = self.render_state.clone().unwrap_or_default();
        if translucent {
            rs.depth_mask = false;
            rs.blending = BlendingState::ALPHA_BLEND;
        } else {
            rs.depth_mask = true;
            rs.blending = BlendingState::DISABLED;
        }
        rs
    }

    /// Generates the default render state given translucency and closed flags.
    pub fn get_default_render_state(
        translucent: bool,
        closed: bool,
        existing: Option<RenderState>,
    ) -> RenderState {
        let mut rs = existing.unwrap_or_default();
        rs.depth_test = DepthTestState {
            enabled: true,
            func: DepthFunction::Less,
        };

        if translucent {
            rs.depth_mask = false;
            rs.blending = BlendingState::ALPHA_BLEND;
        }

        if closed {
            rs.cull = CullState {
                enabled: true,
                face: CullFace::Back,
            };
        }

        rs
    }
}
