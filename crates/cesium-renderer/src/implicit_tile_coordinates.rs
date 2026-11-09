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
    /// Helper for quadtree coordinates
    pub fn quadtree(level: u32, x: u32, y: u32, subtree_levels: Option<u32>) -> Result<Self, DeveloperError> {
        Self::new(
            ImplicitSubdivisionScheme::Quadtree,
            subtree_levels.unwrap_or(2),
            level,
            x,
            y,
            0,
        )
    }

    /// Helper for octree coordinates
    pub fn octree(level: u32, x: u32, y: u32, z: u32, subtree_levels: Option<u32>) -> Result<Self, DeveloperError> {
        Self::new(
            ImplicitSubdivisionScheme::Octree,
            subtree_levels.unwrap_or(2),
            level,
            x,
            y,
            z,
        )
    }

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
        if (x as u64) >= max_dim {
            return Err(DeveloperError::new(format!(
                "Tile x-coordinate {x} is out of range for level {level} (max {max_dim})"
            )));
        }
        if (y as u64) >= max_dim {
            return Err(DeveloperError::new(format!(
                "Tile y-coordinate {y} is out of range for level {level} (max {max_dim})"
            )));
        }
        if subdivision_scheme == ImplicitSubdivisionScheme::Octree && (z as u64) >= max_dim {
            return Err(DeveloperError::new(format!(
                "Tile z-coordinate {z} is out of range for level {level} (max {max_dim})"
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

    /// Checks that two coordinate frames share the same subdivision scheme and subtree level height.
    pub fn check_matching_subtree_shape(&self, other: &Self) -> Result<(), DeveloperError> {
        if self.subdivision_scheme != other.subdivision_scheme {
            return Err(DeveloperError::new("coordinates must have same subdivisionScheme"));
        }
        if self.subtree_levels != other.subtree_levels {
            return Err(DeveloperError::new("coordinates must have same subtreeLevels"));
        }
        Ok(())
    }

    /// Derives the coordinates for a child tile.
    pub fn get_child_coordinates(&self, child_index: usize) -> Result<Self, DeveloperError> {
        let branching = self.subdivision_scheme.branching_factor();
        if child_index >= branching {
            return Err(DeveloperError::new(format!(
                "childIndex must be at least 0 and less than {branching}"
            )));
        }

        let child_level = self.level + 1;
        let (dx, dy, dz) = match self.subdivision_scheme {
            ImplicitSubdivisionScheme::Quadtree => {
                let dx = (child_index & 1) as u32;
                let dy = ((child_index >> 1) & 1) as u32;
                (dx, dy, 0)
            }
            ImplicitSubdivisionScheme::Octree => {
                let dx = (child_index & 1) as u32;
                let dy = ((child_index >> 1) & 1) as u32;
                let dz = ((child_index >> 2) & 1) as u32;
                (dx, dy, dz)
            }
        };

        Self::new(
            self.subdivision_scheme,
            self.subtree_levels,
            child_level,
            2 * self.x + dx,
            2 * self.y + dy,
            2 * self.z + dz,
        )
    }

    /// Computes descendant coordinates given a relative offset coordinate.
    pub fn get_descendant_coordinates(&self, offset_coordinates: &Self) -> Result<Self, DeveloperError> {
        self.check_matching_subtree_shape(offset_coordinates)?;
        let descendant_level = self.level + offset_coordinates.level;
        let descendant_x = (self.x << offset_coordinates.level) + offset_coordinates.x;
        let descendant_y = (self.y << offset_coordinates.level) + offset_coordinates.y;
        let descendant_z = if self.subdivision_scheme == ImplicitSubdivisionScheme::Octree {
            (self.z << offset_coordinates.level) + offset_coordinates.z
        } else {
            0
        };

        Self::new(
            self.subdivision_scheme,
            self.subtree_levels,
            descendant_level,
            descendant_x,
            descendant_y,
            descendant_z,
        )
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

    /// Computes relative offset coordinates between this ancestor and descendant.
    pub fn get_offset_coordinates(&self, descendant: &Self) -> Result<Self, DeveloperError> {
        self.check_matching_subtree_shape(descendant)?;
        if !self.is_equal(descendant) && !self.is_ancestor(descendant)? {
            return Err(DeveloperError::new("this is not an ancestor of descendant"));
        }

        let offset_level = descendant.level - self.level;
        let dim = 1u32 << offset_level;
        let offset_x = descendant.x % dim;
        let offset_y = descendant.y % dim;
        let offset_z = if self.subdivision_scheme == ImplicitSubdivisionScheme::Octree {
            descendant.z % dim
        } else {
            0
        };

        Self::new(
            self.subdivision_scheme,
            self.subtree_levels,
            offset_level,
            offset_x,
            offset_y,
            offset_z,
        )
    }

    /// Child index in parent ([0, branchingFactor)).
    pub fn child_index(&self) -> usize {
        let mut idx = (self.x & 1) as usize;
        idx |= ((self.y & 1) as usize) << 1;
        if self.subdivision_scheme == ImplicitSubdivisionScheme::Octree {
            idx |= ((self.z & 1) as usize) << 2;
        }
        idx
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

    /// Tile index within tree: levelOffset + mortonIndex.
    pub fn tile_index(&self) -> usize {
        let level_offset = if self.subdivision_scheme == ImplicitSubdivisionScheme::Octree {
            ((1usize << (3 * self.level)) - 1) / 7
        } else {
            ((1usize << (2 * self.level)) - 1) / 3
        };
        level_offset + self.morton_index()
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

    /// Creates coordinates from Morton index at `level`.
    pub fn from_morton_index(
        subdivision_scheme: ImplicitSubdivisionScheme,
        subtree_levels: u32,
        level: u32,
        morton_index: usize,
    ) -> Self {
        if subdivision_scheme == ImplicitSubdivisionScheme::Octree {
            let mut x = 0u32;
            let mut y = 0u32;
            let mut z = 0u32;
            for i in 0..21 {
                let bx = ((morton_index >> (3 * i)) & 1) as u32;
                let by = ((morton_index >> (3 * i + 1)) & 1) as u32;
                let bz = ((morton_index >> (3 * i + 2)) & 1) as u32;
                x |= bx << i;
                y |= by << i;
                z |= bz << i;
            }
            Self {
                subdivision_scheme,
                subtree_levels,
                level,
                x,
                y,
                z,
            }
        } else {
            let mut x = 0u32;
            let mut y = 0u32;
            for i in 0..32 {
                let bx = ((morton_index >> (2 * i)) & 1) as u32;
                let by = ((morton_index >> (2 * i + 1)) & 1) as u32;
                x |= bx << i;
                y |= by << i;
            }
            Self {
                subdivision_scheme,
                subtree_levels,
                level,
                x,
                y,
                z: 0,
            }
        }
    }

    /// Creates coordinates from a cumulative tile index relative to subtree root.
    pub fn from_tile_index(
        subdivision_scheme: ImplicitSubdivisionScheme,
        subtree_levels: u32,
        tile_index: usize,
    ) -> Self {
        let (level, morton_index) = if subdivision_scheme == ImplicitSubdivisionScheme::Octree {
            let l = (((7 * tile_index + 1) as f64).log2() / 3.0).floor() as u32;
            let offset = ((1usize << (3 * l)) - 1) / 7;
            (l, tile_index - offset)
        } else {
            let l = (((3 * tile_index + 1) as f64).log2() / 2.0).floor() as u32;
            let offset = ((1usize << (2 * l)) - 1) / 3;
            (l, tile_index - offset)
        };

        Self::from_morton_index(subdivision_scheme, subtree_levels, level, morton_index)
    }

    /// Gets subtree root coordinates containing this tile.
    pub fn get_subtree_coordinates(&self) -> Result<Self, DeveloperError> {
        self.get_ancestor_coordinates(self.level % self.subtree_levels)
    }

    /// Derives the parent subtree coordinates containing this tile.
    pub fn get_parent_subtree_coordinates(&self) -> Result<Self, DeveloperError> {
        let offset = (self.level % self.subtree_levels) + self.subtree_levels;
        self.get_ancestor_coordinates(offset)
    }

    /// Returns whether this tile is an ancestor of another tile.
    pub fn is_ancestor(&self, descendant: &Self) -> Result<bool, DeveloperError> {
        self.check_matching_subtree_shape(descendant)?;
        if descendant.level <= self.level {
            return Ok(false);
        }
        let diff = descendant.level - self.level;
        let anc_x = descendant.x >> diff;
        let anc_y = descendant.y >> diff;
        if self.subdivision_scheme == ImplicitSubdivisionScheme::Octree {
            let anc_z = descendant.z >> diff;
            Ok(self.x == anc_x && self.y == anc_y && self.z == anc_z)
        } else {
            Ok(self.x == anc_x && self.y == anc_y)
        }
    }

    /// Returns whether this tile is the root of the implicit tileset.
    pub fn is_implicit_tileset_root(&self) -> bool {
        self.level == 0
    }

    /// Returns whether this tile is the root of its subtree.
    pub fn is_subtree_root(&self) -> bool {
        self.level.is_multiple_of(self.subtree_levels)
    }

    /// Returns whether this tile is at the bottom row of its subtree.
    pub fn is_bottom_of_subtree(&self) -> bool {
        self.level % self.subtree_levels == self.subtree_levels - 1
    }

    /// True if coordinates are identical.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.subdivision_scheme == other.subdivision_scheme
            && self.subtree_levels == other.subtree_levels
            && self.level == other.level
            && self.x == other.x
            && self.y == other.y
            && (self.subdivision_scheme != ImplicitSubdivisionScheme::Octree || self.z == other.z)
    }

    /// Generates template variables for URI formatting ({level}, {x}, {y}, {z}).
    pub fn get_template_values(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("level".to_string(), self.level.to_string());
        map.insert("x".to_string(), self.x.to_string());
        map.insert("y".to_string(), self.y.to_string());
        if self.subdivision_scheme == ImplicitSubdivisionScheme::Octree {
            map.insert("z".to_string(), self.z.to_string());
        }
        map
    }
}
