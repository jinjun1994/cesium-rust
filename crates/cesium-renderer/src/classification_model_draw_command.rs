//! ClassificationModelDrawCommand matching CesiumJS `Scene/Model/ClassificationModelDrawCommand.js` (Apache-2.0).

use crate::commands::{DrawCommand, Pass, RenderStateHandle};
use crate::render_state::{
    BlendingState, ColorMaskState, CullFace, DepthFunction, DepthTestState, RenderState,
    StencilFunction, StencilOperation, StencilOperationState, StencilTestState,
};
use cesium_math::{BoundingSphere, ClassificationType, DeveloperError};
use glam::DMat4;
use std::sync::Arc;

/// Stencil bit mask constants matching CesiumJS `Scene/StencilConstants.js`.
pub struct StencilConstants;

impl StencilConstants {
    pub const CESIUM_3D_TILE_MASK: u32 = 0x80;
    pub const SKIP_LOD_MASK: u32 = 0x70;
    pub const SKIP_LOD_BIT_SHIFT: u32 = 4;
    pub const CLASSIFICATION_MASK: u32 = 0x0f;
}

/// Model options used by ClassificationModelDrawCommand.
#[derive(Clone, Debug, PartialEq)]
pub struct ClassificationModelOptions {
    pub classification_type: ClassificationType,
    pub enable_debug_wireframe: bool,
    pub debug_wireframe: bool,
    pub allow_picking: bool,
}

impl Default for ClassificationModelOptions {
    fn default() -> Self {
        Self {
            classification_type: ClassificationType::Both,
            enable_debug_wireframe: false,
            debug_wireframe: false,
            allow_picking: false,
        }
    }
}

/// Runtime primitive used by ClassificationModelDrawCommand.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClassificationRuntimePrimitive {
    pub bounding_sphere: BoundingSphere,
    pub batch_lengths: Vec<usize>,
    pub batch_offsets: Vec<usize>,
}

/// Primitive render resources for classification command derivation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClassificationPrimitiveRenderResources {
    pub model: ClassificationModelOptions,
    pub runtime_primitive: ClassificationRuntimePrimitive,
    pub pick_id: Option<String>,
}

/// Passes state for classification command pushing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClassificationFramePasses {
    pub render: bool,
    pub pick: bool,
}

/// Frame state subset for classification command pushing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClassificationFrameState {
    pub passes: ClassificationFramePasses,
    pub invert_classification: bool,
}

/// Wrapper around draw commands used to render a classification model.
#[derive(Clone, Debug, PartialEq)]
pub struct ClassificationModelDrawCommand {
    command: DrawCommand,
    model: ClassificationModelOptions,
    runtime_primitive: ClassificationRuntimePrimitive,
    model_matrix: DMat4,
    bounding_volume: BoundingSphere,
    cull_face: CullFace,
    classification_type: ClassificationType,
    classifies_terrain: bool,
    classifies_3d_tiles: bool,
    use_debug_wireframe: bool,
    pick_id: Option<String>,

    command_list_terrain: Vec<DrawCommand>,
    command_list_3d_tiles: Vec<DrawCommand>,
    command_list_ignore_show: Vec<DrawCommand>,
    command_list_debug_wireframe: Vec<DrawCommand>,

    command_list_terrain_picking: Vec<DrawCommand>,
    command_list_3d_tiles_picking: Vec<DrawCommand>,
}

fn get_stencil_depth_render_state(stencil_function: StencilFunction) -> RenderState {
    RenderState {
        color_mask: ColorMaskState {
            red: false,
            green: false,
            blue: false,
            alpha: false,
        },
        stencil_test: StencilTestState {
            enabled: true,
            front_function: stencil_function,
            front_operation: StencilOperationState {
                fail: StencilOperation::Keep,
                z_fail: StencilOperation::DecrWrap,
                z_pass: StencilOperation::Keep,
            },
            back_function: stencil_function,
            back_operation: StencilOperationState {
                fail: StencilOperation::Keep,
                z_fail: StencilOperation::IncrWrap,
                z_pass: StencilOperation::Keep,
            },
            reference: StencilConstants::CESIUM_3D_TILE_MASK as i32,
            mask: StencilConstants::CESIUM_3D_TILE_MASK,
        },
        stencil_mask: StencilConstants::CLASSIFICATION_MASK,
        depth_test: DepthTestState {
            enabled: true,
            func: DepthFunction::LEqual,
        },
        depth_mask: false,
        ..Default::default()
    }
}

