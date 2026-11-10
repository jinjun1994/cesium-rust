//! ModelDrawCommand matching CesiumJS `Scene/Model/ModelDrawCommand.js` (Apache-2.0).

use std::f64::consts::PI;
use std::sync::Arc;

use cesium_math::{BoundingSphere, Color, DeveloperError, PrimitiveType, SceneMode, ShadowMode};
use glam::DMat4;

use crate::classification_model_draw_command::StencilConstants;
use crate::commands::{
    DrawCommand, Pass, RenderStateHandle, ShaderProgramHandle, UniformMapHandle, VertexArrayHandle,
};
use crate::edge_visibility_pipeline_stage::EdgeDisplayMode;
use crate::render_state::{
    BlendingState, ColorMaskState, CullFace, PolygonOffsetState, RenderState, StencilFunction,
    StencilOperation, StencilOperationState, StencilTestState,
};
use crate::uniform::{UniformMap, UniformValue};

/// Specifies which style commands are needed when styling features.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StyleCommandsNeeded {
    AllOpaque = 0,
    AllTranslucent = 1,
    OpaqueAndTranslucent = 2,
}

impl StyleCommandsNeeded {
    /// Determines which style commands are needed based on whether opaque or translucent features exist.
    pub fn get_style_commands_needed(has_opaque: bool, has_translucent: bool) -> Option<Self> {
        if has_opaque && has_translucent {
            Some(Self::OpaqueAndTranslucent)
        } else if has_opaque {
            Some(Self::AllOpaque)
        } else if has_translucent {
            Some(Self::AllTranslucent)
        } else {
            None
        }
    }
}

/// A wrapper around a DrawCommand managed by ModelDrawCommand, specifying which properties to update.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelDerivedCommand {
    pub command: DrawCommand,
    pub update_shadows: bool,
    pub update_back_face_culling: bool,
    pub update_cull_face: bool,
    pub update_debug_show_bounding_volume: bool,
    pub is_2d: bool,
    pub derived_command_2d: Option<Box<ModelDerivedCommand>>,
}

impl ModelDerivedCommand {
    pub fn new(
        command: DrawCommand,
        update_shadows: bool,
        update_back_face_culling: bool,
        update_cull_face: bool,
        update_debug_show_bounding_volume: bool,
        is_2d: bool,
    ) -> Self {
        Self {
            command,
            update_shadows,
            update_back_face_culling,
            update_cull_face,
            update_debug_show_bounding_volume,
            is_2d,
            derived_command_2d: None,
        }
    }
}

/// Tileset backface command collection matching CesiumJS `Cesium3DTileset`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelDrawCommandTileset {
    pub has_mixed_content: bool,
    pub backface_commands: Vec<DrawCommand>,
}

/// Tile metadata matching CesiumJS `Cesium3DTile`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelDrawCommandTile {
    pub final_resolution: bool,
    pub selection_depth: u32,
}

/// 3D Tiles content container matching CesiumJS `Model.content`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelDrawCommandContent {
    pub tileset: ModelDrawCommandTileset,
    pub tile: ModelDrawCommandTile,
}

/// Model options used by ModelDrawCommand.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelDrawCommandModelOptions {
    pub bounding_sphere_2d: BoundingSphere,
    pub color: Color,
    pub silhouette_size: f64,
    pub silhouette_color: Color,
    pub silhouette_id: u32,
    pub shadows: ShadowMode,
    pub project_to_2d: bool,
    pub edge_display_mode: EdgeDisplayMode,
    pub style_commands_needed: Option<StyleCommandsNeeded>,
    pub content: Option<ModelDrawCommandContent>,
}

impl Default for ModelDrawCommandModelOptions {
    fn default() -> Self {
        Self {
            bounding_sphere_2d: BoundingSphere::new(cesium_math::Cartesian3::ZERO, 1.0),
            color: Color::WHITE,
            silhouette_size: 0.0,
            silhouette_color: Color::RED,
            silhouette_id: 1,
            shadows: ShadowMode::Enabled,
            project_to_2d: false,
            edge_display_mode: EdgeDisplayMode::SurfacesOnly,
            style_commands_needed: None,
            content: None,
        }
    }
}

impl ModelDrawCommandModelOptions {
    pub fn is_translucent(&self) -> bool {
        self.color.alpha > 0.0 && self.color.alpha < 1.0
    }

    pub fn is_invisible(&self) -> bool {
        self.color.alpha == 0.0
    }

    pub fn has_silhouette(&self) -> bool {
        self.silhouette_size > 0.0
    }
}

/// Material properties for the primitive.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelDrawCommandMaterial {
    pub double_sided: bool,
}

/// Model primitive metadata.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelDrawCommandPrimitive {
    pub material: ModelDrawCommandMaterial,
}

/// Runtime primitive used by ModelDrawCommand.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelDrawCommandRuntimePrimitive {
    pub primitive: ModelDrawCommandPrimitive,
    pub bounding_sphere: BoundingSphere,
}

/// Edge geometry for edge command derivation.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelDrawCommandEdgeGeometry {
    pub vertex_array: Option<VertexArrayHandle>,
    pub primitive_type: PrimitiveType,
    pub index_count: usize,
    pub shader_program: Option<ShaderProgramHandle>,
    pub pass: Pass,
    pub line_width: Option<f32>,
}

