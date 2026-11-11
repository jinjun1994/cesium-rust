use std::rc::Rc;

use crate::commands::{ClearCommand, DrawCommand, FramebufferHandle, Pass, ShaderProgramHandle};
use crate::context::Context;
use crate::framebuffer::{Framebuffer, FramebufferManager, FramebufferManagerOptions};
use crate::point_cloud_shading::PointCloudShading;
use crate::texture::Texture;
use cesium_math::{BoundingSphere, Color, DeveloperError, PrimitiveType};

/// A command produced or processed during point cloud eye dome lighting passes.
#[derive(Clone, Debug, PartialEq)]
pub enum PointCloudCommand {
    Draw(Box<DrawCommand>),
    Clear(ClearCommand),
}

/// Rendering frame state passed into `PointCloudEyeDomeLighting::update`.
pub struct PointCloudEyeDomeLightingFrameState<'a> {
    pub context: &'a Context,
    pub command_list: Vec<PointCloudCommand>,
    pub pixel_ratio: f32,
    pub is_picking: bool,
}

impl<'a> PointCloudEyeDomeLightingFrameState<'a> {
    pub fn new(context: &'a Context) -> Self {
        Self {
            context,
            command_list: Vec::new(),
            pixel_ratio: 1.0,
            is_picking: false,
        }
    }
}

/// Eye dome lighting post-processing manager for point clouds.
///
/// Eye dome lighting enhances depth perception of unlit or shaded point clouds
/// by computing a local depth gradient in screen space and attenuating or darkening
/// silhouette boundaries.
#[derive(Debug)]
pub struct PointCloudEyeDomeLighting {
    framebuffer_manager: Option<FramebufferManager>,
    strength: f32,
    radius: f32,
    draw_command: Option<DrawCommand>,
    clear_command: Option<ClearCommand>,
    is_destroyed: bool,
}

impl Default for PointCloudEyeDomeLighting {
    fn default() -> Self {
        Self::new().expect("Valid PointCloudEyeDomeLighting creation")
    }
}

impl PointCloudEyeDomeLighting {
    /// Creates a new `PointCloudEyeDomeLighting` processor.
    pub fn new() -> Result<Self, DeveloperError> {
        let options = FramebufferManagerOptions {
            color_attachments_length: 2,
            depth_stencil: true,
            supports_depth_texture: true,
            create_color_attachments: true,
            create_depth_attachments: true,
            ..Default::default()
        };

        let manager = FramebufferManager::new(options)?;

        let clear_command = ClearCommand {
            color: Some(Color::new(0.0, 0.0, 0.0, 0.0)),
            depth: Some(1.0),
            stencil: Some(0),
            pass: Some(Pass::Cesium3dTile),
            ..Default::default()
        };

        let draw_command = DrawCommand {
            primitive_type: PrimitiveType::Triangles,
            pass: Some(Pass::Cesium3dTile),
            shader_program: Some(ShaderProgramHandle::Name("PointCloudEyeDomeLighting".to_string())),
            ..Default::default()
        };

        Ok(Self {
            framebuffer_manager: Some(manager),
            strength: 1.0,
            radius: 1.0,
            draw_command: Some(draw_command),
            clear_command: Some(clear_command),
            is_destroyed: false,
        })
    }

    /// Determines whether eye dome lighting is supported on the target hardware context.
    pub fn is_supported(context: &Context) -> bool {
        context.draw_buffers && context.fragment_depth
    }

    /// Gets the offscreen framebuffer where hijacked point primitives render.
    pub fn framebuffer(&self) -> Option<&Framebuffer> {
        self.framebuffer_manager
            .as_ref()
            .and_then(|m| m.framebuffer())
    }

    /// Color G-buffer texture (attachment 0).
    pub fn color_g_buffer(&self) -> Option<&Texture> {
        self.framebuffer_manager
            .as_ref()
            .and_then(|m| m.get_color_texture(0).ok().flatten())
    }

    /// Packed depth G-buffer texture (attachment 1).
    pub fn depth_g_buffer(&self) -> Option<&Texture> {
        self.framebuffer_manager
            .as_ref()
            .and_then(|m| m.get_color_texture(1).ok().flatten())
    }

    /// Updates resources and hijacks point commands to inject the EDL offscreen pass and screen quad blend.
    pub fn update(
        &mut self,
        frame_state: &mut PointCloudEyeDomeLightingFrameState,
        command_start: usize,
        point_cloud_shading: &PointCloudShading,
        bounding_volume: Option<BoundingSphere>,
    ) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        // EDL is not supported on this context or disabled in shading settings
        if !Self::is_supported(frame_state.context) || !point_cloud_shading.eye_dome_lighting {
            return Ok(());
        }

        // Do not change commands for pick calls
        if frame_state.is_picking {
            return Ok(());
        }

        self.strength = point_cloud_shading.eye_dome_lighting_strength as f32;
        self.radius =
            (point_cloud_shading.eye_dome_lighting_radius * frame_state.pixel_ratio as f64) as f32;

        let width = frame_state.context.drawing_buffer_width;
        let height = frame_state.context.drawing_buffer_height;

        let manager = self
            .framebuffer_manager
            .as_mut()
            .ok_or_else(|| DeveloperError::new("FramebufferManager not initialized"))?;

        manager.update(Some(width), Some(height), Some(1), None, None)?;

        let fb_rc = manager.framebuffer().cloned().map(Rc::new);

        // Hijack existing point commands in [command_start..command_end]
        let command_end = frame_state.command_list.len();
        for i in command_start..command_end {
            if let PointCloudCommand::Draw(ref mut cmd) = frame_state.command_list[i] {
                if cmd.primitive_type == PrimitiveType::Points && cmd.pass != Some(Pass::Translucent) {
                    cmd.cast_shadows = false;
                    cmd.receive_shadows = false;
                    cmd.dirty = true;
                    // Direct command to offscreen EDL framebuffer
                    cmd.framebuffer = fb_rc.clone().map(FramebufferHandle::Framebuffer);
                    // Switch to derived EyeDomeLighting shader
                    let derived_name = match &cmd.shader_program {
                        Some(ShaderProgramHandle::Name(n)) => format!("{}_EC", n),
                        _ => "PointCloud_EC".to_string(),
                    };
                    cmd.shader_program = Some(ShaderProgramHandle::Name(derived_name));
                }
            }
        }

        // Blend EDL into main framebuffer by appending post-processing draw & clear commands
        let mut blend_cmd = self
            .draw_command
            .clone()
            .ok_or_else(|| DeveloperError::new("DrawCommand not initialized"))?;
        blend_cmd.bounding_volume = bounding_volume;

        let mut clear_cmd = self
            .clear_command
            .clone()
            .ok_or_else(|| DeveloperError::new("ClearCommand not initialized"))?;
        clear_cmd.framebuffer = fb_rc;

        frame_state.command_list.push(PointCloudCommand::Draw(Box::new(blend_cmd)));
        frame_state.command_list.push(PointCloudCommand::Clear(clear_cmd));

        Ok(())
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys all WebGL resources.
    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            if let Some(mut m) = self.framebuffer_manager.take() {
                m.destroy();
            }
            self.draw_command = None;
            self.clear_command = None;
            self.is_destroyed = true;
        }
    }
}