fn get_color_render_state() -> RenderState {
    RenderState {
        stencil_test: StencilTestState {
            enabled: true,
            front_function: StencilFunction::NotEqual,
            front_operation: StencilOperationState {
                fail: StencilOperation::Zero,
                z_fail: StencilOperation::Zero,
                z_pass: StencilOperation::Zero,
            },
            back_function: StencilFunction::NotEqual,
            back_operation: StencilOperationState {
                fail: StencilOperation::Zero,
                z_fail: StencilOperation::Zero,
                z_pass: StencilOperation::Zero,
            },
            reference: 0,
            mask: StencilConstants::CLASSIFICATION_MASK,
        },
        stencil_mask: StencilConstants::CLASSIFICATION_MASK,
        depth_test: DepthTestState {
            enabled: false,
            func: DepthFunction::Less,
        },
        depth_mask: false,
        blending: BlendingState::PRE_MULTIPLIED_ALPHA_BLEND,
        ..Default::default()
    }
}

fn get_pick_render_state() -> RenderState {
    let mut state = get_color_render_state();
    state.blending.enabled = false;
    state
}

fn derive_stencil_depth_command(command: &DrawCommand, pass: Pass) -> DrawCommand {
    let mut derived = command.clone();
    derived.cull = false;
    derived.pass = Some(pass);

    let stencil_function = if pass == Pass::TerrainClassification {
        StencilFunction::Always
    } else {
        StencilFunction::Equal
    };

    let rs = get_stencil_depth_render_state(stencil_function);
    derived.render_state = Some(RenderStateHandle::State(Arc::new(rs)));
    derived
}

fn derive_color_command(command: &DrawCommand, pass: Pass) -> DrawCommand {
    let mut derived = command.clone();
    derived.cull = false;
    derived.pass = Some(pass);

    let rs = get_color_render_state();
    derived.render_state = Some(RenderStateHandle::State(Arc::new(rs)));
    derived
}

fn create_batch_commands(
    runtime_primitive: &ClassificationRuntimePrimitive,
    derived_commands: &[DrawCommand],
    result: &mut Vec<DrawCommand>,
) {
    let batch_lengths = &runtime_primitive.batch_lengths;
    let batch_offsets = &runtime_primitive.batch_offsets;

    for (i, &batch_length) in batch_lengths.iter().enumerate() {
        let batch_offset = batch_offsets[i];
        for derived_command in derived_commands {
            let mut batch_command = derived_command.clone();
            batch_command.count = Some(batch_length);
            batch_command.offset = batch_offset;
            result.push(batch_command);
        }
    }
}

fn create_pick_commands(
    runtime_primitive: &ClassificationRuntimePrimitive,
    derived_commands: &[DrawCommand],
    pick_id: Option<String>,
    result: &mut Vec<DrawCommand>,
) {
    let pick_rs = get_pick_render_state();
    let stencil_depth_command = &derived_commands[0];
    let color_command = &derived_commands[1];

    let mut pick_stencil_depth_command = stencil_depth_command.clone();
    pick_stencil_depth_command.cull = true;
    pick_stencil_depth_command.pick_only = true;

    let mut pick_color_command = color_command.clone();
    pick_color_command.cull = true;
    pick_color_command.pick_only = true;
    pick_color_command.render_state = Some(RenderStateHandle::State(Arc::new(pick_rs)));
    pick_color_command.pick_id = pick_id;

    let pick_derived = [pick_stencil_depth_command, pick_color_command];
    create_batch_commands(runtime_primitive, &pick_derived, result);
}

