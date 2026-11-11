//! Vector3DTileGeometry matching CesiumJS `Scene/Vector3DTileGeometry.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{BoundingSphere, Cartesian3, ClassificationType, Color, Matrix4};

use crate::commands::{DrawCommand, Pass, ShaderProgramHandle};
use crate::model_3d_tile_content::Cesium3DTileStyle;
use crate::scene_primitive::PrimitiveRenderFrameState;
use crate::vector_3d_tile_content::Vector3DTileFeature;

/// Packed element lengths matching CesiumJS `Vector3DTileGeometry`.
pub const PACKED_BOX_LENGTH: usize = 19;
pub const PACKED_CYLINDER_LENGTH: usize = 18;
pub const PACKED_ELLIPSOID_LENGTH: usize = 19;
pub const PACKED_SPHERE_LENGTH: usize = 4;

/// A batch of geometry primitives with a unified color and range.
#[derive(Debug, Clone)]
pub struct Vector3DTileBatch {
    pub color: Color,
    pub offset: usize,
    pub count: usize,
    pub batch_ids: Vec<u16>,
}

/// Options to construct a [`Vector3DTileGeometry`].
#[derive(Debug, Clone, Default)]
pub struct Vector3DTileGeometryOptions {
    pub boxes: Option<Vec<f32>>,
    pub box_batch_ids: Option<Vec<u16>>,
    pub cylinders: Option<Vec<f32>>,
    pub cylinder_batch_ids: Option<Vec<u16>>,
    pub ellipsoids: Option<Vec<f32>>,
    pub ellipsoid_batch_ids: Option<Vec<u16>>,
    pub spheres: Option<Vec<f32>>,
    pub sphere_batch_ids: Option<Vec<u16>>,
    pub center: Option<Cartesian3>,
    pub model_matrix: Option<Matrix4>,
    pub bounding_volume: Option<BoundingSphere>,
    pub classification_type: Option<ClassificationType>,
    pub batch_colors: Option<HashMap<usize, Color>>,
}

/// Helper to pack a list of boxes into a float array.
pub fn pack_boxes(boxes: &[(Cartesian3, Matrix4)]) -> Vec<f32> {
    let mut packed = Vec::with_capacity(boxes.len() * PACKED_BOX_LENGTH);
    for (dim, mat) in boxes {
        packed.push(dim.x as f32);
        packed.push(dim.y as f32);
        packed.push(dim.z as f32);
        for &col in &mat.values {
            packed.push(col as f32);
        }
    }
    packed
}

/// Helper to pack a list of cylinders into a float array.
pub fn pack_cylinders(cylinders: &[(f32, f32, Matrix4)]) -> Vec<f32> {
    let mut packed = Vec::with_capacity(cylinders.len() * PACKED_CYLINDER_LENGTH);
    for (radius, length, mat) in cylinders {
        packed.push(*radius);
        packed.push(*length);
        for &col in &mat.values {
            packed.push(col as f32);
        }
    }
    packed
}

/// Helper to pack a list of ellipsoids into a float array.
pub fn pack_ellipsoids(ellipsoids: &[(Cartesian3, Matrix4)]) -> Vec<f32> {
    let mut packed = Vec::with_capacity(ellipsoids.len() * PACKED_ELLIPSOID_LENGTH);
    for (radii, mat) in ellipsoids {
        packed.push(radii.x as f32);
        packed.push(radii.y as f32);
        packed.push(radii.z as f32);
        for &col in &mat.values {
            packed.push(col as f32);
        }
    }
    packed
}

/// Helper to pack a list of spheres into a float array.
pub fn pack_spheres(spheres: &[(f32, Cartesian3)]) -> Vec<f32> {
    let mut packed = Vec::with_capacity(spheres.len() * PACKED_SPHERE_LENGTH);
    for (radius, translation) in spheres {
        packed.push(*radius);
        packed.push(translation.x as f32);
        packed.push(translation.y as f32);
        packed.push(translation.z as f32);
    }
    packed
}

/// Renders a batch of box, cylinder, ellipsoid and/or sphere geometries classification volumes.
#[derive(Debug)]
pub struct Vector3DTileGeometry {
    pub classification_type: ClassificationType,
    pub debug_wireframe: bool,
    pub force_rebatch: bool,
    pub ready: bool,
    pub is_destroyed: bool,

