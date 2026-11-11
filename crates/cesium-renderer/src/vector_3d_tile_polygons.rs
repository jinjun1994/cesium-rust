//! Vector3DTilePolygons matching CesiumJS `Scene/Vector3DTilePolygons.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{BoundingSphere, Cartesian3, Cartographic, ClassificationType, Color, Ellipsoid, GeoRectangle};

use crate::commands::{DrawCommand, Pass, ShaderProgramHandle};
use crate::model_3d_tile_content::Cesium3DTileStyle;
use crate::scene_primitive::PrimitiveRenderFrameState;
use crate::vector_3d_tile_content::Vector3DTileFeature;
use crate::vector_3d_tile_geometry::Vector3DTileBatch;
use crate::vector_3d_tile_points::{un_zig_zag, zig_zag};

const MAX_SHORT: f64 = 32767.0;

/// Encodes 2D cartographic polygon positions into quantised zig-zag delta buffer.
pub fn encode_polygon_positions(rectangle: &GeoRectangle, positions: &[Cartographic]) -> Vec<u16> {
    let length = positions.len();
    let mut buffer = vec![0u16; length * 2];

    let mut last_u: i32 = 0;
    let mut last_v: i32 = 0;

    let width = rectangle.width();
    let height = rectangle.height();

    for (i, pos) in positions.iter().enumerate() {
        let mut u = (pos.longitude - rectangle.west) / width;
        let mut v = (pos.latitude - rectangle.south) / height;

        u = u.clamp(0.0, 1.0);
        v = v.clamp(0.0, 1.0);

        let u_int = (u * MAX_SHORT).floor() as i32;
        let v_int = (v * MAX_SHORT).floor() as i32;

        buffer[i] = zig_zag(u_int - last_u);
        buffer[i + length] = zig_zag(v_int - last_v);

        last_u = u_int;
        last_v = v_int;
    }

    buffer
}

/// Options to construct a [`Vector3DTilePolygons`].
#[derive(Debug, Clone, Default)]
pub struct Vector3DTilePolygonsOptions {
    pub positions: Vec<u16>,
    pub counts: Vec<u32>,
    pub indices: Vec<u32>,
    pub index_counts: Vec<u32>,
    pub batch_ids: Vec<u16>,
    pub rectangle: GeoRectangle,
    pub minimum_height: f64,
    pub maximum_height: f64,
    pub polygon_minimum_heights: Option<Vec<f32>>,
    pub polygon_maximum_heights: Option<Vec<f32>>,
    pub classification_type: Option<ClassificationType>,
    pub bounding_volume: Option<BoundingSphere>,
    pub batch_colors: Option<HashMap<usize, Color>>,
}

/// Represents a batch of pre-triangulated polygons draped on terrain and/or 3D Tiles.
#[derive(Debug)]
pub struct Vector3DTilePolygons {
    pub classification_type: ClassificationType,
    pub debug_wireframe: bool,
    pub force_rebatch: bool,
    pub ready: bool,
    pub is_destroyed: bool,

    pub rectangle: GeoRectangle,
    pub minimum_height: f64,
    pub maximum_height: f64,
    pub polygon_minimum_heights: Option<Vec<f32>>,
    pub polygon_maximum_heights: Option<Vec<f32>>,

    pub center: Cartesian3,
    pub bounding_volume: Option<BoundingSphere>,

    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
    pub vertex_batch_ids: Vec<u16>,
    pub batches: Vec<Vector3DTileBatch>,
    pub batch_colors: HashMap<usize, Color>,

    pub triangles_length: usize,
    pub geometry_byte_length: usize,
}

impl Vector3DTilePolygons {
    /// Creates a new `Vector3DTilePolygons`.
    pub fn new(options: Vector3DTilePolygonsOptions) -> Self {
        let classification_type = options
            .classification_type
            .unwrap_or(ClassificationType::Both);

        let center_carto = options.rectangle.center();
        let center = Ellipsoid::WGS84
            .cartographic_to_cartesian(Cartographic::new(
                center_carto.longitude,
                center_carto.latitude,
                (options.minimum_height + options.maximum_height) * 0.5,
            ))
            .map(|p| Cartesian3::new(p.x, p.y, p.z))
            .unwrap_or(Cartesian3::ZERO);

        let mut instance = Self {
            classification_type,
            debug_wireframe: false,
            force_rebatch: false,
            ready: false,
            is_destroyed: false,
            rectangle: options.rectangle,
            minimum_height: options.minimum_height,
            maximum_height: options.maximum_height,
            polygon_minimum_heights: options.polygon_minimum_heights.clone(),
            polygon_maximum_heights: options.polygon_maximum_heights.clone(),
            center,
            bounding_volume: options.bounding_volume,
            positions: Vec::new(),
            indices: Vec::new(),
            vertex_batch_ids: Vec::new(),
            batches: Vec::new(),
            batch_colors: options.batch_colors.clone().unwrap_or_default(),
            triangles_length: 0,
            geometry_byte_length: 0,
        };

        instance.build_polygons(&options);
        instance.ready = true;
        instance
    }