/// Primitive render resources for ModelDrawCommand.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelPrimitiveRenderResources {
    pub model: ModelDrawCommandModelOptions,
    pub runtime_primitive: ModelDrawCommandRuntimePrimitive,
    pub has_silhouette: bool,
    pub has_skip_level_of_detail: bool,
    pub planar_fill_id_pass: bool,
    pub edge_geometry: Option<ModelDrawCommandEdgeGeometry>,
}

/// Passes state for frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModelDrawCommandFramePasses {
    pub snap: bool,
    pub pick: bool,
}

/// Frame state subset needed by ModelDrawCommand.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelDrawCommandFrameState {
    pub mode: SceneMode,
    pub ellipsoid_maximum_radius: f64,
    pub passes: ModelDrawCommandFramePasses,
    pub edge_visibility_requested: bool,
}

impl Default for ModelDrawCommandFrameState {
    fn default() -> Self {
        Self {
            mode: SceneMode::Scene3D,
            ellipsoid_maximum_radius: 6378137.0,
            passes: ModelDrawCommandFramePasses::default(),
            edge_visibility_requested: false,
        }
    }
}

fn math_sign(val: f64) -> f64 {
    if val > 0.0 {
        1.0
    } else if val < 0.0 {
        -1.0
    } else {
        0.0
    }
}

fn get_render_state(command: &DrawCommand) -> RenderState {
    match &command.render_state {
        Some(RenderStateHandle::State(rs)) => (**rs).clone(),
        _ => RenderState::default(),
    }
}

/// A wrapper around draw commands used to render a ModelRuntimePrimitive.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelDrawCommand {
    command: DrawCommand,
    model: ModelDrawCommandModelOptions,
    runtime_primitive: ModelDrawCommandRuntimePrimitive,
    primitive_render_resources: ModelPrimitiveRenderResources,

    model_matrix: DMat4,
    bounding_volume: BoundingSphere,
    model_matrix_2d: DMat4,
    bounding_volume_2d: BoundingSphere,
    model_matrix_2d_dirty: bool,

    back_face_culling: bool,
    cull_face: CullFace,
    shadows: ShadowMode,
    debug_show_bounding_volume: bool,

    uses_back_face_culling: bool,
    needs_translucent_command: bool,
    needs_skip_level_of_detail_commands: bool,
    needs_silhouette_commands: bool,
    needs_edge_commands: bool,
    needs_planar_fill_id_command: bool,

    original_command: Option<ModelDerivedCommand>,
    translucent_command: Option<ModelDerivedCommand>,
    skip_lod_backface_command: Option<ModelDerivedCommand>,
    skip_lod_stencil_command: Option<ModelDerivedCommand>,
    silhouette_model_command: Option<ModelDerivedCommand>,
    silhouette_color_command: Option<ModelDerivedCommand>,
    edge_command: Option<ModelDerivedCommand>,
    edge_snap_command: Option<ModelDerivedCommand>,
    planar_fill_id_command: Option<ModelDerivedCommand>,

    derived_commands: Vec<ModelDerivedCommand>,
    has_2d_commands: bool,
}

impl ModelDrawCommand {
    /// Creates a ModelDrawCommand directly wrapping a DrawCommand.
    pub fn from_command(command: DrawCommand) -> Self {
        let model_matrix = command.model_matrix.unwrap_or(DMat4::IDENTITY);
        let bounding_volume = command.bounding_volume.unwrap_or_default();
        Self {
            command,
            model: ModelDrawCommandModelOptions::default(),
            runtime_primitive: ModelDrawCommandRuntimePrimitive::default(),
            primitive_render_resources: ModelPrimitiveRenderResources::default(),
            model_matrix,
            bounding_volume,
            model_matrix_2d: DMat4::IDENTITY,
            bounding_volume_2d: BoundingSphere::default(),
            model_matrix_2d_dirty: false,
            back_face_culling: true,
            cull_face: CullFace::Back,
            shadows: ShadowMode::Disabled,
            debug_show_bounding_volume: false,
            uses_back_face_culling: false,
            needs_translucent_command: false,
            needs_skip_level_of_detail_commands: false,
            needs_silhouette_commands: false,
            needs_edge_commands: false,
            needs_planar_fill_id_command: false,
            original_command: None,
            translucent_command: None,
            skip_lod_backface_command: None,
            skip_lod_stencil_command: None,
            silhouette_model_command: None,
            silhouette_color_command: None,
            edge_command: None,
            edge_snap_command: None,
            planar_fill_id_command: None,
            derived_commands: Vec::new(),
            has_2d_commands: false,
        }
    }

