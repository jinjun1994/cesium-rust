use cesium_math::{Cartesian3, CullingVolume, OrientedBoundingBox, EPSILON7};
use glam::DVec3;

use crate::keyframe_node::KeyframeNode;
use crate::voxel_box_shape::VoxelShape;

/// A node within a voxel spatial octree representing a subvolume at a given level and tile coordinate.
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialNode {
    pub level: u32,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub dimensions: Cartesian3,
    pub oriented_bounding_box: OrientedBoundingBox,
    pub approximate_voxel_size: f64,
    pub screen_space_error: f64,
    pub visited_frame_number: i64,
    pub children: Option<Vec<SpatialNode>>,
    pub keyframe_nodes: Vec<KeyframeNode<()>>,
    pub renderable_keyframe_nodes: Vec<KeyframeNode<()>>,
    pub renderable_keyframe_node_lerp: f64,
    pub renderable_keyframe_node_previous: Option<i32>,
    pub renderable_keyframe_node_next: Option<i32>,
}

impl SpatialNode {
    /// Constructs a new SpatialNode, computing its oriented bounding box and approximate voxel size.
    pub fn new(
        level: u32,
        x: u32,
        y: u32,
        z: u32,
        shape: &(impl VoxelShape + ?Sized),
        dimensions: Cartesian3,
    ) -> Self {
        let mut node = Self {
            level,
            x,
            y,
            z,
            dimensions,
            oriented_bounding_box: OrientedBoundingBox::default(),
            approximate_voxel_size: 0.0,
            screen_space_error: 0.0,
            visited_frame_number: -1,
            children: None,
            keyframe_nodes: Vec::new(),
            renderable_keyframe_nodes: Vec::new(),
            renderable_keyframe_node_lerp: 0.0,
            renderable_keyframe_node_previous: None,
            renderable_keyframe_node_next: None,
        };

        node.compute_bounding_volumes(shape);
        node
    }

    /// Computes the tile's oriented bounding box and calculates approximate voxel size.
    pub fn compute_bounding_volumes(&mut self, shape: &(impl VoxelShape + ?Sized)) {
        self.oriented_bounding_box =
            shape.compute_oriented_bounding_box_for_tile(self.level, self.x, self.y, self.z);

        let half_scale_x = self.oriented_bounding_box.half_axes.x_axis.length();
        let half_scale_y = self.oriented_bounding_box.half_axes.y_axis.length();
        let half_scale_z = self.oriented_bounding_box.half_axes.z_axis.length();
        let maximum_scale = 2.0 * half_scale_x.max(half_scale_y).max(half_scale_z);

        let min_dim = self.dimensions.x.min(self.dimensions.y).min(self.dimensions.z);
        self.approximate_voxel_size = if min_dim > 0.0 {
            maximum_scale / min_dim
        } else {
            0.0
        };
    }

    /// Divides this spatial node into 8 child nodes according to octree indexing rules.
    pub fn construct_child_nodes(
        &mut self,
        shape: &(impl VoxelShape + ?Sized),
        dimensions: Cartesian3,
    ) {
        let child_level = self.level + 1;
        let x_min = self.x * 2;
        let y_min = self.y * 2;
        let z_min = self.z * 2;
        let x_max = x_min + 1;
        let y_max = y_min + 1;
        let z_max = z_min + 1;

        let child_coords = [
            (child_level, x_min, y_min, z_min),
            (child_level, x_max, y_min, z_min),
            (child_level, x_min, y_max, z_min),
            (child_level, x_max, y_max, z_min),
            (child_level, x_min, y_min, z_max),
            (child_level, x_max, y_min, z_max),
            (child_level, x_min, y_max, z_max),
            (child_level, x_max, y_max, z_max),
        ];

        let children = child_coords
            .into_iter()
            .map(|(lvl, cx, cy, cz)| Self::new(lvl, cx, cy, cz, shape, dimensions))
            .collect();

        self.children = Some(children);
    }

    /// Computes the screen space error for this node relative to the camera position.
    pub fn compute_screen_space_error(
        &mut self,
        camera_position: Cartesian3,
        screen_space_error_multiplier: f64,
    ) {
        let dist_sq = self
            .oriented_bounding_box
            .distance_squared_to(DVec3::new(camera_position.x, camera_position.y, camera_position.z))
            .unwrap_or(0.0);
        let distance = dist_sq.sqrt().max(EPSILON7);
        self.screen_space_error = screen_space_error_multiplier * (self.approximate_voxel_size / distance);
    }

    /// Appends or inserts a keyframe node in sorted order by keyframe index.
    pub fn create_keyframe_node(&mut self, keyframe: i32) {
        if let Err(insert_idx) = self.keyframe_nodes.binary_search_by_key(&keyframe, |k| k.keyframe) {
            let keyframe_node = KeyframeNode::new((), keyframe);
            self.keyframe_nodes.insert(insert_idx, keyframe_node);
        }
    }

    /// Checks if this spatial node has been visited during the specified frame.
    pub fn is_visited(&self, frame_number: i64) -> bool {
        self.visited_frame_number == frame_number
    }

    /// Computes the visibility mask of this spatial node within the given culling volume.
    pub fn visibility(&self, culling_volume: &CullingVolume, parent_plane_mask: u32) -> u32 {
        culling_volume.compute_visibility_with_plane_mask(&self.oriented_bounding_box, parent_plane_mask)
    }
}
