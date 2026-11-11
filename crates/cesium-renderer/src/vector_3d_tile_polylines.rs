//! Vector3DTilePolylines matching CesiumJS `Scene/Vector3DTilePolylines.js` (Apache-2.0).

use cesium_math::{BoundingSphere, Cartesian3, Color, Ellipsoid, GeoRectangle};

use crate::commands::{DrawCommand, Pass, ShaderProgramHandle};
use crate::model_3d_tile_content::Cesium3DTileStyle;
use crate::scene_primitive::PrimitiveRenderFrameState;
use crate::vector_3d_tile_content::Vector3DTileFeature;
use crate::vector_3d_tile_points::decode_positions;

/// Options to construct a [`Vector3DTilePolylines`].
#[derive(Debug, Clone, Default)]
pub struct Vector3DTilePolylinesOptions {
    pub positions: Vec<u16>,
    pub widths: Vec<u16>,
    pub counts: Vec<u32>,
    pub batch_ids: Vec<u16>,
    pub rectangle: GeoRectangle,
    pub minimum_height: f64,
    pub maximum_height: f64,
    pub center: Option<Cartesian3>,
    pub bounding_volume: Option<BoundingSphere>,
    pub keep_decoded_positions: bool,
}

/// Represents a batch of polylines draped on terrain or 3D Tiles.
#[derive(Debug)]
pub struct Vector3DTilePolylines {
    pub rectangle: GeoRectangle,
    pub minimum_height: f64,
    pub maximum_height: f64,
    pub center: Cartesian3,
    pub bounding_volume: Option<BoundingSphere>,
    pub keep_decoded_positions: bool,

    pub widths: Vec<u16>,
    pub counts: Vec<u32>,
    pub batch_ids: Vec<u16>,

    pub decoded_positions: Option<Vec<Cartesian3>>,
    pub decoded_position_offsets: Option<Vec<usize>>,

    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
    pub vertex_batch_ids: Vec<u16>,

    pub highlight_color: Color,
    pub triangles_length: usize,
    pub geometry_byte_length: usize,
    pub ready: bool,
    pub is_destroyed: bool,
}

impl Vector3DTilePolylines {
    /// Creates a new `Vector3DTilePolylines`.
    pub fn new(options: Vector3DTilePolylinesOptions) -> Self {
        let center = options
            .center
            .or_else(|| {
                options
                    .bounding_volume
                    .as_ref()
                    .map(|b| Cartesian3::new(b.center.x, b.center.y, b.center.z))
            })
            .or_else(|| {
                let c = options.rectangle.center();
                Ellipsoid::WGS84
                    .cartographic_to_cartesian(cesium_math::Cartographic::new(
                        c.longitude,
                        c.latitude,
                        (options.minimum_height + options.maximum_height) * 0.5,
                    ))
                    .map(|p| Cartesian3::new(p.x, p.y, p.z))
            })
            .unwrap_or(Cartesian3::ZERO);

        let mut instance = Self {
            rectangle: options.rectangle,
            minimum_height: options.minimum_height,
            maximum_height: options.maximum_height,
            center,
            bounding_volume: options.bounding_volume,
            keep_decoded_positions: options.keep_decoded_positions,
            widths: options.widths,
            counts: options.counts,
            batch_ids: options.batch_ids,
            decoded_positions: None,
            decoded_position_offsets: None,
            positions: Vec::new(),
            indices: Vec::new(),
            vertex_batch_ids: Vec::new(),
            highlight_color: Color::WHITE,
            triangles_length: 0,
            geometry_byte_length: 0,
            ready: false,
            is_destroyed: false,
        };

        instance.build_polylines(&options.positions);
        instance.ready = true;
        instance
    }

