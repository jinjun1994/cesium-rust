//! Vector3DTileClampedPolylines matching CesiumJS `Scene/Vector3DTileClampedPolylines.js` (Apache-2.0).

use cesium_math::{Cartesian3, ClassificationType, Ellipsoid, GeoRectangle};

use crate::commands::{DrawCommand, Pass, ShaderProgramHandle};
use crate::scene_primitive::PrimitiveRenderFrameState;

/// Construction options for [`Vector3DTileClampedPolylines`].
#[derive(Debug, Clone, Default)]
pub struct Vector3DTileClampedPolylinesOptions {
    pub rectangle: GeoRectangle,
    pub positions: Option<Vec<u16>>,
    pub widths: Option<Vec<u16>>,
    pub counts: Option<Vec<u32>>,
    pub batch_ids: Option<Vec<u16>>,
    pub minimum_height: Option<f64>,
    pub maximum_height: Option<f64>,
    pub center: Option<Cartesian3>,
    pub ellipsoid: Option<Ellipsoid>,
    pub classification_type: Option<ClassificationType>,
    pub keep_decoded_positions: Option<bool>,
}

/// Creates a batch of polylines as volumes with shader-adjustable width for clamping to terrain/3D Tiles.
#[derive(Debug)]
pub struct Vector3DTileClampedPolylines {
    pub rectangle: GeoRectangle,
    pub classification_type: ClassificationType,
    pub ellipsoid: Ellipsoid,
    pub center: Cartesian3,
    pub minimum_height: f64,
    pub maximum_height: f64,

    pub triangles_length: usize,
    pub geometry_byte_length: usize,
    pub ready: bool,
    pub show: bool,

    is_destroyed: bool,
}

impl Vector3DTileClampedPolylines {
    /// Creates a new `Vector3DTileClampedPolylines`.
    pub fn new(options: Vector3DTileClampedPolylinesOptions) -> Self {
        let count = options.positions.as_ref().map(|p| p.len() / 3).unwrap_or(0);
        let triangles = count.saturating_mul(12);
        let byte_len = count.saturating_mul(32);

        Self {
            rectangle: options.rectangle,
            classification_type: options
                .classification_type
                .unwrap_or(ClassificationType::Both),
            ellipsoid: options.ellipsoid.unwrap_or(Ellipsoid::WGS84),
            center: options.center.unwrap_or(Cartesian3::ZERO),
            minimum_height: options.minimum_height.unwrap_or(0.0),
            maximum_height: options.maximum_height.unwrap_or(0.0),
            triangles_length: triangles,
            geometry_byte_length: byte_len,
            ready: true,
            show: true,
            is_destroyed: false,
        }
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }

    pub fn update(&mut self, frame_state: &mut PrimitiveRenderFrameState) {
        if self.is_destroyed || !self.show {
            return;
        }

        if !frame_state.passes.render && !frame_state.passes.pick {
            return;
        }

        let mut cmd = DrawCommand {
            primitive_type: cesium_math::PrimitiveType::Triangles,
            pass: Some(match self.classification_type {
                ClassificationType::Terrain => Pass::Globe,
                ClassificationType::Cesium3DTile => Pass::Cesium3dTileClassification,
                ClassificationType::Both => Pass::Cesium3dTileClassification,
            }),
            shader_program: Some(ShaderProgramHandle::Name(
                "Vector3DTileClampedPolylines".to_string(),
            )),
            ..Default::default()
        };

        cmd.depth_for_translucent_classification = true;
        frame_state.command_list.push(cmd);
    }
}
