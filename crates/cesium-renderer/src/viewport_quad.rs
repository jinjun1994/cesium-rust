//! ViewportQuad primitive matching CesiumJS `Scene/ViewportQuad.js` (Apache-2.0).

use std::sync::Arc;

use cesium_math::{BoundingRectangle, Color, DeveloperError};

use crate::commands::{
    DrawCommand, DrawCommandOptions, Pass, RenderStateHandle, ShaderProgramHandle,
    UniformMapHandle,
};
use crate::material::Material;
use crate::primitive_collection::next_primitive_id;
use crate::render_state::{BlendingState, RenderState};
use crate::scene_primitive::PrimitiveRenderFrameState;

/// A viewport-aligned quad primitive.
#[derive(Clone, Debug)]
pub struct ViewportQuad {
    id: u64,
    pub show: bool,
    pub rectangle: Option<BoundingRectangle>,
    pub material: Option<Material>,
    overlay_command: Option<DrawCommand>,
    render_state: Option<Arc<RenderState>>,
    destroyed: bool,
}

impl Default for ViewportQuad {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl ViewportQuad {
    /// Creates a new ViewportQuad with an optional bounding rectangle and material.
    pub fn new(rectangle: Option<BoundingRectangle>, material: Option<Material>) -> Self {
        let rect = rectangle.unwrap_or_default();
        let mat = material.unwrap_or_else(|| {
            Material::from_color(Color::new(1.0, 1.0, 1.0, 1.0))
        });

        Self {
            id: next_primitive_id(),
            show: true,
            rectangle: Some(rect),
            material: Some(mat),
            overlay_command: None,
            render_state: None,
            destroyed: false,
        }
    }

    /// Returns the unique primitive ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Updates the primitive and populates the draw command list on the frame state.
    pub fn update(
        &mut self,
        frame_state: &mut PrimitiveRenderFrameState,
    ) -> Result<(), DeveloperError> {
        if self.is_destroyed() {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        if !self.show {
            return Ok(());
        }

        let material = self
            .material
            .as_ref()
            .ok_or_else(|| DeveloperError::new("this.material must be defined."))?;

        let rectangle = self
            .rectangle
            .ok_or_else(|| DeveloperError::new("this.rectangle must be defined."))?;

        let rs = match &self.render_state {
            Some(existing) if existing.viewport == Some(rectangle) => existing.clone(),
            _ => {
                let state = RenderState {
                    blending: BlendingState::ALPHA_BLEND,
                    viewport: Some(rectangle),
                    ..Default::default()
                };
                let arc_state = RenderState::from_cache(Some(state));
                self.render_state = Some(arc_state.clone());
                arc_state
            }
        };

        if frame_state.passes.render {
            let mut uniform_map = std::collections::HashMap::new();
            for (k, v) in &material.uniforms {
                uniform_map.insert(k.clone(), v.to_string());
            }

            let command = DrawCommand::new(DrawCommandOptions {
                shader_program: Some(ShaderProgramHandle::Name("czm_viewportQuad".to_string())),
                render_state: Some(RenderStateHandle::State(rs)),
                uniform_map: Some(UniformMapHandle::Map(uniform_map)),
                pass: Some(Pass::Overlay),
                owner: Some("ViewportQuad".to_string()),
                cull: Some(false),
                ..Default::default()
            });

            self.overlay_command = Some(command.clone());
            frame_state.command_list.push(command);
        }

        Ok(())
    }

    /// Returns the active overlay draw command if available.
    pub fn overlay_command(&self) -> Option<&DrawCommand> {
        self.overlay_command.as_ref()
    }

    /// Returns true if this object was destroyed; otherwise, false.
    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    /// Destroys resources held by this object.
    pub fn destroy(&mut self) {
        self.destroyed = true;
        self.overlay_command = None;
        self.render_state = None;
    }
}