    /// Creates a new ModelDrawCommand from the command and primitive render resources.
    pub fn new(
        command: Option<DrawCommand>,
        render_resources: Option<ModelPrimitiveRenderResources>,
    ) -> Result<Self, DeveloperError> {
        let command = command.ok_or_else(|| DeveloperError::new("command is required"))?;
        let render_resources = render_resources
            .ok_or_else(|| DeveloperError::new("primitiveRenderResources is required"))?;

        let model = render_resources.model.clone();
        let runtime_primitive = render_resources.runtime_primitive.clone();

        let is_translucent = command.pass == Some(Pass::Translucent);
        let is_double_sided = runtime_primitive.primitive.material.double_sided;
        let uses_back_face_culling = !is_double_sided && !is_translucent;
        let has_silhouette = render_resources.has_silhouette;

        let needs_translucent_command = !is_translucent && !has_silhouette;
        let needs_skip_level_of_detail_commands =
            render_resources.has_skip_level_of_detail && !is_translucent;
        let needs_silhouette_commands = has_silhouette;
        let needs_edge_commands = render_resources.edge_geometry.is_some();
        let needs_planar_fill_id_command = render_resources.planar_fill_id_pass;

        let model_matrix = command.model_matrix.unwrap_or(DMat4::IDENTITY);
        let bounding_volume = command.bounding_volume.unwrap_or_else(|| {
            runtime_primitive
                .bounding_sphere
                .transform(model_matrix)
                .unwrap_or(runtime_primitive.bounding_sphere)
        });

        let (cull_enabled, cull_face) = match &command.render_state {
            Some(RenderStateHandle::State(rs)) => (rs.cull.enabled, rs.cull.face),
            _ => (false, CullFace::Back),
        };

        let shadows = model.shadows;
        let debug_show_bounding_volume = command.debug_show_bounding_volume;

        let mut instance = Self {
            command,
            model,
            runtime_primitive,
            primitive_render_resources: render_resources,

            model_matrix,
            bounding_volume,
            model_matrix_2d: DMat4::IDENTITY,
            bounding_volume_2d: BoundingSphere::default(),
            model_matrix_2d_dirty: false,

            back_face_culling: cull_enabled,
            cull_face,
            shadows,
            debug_show_bounding_volume,

            uses_back_face_culling,
            needs_translucent_command,
            needs_skip_level_of_detail_commands,
            needs_silhouette_commands,
            needs_edge_commands,
            needs_planar_fill_id_command,

            original_command: None,
            translucent_command: None,
            skip_lod_backface_command: None,
            skip_lod_stencil_command: None,
            silhouette_model_command: None,
            silhouette_color_command: None,
            edge_command: None,
            edge_snap_command: None,
            planar_fill_id_command: None,

            derived_commands: Vec::new(),
            has_2d_commands: false,
        };

        instance.initialize();
        Ok(instance)
    }

    fn initialize(&mut self) {
        let mut cmd = self.command.clone();
        cmd.model_matrix = Some(self.model_matrix);
        cmd.bounding_volume = Some(self.bounding_volume);

        let orig = ModelDerivedCommand::new(
            cmd,
            true,
            self.uses_back_face_culling,
            self.uses_back_face_culling,
            true,
            false,
        );
        self.original_command = Some(orig.clone());
        self.derived_commands.push(orig);

        if self.needs_translucent_command {
            let trans_cmd = derive_translucent_command(&self.command);
            let derived = ModelDerivedCommand::new(trans_cmd, true, false, false, true, false);
            self.translucent_command = Some(derived.clone());
            self.derived_commands.push(derived);
        }

        if self.needs_skip_level_of_detail_commands {
            let backface_cmd = derive_skip_lod_backface_command(&self.command);
            let backface_derived = ModelDerivedCommand::new(
                backface_cmd,
                false,
                false,
                self.uses_back_face_culling,
                false,
                false,
            );
            self.skip_lod_backface_command = Some(backface_derived.clone());
            self.derived_commands.push(backface_derived);

            let stencil_cmd = derive_skip_lod_stencil_command(&self.command);
            let stencil_derived = ModelDerivedCommand::new(
                stencil_cmd,
                true,
                self.uses_back_face_culling,
                self.uses_back_face_culling,
                true,
                false,
            );
            self.skip_lod_stencil_command = Some(stencil_derived.clone());
            self.derived_commands.push(stencil_derived);
        }

        if self.needs_silhouette_commands {
            let sil_model_cmd = derive_silhouette_model_command(
                &self.command,
                self.model.silhouette_id,
                self.model.is_invisible(),
            );
            let sil_model_derived = ModelDerivedCommand::new(
                sil_model_cmd,
                true,
                self.uses_back_face_culling,
                self.uses_back_face_culling,
                true,
                false,
            );
            self.silhouette_model_command = Some(sil_model_derived.clone());
            self.derived_commands.push(sil_model_derived);

            let sil_color_cmd = derive_silhouette_color_command(
                &self.command,
                self.model.silhouette_id,
                self.model.silhouette_color,
            );
            let sil_color_derived =
                ModelDerivedCommand::new(sil_color_cmd, false, false, false, false, false);
            self.silhouette_color_command = Some(sil_color_derived.clone());
            self.derived_commands.push(sil_color_derived);
        }

        if self.needs_edge_commands {
            if let Some(edge_geometry) = &self.primitive_render_resources.edge_geometry {
                let color_line_width = edge_geometry.line_width.unwrap_or(1.0);
                let snap_line_width = 1.0;

                let edge_cmd = derive_edge_command(&self.command, edge_geometry, color_line_width);
                let edge_derived =
                    ModelDerivedCommand::new(edge_cmd, false, false, false, false, false);
                self.edge_command = Some(edge_derived.clone());
                self.derived_commands.push(edge_derived);

                let edge_snap_cmd =
                    derive_edge_command(&self.command, edge_geometry, snap_line_width);
                let edge_snap_derived =
                    ModelDerivedCommand::new(edge_snap_cmd, false, false, false, false, false);
                self.edge_snap_command = Some(edge_snap_derived.clone());
                self.derived_commands.push(edge_snap_derived);
            }
        }

        if self.needs_planar_fill_id_command {
            let planar_cmd = derive_planar_fill_id_command(&self.command);
            let planar_derived =
                ModelDerivedCommand::new(planar_cmd, false, false, false, false, false);
            self.planar_fill_id_command = Some(planar_derived.clone());
            self.derived_commands.push(planar_derived);
        }
    }

