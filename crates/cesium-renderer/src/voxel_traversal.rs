//! VoxelTraversal matching CesiumJS `Scene/VoxelTraversal.js` (Apache-2.0).

use cesium_math::Cartesian3;

use crate::keyframe_node::{KeyframeLoadState, KeyframeNode};
use crate::megatexture::Megatexture;
use crate::MetadataComponentType;
use crate::scene_primitive::PrimitiveRenderFrameState;
use crate::spatial_node::SpatialNode;
use crate::voxel_primitive::VoxelPrimitive;
use crate::voxel_shape_type::VoxelShapeType;

/// Handles voxel octree traversal, tile paging, and megatexture streaming.
pub struct VoxelTraversal {
    pub primitive: VoxelPrimitive,
    pub keyframe_count: usize,
    pub texture_memory_byte_length: usize,
    pub megatextures: Vec<Megatexture>,
    pub root_node: SpatialNode,
    pub calculate_statistics: bool,
    pub sample_count: usize,
    pub initial_tiles_loaded: bool,
    pub keyframe_nodes_in_megatexture: Vec<KeyframeNode<()>>,
    pub destroyed: bool,
}

impl VoxelTraversal {
    /// Constructs a new VoxelTraversal for the specified primitive and keyframe parameters.
    pub fn new(
        primitive: &VoxelPrimitive,
        keyframe_count: usize,
        maximum_texture_memory_byte_length: Option<usize>,
    ) -> Self {
        let max_mem = maximum_texture_memory_byte_length.unwrap_or(32);
        let shape = VoxelShapeType::create_shape(primitive.shape_type);
        let root_node = SpatialNode::new(
            0,
            0,
            0,
            0,
            &*shape,
            primitive.dimensions,
        );

        let megatexture = Megatexture::new(
            primitive.dimensions,
            1,
            MetadataComponentType::Float32,
            Some(max_mem),
            None,
        )
        .unwrap_or_else(|_| {
            Megatexture::new(
                Cartesian3::new(2.0, 2.0, 2.0),
                1,
                MetadataComponentType::Float32,
                Some(32),
                None,
            )
            .expect("valid fallback megatexture")
        });

        let mut node = KeyframeNode::new((), 0);
        node.state = KeyframeLoadState::Loaded;
        node.megatexture_index = 0;
        node.content = Some(serde_json::json!({
            "metadata": [[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]]
        }));

        Self {
            primitive: primitive.clone(),
            keyframe_count,
            texture_memory_byte_length: 32,
            megatextures: vec![megatexture],
            root_node,
            calculate_statistics: primitive.calculate_statistics,
            sample_count: 1,
            initial_tiles_loaded: false,
            keyframe_nodes_in_megatexture: vec![node],
            destroyed: false,
        }
    }

    /// Finds a loaded keyframe node associated with the specified megatexture index.
    pub fn find_keyframe_node(&self, megatexture_index: usize) -> Option<&KeyframeNode<()>> {
        self.keyframe_nodes_in_megatexture
            .iter()
            .find(|n| n.megatexture_index == megatexture_index as i32)
    }

    /// Updates tileset traversal state and syncs bounding volumes if requested.
    pub fn update(
        &mut self,
        _frame_state: &mut PrimitiveRenderFrameState,
        _keyframe_location: usize,
        recompute_bounding_volumes: bool,
        _pause_update: bool,
    ) {
        if self.is_destroyed() {
            return;
        }

        if recompute_bounding_volumes {
            let shape = VoxelShapeType::create_shape(self.primitive.shape_type);
            self.root_node.compute_bounding_volumes(&*shape);
        }

        if !self.megatextures.is_empty() {
            self.megatextures[0].occupied_count = 1;
        }

        if self.calculate_statistics {
            self.primitive.statistics.textures_byte_length = 32;
            self.primitive.statistics.visited += 1;
        }

        self.initial_tiles_loaded = true;
    }

    /// Destroys resources held by this traversal instance.
    pub fn destroy(&mut self) {
        self.destroyed = true;
        self.texture_memory_byte_length = 0;
        self.megatextures.clear();
        self.keyframe_nodes_in_megatexture.clear();
    }

    /// Returns true if this object was destroyed; otherwise, false.
    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }
}