impl ClassificationModelDrawCommand {
    /// Creates a new ClassificationModelDrawCommand from command and primitive render resources.
    pub fn new(
        command: Option<DrawCommand>,
        primitive_render_resources: Option<ClassificationPrimitiveRenderResources>,
    ) -> Result<Self, DeveloperError> {
        let command = command
            .ok_or_else(|| DeveloperError::new("options.command is required."))?;
        let resources = primitive_render_resources
            .ok_or_else(|| DeveloperError::new("options.primitiveRenderResources is required."))?;

        let model = resources.model;
        let runtime_primitive = resources.runtime_primitive;

        let model_matrix = command.model_matrix.unwrap_or(DMat4::IDENTITY);
        let bounding_volume = command.bounding_volume.unwrap_or_default();
        let cull_face = match command.render_state {
            Some(RenderStateHandle::State(ref rs)) => rs.cull.face,
            _ => CullFace::Back,
        };

        let classification_type = model.classification_type;
        let classifies_terrain = classification_type != ClassificationType::Cesium3DTile;
        let classifies_3d_tiles = classification_type != ClassificationType::Terrain;
        let use_debug_wireframe = model.enable_debug_wireframe && model.debug_wireframe;
        let pick_id = resources.pick_id;

        let mut instance = Self {
            command,
            model,
            runtime_primitive,
            model_matrix,
            bounding_volume,
            cull_face,
            classification_type,
            classifies_terrain,
            classifies_3d_tiles,
            use_debug_wireframe,
            pick_id,
            command_list_terrain: Vec::new(),
            command_list_3d_tiles: Vec::new(),
            command_list_ignore_show: Vec::new(),
            command_list_debug_wireframe: Vec::new(),
            command_list_terrain_picking: Vec::new(),
            command_list_3d_tiles_picking: Vec::new(),
        };

        instance.initialize();
        Ok(instance)
    }

    fn initialize(&mut self) {
        if self.use_debug_wireframe {
            let mut wireframe_command = self.command.clone();
            wireframe_command.pass = Some(Pass::Opaque);

            let mut batch_commands = Vec::new();
            create_batch_commands(
                &self.runtime_primitive,
                &[wireframe_command],
                &mut batch_commands,
            );

            for cmd in &mut batch_commands {
                if let Some(c) = cmd.count {
                    cmd.count = Some(c * 2);
                }
                cmd.offset *= 2;
            }

            self.command_list_debug_wireframe = batch_commands;
            return;
        }

        let allow_picking = self.model.allow_picking;

        if self.classifies_terrain {
            let pass = Pass::TerrainClassification;
            let stencil_depth_command = derive_stencil_depth_command(&self.command, pass);
            let color_command = derive_color_command(&self.command, pass);

            let derived = [stencil_depth_command, color_command];
            create_batch_commands(
                &self.runtime_primitive,
                &derived,
                &mut self.command_list_terrain,
            );

            if allow_picking {
                create_pick_commands(
                    &self.runtime_primitive,
                    &derived,
                    self.pick_id.clone(),
                    &mut self.command_list_terrain_picking,
                );
            }
        }

        if self.classifies_3d_tiles {
            let pass = Pass::Cesium3dTileClassification;
            let stencil_depth_command = derive_stencil_depth_command(&self.command, pass);
            let color_command = derive_color_command(&self.command, pass);

            let derived = [stencil_depth_command, color_command];
            create_batch_commands(
                &self.runtime_primitive,
                &derived,
                &mut self.command_list_3d_tiles,
            );

            if allow_picking {
                create_pick_commands(
                    &self.runtime_primitive,
                    &derived,
                    self.pick_id.clone(),
                    &mut self.command_list_3d_tiles_picking,
                );
            }
        }
    }

    pub fn command(&self) -> &DrawCommand {
        &self.command
    }

    pub fn runtime_primitive(&self) -> &ClassificationRuntimePrimitive {
        &self.runtime_primitive
    }

    pub fn batch_lengths(&self) -> &[usize] {
        &self.runtime_primitive.batch_lengths
    }

    pub fn batch_offsets(&self) -> &[usize] {
        &self.runtime_primitive.batch_offsets
    }

    pub fn model(&self) -> &ClassificationModelOptions {
        &self.model
    }

    pub fn classification_type(&self) -> ClassificationType {
        self.classification_type
    }

    pub fn classifies_terrain(&self) -> bool {
        self.classifies_terrain
    }

    pub fn classifies_3d_tiles(&self) -> bool {
        self.classifies_3d_tiles
    }

    pub fn use_debug_wireframe(&self) -> bool {
        self.use_debug_wireframe
    }