    fn build_polygons(&mut self, options: &Vector3DTilePolygonsOptions) {
        let total_verts = options.positions.len() / 2;
        let mut u_int: i32 = 0;
        let mut v_int: i32 = 0;

        let width = self.rectangle.width();
        let height = self.rectangle.height();

        let mut polygon_positions = Vec::with_capacity(total_verts);

        for i in 0..total_verts {
            u_int += un_zig_zag(options.positions[i]);
            v_int += un_zig_zag(options.positions[i + total_verts]);

            let lon = self.rectangle.west + (u_int as f64 / MAX_SHORT) * width;
            let lat = self.rectangle.south + (v_int as f64 / MAX_SHORT) * height;

            let carto = Cartographic::new(lon, lat, self.minimum_height);
            let world = Ellipsoid::WGS84
                .cartographic_to_cartesian(carto)
                .map(|p| Cartesian3::new(p.x, p.y, p.z))
                .unwrap_or(Cartesian3::ZERO);
            let rtc = world - self.center;

            self.positions.push(rtc.x as f32);
            self.positions.push(rtc.y as f32);
            self.positions.push(rtc.z as f32);
            polygon_positions.push(rtc);
        }

        // Copy indices
        self.indices.clone_from(&options.indices);

        // Assign vertex batch ids and create batches
        let mut index_offset = 0;

        for (poly_idx, &count) in options.counts.iter().enumerate() {
            let batch_id = options
                .batch_ids
                .get(poly_idx)
                .copied()
                .unwrap_or(poly_idx as u16);
            let idx_count = options
                .index_counts
                .get(poly_idx)
                .copied()
                .unwrap_or(count);

            for _ in 0..count {
                self.vertex_batch_ids.push(batch_id);
            }

            let color = self
                .batch_colors
                .get(&(batch_id as usize))
                .copied()
                .unwrap_or(Color::WHITE);
            self.batches.push(Vector3DTileBatch {
                color,
                offset: index_offset,
                count: idx_count as usize,
                batch_ids: vec![batch_id],
            });

            index_offset += idx_count as usize;
        }

        self.triangles_length = self.indices.len() / 3;
        self.geometry_byte_length =
            self.positions.len() * 4 + self.indices.len() * 4 + self.vertex_batch_ids.len() * 2;
    }

    /// Updates the color for a given batch ID.
    pub fn update_commands(&mut self, batch_id: usize, color: Color) {
        self.batch_colors.insert(batch_id, color);
        for batch in &mut self.batches {
            if batch.batch_ids.contains(&(batch_id as u16)) {
                batch.color = color;
            }
        }
    }

    /// Sets debug highlight settings.
    pub fn apply_debug_settings(&mut self, enabled: bool, color: Color) {
        let col = if enabled { color } else { Color::WHITE };
        for batch in &mut self.batches {
            batch.color = col;
        }
    }

    /// Applies style expression.
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
            for (idx, _) in features.iter().enumerate() {
                self.update_commands(idx, color);
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

        let passes = match self.classification_type {
            ClassificationType::Terrain => vec![Pass::Globe],
            ClassificationType::Cesium3DTile => vec![Pass::Cesium3dTileClassification],
            ClassificationType::Both => vec![Pass::Globe, Pass::Cesium3dTileClassification],
        };

        for pass in passes {
            let cmd = DrawCommand {
                pass: Some(pass),
                shader_program: Some(ShaderProgramHandle::Name(if self.debug_wireframe {
                    "Vector3DTilePolygonsWireframe".to_string()
                } else {
                    "Vector3DTilePolygons".to_string()
                })),
                ..Default::default()
            };
            frame_state.command_list.push(cmd);
        }
    }

    /// Destroys resources.
    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.positions.clear();
            self.indices.clear();
            self.vertex_batch_ids.clear();
            self.batches.clear();
            self.is_destroyed = true;
            self.ready = false;
        }
    }

    /// Returns whether this primitive is destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}
