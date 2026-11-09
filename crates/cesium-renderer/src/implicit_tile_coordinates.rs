use std::collections::HashMap;
use cesium_math::DeveloperError;
use crate::implicit_subdivision_scheme::ImplicitSubdivisionScheme;

/// Coordinates of a tile within an implicit tileset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImplicitTileCoordinates {
    pub subdivision_scheme: ImplicitSubdivisionScheme,
    pub subtree_levels: u32,
    pub level: u32,
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl ImplicitTileCoordinates {
    /// Creates new coordinates with validation.
    pub fn new(
        subdivision_scheme: ImplicitSubdivisionScheme,
        subtree_levels: u32,
        level: u32,
        x: u32,
        y: u32,
        z: u32,
    ) -> Result<Self, DeveloperError> {
        let max_dim = 1u64 << level;
        if (x as u64) >= max_dim || (y as u64) >= max_dim {
            return Err(DeveloperError::new(format!(
                "Tile coordinates ({level}, {x}, {y}, {z}) out of bounds for dimension {max_dim}"
            )));
        }
        if subdivision_scheme == ImplicitSubdivisionScheme::Octree && (z as u64) >= max_dim {
            return Err(DeveloperError::new(format!(
                "Tile z-coordinate {z} out of bounds for level {level}"
            )));
        }
        Ok(Self {
            subdivision_scheme,
            subtree_levels,
            level,
            x,
            y,
            z: if subdivision_scheme == ImplicitSubdivisionScheme::Quadtree { 0 } else { z },
        })
    }

    /// Derives the coordinates for a child tile.
    pub fn get_child_coordinates(&self, child_index: usize) -> Self {
        let child_level = self.level + 1;
        let branching = self.subdivision_scheme.branching_factor();
        let idx = child_index % branching;

        let (dx, dy, dz) = match self.subdivision_scheme {
            ImplicitSubdivisionScheme::Quadtree => {
                let dx = (idx & 1) as u32;
                let dy = ((idx >> 1) & 1) as u32;
                (dx, dy, 0)
            }
            ImplicitSubdivisionScheme::Octree => {
                let dx = (idx & 1) as u32;
                let dy = ((idx >> 1) & 1) as u32;
                let dz = ((idx >> 2) & 1) as u32;
                (dx, dy, dz)
            }
        };

        Self {
            subdivision_scheme: self.subdivision_scheme,
            subtree_levels: self.subtree_levels,
            level: child_level,
            x: self.x * 2 + dx,
            y: self.y * 2 + dy,
            z: self.z * 2 + dz,
        }
    }

    /// Computes Morton index at current level.
    pub fn morton_index(&self) -> usize {
        let mut morton = 0usize;
        for i in 0..self.level {
            let bx = ((self.x >> i) & 1) as usize;
            let by = ((self.y >> i) & 1) as usize;
            if self.subdivision_scheme == ImplicitSubdivisionScheme::Quadtree {
                morton |= (bx | (by << 1)) << (2 * i);
            } else {
                let bz = ((self.z >> i) & 1) as usize;
                morton |= (bx | (by << 1) | (bz << 2)) << (3 * i);
            }
        }
        morton
    }

    /// Computes cumulative tile index relative to subtree root.
    pub fn relative_tile_index(&self, root_level: u32) -> usize {
        let rel_level = self.level.saturating_sub(root_level);
        let branching = self.subdivision_scheme.branching_factor();
        let mut offset = 0;
        let mut level_tiles = 1;
        for _ in 0..rel_level {
            offset += level_tiles;
            level_tiles *= branching;
        }
        offset + self.morton_index()
    }

    /// Derives ancestor coordinates `offset_levels` above this tile.
    pub fn get_ancestor_coordinates(&self, offset_levels: u32) -> Result<Self, DeveloperError> {
        if offset_levels > self.level {
            return Err(DeveloperError::new(
                "ancestor cannot be above the tileset root",
            ));
        }
        let divisor = 1u32 << offset_levels;
        let ancestor_level = self.level - offset_levels;
        let ancestor_x = self.x / divisor;
        let ancestor_y = self.y / divisor;
        let ancestor_z = if self.subdivision_scheme == ImplicitSubdivisionScheme::Octree {
            self.z / divisor
        } else {
            0
        };

        Ok(Self {
            subdivision_scheme: self.subdivision_scheme,
            subtree_levels: self.subtree_levels,
            level: ancestor_level,
            x: ancestor_x,
            y: ancestor_y,
            z: ancestor_z,
        })
    }

    /// Derives the parent subtree coordinates containing this tile.
    pub fn get_parent_subtree_coordinates(&self) -> Result<Self, DeveloperError> {
        let offset = (self.level % self.subtree_levels) + self.subtree_levels;
        self.get_ancestor_coordinates(offset)
    }

    /// Returns whether this tile is an ancestor of another tile.
    pub fn is_ancestor(&self, descendant: &Self) -> bool {
        if descendant.level <= self.level {
            return false;
        }
        let diff = descendant.level - self.level;
        let anc_x = descendant.x >> diff;
        let anc_y = descendant.y >> diff;
        if self.subdivision_scheme == ImplicitSubdivisionScheme::Octree {
            let anc_z = descendant.z >> diff;
            self.x == anc_x && self.y == anc_y && self.z == anc_z
        } else {
            self.x == anc_x && self.y == anc_y
        }
    }

    /// True if coordinates are identical.
    pub fn is_equal(&self, other: &Self) -> bool {
        self == other
    }

    /// Generates template variables for URI formatting ({level}, {x}, {y}, {z}).
    pub fn get_template_values(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("level".to_string(), self.level.to_string());
        map.insert("x".to_string(), self.x.to_string());
        map.insert("y".to_string(), self.y.to_string());
        map.insert("z".to_string(), self.z.to_string());
        map
    }
}
