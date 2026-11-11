//! VoxelCell matching CesiumJS `Scene/VoxelCell.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{Cartesian3, DeveloperError, OrientedBoundingBox};

use crate::spatial_node::SpatialNode;
use crate::voxel_box_shape::VoxelShape;

/// A cell from a voxel primitive, providing access to sample metadata and bounding volume.
#[derive(Debug, Clone, PartialEq)]
pub struct VoxelCell {
    pub primitive_id: u64,
    pub tile_index: usize,
    pub sample_index: usize,
    pub metadata: HashMap<String, Vec<f32>>,
    pub oriented_bounding_box: OrientedBoundingBox,
}

impl VoxelCell {
    /// Constructs a new VoxelCell.
    pub fn new(primitive_id: u64, tile_index: usize, sample_index: usize) -> Self {
        Self {
            primitive_id,
            tile_index,
            sample_index,
            metadata: HashMap::new(),
            oriented_bounding_box: OrientedBoundingBox::default(),
        }
    }

    /// Constructs a VoxelCell from node details and computes its metadata and bounding box.
    #[allow(clippy::too_many_arguments)]
    pub fn from_keyframe_node_details(
        primitive_id: Option<u64>,
        tile_index: Option<usize>,
        sample_index: Option<usize>,
        shape: Option<&impl VoxelShape>,
        spatial_node: Option<&SpatialNode>,
        tile_dimensions: Option<Cartesian3>,
        padding_before: Option<Cartesian3>,
        metadata: Option<HashMap<String, Vec<f32>>>,
    ) -> Result<Self, DeveloperError> {
        let prim_id = primitive_id.ok_or_else(|| DeveloperError::new("primitive is required."))?;
        let t_idx = tile_index.ok_or_else(|| DeveloperError::new("tileIndex is required."))?;
        let s_idx = sample_index.ok_or_else(|| DeveloperError::new("sampleIndex is required."))?;
        let shape = shape.ok_or_else(|| DeveloperError::new("keyframeNode is required."))?;
        let node = spatial_node.ok_or_else(|| DeveloperError::new("spatialNode is required."))?;

        let dims = tile_dimensions.unwrap_or(Cartesian3::new(2.0, 2.0, 2.0));
        let padding = padding_before.unwrap_or(Cartesian3::ZERO);

        let obb = compute_cell_oriented_bounding_box(shape, node, s_idx, dims, padding);

        Ok(Self {
            primitive_id: prim_id,
            tile_index: t_idx,
            sample_index: s_idx,
            metadata: metadata.unwrap_or_default(),
            oriented_bounding_box: obb,
        })
    }

    /// Returns true if this cell contains the given metadata property.
    pub fn has_property(&self, name: &str) -> bool {
        self.metadata.contains_key(name)
    }

    /// Returns the sorted names of metadata properties.
    pub fn get_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.metadata.keys().cloned().collect();
        names.sort();
        names
    }

    /// Returns the sample metadata slice for the given property name if present.
    pub fn get_property(&self, name: &str) -> Option<&[f32]> {
        self.metadata.get(name).map(|v| v.as_slice())
    }
}

/// Computes the OrientedBoundingBox enclosing a sample within a spatial node.
pub fn compute_cell_oriented_bounding_box(
    shape: &impl VoxelShape,
    spatial_node: &SpatialNode,
    sample_index: usize,
    tile_dimensions: Cartesian3,
    padding_before: Cartesian3,
) -> OrientedBoundingBox {
    let padded = spatial_node.dimensions;
    let slice_size = (padded.x * padded.y).max(1.0) as usize;
    let z_idx = sample_index / slice_size;
    let rem = sample_index % slice_size;
    let y_idx = rem / (padded.x.max(1.0) as usize);
    let x_idx = rem % (padded.x.max(1.0) as usize);

    let coord = Cartesian3::new(x_idx as f64, y_idx as f64, z_idx as f64);
    let unpadded = coord - padding_before;
    let tile_uv = Cartesian3::new(
        unpadded.x / tile_dimensions.x,
        unpadded.y / tile_dimensions.y,
        unpadded.z / tile_dimensions.z,
    );

    shape.compute_oriented_bounding_box_for_sample(spatial_node, tile_dimensions, tile_uv)
}
