//! CustomShader matching CesiumJS `Scene/Model/CustomShader.js` (Apache-2.0).

/// A user-defined shader for modifying materials and geometry of 3D Models and 3D Tilesets.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CustomShader {
    pub vertex_shader_text: Option<String>,
    pub fragment_shader_text: Option<String>,
}

impl CustomShader {
    pub fn new() -> Self {
        Self::default()
    }
}