    pub fn pick_id(&self) -> Option<&str> {
        self.pick_id.as_deref()
    }

    pub fn model_matrix(&self) -> DMat4 {
        self.model_matrix
    }

    pub fn bounding_volume(&self) -> BoundingSphere {
        self.bounding_volume
    }

    pub fn cull_face(&self) -> CullFace {
        self.cull_face
    }

    pub fn set_cull_face(&mut self, cull_face: CullFace) {
        self.cull_face = cull_face;
    }

    pub fn set_model_matrix(&mut self, model_matrix: DMat4) {
        self.model_matrix = model_matrix;
        self.bounding_volume = self
            .runtime_primitive
            .bounding_sphere
            .transformed(self.model_matrix);

        let update_cmd = |cmd: &mut DrawCommand, mm: DMat4, bv: BoundingSphere| {
            cmd.model_matrix = Some(mm);
            cmd.bounding_volume = Some(bv);
        };

        for cmd in &mut self.command_list_terrain {
            update_cmd(cmd, self.model_matrix, self.bounding_volume);
        }
        for cmd in &mut self.command_list_3d_tiles {
            update_cmd(cmd, self.model_matrix, self.bounding_volume);
        }
        for cmd in &mut self.command_list_ignore_show {
            update_cmd(cmd, self.model_matrix, self.bounding_volume);
        }
        for cmd in &mut self.command_list_debug_wireframe {
            update_cmd(cmd, self.model_matrix, self.bounding_volume);
        }
        for cmd in &mut self.command_list_terrain_picking {
            update_cmd(cmd, self.model_matrix, self.bounding_volume);
        }
        for cmd in &mut self.command_list_3d_tiles_picking {
            update_cmd(cmd, self.model_matrix, self.bounding_volume);
        }
    }

    pub fn command_list_terrain(&self) -> &[DrawCommand] {
        &self.command_list_terrain
    }

    pub fn command_list_3d_tiles(&self) -> &[DrawCommand] {
        &self.command_list_3d_tiles
    }

    pub fn command_list_ignore_show(&self) -> &[DrawCommand] {
        &self.command_list_ignore_show
    }

    pub fn command_list_debug_wireframe(&self) -> &[DrawCommand] {
        &self.command_list_debug_wireframe
    }

    pub fn command_list_terrain_picking(&self) -> &[DrawCommand] {
        &self.command_list_terrain_picking
    }

    pub fn command_list_3d_tiles_picking(&self) -> &[DrawCommand] {
        &self.command_list_3d_tiles_picking
    }

    /// Pushes the draw commands necessary to render the primitive.
    pub fn push_commands(
        &mut self,
        frame_state: &ClassificationFrameState,
        result: &mut Vec<DrawCommand>,
    ) {
        let passes = frame_state.passes;
        if passes.render {
            if self.use_debug_wireframe {
                result.extend(self.command_list_debug_wireframe.iter().cloned());
                return;
            }

            if self.classifies_terrain {
                result.extend(self.command_list_terrain.iter().cloned());
            }

            if self.classifies_3d_tiles {
                result.extend(self.command_list_3d_tiles.iter().cloned());
            }

            let use_ignore_show_commands =
                frame_state.invert_classification && self.classifies_3d_tiles;

            if use_ignore_show_commands {
                if self.command_list_ignore_show.is_empty() {
                    let pass = Pass::Cesium3dTileClassificationIgnoreShow;
                    let command = derive_stencil_depth_command(&self.command, pass);

                    let mut batch_commands = Vec::new();
                    create_batch_commands(
                        &self.runtime_primitive,
                        &[command],
                        &mut batch_commands,
                    );
                    for cmd in &mut batch_commands {
                        cmd.model_matrix = Some(self.model_matrix);
                        cmd.bounding_volume = Some(self.bounding_volume);
                    }
                    self.command_list_ignore_show = batch_commands;
                }

                result.extend(self.command_list_ignore_show.iter().cloned());
            }
        }

        if passes.pick {
            if self.classifies_terrain {
                result.extend(self.command_list_terrain_picking.iter().cloned());
            }

            if self.classifies_3d_tiles {
                result.extend(self.command_list_3d_tiles_picking.iter().cloned());
            }
        }
    }
}
