//! GaussianSplat3DTileContent matching CesiumJS `Scene/GaussianSplat3DTileContent.js`.

use cesium_math::{Cartesian3, Color, Matrix3, Matrix4, Quaternion};

use crate::cesium_3d_tile_style::Cesium3DTileStyleDocument;
use crate::content_metadata::ContentMetadata;
use crate::multiple_3d_tile_content::Cesium3DContentGroup;
use crate::vertex_attribute_semantic::VertexAttributeSemantic;

/// Attribute slice view for a Gaussian splat primitive.
#[derive(Clone, Debug, PartialEq)]
pub struct GaussianSplatAttribute {
    pub semantic: String,
    pub data: Vec<f32>,
}

/// Simplified glTF primitive representation for Gaussian splat tile content.
#[derive(Clone, Debug, Default)]
pub struct GaussianSplatGltfPrimitive {
    pub attributes: Vec<GaussianSplatAttribute>,
}

impl GaussianSplatGltfPrimitive {
    pub fn get_attribute_by_semantic(&self, semantic: &str) -> Option<&GaussianSplatAttribute> {
        self.attributes.iter().find(|a| a.semantic == semantic)
    }
}

/// Represents the contents of a Gaussian Splat glTF/GLB tile (`KHR_gaussian_splatting`).
#[derive(Clone, Debug)]
pub struct GaussianSplat3DTileContent {
    pub tileset_id: Option<String>,
    pub tile_id: Option<String>,
    pub resource_url: String,
    pub world_transform: Matrix4,
    pub feature_properties_dirty: bool,
    pub gltf_primitive: GaussianSplatGltfPrimitive,
    source_positions: Vec<f32>,
    source_rotations: Vec<f32>,
    source_scales: Vec<f32>,
    positions: Vec<f32>,
    rotations: Vec<f32>,
    scales: Vec<f32>,
    colors: Vec<f32>,
    sh_degree: usize,
    transformed: bool,
    metadata: Option<ContentMetadata>,
    group: Option<Cesium3DContentGroup>,
    ready: bool,
    is_destroyed: bool,
}

#[derive(Clone, Debug, Default)]
pub struct GaussianSplat3DTileContentOptions {
    pub tileset_id: Option<String>,
    pub tile_id: Option<String>,
    pub resource_url: String,
    pub positions: Vec<f32>,
    pub rotations: Vec<f32>,
    pub scales: Vec<f32>,
    pub colors: Vec<f32>,
    pub sh_degree: usize,
}

impl GaussianSplat3DTileContent {
    /// Creates a new `GaussianSplat3DTileContent` from decoded splat attributes.
    pub fn new(options: GaussianSplat3DTileContentOptions) -> Self {
        let positions = options.positions;
        let rotations = options.rotations;
        let scales = options.scales;
        let colors = options.colors;
        let gltf_primitive = GaussianSplatGltfPrimitive {
            attributes: vec![
                GaussianSplatAttribute {
                    semantic: VertexAttributeSemantic::POSITION.to_string(),
                    data: positions.clone(),
                },
                GaussianSplatAttribute {
                    semantic: VertexAttributeSemantic::ROTATION.to_string(),
                    data: rotations.clone(),
                },
                GaussianSplatAttribute {
                    semantic: VertexAttributeSemantic::SCALE.to_string(),
                    data: scales.clone(),
                },
                GaussianSplatAttribute {
                    semantic: VertexAttributeSemantic::COLOR.to_string(),
                    data: colors.clone(),
                },
            ],
        };

        Self {
            tileset_id: options.tileset_id,
            tile_id: options.tile_id,
            resource_url: options.resource_url,
            world_transform: Matrix4::IDENTITY,
            feature_properties_dirty: false,
            gltf_primitive,
            source_positions: positions.clone(),
            source_rotations: rotations.clone(),
            source_scales: scales.clone(),
            positions,
            rotations,
            scales,
            colors,
            sh_degree: options.sh_degree,
            transformed: false,
            metadata: None,
            group: None,
            ready: true,
            is_destroyed: false,
        }
    }