    pub center: Cartesian3,
    pub model_matrix: Matrix4,
    pub bounding_volume: Option<BoundingSphere>,

    pub positions: Vec<f32>,
    pub vertex_batch_ids: Vec<u16>,
    pub indices: Vec<u32>,
    pub batches: Vec<Vector3DTileBatch>,
    pub batch_colors: HashMap<usize, Color>,

    pub triangles_length: usize,
    pub geometry_byte_length: usize,
}

impl Vector3DTileGeometry {
    /// Creates a new `Vector3DTileGeometry`.
    pub fn new(options: Vector3DTileGeometryOptions) -> Self {
        let classification_type = options
            .classification_type
            .unwrap_or(ClassificationType::Both);
        let center = options
            .center
            .or_else(|| {
                options
                    .bounding_volume
                    .as_ref()
                    .map(|b| Cartesian3::new(b.center.x, b.center.y, b.center.z))
            })
            .unwrap_or(Cartesian3::ZERO);
        let model_matrix = options.model_matrix.unwrap_or(Matrix4::IDENTITY);
        let bounding_volume = options.bounding_volume;

        let mut instance = Self {
            classification_type,
            debug_wireframe: false,
            force_rebatch: false,
            ready: false,
            is_destroyed: false,
            center,
            model_matrix,
            bounding_volume,
            positions: Vec::new(),
            vertex_batch_ids: Vec::new(),
            indices: Vec::new(),
            batches: Vec::new(),
            batch_colors: options.batch_colors.clone().unwrap_or_default(),
            triangles_length: 0,
            geometry_byte_length: 0,
        };

        instance.build_geometries(&options);
        instance.ready = true;
        instance
    }

    fn build_geometries(&mut self, options: &Vector3DTileGeometryOptions) {
        // Build boxes
        if let (Some(boxes), Some(batch_ids)) = (&options.boxes, &options.box_batch_ids) {
            let count = boxes.len() / PACKED_BOX_LENGTH;
            for i in 0..count {
                let offset = i * PACKED_BOX_LENGTH;
                let dx = boxes[offset] as f64;
                let dy = boxes[offset + 1] as f64;
                let dz = boxes[offset + 2] as f64;
                let mut mat_vals = [0.0; 16];
                for (j, slot) in mat_vals.iter_mut().enumerate() {
                    *slot = boxes[offset + 3 + j] as f64;
                }
                let local_mat = Matrix4::from_column_major_array(mat_vals);
                let full_mat = self.model_matrix.multiply(&local_mat);
                let batch_id = batch_ids.get(i).copied().unwrap_or(0);

                self.add_box_geometry(dx, dy, dz, &full_mat, batch_id);
            }
        }

        // Build cylinders
        if let (Some(cylinders), Some(batch_ids)) = (&options.cylinders, &options.cylinder_batch_ids) {
            let count = cylinders.len() / PACKED_CYLINDER_LENGTH;
            for i in 0..count {
                let offset = i * PACKED_CYLINDER_LENGTH;
                let radius = cylinders[offset] as f64;
                let length = cylinders[offset + 1] as f64;
                let mut mat_vals = [0.0; 16];
                for (j, slot) in mat_vals.iter_mut().enumerate() {
                    *slot = cylinders[offset + 2 + j] as f64;
                }
                let local_mat = Matrix4::from_column_major_array(mat_vals);
                let full_mat = self.model_matrix.multiply(&local_mat);
                let batch_id = batch_ids.get(i).copied().unwrap_or(0);

                self.add_cylinder_geometry(radius, length, &full_mat, batch_id);
            }
        }

        // Build ellipsoids
        if let (Some(ellipsoids), Some(batch_ids)) = (&options.ellipsoids, &options.ellipsoid_batch_ids) {
            let count = ellipsoids.len() / PACKED_ELLIPSOID_LENGTH;
            for i in 0..count {
                let offset = i * PACKED_ELLIPSOID_LENGTH;
                let rx = ellipsoids[offset] as f64;
                let ry = ellipsoids[offset + 1] as f64;
                let rz = ellipsoids[offset + 2] as f64;
                let mut mat_vals = [0.0; 16];
                for (j, slot) in mat_vals.iter_mut().enumerate() {
                    *slot = ellipsoids[offset + 3 + j] as f64;
                }
                let local_mat = Matrix4::from_column_major_array(mat_vals);
                let full_mat = self.model_matrix.multiply(&local_mat);
                let batch_id = batch_ids.get(i).copied().unwrap_or(0);

                self.add_ellipsoid_geometry(rx, ry, rz, &full_mat, batch_id);
            }
        }

        // Build spheres
        if let (Some(spheres), Some(batch_ids)) = (&options.spheres, &options.sphere_batch_ids) {
            let count = spheres.len() / PACKED_SPHERE_LENGTH;
            for i in 0..count {
                let offset = i * PACKED_SPHERE_LENGTH;
                let radius = spheres[offset] as f64;
                let tx = spheres[offset + 1] as f64;
                let ty = spheres[offset + 2] as f64;
                let tz = spheres[offset + 3] as f64;
                let trans_mat = Matrix4::from_translation(Cartesian3::new(tx, ty, tz));
                let full_mat = self.model_matrix.multiply(&trans_mat);
                let batch_id = batch_ids.get(i).copied().unwrap_or(0);

                self.add_ellipsoid_geometry(radius, radius, radius, &full_mat, batch_id);
            }
        }

        self.triangles_length = self.indices.len() / 3;
        self.geometry_byte_length =
            self.positions.len() * 4 + self.indices.len() * 4 + self.vertex_batch_ids.len() * 2;
    }