    fn build_polylines(&mut self, encoded_positions: &[u16]) {
        let decoded = decode_positions(
            &self.rectangle,
            self.minimum_height,
            self.maximum_height,
            encoded_positions,
        );

        let mut offsets = Vec::with_capacity(self.counts.len() + 1);
        let mut running_offset = 0;
        offsets.push(running_offset);

        for &cnt in &self.counts {
            running_offset += cnt as usize;
            offsets.push(running_offset);
        }

        if self.keep_decoded_positions {
            self.decoded_positions = Some(decoded.clone());
            self.decoded_position_offsets = Some(offsets.clone());
        }

        // Triangulate polylines: each segment between two points is expanded into a quad (2 triangles)
        let mut vert_start_idx = 0u32;
        for (line_idx, &count) in self.counts.iter().enumerate() {
            let start = offsets[line_idx];
            let end = offsets[line_idx + 1];
            let batch_id = self
                .batch_ids
                .get(line_idx)
                .copied()
                .unwrap_or(line_idx as u16);

            if end > start + 1 && end <= decoded.len() {
                for i in start..end - 1 {
                    let p0 = decoded[i] - self.center;
                    let p1 = decoded[i + 1] - self.center;

                    // 4 vertices of segment quad
                    self.positions.push(p0.x as f32);
                    self.positions.push(p0.y as f32);
                    self.positions.push(p0.z as f32);

                    self.positions.push(p0.x as f32);
                    self.positions.push(p0.y as f32);
                    self.positions.push(p0.z as f32);

                    self.positions.push(p1.x as f32);
                    self.positions.push(p1.y as f32);
                    self.positions.push(p1.z as f32);

                    self.positions.push(p1.x as f32);
                    self.positions.push(p1.y as f32);
                    self.positions.push(p1.z as f32);

                    for _ in 0..4 {
                        self.vertex_batch_ids.push(batch_id);
                    }

                    // 2 triangles
                    self.indices.push(vert_start_idx);
                    self.indices.push(vert_start_idx + 1);
                    self.indices.push(vert_start_idx + 2);

                    self.indices.push(vert_start_idx + 1);
                    self.indices.push(vert_start_idx + 3);
                    self.indices.push(vert_start_idx + 2);

                    vert_start_idx += 4;
                }
            } else if count == 1 && start < decoded.len() {
                // degenerate point
                let p = decoded[start] - self.center;
                self.positions.push(p.x as f32);
                self.positions.push(p.y as f32);
                self.positions.push(p.z as f32);
                self.vertex_batch_ids.push(batch_id);
                vert_start_idx += 1;
            }
        }

        self.triangles_length = self.indices.len() / 3;
        self.geometry_byte_length =
            self.positions.len() * 4 + self.indices.len() * 4 + self.vertex_batch_ids.len() * 2;
    }

    /// Gets decoded polyline positions for a given batch ID.
    pub fn get_positions(&self, batch_id: usize) -> Option<Vec<Cartesian3>> {
        let (positions, offsets) = match (&self.decoded_positions, &self.decoded_position_offsets) {
            (Some(p), Some(o)) => (p, o),
            _ => return None,
        };

        let mut results = Vec::new();
        for (i, &bid) in self.batch_ids.iter().enumerate() {
            if bid as usize == batch_id && i + 1 < offsets.len() {
                let start = offsets[i];
                let end = offsets[i + 1];
                if end <= positions.len() {
                    results.extend_from_slice(&positions[start..end]);
                }
            }
        }

        if results.is_empty() {
            None
        } else {
            Some(results)
        }
    }

    /// Creates features mapping.
    pub fn create_features(&self, features: &mut Vec<Vector3DTileFeature>) {
        features.clear();
        for (i, &bid) in self.batch_ids.iter().enumerate() {
            let mut feat = Vector3DTileFeature::new(bid as usize);
            feat.properties.insert(
                "polyline_index".to_string(),
                serde_json::Value::from(i as u64),
            );
            features.push(feat);
        }
    }

    /// Sets debug highlight settings.
    pub fn apply_debug_settings(&mut self, enabled: bool, color: Color) {
        if enabled {
            self.highlight_color = color;
        } else {
            self.highlight_color = Color::WHITE;
        }
    }

    /// Applies style expressions.
    pub fn apply_style(
        &mut self,
        style: Option<&Cesium3DTileStyle>,
        features: &mut [Vector3DTileFeature],
    ) {
        let Some(s) = style else {
            for feat in features {
                feat.show = true;
                feat.color = Color::WHITE;
            }
            return;
        };

        if let Some(ref show_expr) = s.show_expression {
            let show_val = show_expr != "false";
            for feat in features.iter_mut() {
                feat.show = show_val;
            }
        }

        if let Some(ref color_expr) = s.color_expression {
            let color = if color_expr.contains("blue") {
                Color::BLUE
            } else if color_expr.contains("black") {
                Color::BLACK
            } else if color_expr.contains("red") {
                Color::RED
            } else {
                Color::WHITE
            };

            for feat in features.iter_mut() {
                feat.color = color;
            }
        }
    }

    /// Updates the render pass state.
    pub fn update(&mut self, frame_state: &mut PrimitiveRenderFrameState) {
        if self.is_destroyed || !self.ready {
            return;
        }

        let is_render = frame_state.passes.render;
        let is_pick = frame_state.passes.pick;

        if !is_render && !is_pick {
            return;
        }

        let cmd = DrawCommand {
            pass: Some(Pass::Translucent),
            shader_program: Some(ShaderProgramHandle::Name(
                "Vector3DTilePolylines".to_string(),
            )),
            ..Default::default()
        };

        frame_state.command_list.push(cmd);
    }

    /// Destroys resources.
    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.positions.clear();
            self.indices.clear();
            self.vertex_batch_ids.clear();
            self.decoded_positions = None;
            self.decoded_position_offsets = None;
            self.is_destroyed = true;
            self.ready = false;
        }
    }

    /// Returns whether this primitive is destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}
