use std::sync::Arc;
use cesium_math::DeveloperError;
use crate::implicit_tile_coordinates::ImplicitTileCoordinates;
use crate::implicit_subtree::ImplicitSubtree;

/// Options for configuring an [`ImplicitSubtreeCache`].
#[derive(Debug, Clone, Default)]
pub struct ImplicitSubtreeCacheOptions {
    /// Maximum number of subtrees to hold in memory (0 for unlimited).
    pub maximum_subtree_count: Option<usize>,
}

#[derive(Clone)]
struct ImplicitSubtreeCacheNode {
    subtree: Arc<ImplicitSubtree>,
    coordinates: ImplicitTileCoordinates,
    stamp: usize,
}

/// LRU-like priority cache for loaded implicit subtrees.
#[derive(Clone, Default)]
pub struct ImplicitSubtreeCache {
    maximum_subtree_count: usize,
    subtree_request_counter: usize,
    queue: Vec<ImplicitSubtreeCacheNode>,
}

impl ImplicitSubtreeCache {
    /// Creates a new `ImplicitSubtreeCache`.
    pub fn new(options: Option<ImplicitSubtreeCacheOptions>) -> Self {
        let opts = options.unwrap_or_default();
        Self {
            maximum_subtree_count: opts.maximum_subtree_count.unwrap_or(0),
            subtree_request_counter: 0,
            queue: Vec::new(),
        }
    }

    /// Number of subtrees currently stored in the cache.
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Subtree request counter tracking total additions.
    pub fn subtree_request_counter(&self) -> usize {
        self.subtree_request_counter
    }

    /// Maximum subtree capacity (0 for unlimited).
    pub fn maximum_subtree_count(&self) -> usize {
        self.maximum_subtree_count
    }

    /// Finds a cached subtree matching the given root coordinates.
    pub fn find(&self, coord: &ImplicitTileCoordinates) -> Option<Arc<ImplicitSubtree>> {
        self.queue
            .iter()
            .find(|node| node.coordinates.is_equal(coord))
            .map(|node| node.subtree.clone())
    }

    /// Adds a subtree to the cache, enforcing parent subtree existence and cache trimming.
    pub fn add_subtree(
        &mut self,
        subtree: Arc<ImplicitSubtree>,
        coord: ImplicitTileCoordinates,
    ) -> Result<(), DeveloperError> {
        let current_stamp = self.subtree_request_counter;
        self.subtree_request_counter += 1;

        if coord.level > 0 {
            let parent_coord = coord.get_parent_subtree_coordinates()?;
            if self.find(&parent_coord).is_none() {
                return Err(DeveloperError::new("parent node needs to exist"));
            }
        }

        self.queue.push(ImplicitSubtreeCacheNode {
            subtree,
            coordinates: coord,
            stamp: current_stamp,
        });

        if self.maximum_subtree_count > 0 {
            while self.queue.len() > self.maximum_subtree_count {
                // Find lowest priority node (earliest stamp that is not the newly added node)
                let mut candidate_idx = None;
                let mut min_stamp = usize::MAX;

                for (i, node) in self.queue.iter().enumerate() {
                    if node.stamp != current_stamp && node.stamp < min_stamp {
                        min_stamp = node.stamp;
                        candidate_idx = Some(i);
                    }
                }

                if let Some(idx) = candidate_idx {
                    self.queue.remove(idx);
                } else {
                    break;
                }
            }
        }

        Ok(())
    }
}
