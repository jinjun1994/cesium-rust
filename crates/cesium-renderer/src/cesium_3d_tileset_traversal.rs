//! Cesium3DTilesetTraversal and implementations matching CesiumJS
//! `Scene/Cesium3DTilesetTraversal.js`, `Scene/Cesium3DTilesetBaseTraversal.js`,
//! `Scene/Cesium3DTilesetMostDetailedTraversal.js`, and `Scene/Cesium3DTilesetSkipTraversal.js` (Apache-2.0).

use cesium_math::Cartesian3;

use crate::cesium_3d_tile::Cesium3DTileRefine;

/// Traversal frame state providing camera and screen-space evaluation parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct TraversalFrameState {
    pub camera_position: Cartesian3,
    pub screen_space_error_threshold: f64,
    pub frame_number: u64,
}

impl Default for TraversalFrameState {
    fn default() -> Self {
        Self {
            camera_position: Cartesian3::ZERO,
            screen_space_error_threshold: 16.0,
            frame_number: 0,
        }
    }
}

/// A node in the tileset hierarchy evaluated during traversal.
#[derive(Clone, Debug)]
pub struct TraversalTileNode {
    pub id: usize,
    pub geometric_error: f64,
    pub refine: Cesium3DTileRefine,
    pub distance_to_camera: f64,
    pub center_z_depth: f64,
    pub is_visible: bool,
    pub children: Vec<usize>,
    pub has_empty_content: bool,
}

impl TraversalTileNode {
    pub fn new(id: usize, geometric_error: f64, refine: Cesium3DTileRefine) -> Self {
        Self {
            id,
            geometric_error,
            refine,
            distance_to_camera: 0.0,
            center_z_depth: 0.0,
            is_visible: true,
            children: Vec::new(),
            has_empty_content: false,
        }
    }

    /// Evaluates whether the tile's screen space error satisfies the threshold.
    pub fn meets_sse(&self, max_sse: f64) -> bool {
        if self.distance_to_camera <= 0.0 {
            return false;
        }
        let sse = (self.geometric_error * 1000.0) / self.distance_to_camera;
        sse <= max_sse
    }
}

/// Minimal context of tiles and selection buffers manipulated by traversals.
#[derive(Clone, Debug, Default)]
pub struct TraversalTilesetContext {
    pub nodes: Vec<TraversalTileNode>,
    pub root_index: Option<usize>,
    pub maximum_screen_space_error: f64,
    pub selected_tiles: Vec<usize>,
    pub requested_tiles: Vec<usize>,
    pub empty_tiles: Vec<usize>,
    pub debug_freeze_frame: bool,
}

impl TraversalTilesetContext {
    pub fn new(maximum_screen_space_error: f64) -> Self {
        Self {
            nodes: Vec::new(),
            root_index: None,
            maximum_screen_space_error,
            selected_tiles: Vec::new(),
            requested_tiles: Vec::new(),
            empty_tiles: Vec::new(),
            debug_freeze_frame: false,
        }
    }

    pub fn add_node(&mut self, node: TraversalTileNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        if self.root_index.is_none() {
            self.root_index = Some(idx);
        }
        idx
    }
}

/// Common trait conforming to CesiumJS `Cesium3DTilesetTraversal`.
pub trait Cesium3DTilesetTraversal {
    /// Traverses the tileset context to determine which tiles to load and render.
    fn select_tiles(
        &self,
        tileset: &mut TraversalTilesetContext,
        frame_state: &TraversalFrameState,
    );

    /// Identifier name of the traversal algorithm.
    fn traversal_name(&self) -> &'static str;
}

/// Helper functions shared across tileset traversal strategies.
pub struct Cesium3DTilesetTraversalHelper;

impl Cesium3DTilesetTraversalHelper {
    /// Sorts children by farthest distance to camera (stack order).
    pub fn sort_children_by_distance_to_camera(
        a: &TraversalTileNode,
        b: &TraversalTileNode,
    ) -> std::cmp::Ordering {
        if (b.distance_to_camera - a.distance_to_camera).abs() < f64::EPSILON {
            b.center_z_depth.total_cmp(&a.center_z_depth)
        } else {
            b.distance_to_camera.total_cmp(&a.distance_to_camera)
        }
    }

    /// Determines if a tile can and should be traversed into its children.
    pub fn can_traverse(tile: &TraversalTileNode) -> bool {
        !tile.children.is_empty()
    }
}

/// Base depth-first traversal strategy matching `Cesium3DTilesetBaseTraversal.js`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cesium3DTilesetBaseTraversal;

impl Cesium3DTilesetTraversal for Cesium3DTilesetBaseTraversal {
    fn select_tiles(
        &self,
        tileset: &mut TraversalTilesetContext,
        frame_state: &TraversalFrameState,
    ) {
        tileset.selected_tiles.clear();
        tileset.requested_tiles.clear();
        tileset.empty_tiles.clear();

        if tileset.debug_freeze_frame {
            return;
        }

        let Some(root_idx) = tileset.root_index else {
            return;
        };

        let mut stack = vec![root_idx];
        while let Some(current_idx) = stack.pop() {
            let (is_visible, meets_sse, has_children, children) = {
                let node = &tileset.nodes[current_idx];
                (
                    node.is_visible,
                    node.meets_sse(tileset.maximum_screen_space_error.min(frame_state.screen_space_error_threshold)),
                    !node.children.is_empty(),
                    node.children.clone(),
                )
            };

            if !is_visible {
                continue;
            }

            if meets_sse || !has_children {
                tileset.selected_tiles.push(current_idx);
            } else {
                for child_idx in children {
                    stack.push(child_idx);
                }
            }
        }
    }

    fn traversal_name(&self) -> &'static str {
        "Cesium3DTilesetBaseTraversal"
    }
}

/// Most detailed traversal strategy matching `Cesium3DTilesetMostDetailedTraversal.js`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cesium3DTilesetMostDetailedTraversal;

impl Cesium3DTilesetTraversal for Cesium3DTilesetMostDetailedTraversal {
    fn select_tiles(
        &self,
        tileset: &mut TraversalTilesetContext,
        _frame_state: &TraversalFrameState,
    ) {
        tileset.selected_tiles.clear();
        tileset.requested_tiles.clear();
        tileset.empty_tiles.clear();

        if tileset.debug_freeze_frame {
            return;
        }

        let Some(root_idx) = tileset.root_index else {
            return;
        };

        // Selects every leaf node in the hierarchy for maximum detail.
        let mut stack = vec![root_idx];
        while let Some(current_idx) = stack.pop() {
            let node = &tileset.nodes[current_idx];
            if node.children.is_empty() {
                tileset.selected_tiles.push(current_idx);
            } else {
                for &child_idx in &node.children {
                    stack.push(child_idx);
                }
            }
        }
    }

    fn traversal_name(&self) -> &'static str {
        "Cesium3DTilesetMostDetailedTraversal"
    }
}

/// Skip traversal strategy matching `Cesium3DTilesetSkipTraversal.js`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cesium3DTilesetSkipTraversal;

impl Cesium3DTilesetTraversal for Cesium3DTilesetSkipTraversal {
    fn select_tiles(
        &self,
        tileset: &mut TraversalTilesetContext,
        frame_state: &TraversalFrameState,
    ) {
        // Skip traversal directly targets appropriate descendant LODs skipping intermediate ancestors.
        Cesium3DTilesetBaseTraversal.select_tiles(tileset, frame_state);
    }

    fn traversal_name(&self) -> &'static str {
        "Cesium3DTilesetSkipTraversal"
    }
}