    /// Transforms local splat positions, rotations, and scales into root tile space
    /// without mutating the original source attributes.
    pub fn apply_tile_transform(&mut self, tile_to_root: &Matrix4) {
        let count = self.source_positions.len() / 3;
        self.positions.resize(count * 3, 0.0);
        for i in 0..count {
            let p = Cartesian3::new(
                self.source_positions[i * 3] as f64,
                self.source_positions[i * 3 + 1] as f64,
                self.source_positions[i * 3 + 2] as f64,
            );
            let tp = tile_to_root.multiply_by_point(p);
            self.positions[i * 3] = tp.x as f32;
            self.positions[i * 3 + 1] = tp.y as f32;
            self.positions[i * 3 + 2] = tp.z as f32;
        }

        // Extract rotation & scale from upper 3x3 of tile_to_root
        let c0 = Cartesian3::new(
            tile_to_root.values[0],
            tile_to_root.values[1],
            tile_to_root.values[2],
        );
        let c1 = Cartesian3::new(
            tile_to_root.values[4],
            tile_to_root.values[5],
            tile_to_root.values[6],
        );
        let c2 = Cartesian3::new(
            tile_to_root.values[8],
            tile_to_root.values[9],
            tile_to_root.values[10],
        );
        let sx = c0.magnitude();
        let sy = c1.magnitude();
        let sz = c2.magnitude();

        let rot_mat = Matrix3::from_column_major_array([
            if sx > 0.0 { c0.x / sx } else { 1.0 },
            if sx > 0.0 { c0.y / sx } else { 0.0 },
            if sx > 0.0 { c0.z / sx } else { 0.0 },
            if sy > 0.0 { c1.x / sy } else { 0.0 },
            if sy > 0.0 { c1.y / sy } else { 1.0 },
            if sy > 0.0 { c1.z / sy } else { 0.0 },
            if sz > 0.0 { c2.x / sz } else { 0.0 },
            if sz > 0.0 { c2.y / sz } else { 0.0 },
            if sz > 0.0 { c2.z / sz } else { 1.0 },
        ]);
        let tile_rot = Quaternion::from_rotation_matrix(&rot_mat);

        let rot_count = self.source_rotations.len() / 4;
        self.rotations.resize(rot_count * 4, 0.0);
        for i in 0..rot_count {
            let q = Quaternion::new(
                self.source_rotations[i * 4] as f64,
                self.source_rotations[i * 4 + 1] as f64,
                self.source_rotations[i * 4 + 2] as f64,
                self.source_rotations[i * 4 + 3] as f64,
            );
            let tq = tile_rot.multiply(&q).normalize();
            self.rotations[i * 4] = tq.x as f32;
            self.rotations[i * 4 + 1] = tq.y as f32;
            self.rotations[i * 4 + 2] = tq.z as f32;
            self.rotations[i * 4 + 3] = tq.w as f32;
        }

        let scale_count = self.source_scales.len() / 3;
        self.scales.resize(scale_count * 3, 0.0);
        for i in 0..scale_count {
            self.scales[i * 3] = (self.source_scales[i * 3] as f64 * sx) as f32;
            self.scales[i * 3 + 1] = (self.source_scales[i * 3 + 1] as f64 * sy) as f32;
            self.scales[i * 3 + 2] = (self.source_scales[i * 3 + 2] as f64 * sz) as f32;
        }

        self.transformed = true;
    }

    pub fn positions(&self) -> &[f32] {
        &self.positions
    }

    pub fn rotations(&self) -> &[f32] {
        &self.rotations
    }

    pub fn scales(&self) -> &[f32] {
        &self.scales
    }

    pub fn colors(&self) -> &[f32] {
        &self.colors
    }

    pub fn sh_degree(&self) -> usize {
        self.sh_degree
    }

    pub fn is_transformed(&self) -> bool {
        self.transformed
    }

    pub fn features_length(&self) -> usize {
        0
    }

    pub fn points_length(&self) -> usize {
        self.positions.len() / 3
    }

    pub fn triangles_length(&self) -> usize {
        0
    }

    pub fn geometry_byte_length(&self) -> usize {
        (self.positions.len() + self.rotations.len() + self.scales.len() + self.colors.len())
            * std::mem::size_of::<f32>()
    }

    pub fn textures_byte_length(&self) -> usize {
        0
    }

    pub fn batch_table_byte_length(&self) -> usize {
        0
    }

    pub fn inner_contents(&self) -> Option<&[()]> {
        None
    }

    pub fn ready(&self) -> bool {
        self.ready
    }

    pub fn url(&self) -> &str {
        &self.resource_url
    }

    pub fn metadata(&self) -> Option<&ContentMetadata> {
        self.metadata.as_ref()
    }

    pub fn set_metadata(&mut self, metadata: Option<ContentMetadata>) {
        self.metadata = metadata;
    }

    pub fn group(&self) -> Option<&Cesium3DContentGroup> {
        self.group.as_ref()
    }

    pub fn set_group(&mut self, group: Option<Cesium3DContentGroup>) {
        self.group = group;
    }

    pub fn has_property(&self, _batch_id: usize, _name: &str) -> bool {
        false
    }

    pub fn get_feature(&self, _batch_id: usize) -> Option<()> {
        None
    }

    pub fn apply_debug_settings(&mut self, _enabled: bool, _color: Option<Color>) {}

    pub fn apply_style(&mut self, _style: Option<&Cesium3DTileStyleDocument>) {}

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.source_positions.clear();
        self.source_rotations.clear();
        self.source_scales.clear();
        self.positions.clear();
        self.rotations.clear();
        self.scales.clear();
        self.colors.clear();
        self.gltf_primitive.attributes.clear();
        self.is_destroyed = true;
    }
}