    fn add_box_geometry(
        &mut self,
        dx: f64,
        dy: f64,
        dz: f64,
        transform: &Matrix4,
        batch_id: u16,
    ) {
        let hx = dx * 0.5;
        let hy = dy * 0.5;
        let hz = dz * 0.5;

        let local_corners = [
            Cartesian3::new(-hx, -hy, -hz),
            Cartesian3::new(hx, -hy, -hz),
            Cartesian3::new(hx, hy, -hz),
            Cartesian3::new(-hx, hy, -hz),
            Cartesian3::new(-hx, -hy, hz),
            Cartesian3::new(hx, -hy, hz),
            Cartesian3::new(hx, hy, hz),
            Cartesian3::new(-hx, hy, hz),
        ];

        let start_index = (self.positions.len() / 3) as u32;
        let index_offset = self.indices.len();

        for p in &local_corners {
            let world = transform.multiply_by_point(*p);
            let rtc = world - self.center;
            self.positions.push(rtc.x as f32);
            self.positions.push(rtc.y as f32);
            self.positions.push(rtc.z as f32);
            self.vertex_batch_ids.push(batch_id);
        }

        #[rustfmt::skip]
        let box_indices: [u32; 36] = [
            0, 2, 1, 0, 3, 2, // bottom
            4, 5, 6, 4, 6, 7, // top
            0, 1, 5, 0, 5, 4, // front
            1, 2, 6, 1, 6, 5, // right
            2, 3, 7, 2, 7, 6, // back
            3, 0, 4, 3, 4, 7, // left
        ];

        for idx in box_indices {
            self.indices.push(start_index + idx);
        }

        let color = self
            .batch_colors
            .get(&(batch_id as usize))
            .copied()
            .unwrap_or(Color::WHITE);
        self.batches.push(Vector3DTileBatch {
            color,
            offset: index_offset,
            count: 36,
            batch_ids: vec![batch_id],
        });
    }

