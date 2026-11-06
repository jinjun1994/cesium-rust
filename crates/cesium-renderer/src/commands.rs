//! Renderer commands matching CesiumJS `Renderer/ClearCommand.js`, `Renderer/DrawCommand.js`, and `Renderer/Pass.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{BoundingSphere, Color, OrientedBoundingBox, PrimitiveType};
use glam::DMat4;

/// The render pass for a command, matching CesiumJS `Renderer/Pass.js`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Pass {
    Environment = 0,
    Compute = 1,
    Globe = 2,
    TerrainClassification = 3,
    Cesium3dTileEdges = 4,
    Cesium3dTilePlanarFillId = 5,
    Cesium3dTile = 6,
    Cesium3dTileClassification = 7,
    Cesium3dTileClassificationIgnoreShow = 8,
    Opaque = 9,
    Translucent = 10,
    Voxels = 11,
    GaussianSplats = 12,
    Cesium3dTileEdgesDirect = 13,
    Overlay = 14,
}

pub const NUMBER_OF_PASSES: usize = 15;

/// Options for configuring a ClearCommand.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClearCommandOptions {
    pub color: Option<Color>,
    pub depth: Option<f64>,
    pub stencil: Option<i32>,
    pub render_state: Option<String>,
    pub framebuffer: Option<String>,
    pub owner: Option<String>,
    pub pass: Option<Pass>,
}

/// Represents a command to the renderer for clearing a framebuffer.
#[derive(Clone, Debug, PartialEq)]
pub struct ClearCommand {
    pub color: Option<Color>,
    pub depth: Option<f64>,
    pub stencil: Option<i32>,
    pub render_state: Option<String>,
    pub framebuffer: Option<String>,
    pub owner: Option<String>,
    pub pass: Option<Pass>,
}

impl Default for ClearCommand {
    fn default() -> Self {
        Self::new(ClearCommandOptions::default())
    }
}

impl ClearCommand {
    /// Clears color to (0.0, 0.0, 0.0, 0.0); depth to 1.0; and stencil to 0.
    pub const ALL: Self = Self {
        color: Some(Color::new(0.0, 0.0, 0.0, 0.0)),
        depth: Some(1.0),
        stencil: Some(0),
        render_state: None,
        framebuffer: None,
        owner: None,
        pass: None,
    };

    /// Creates a new ClearCommand with options.
    pub fn new(options: ClearCommandOptions) -> Self {
        Self {
            color: options.color,
            depth: options.depth,
            stencil: options.stencil,
            render_state: options.render_state,
            framebuffer: options.framebuffer,
            owner: options.owner,
            pass: options.pass,
        }
    }
}

/// Options for configuring a DrawCommand.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawCommandOptions {
    pub bounding_volume: Option<BoundingSphere>,
    pub oriented_bounding_box: Option<OrientedBoundingBox>,
    pub model_matrix: Option<DMat4>,
    pub primitive_type: Option<PrimitiveType>,
    pub vertex_array: Option<String>,
    pub count: Option<usize>,
    pub offset: Option<usize>,
    pub instance_count: Option<usize>,
    pub shader_program: Option<String>,
    pub uniform_map: Option<HashMap<String, String>>,
    pub render_state: Option<String>,
    pub framebuffer: Option<String>,
    pub pass: Option<Pass>,
    pub owner: Option<String>,
    pub pick_id: Option<String>,
    pub snap_id: Option<String>,
    pub pick_metadata_allowed: Option<bool>,
    pub cull: Option<bool>,
    pub occlude: Option<bool>,
    pub execute_in_closest_frustum: Option<bool>,
    pub debug_show_bounding_volume: Option<bool>,
    pub cast_shadows: Option<bool>,
    pub receive_shadows: Option<bool>,
    pub pick_only: Option<bool>,
    pub depth_for_translucent_classification: Option<bool>,
}

/// Represents a command to the renderer for drawing.
#[derive(Clone, Debug, PartialEq)]
pub struct DrawCommand {
    pub bounding_volume: Option<BoundingSphere>,
    pub oriented_bounding_box: Option<OrientedBoundingBox>,
    pub model_matrix: Option<DMat4>,
    pub primitive_type: PrimitiveType,
    pub vertex_array: Option<String>,
    pub count: Option<usize>,
    pub offset: usize,
    pub instance_count: usize,
    pub shader_program: Option<String>,
    pub uniform_map: Option<HashMap<String, String>>,
    pub render_state: Option<String>,
    pub framebuffer: Option<String>,
    pub pass: Option<Pass>,
    pub owner: Option<String>,
    pub pick_id: Option<String>,
    pub snap_id: Option<String>,
    pub pick_metadata_allowed: bool,
    pub cull: bool,
    pub occlude: bool,
    pub execute_in_closest_frustum: bool,
    pub debug_show_bounding_volume: bool,
    pub debug_overlapping_frustums: i32,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
    pub pick_only: bool,
    pub depth_for_translucent_classification: bool,
    pub dirty: bool,
}

impl Default for DrawCommand {
    fn default() -> Self {
        Self::new(DrawCommandOptions::default())
    }
}

impl DrawCommand {
    /// Creates a new DrawCommand with the given options.
    pub fn new(options: DrawCommandOptions) -> Self {
        Self {
            bounding_volume: options.bounding_volume,
            oriented_bounding_box: options.oriented_bounding_box,
            model_matrix: options.model_matrix,
            primitive_type: options.primitive_type.unwrap_or(PrimitiveType::Triangles),
            vertex_array: options.vertex_array,
            count: options.count,
            offset: options.offset.unwrap_or(0),
            instance_count: options.instance_count.unwrap_or(0),
            shader_program: options.shader_program,
            uniform_map: options.uniform_map,
            render_state: options.render_state,
            framebuffer: options.framebuffer,
            pass: options.pass,
            owner: options.owner,
            pick_id: options.pick_id,
            snap_id: options.snap_id,
            pick_metadata_allowed: options.pick_metadata_allowed.unwrap_or(false),
            cull: options.cull.unwrap_or(true),
            occlude: options.occlude.unwrap_or(true),
            execute_in_closest_frustum: options.execute_in_closest_frustum.unwrap_or(false),
            debug_show_bounding_volume: options.debug_show_bounding_volume.unwrap_or(false),
            debug_overlapping_frustums: 0,
            cast_shadows: options.cast_shadows.unwrap_or(false),
            receive_shadows: options.receive_shadows.unwrap_or(false),
            pick_only: options.pick_only.unwrap_or(false),
            depth_for_translucent_classification: options
                .depth_for_translucent_classification
                .unwrap_or(false),
            dirty: true,
        }
    }

    /// Shallow clones a DrawCommand, optionally reusing an existing result.
    pub fn shallow_clone(
        command: Option<&DrawCommand>,
        result: Option<&mut DrawCommand>,
    ) -> Option<DrawCommand> {
        let cmd = command?;
        if let Some(res) = result {
            *res = cmd.clone();
            res.dirty = true;
            return Some(res.clone());
        }
        let mut cloned = cmd.clone();
        cloned.dirty = true;
        Some(cloned)
    }
}