    pub fn command(&self) -> &DrawCommand {
        &self.command
    }

    pub fn runtime_primitive(&self) -> &ModelDrawCommandRuntimePrimitive {
        &self.runtime_primitive
    }

    pub fn model(&self) -> &ModelDrawCommandModelOptions {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut ModelDrawCommandModelOptions {
        &mut self.model
    }

    pub fn primitive_type(&self) -> PrimitiveType {
        self.command.primitive_type
    }

    pub fn model_matrix(&self) -> DMat4 {
        self.model_matrix
    }

    pub fn model_matrix_2d(&self) -> DMat4 {
        self.model_matrix_2d
    }

    pub fn set_model_matrix(&mut self, value: DMat4) {
        self.model_matrix = value;
        self.model_matrix_2d_dirty = true;
        self.bounding_volume = self
            .runtime_primitive
            .bounding_sphere
            .transform(value)
            .unwrap_or(self.runtime_primitive.bounding_sphere);

        // Update commands
        for cmd in &mut self.derived_commands {
            if !cmd.is_2d {
                cmd.command.model_matrix = Some(self.model_matrix);
                cmd.command.bounding_volume = Some(self.bounding_volume);
            }
        }
        self.sync_named_commands_3d();
    }

    pub fn bounding_volume(&self) -> BoundingSphere {
        self.bounding_volume
    }

    pub fn set_bounding_volume(&mut self, value: BoundingSphere) {
        self.bounding_volume = value;
        for cmd in &mut self.derived_commands {
            if !cmd.is_2d {
                cmd.command.bounding_volume = Some(self.bounding_volume);
            }
        }
        self.sync_named_commands_3d();
    }

    pub fn shadows(&self) -> ShadowMode {
        self.shadows
    }

    pub fn set_shadows(&mut self, value: ShadowMode) {
        self.shadows = value;
        let cast = ShadowMode::cast_shadows(value);
        let receive = ShadowMode::receive_shadows(value);

        for cmd in &mut self.derived_commands {
            if cmd.update_shadows {
                cmd.command.cast_shadows = cast;
                cmd.command.receive_shadows = receive;
            }
        }
        self.sync_named_commands();
    }

    pub fn back_face_culling(&self) -> bool {
        self.back_face_culling
    }

    pub fn set_back_face_culling(&mut self, value: bool) {
        if self.back_face_culling == value {
            return;
        }
        self.back_face_culling = value;

        for cmd in &mut self.derived_commands {
            if cmd.update_back_face_culling {
                let mut rs = get_render_state(&cmd.command);
                rs.cull.enabled = value;
                cmd.command.render_state = Some(RenderStateHandle::State(Arc::new(rs)));
            }
        }
        self.sync_named_commands();
    }

    pub fn cull_face(&self) -> CullFace {
        self.cull_face
    }

    pub fn set_cull_face(&mut self, value: CullFace) {
        if self.cull_face == value {
            return;
        }
        self.cull_face = value;

        for cmd in &mut self.derived_commands {
            if cmd.update_cull_face {
                let mut rs = get_render_state(&cmd.command);
                rs.cull.face = value;
                cmd.command.render_state = Some(RenderStateHandle::State(Arc::new(rs)));
            }
        }
        self.sync_named_commands();
    }

    pub fn debug_show_bounding_volume(&self) -> bool {
        self.debug_show_bounding_volume
    }

    pub fn set_debug_show_bounding_volume(&mut self, value: bool) {
        if self.debug_show_bounding_volume == value {
            return;
        }
        self.debug_show_bounding_volume = value;

        for cmd in &mut self.derived_commands {
            if cmd.update_debug_show_bounding_volume {
                cmd.command.debug_show_bounding_volume = value;
            }
        }
        self.sync_named_commands();
    }

    pub fn original_command(&self) -> Option<&ModelDerivedCommand> {
        self.original_command.as_ref()
    }

    pub fn translucent_command(&self) -> Option<&ModelDerivedCommand> {
        self.translucent_command.as_ref()
    }

    pub fn skip_lod_backface_command(&self) -> Option<&ModelDerivedCommand> {
        self.skip_lod_backface_command.as_ref()
    }

    pub fn skip_lod_stencil_command(&self) -> Option<&ModelDerivedCommand> {
        self.skip_lod_stencil_command.as_ref()
    }

    pub fn silhouette_model_command(&self) -> Option<&ModelDerivedCommand> {
        self.silhouette_model_command.as_ref()
    }

    pub fn silhouette_color_command(&self) -> Option<&ModelDerivedCommand> {
        self.silhouette_color_command.as_ref()
    }

    pub fn edge_command(&self) -> Option<&ModelDerivedCommand> {
        self.edge_command.as_ref()
    }

    pub fn edge_snap_command(&self) -> Option<&ModelDerivedCommand> {
        self.edge_snap_command.as_ref()
    }

    pub fn planar_fill_id_command(&self) -> Option<&ModelDerivedCommand> {
        self.planar_fill_id_command.as_ref()
    }

    pub fn derived_commands(&self) -> &[ModelDerivedCommand] {
        &self.derived_commands
    }

    pub fn needs_planar_fill_id_command(&self) -> bool {
        self.needs_planar_fill_id_command
    }

    fn sync_named_commands_3d(&mut self) {
        if let Some(cmd) = &mut self.original_command {
            cmd.command.model_matrix = Some(self.model_matrix);
            cmd.command.bounding_volume = Some(self.bounding_volume);
        }
        if let Some(cmd) = &mut self.translucent_command {
            cmd.command.model_matrix = Some(self.model_matrix);
            cmd.command.bounding_volume = Some(self.bounding_volume);
        }
        if let Some(cmd) = &mut self.skip_lod_backface_command {
            cmd.command.model_matrix = Some(self.model_matrix);
            cmd.command.bounding_volume = Some(self.bounding_volume);
        }
        if let Some(cmd) = &mut self.skip_lod_stencil_command {
            cmd.command.model_matrix = Some(self.model_matrix);
            cmd.command.bounding_volume = Some(self.bounding_volume);
        }
        if let Some(cmd) = &mut self.silhouette_model_command {
            cmd.command.model_matrix = Some(self.model_matrix);
            cmd.command.bounding_volume = Some(self.bounding_volume);
        }
        if let Some(cmd) = &mut self.silhouette_color_command {
            cmd.command.model_matrix = Some(self.model_matrix);
            cmd.command.bounding_volume = Some(self.bounding_volume);
        }
        if let Some(cmd) = &mut self.edge_command {
            cmd.command.model_matrix = Some(self.model_matrix);
            cmd.command.bounding_volume = Some(self.bounding_volume);
        }
        if let Some(cmd) = &mut self.edge_snap_command {
            cmd.command.model_matrix = Some(self.model_matrix);
            cmd.command.bounding_volume = Some(self.bounding_volume);
        }
        if let Some(cmd) = &mut self.planar_fill_id_command {
            cmd.command.model_matrix = Some(self.model_matrix);
            cmd.command.bounding_volume = Some(self.bounding_volume);
        }
    }

    fn sync_named_commands(&mut self) {
        // Sync modified command properties from derived_commands back to the named fields
        let mut idx = 0;
        if let Some(orig_cmd) = &mut self.original_command {
            if idx < self.derived_commands.len() {
                orig_cmd.command = self.derived_commands[idx].command.clone();
                idx += 1;
            }
        }
        if let Some(trans_cmd) = &mut self.translucent_command {
            if idx < self.derived_commands.len() {
                trans_cmd.command = self.derived_commands[idx].command.clone();
                idx += 1;
            }
        }
        if let Some(back_cmd) = &mut self.skip_lod_backface_command {
            if idx < self.derived_commands.len() {
                back_cmd.command = self.derived_commands[idx].command.clone();
                idx += 1;
            }
        }
        if let Some(sten_cmd) = &mut self.skip_lod_stencil_command {
            if idx < self.derived_commands.len() {
                sten_cmd.command = self.derived_commands[idx].command.clone();
                idx += 1;
            }
        }
        if let Some(sil_m) = &mut self.silhouette_model_command {
            if idx < self.derived_commands.len() {
                sil_m.command = self.derived_commands[idx].command.clone();
                idx += 1;
            }
        }
        if let Some(sil_c) = &mut self.silhouette_color_command {
            if idx < self.derived_commands.len() {
                sil_c.command = self.derived_commands[idx].command.clone();
                idx += 1;
            }
        }
        if let Some(edge) = &mut self.edge_command {
            if idx < self.derived_commands.len() {
                edge.command = self.derived_commands[idx].command.clone();
                idx += 1;
            }
        }
        if let Some(snap) = &mut self.edge_snap_command {
            if idx < self.derived_commands.len() {
                snap.command = self.derived_commands[idx].command.clone();
                idx += 1;
            }
        }
        if let Some(planar) = &mut self.planar_fill_id_command {
            if idx < self.derived_commands.len() {
                planar.command = self.derived_commands[idx].command.clone();
            }
        }
    }

    pub fn should_use_2d_commands(&self, frame_state: &ModelDrawCommandFrameState) -> bool {
        if frame_state.mode != SceneMode::Scene2D || self.model.project_to_2d {
            return false;
        }

        let bs = &self.model.bounding_sphere_2d;
        let left = bs.center.y - bs.radius;
        let right = bs.center.y + bs.radius;
        let idl_2d = frame_state.ellipsoid_maximum_radius * PI;

        (left < idl_2d && right > idl_2d) || (left < -idl_2d && right > -idl_2d)
    }

    pub fn update_model_matrix_2d(&mut self, frame_state: &ModelDrawCommandFrameState) {
        self.model_matrix_2d = self.model_matrix;
        // In column-major 4x4 matrix, translation y is at index [13], or w_axis.y
        let y = self.model_matrix.w_axis.y;
        self.model_matrix_2d.w_axis.y -=
            math_sign(y) * 2.0 * PI * frame_state.ellipsoid_maximum_radius;

        self.bounding_volume_2d = self
            .runtime_primitive
            .bounding_sphere
            .transform(self.model_matrix_2d)
            .unwrap_or(self.runtime_primitive.bounding_sphere);

        // Update 2D commands
        for cmd in &mut self.derived_commands {
            if cmd.is_2d {
                cmd.command.model_matrix = Some(self.model_matrix_2d);
                cmd.command.bounding_volume = Some(self.bounding_volume_2d);
            }
        }
        self.sync_2d_commands_to_named();
    }

    fn sync_2d_commands_to_named(&mut self) {
        let mm2d = self.model_matrix_2d;
        let bv2d = self.bounding_volume_2d;

        let update_field_2d = |field: &mut Option<ModelDerivedCommand>| {
            if let Some(f) = field {
                if let Some(c2d) = &mut f.derived_command_2d {
                    c2d.command.model_matrix = Some(mm2d);
                    c2d.command.bounding_volume = Some(bv2d);
                }
            }
        };

        update_field_2d(&mut self.original_command);
        update_field_2d(&mut self.translucent_command);
        update_field_2d(&mut self.skip_lod_backface_command);
        update_field_2d(&mut self.skip_lod_stencil_command);
        update_field_2d(&mut self.silhouette_model_command);
        update_field_2d(&mut self.silhouette_color_command);
        update_field_2d(&mut self.edge_command);
        update_field_2d(&mut self.edge_snap_command);
        update_field_2d(&mut self.planar_fill_id_command);
    }

    pub fn derive_2d_commands(&mut self) {
        let mm2d = self.model_matrix_2d;
        let bv2d = self.bounding_volume_2d;

        let derive_one = |field: &mut Option<ModelDerivedCommand>,
                          derived_commands: &mut Vec<ModelDerivedCommand>| {
            if let Some(f) = field {
                let mut cmd2d = f.command.clone();
                cmd2d.model_matrix = Some(mm2d);
                cmd2d.bounding_volume = Some(bv2d);
                let derived2d = ModelDerivedCommand::new(
                    cmd2d,
                    false, // Shadows disabled for 2D
                    f.update_back_face_culling,
                    f.update_cull_face,
                    f.update_debug_show_bounding_volume,
                    true,
                );
                f.derived_command_2d = Some(Box::new(derived2d.clone()));
                derived_commands.push(derived2d);
            }
        };

        derive_one(&mut self.original_command, &mut self.derived_commands);
        derive_one(&mut self.translucent_command, &mut self.derived_commands);
        derive_one(
            &mut self.skip_lod_backface_command,
            &mut self.derived_commands,
        );
        derive_one(&mut self.skip_lod_stencil_command, &mut self.derived_commands);
        derive_one(&mut self.silhouette_model_command, &mut self.derived_commands);
        derive_one(&mut self.silhouette_color_command, &mut self.derived_commands);
        derive_one(&mut self.edge_command, &mut self.derived_commands);
        derive_one(&mut self.edge_snap_command, &mut self.derived_commands);
        derive_one(&mut self.planar_fill_id_command, &mut self.derived_commands);

        self.has_2d_commands = true;
    }

    pub fn push_commands(
        &mut self,
        frame_state: &ModelDrawCommandFrameState,
        result: &mut Vec<DrawCommand>,
    ) {
        let use_2d = self.should_use_2d_commands(frame_state);

        if use_2d && !self.has_2d_commands {
            self.derive_2d_commands();
            self.model_matrix_2d_dirty = true;
        }

        if self.model_matrix_2d_dirty {
            self.update_model_matrix_2d(frame_state);
            self.model_matrix_2d_dirty = false;
        }

        if self.needs_translucent_command {
            if let Some(style) = self.model.style_commands_needed {
                if style != StyleCommandsNeeded::AllOpaque {
                    if let Some(cmd) = &self.translucent_command {
                        push_command_pair(result, cmd, use_2d);
                    }
                }
                if style == StyleCommandsNeeded::AllTranslucent {
                    return;
                }
            }
        }

        if self.needs_skip_level_of_detail_commands {
            if let Some(content) = &self.model.content {
                let has_mixed = content.tileset.has_mixed_content;
                let final_res = content.tile.final_resolution;
                let sel_depth = content.tile.selection_depth;

                if has_mixed {
                    if !final_res {
                        if let Some(back_cmd) = &self.skip_lod_backface_command {
                            if let Some(content_mut) = &mut self.model.content {
                                push_command_pair(
                                    &mut content_mut.tileset.backface_commands,
                                    back_cmd,
                                    use_2d,
                                );
                            }
                        }
                    }

                    self.update_skip_lod_stencil_command(sel_depth, use_2d);
                    if let Some(stencil_cmd) = &self.skip_lod_stencil_command {
                        push_command_pair(result, stencil_cmd, use_2d);
                    }
                    return;
                }
            }
        }

        if self.needs_silhouette_commands {
            if let Some(sil_model_cmd) = &self.silhouette_model_command {
                push_command_pair(result, sil_model_cmd, use_2d);
            }
            return;
        }

        if self.model.edge_display_mode == EdgeDisplayMode::EdgesOnly && self.needs_edge_commands {
            return;
        }

        if let Some(orig) = &self.original_command {
            push_command_pair(result, orig, use_2d);
        }
    }

    pub fn push_silhouette_commands(
        &self,
        frame_state: &ModelDrawCommandFrameState,
        result: &mut Vec<DrawCommand>,
    ) {
        let use_2d = self.should_use_2d_commands(frame_state);
        if let Some(cmd) = &self.silhouette_color_command {
            push_command_pair(result, cmd, use_2d);
        }
    }

    pub fn push_edge_commands(
        &mut self,
        frame_state: &mut ModelDrawCommandFrameState,
        result: &mut Vec<DrawCommand>,
    ) {
        if self.edge_command.is_none() {
            return;
        }

        let mode = self.model.edge_display_mode;
        let passes = frame_state.passes;
        let use_2d = self.should_use_2d_commands(frame_state);

        if passes.snap {
            if let Some(cmd) = &mut self.edge_snap_command {
                cmd.command.pass = Some(Pass::Cesium3dTileEdgesDirect);
                if let Some(cmd2d) = &mut cmd.derived_command_2d {
                    cmd2d.command.pass = Some(Pass::Cesium3dTileEdgesDirect);
                }
                push_command_pair(result, cmd, use_2d);
            }
            return;
        }

        if passes.pick {
            if mode != EdgeDisplayMode::EdgesOnly {
                return;
            }
            if let Some(cmd) = &mut self.edge_command {
                cmd.command.pass = Some(Pass::Cesium3dTileEdgesDirect);
                if let Some(cmd2d) = &mut cmd.derived_command_2d {
                    cmd2d.command.pass = Some(Pass::Cesium3dTileEdgesDirect);
                }
                push_command_pair(result, cmd, use_2d);
            }
            return;
        }

        if mode == EdgeDisplayMode::SurfacesOnly {
            return;
        }

        let edge_pass = if mode == EdgeDisplayMode::EdgesOnly {
            Pass::Cesium3dTileEdgesDirect
        } else {
            Pass::Cesium3dTileEdges
        };

        if edge_pass == Pass::Cesium3dTileEdges {
            frame_state.edge_visibility_requested = true;
        }

        if let Some(cmd) = &mut self.edge_command {
            cmd.command.pass = Some(edge_pass);
            if let Some(cmd2d) = &mut cmd.derived_command_2d {
                cmd2d.command.pass = Some(edge_pass);
            }
            push_command_pair(result, cmd, use_2d);
        }
    }

    pub fn push_planar_fill_id_commands(
        &self,
        frame_state: &ModelDrawCommandFrameState,
        result: &mut Vec<DrawCommand>,
    ) {
        if let Some(cmd) = &self.planar_fill_id_command {
            let use_2d = self.should_use_2d_commands(frame_state);
            push_command_pair(result, cmd, use_2d);
        }
    }

    fn update_skip_lod_stencil_command(&mut self, selection_depth: u32, use_2d: bool) {
        if let Some(sten) = &mut self.skip_lod_stencil_command {
            let last_depth = get_last_selection_depth(&sten.command);
            if selection_depth != last_depth {
                let skip_lod_ref = get_stencil_reference(selection_depth);
                let mut rs = get_render_state(&sten.command);
                rs.stencil_test.reference = skip_lod_ref;
                let handle = RenderStateHandle::State(Arc::new(rs.clone()));
                sten.command.render_state = Some(handle);

                if use_2d {
                    if let Some(cmd2d) = &mut sten.derived_command_2d {
                        cmd2d.command.render_state = Some(RenderStateHandle::State(Arc::new(rs)));
                    }
                }
            }
        }
    }
}

fn push_command_pair(list: &mut Vec<DrawCommand>, cmd: &ModelDerivedCommand, use_2d: bool) {
    list.push(cmd.command.clone());
    if use_2d {
        if let Some(cmd2d) = &cmd.derived_command_2d {
            list.push(cmd2d.command.clone());
        }
    }
}

fn get_last_selection_depth(command: &DrawCommand) -> u32 {
    match &command.render_state {
        Some(RenderStateHandle::State(rs)) => {
            let reference = rs.stencil_test.reference as u32;
            (reference & StencilConstants::SKIP_LOD_MASK) >> StencilConstants::SKIP_LOD_BIT_SHIFT
        }
        _ => 0,
    }
}

fn get_stencil_reference(selection_depth: u32) -> i32 {
    (StencilConstants::CESIUM_3D_TILE_MASK
        | (selection_depth << StencilConstants::SKIP_LOD_BIT_SHIFT)) as i32
}

pub fn derive_translucent_command(command: &DrawCommand) -> DrawCommand {
    let mut derived = DrawCommand::shallow_clone(Some(command), None).unwrap_or_default();
    derived.pass = Some(Pass::Translucent);
    let mut rs = get_render_state(command);
    rs.cull.enabled = false;
    rs.depth_mask = false;
    rs.blending = BlendingState::ALPHA_BLEND;
    derived.render_state = Some(RenderStateHandle::State(Arc::new(rs)));
    derived
}

pub fn derive_silhouette_model_command(
    command: &DrawCommand,
    silhouette_id: u32,
    is_invisible: bool,
) -> DrawCommand {
    let stencil_reference = (silhouette_id % 255) as i32;
    let mut derived = DrawCommand::shallow_clone(Some(command), None).unwrap_or_default();
    let mut rs = get_render_state(command);
    rs.stencil_test = StencilTestState {
        enabled: true,
        front_function: StencilFunction::Always,
        back_function: StencilFunction::Always,
        reference: stencil_reference,
        mask: !0,
        front_operation: StencilOperationState {
            fail: StencilOperation::Keep,
            z_fail: StencilOperation::Keep,
            z_pass: StencilOperation::Replace,
        },
        back_operation: StencilOperationState {
            fail: StencilOperation::Keep,
            z_fail: StencilOperation::Keep,
            z_pass: StencilOperation::Replace,
        },
    };
    if is_invisible {
        rs.color_mask = ColorMaskState {
            red: false,
            green: false,
            blue: false,
            alpha: false,
        };
    }
    derived.render_state = Some(RenderStateHandle::State(Arc::new(rs)));
    derived
}

pub fn derive_silhouette_color_command(
    command: &DrawCommand,
    silhouette_id: u32,
    silhouette_color: Color,
) -> DrawCommand {
    let stencil_reference = (silhouette_id % 255) as i32;
    let mut derived = DrawCommand::shallow_clone(Some(command), None).unwrap_or_default();
    let mut rs = get_render_state(command);
    rs.cull.enabled = false;

    let silhouette_translucent =
        command.pass == Some(Pass::Translucent) || silhouette_color.alpha < 1.0;
    if silhouette_translucent {
        derived.pass = Some(Pass::Translucent);
        rs.depth_mask = false;
        rs.blending = BlendingState::ALPHA_BLEND;
    }

    rs.stencil_test = StencilTestState {
        enabled: true,
        front_function: StencilFunction::NotEqual,
        back_function: StencilFunction::NotEqual,
        reference: stencil_reference,
        mask: !0,
        front_operation: StencilOperationState {
            fail: StencilOperation::Keep,
            z_fail: StencilOperation::Keep,
            z_pass: StencilOperation::Keep,
        },
        back_operation: StencilOperationState {
            fail: StencilOperation::Keep,
            z_fail: StencilOperation::Keep,
            z_pass: StencilOperation::Keep,
        },
    };

    let mut uniform_map = match &command.uniform_map {
        Some(UniformMapHandle::UniformMap(map)) => (**map).clone(),
        _ => UniformMap::new(),
    };
    uniform_map.set_value("model_silhouettePass", UniformValue::Bool(true));

    derived.render_state = Some(RenderStateHandle::State(Arc::new(rs)));
    derived.uniform_map = Some(UniformMapHandle::from(uniform_map));
    derived.cast_shadows = false;
    derived.receive_shadows = false;
    derived
}

pub fn derive_skip_lod_backface_command(command: &DrawCommand) -> DrawCommand {
    let mut derived = DrawCommand::shallow_clone(Some(command), None).unwrap_or_default();
    let mut rs = get_render_state(command);
    rs.cull.enabled = true;
    rs.cull.face = CullFace::Front;
    rs.color_mask = ColorMaskState {
        red: false,
        green: false,
        blue: false,
        alpha: false,
    };
    rs.polygon_offset = PolygonOffsetState {
        enabled: true,
        factor: 5.0,
        units: 5.0,
    };

    let mut uniform_map = match &command.uniform_map {
        Some(UniformMapHandle::UniformMap(map)) => (**map).clone(),
        _ => UniformMap::new(),
    };
    uniform_map.set_value("u_polygonOffset", UniformValue::Vec2([5.0, 5.0]));

    derived.render_state = Some(RenderStateHandle::State(Arc::new(rs)));
    derived.uniform_map = Some(UniformMapHandle::from(uniform_map));
    derived.cast_shadows = false;
    derived.receive_shadows = false;
    derived
}

pub fn derive_skip_lod_stencil_command(command: &DrawCommand) -> DrawCommand {
    let mut derived = DrawCommand::shallow_clone(Some(command), None).unwrap_or_default();
    let mut rs = get_render_state(command);
    rs.stencil_test = StencilTestState {
        enabled: true,
        front_function: StencilFunction::GEqual,
        back_function: StencilFunction::GEqual,
        reference: StencilConstants::CESIUM_3D_TILE_MASK as i32,
        mask: StencilConstants::SKIP_LOD_MASK,
        front_operation: StencilOperationState {
            fail: StencilOperation::Keep,
            z_fail: StencilOperation::Keep,
            z_pass: StencilOperation::Replace,
        },
        back_operation: StencilOperationState {
            fail: StencilOperation::Keep,
            z_fail: StencilOperation::Keep,
            z_pass: StencilOperation::Replace,
        },
    };
    rs.stencil_mask = StencilConstants::CESIUM_3D_TILE_MASK | StencilConstants::SKIP_LOD_MASK;
    derived.render_state = Some(RenderStateHandle::State(Arc::new(rs)));
    derived
}

pub fn derive_edge_command(
    command: &DrawCommand,
    edge_geometry: &ModelDrawCommandEdgeGeometry,
    line_width: f32,
) -> DrawCommand {
    let mut derived = DrawCommand::shallow_clone(Some(command), None).unwrap_or_default();
    derived.vertex_array = edge_geometry.vertex_array.clone();
    derived.primitive_type = edge_geometry.primitive_type;
    derived.count = Some(edge_geometry.index_count);
    if let Some(shader_program) = &edge_geometry.shader_program {
        derived.shader_program = Some(shader_program.clone());
    }
    derived.pass = Some(edge_geometry.pass);

    let mut uniform_map = match &command.uniform_map {
        Some(UniformMapHandle::UniformMap(map)) => (**map).clone(),
        _ => UniformMap::new(),
    };
    uniform_map.set_value("u_isEdgePass", UniformValue::Bool(true));
    uniform_map.set_value("u_lineWidth", UniformValue::Float(line_width));

    derived.uniform_map = Some(UniformMapHandle::from(uniform_map));
    derived.cast_shadows = false;
    derived.receive_shadows = false;
    derived
}

pub fn derive_planar_fill_id_command(command: &DrawCommand) -> DrawCommand {
    let mut derived = DrawCommand::shallow_clone(Some(command), None).unwrap_or_default();
    derived.pass = Some(Pass::Cesium3dTilePlanarFillId);
    derived.cast_shadows = false;
    derived.receive_shadows = false;

    let mut rs = get_render_state(command);
    rs.blending.enabled = false;
    derived.render_state = Some(RenderStateHandle::State(Arc::new(rs)));

    let mut uniform_map = match &command.uniform_map {
        Some(UniformMapHandle::UniformMap(map)) => (**map).clone(),
        _ => UniformMap::new(),
    };
    uniform_map.set_value("u_isPlanarFillIdPass", UniformValue::Bool(true));
    derived.uniform_map = Some(UniformMapHandle::from(uniform_map));

    derived
}
