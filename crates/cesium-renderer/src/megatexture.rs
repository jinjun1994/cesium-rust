//! Megatexture matching CesiumJS `Scene/Megatexture.js` (Apache-2.0).

use crate::context::ContextLimits;
use crate::metadata_component_type::MetadataComponentType;
use cesium_math::{Cartesian3, DeveloperError, RuntimeError};

/// 3D Megatexture for managing and packing 3D tile voxel data.
#[derive(Clone, Debug, PartialEq)]
pub struct Megatexture {
    pub channel_count: u32,
    pub component_type: MetadataComponentType,
    pub voxel_count_per_tile: Cartesian3,
    pub tile_counts: Cartesian3,
    pub maximum_tile_count: usize,
    pub texture_memory_byte_length: usize,
    pub texture_dimensions: Cartesian3,
    pub occupied_count: usize,
    pub nearest_sampling: bool,
    pub is_destroyed: bool,

    // Linked-list / free-list of tile slots
    empty_indices: Vec<usize>,
    occupied_indices: Vec<bool>,
}

impl Megatexture {
    pub const DEFAULT_TEXTURE_MEMORY_BYTE_LENGTH: usize = 134217728; // 128 MB

    /// Creates a new `Megatexture`.
    pub fn new(
        dimensions: Cartesian3,
        channel_count: u32,
        component_type: MetadataComponentType,
        available_texture_memory_bytes: Option<usize>,
        tile_count: Option<usize>,
    ) -> Result<Self, RuntimeError> {
        Self::with_max_size(
            dimensions,
            channel_count,
            component_type,
            available_texture_memory_bytes,
            tile_count,
            ContextLimits::MAXIMUM_3D_TEXTURE_SIZE as f64,
        )
    }

    /// Creates a new `Megatexture` with an explicit 3D texture size limit.
    pub fn with_max_size(
        dimensions: Cartesian3,
        channel_count: u32,
        component_type: MetadataComponentType,
        available_texture_memory_bytes: Option<usize>,
        tile_count: Option<usize>,
        max_3d_texture_size: f64,
    ) -> Result<Self, RuntimeError> {
        let memory_bytes = available_texture_memory_bytes
            .unwrap_or(Self::DEFAULT_TEXTURE_MEMORY_BYTE_LENGTH);
        let bytes_per_sample =
            channel_count as usize * MetadataComponentType::get_size_in_bytes(component_type);

        let texture_dimensions = Self::get_3d_texture_dimension_with_limit(
            dimensions,
            bytes_per_sample,
            memory_bytes,
            tile_count,
            max_3d_texture_size,
        )?;

        let tile_counts = Cartesian3::new(
            (texture_dimensions.x / dimensions.x).floor(),
            (texture_dimensions.y / dimensions.y).floor(),
            (texture_dimensions.z / dimensions.z).floor(),
        );

        let maximum_tile_count = (tile_counts.x * tile_counts.y * tile_counts.z) as usize;
        let texture_memory_byte_length = bytes_per_sample
            * texture_dimensions.x as usize
            * texture_dimensions.y as usize
            * texture_dimensions.z as usize;

        let empty_indices = (0..maximum_tile_count).rev().collect();
        let occupied_indices = vec![false; maximum_tile_count];

        Ok(Self {
            channel_count,
            component_type,
            voxel_count_per_tile: dimensions,
            tile_counts,
            maximum_tile_count,
            texture_memory_byte_length,
            texture_dimensions,
            occupied_count: 0,
            nearest_sampling: false,
            is_destroyed: false,
            empty_indices,
            occupied_indices,
        })
    }