    fn add_cylinder_geometry(
        &mut self,
        radius: f64,
        length: f64,
        transform: &Matrix4,
        batch_id: u16,
    ) {
        let slices = 16;
        let hl = length * 0.5;
        let start_index = (self.positions.len() / 3) as u32;
        let index_offset = self.indices.len();

        // Generate top & bottom rings
        for i in 0..slices {
            let theta = (i as f64 / slices as f64) * std::f64::consts::TAU;
            let x = radius * theta.cos();
            let y = radius * theta.sin();

            // Bottom vertex
            let p_bot = transform.multiply_by_point(Cartesian3::new(x, y, -hl));
            let rtc_bot = p_bot - self.center;
            self.positions.push(rtc_bot.x as f32);
            self.positions.push(rtc_bot.y as f32);
            self.positions.push(rtc_bot.z as f32);
            self.vertex_batch_ids.push(batch_id);

            // Top vertex
            let p_top = transform.multiply_by_point(Cartesian3::new(x, y, hl));
            let rtc_top = p_top - self.center;
            self.positions.push(rtc_top.x as f32);
            self.positions.push(rtc_top.y as f32);
            self.positions.push(rtc_top.z as f32);
            self.vertex_batch_ids.push(batch_id);
        }

        // Indices for side quads
        for i in 0..slices {
            let next = (i + 1) % slices;
            let b0 = start_index + (i * 2) as u32;
            let t0 = b0 + 1;
            let b1 = start_index + (next * 2) as u32;
            let t1 = b1 + 1;

            self.indices.push(b0);
            self.indices.push(t0);
            self.indices.push(b1);

            self.indices.push(b1);
            self.indices.push(t0);
            self.indices.push(t1);
        }

        let count = self.indices.len() - index_offset;
        let color = self
            .batch_colors
            .get(&(batch_id as usize))
            .copied()
            .unwrap_or(Color::WHITE);
        self.batches.push(Vector3DTileBatch {
            color,
            offset: index_offset,
            count,
            batch_ids: vec![batch_id],
        });
    }

    fn add_ellipsoid_geometry(
        &mut self,
        rx: f64,
        ry: f64,
        rz: f64,
        transform: &Matrix4,
        batch_id: u16,
    ) {
        let lat_segments = 8;
        let lon_segments = 12;
        let start_index = (self.positions.len() / 3) as u32;
        let index_offset = self.indices.len();

        for i in 0..=lat_segments {
            let theta = (i as f64 / lat_segments as f64) * std::f64::consts::PI;
            let sin_t = theta.sin();
            let cos_t = theta.cos();

            for j in 0..=lon_segments {
                let phi = (j as f64 / lon_segments as f64) * std::f64::consts::TAU;
                let x = rx * sin_t * phi.cos();
                let y = ry * sin_t * phi.sin();
                let z = rz * cos_t;

                let pt = transform.multiply_by_point(Cartesian3::new(x, y, z));
                let rtc = pt - self.center;
                self.positions.push(rtc.x as f32);
                self.positions.push(rtc.y as f32);
                self.positions.push(rtc.z as f32);
                self.vertex_batch_ids.push(batch_id);
            }
        }

        for i in 0..lat_segments {
            for j in 0..lon_segments {
                let first = start_index + (i * (lon_segments + 1) + j) as u32;
                let second = first + (lon_segments + 1) as u32;

                self.indices.push(first);
                self.indices.push(second);
                self.indices.push(first + 1);

                self.indices.push(second);
                self.indices.push(second + 1);
                self.indices.push(first + 1);
            }
        }

        let count = self.indices.len() - index_offset;
        let color = self
            .batch_colors
            .get(&(batch_id as usize))
            .copied()
            .unwrap_or(Color::WHITE);
        self.batches.push(Vector3DTileBatch {
            color,
            offset: index_offset,
            count,
            batch_ids: vec![batch_id],
        });
    }

    /// Updates the command color for a specific batch.
    pub fn update_commands(&mut self, batch_id: usize, color: Color) {
        self.batch_colors.insert(batch_id, color);
        for batch in &mut self.batches {
            if batch.batch_ids.contains(&(batch_id as u16)) {
                batch.color = color;
            }
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

    /// Updates the render frame commands.
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
                    "Vector3DTileGeometryWireframe".to_string()
                } else {
                    "Vector3DTileGeometry".to_string()
                })),
                ..Default::default()
            };
            frame_state.command_list.push(cmd);
        }
    }

    /// Destroys geometry and clears buffers.
    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.positions.clear();
            self.vertex_batch_ids.clear();
            self.indices.clear();
            self.batches.clear();
            self.is_destroyed = true;
            self.ready = false;
        }
    }

    /// Returns whether this geometry is destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}