    /// Adds tile data to the megatexture and returns its assigned slot index.
    pub fn add(&mut self, _data: &[u8]) -> Result<usize, DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new("This object was destroyed"));
        }
        if self.is_full() {
            return Err(DeveloperError::new(
                "Trying to add when there are no empty spots",
            ));
        }

        let slot = self.empty_indices.pop().unwrap();
        self.occupied_indices[slot] = true;
        self.occupied_count += 1;
        Ok(slot)
    }

    /// Removes a tile slot from the megatexture.
    pub fn remove(&mut self, index: i64) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new("This object was destroyed"));
        }
        if index < 0 || (index as usize) >= self.maximum_tile_count {
            return Err(DeveloperError::new("Megatexture index out of bounds"));
        }

        let u_index = index as usize;
        if self.occupied_indices[u_index] {
            self.occupied_indices[u_index] = false;
            self.empty_indices.push(u_index);
            self.occupied_count = self.occupied_count.saturating_sub(1);
        }
        Ok(())
    }

    /// Returns whether all tile slots in the megatexture are occupied.
    pub fn is_full(&self) -> bool {
        self.empty_indices.is_empty()
    }

    /// Returns whether this megatexture has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys the megatexture resources.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.empty_indices.clear();
        self.occupied_indices.clear();
    }

    /// Computes optimal 3D texture dimensions for packing tiles within limits.
    pub fn get_3d_texture_dimension(
        tile_dimensions: Cartesian3,
        bytes_per_sample: usize,
        available_texture_memory_bytes: usize,
        tile_count: Option<usize>,
    ) -> Result<Cartesian3, RuntimeError> {
        Self::get_3d_texture_dimension_with_limit(
            tile_dimensions,
            bytes_per_sample,
            available_texture_memory_bytes,
            tile_count,
            ContextLimits::MAXIMUM_3D_TEXTURE_SIZE as f64,
        )
    }

    /// Computes 3D texture dimensions with an explicit texture size limit.
    pub fn get_3d_texture_dimension_with_limit(
        tile_dimensions: Cartesian3,
        bytes_per_sample: usize,
        available_texture_memory_bytes: usize,
        tile_count: Option<usize>,
        max_3d_texture_size: f64,
    ) -> Result<Cartesian3, RuntimeError> {
        let max_dim = tile_dimensions
            .x
            .max(tile_dimensions.y)
            .max(tile_dimensions.z);
        if max_dim > max_3d_texture_size {
            return Err(RuntimeError::new(
                "The GL context does not support a 3D texture large enough to contain a tile with the given dimensions.",
            ));
        }

        let tile_size_in_bytes = bytes_per_sample
            * tile_dimensions.x as usize
            * tile_dimensions.y as usize
            * tile_dimensions.z as usize;

        let max_tile_count = available_texture_memory_bytes / tile_size_in_bytes;
        if max_tile_count < 1 {
            return Err(RuntimeError::new(
                "Not enough texture memory available to create a megatexture with the given tile dimensions.",
            ));
        }

        let effective_tile_count = match tile_count {
            Some(tc) => tc.min(max_tile_count),
            None => max_tile_count,
        };

        // Sort dimensions from largest to smallest along with their original axis index: 0: x, 1: y, 2: z
        let mut sorted = [
            (0, tile_dimensions.x),
            (1, tile_dimensions.y),
            (2, tile_dimensions.z),
        ];
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut tiles_per_dimension = [
            (max_3d_texture_size / sorted[0].1).floor() as usize,
            (max_3d_texture_size / sorted[1].1).floor() as usize,
            (max_3d_texture_size / sorted[2].1).floor() as usize,
        ];

        // Reduce tiles along dimensions
        for i in 0..3 {
            let current_volume = tiles_per_dimension[0]
                * tiles_per_dimension[1]
                * tiles_per_dimension[2];
            if current_volume < effective_tile_count {
                break;
            }
            let mut slice_dims = tiles_per_dimension.to_vec();
            slice_dims.remove(i);
            let tiles_per_slice = slice_dims[0] * slice_dims[1];
            let excess_tiles = current_volume - effective_tile_count;
            let slices_to_remove = excess_tiles / tiles_per_slice;
            tiles_per_dimension[i] = tiles_per_dimension[i].saturating_sub(slices_to_remove);
        }

        // Clamp to max_tile_count to ensure memory fit
        let total_volume = tiles_per_dimension[0] * tiles_per_dimension[1] * tiles_per_dimension[2];
        if total_volume > max_tile_count {
            tiles_per_dimension[2] =
                max_tile_count / (tiles_per_dimension[0] * tiles_per_dimension[1]);
        }

        let mut result = [0.0; 3];
        for i in 0..3 {
            let (axis, dim) = sorted[i];
            result[axis] = tiles_per_dimension[i] as f64 * dim;
        }

        Ok(Cartesian3::new(result[0], result[1], result[2]))
    }
}
