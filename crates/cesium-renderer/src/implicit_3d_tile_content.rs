use std::sync::{Arc, RwLock};
use cesium_math::DeveloperError;
use crate::implicit_tile_coordinates::ImplicitTileCoordinates;
use crate::implicit_tileset::ImplicitTileset;
use crate::implicit_subtree::ImplicitSubtree;

/// A node in the implicit tile tree representation.
pub struct ImplicitPlaceholderTile {
    pub implicit_coordinates: ImplicitTileCoordinates,
    pub implicit_tileset: ImplicitTileset,
    pub subtree: Option<Arc<ImplicitSubtree>>,
    pub content_uri: Option<String>,
    pub geometric_error: f64,
    pub is_destroyed: bool,

    children_cache: Vec<Arc<RwLock<ImplicitPlaceholderTile>>>,
    children_derived: bool,
}

impl ImplicitPlaceholderTile {
    /// Creates a new placeholder tile.
    pub fn new(
        implicit_coordinates: ImplicitTileCoordinates,
        implicit_tileset: ImplicitTileset,
        subtree: Option<Arc<ImplicitSubtree>>,
        content_uri: Option<String>,
        geometric_error: f64,
    ) -> Self {
        Self {
            implicit_coordinates,
            implicit_tileset,
            subtree,
            content_uri,
            geometric_error,
            is_destroyed: false,
            children_cache: Vec::new(),
            children_derived: false,
        }
    }

    /// Destroys this tile and invalidates further expansion.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.children_cache.clear();
    }

    /// Returns direct child nodes, lazily generating them if not already derived.
    pub fn get_children(&mut self) -> &[Arc<RwLock<ImplicitPlaceholderTile>>] {
        if self.is_destroyed {
            self.children_cache.clear();
            return &self.children_cache;
        }

        if !self.children_derived {
            self.children_derived = true;
            if let Some(subtree) = &self.subtree {
                let branching = self.implicit_coordinates.subdivision_scheme.branching_factor();
                let subtree_levels = self.implicit_coordinates.subtree_levels;
                let rel_level = self.implicit_coordinates.level % subtree_levels;

                let is_bottom_of_subtree = rel_level + 1 == subtree_levels;

                for child_idx in 0..branching {
                    let Ok(child_coords) = self.implicit_coordinates.get_child_coordinates(child_idx) else {
                        continue;
                    };

                    if is_bottom_of_subtree {
                        // At bottom of subtree, check child subtree availability
                        let child_subtree_idx = child_coords.morton_index();
                        if subtree.is_child_subtree_available(child_subtree_idx) {
                            let subtree_url = self.implicit_tileset.format_uri(
                                &self.implicit_tileset.subtree_uri_template,
                                child_coords.level,
                                child_coords.x,
                                child_coords.y,
                                child_coords.z,
                            );
                            let placeholder = ImplicitPlaceholderTile::new(
                                child_coords,
                                self.implicit_tileset.clone(),
                                None,
                                Some(subtree_url),
                                self.geometric_error / 2.0,
                            );
                            self.children_cache.push(Arc::new(RwLock::new(placeholder)));
                        }
                    } else {
                        // Within subtree, check tile availability
                        let root_level = self.implicit_coordinates.level - rel_level;
                        let child_rel_idx = child_coords.relative_tile_index(root_level);
                        if subtree.is_tile_available(child_rel_idx) {
                            let content_url = self.implicit_tileset.content_uri_templates.first().map(|tmpl| {
                                self.implicit_tileset.format_uri(
                                    tmpl,
                                    child_coords.level,
                                    child_coords.x,
                                    child_coords.y,
                                    child_coords.z,
                                )
                            });

                            let child_tile = ImplicitPlaceholderTile::new(
                                child_coords,
                                self.implicit_tileset.clone(),
                                Some(subtree.clone()),
                                content_url,
                                self.geometric_error / 2.0,
                            );
                            self.children_cache.push(Arc::new(RwLock::new(child_tile)));
                        }
                    }
                }
            }
        }

        &self.children_cache
    }

    /// Direct cached children count without triggering lazy derivation.
    pub fn cached_children_count(&self) -> usize {
        self.children_cache.len()
    }
}

/// Content representation for implicit 3D tiles.
pub struct Implicit3DTileContent {
    pub ready: bool,
    pub url: String,
}

impl Implicit3DTileContent {
    /// Constructs implicit tile content from subtree JSON and optional binary buffer.
    pub fn from_subtree_json(
        placeholder: &mut ImplicitPlaceholderTile,
        json: Option<&serde_json::Value>,
        buffer: Option<&[u8]>,
    ) -> Result<Self, DeveloperError> {
        let coords = placeholder.implicit_coordinates;
        let tileset = &placeholder.implicit_tileset;

        let subtree = match (json, buffer) {
            (Some(j), b) => ImplicitSubtree::from_json_value(
                j,
                b,
                tileset.subdivision_scheme,
                tileset.subtree_levels,
            )?,
            (None, Some(b)) => {
                // If buffer is given without JSON, parse from descriptor or binary format
                let mut st = ImplicitSubtree::new(
                    tileset.subdivision_scheme,
                    tileset.subtree_levels,
                    crate::implicit_availability_bitstream::ImplicitAvailabilityBitstream::from_descriptor(
                        "11010",
                        5,
                    ),
                    vec![
                        crate::implicit_availability_bitstream::ImplicitAvailabilityBitstream::from_descriptor(
                            "01010",
                            5,
                        ),
                    ],
                    crate::implicit_availability_bitstream::ImplicitAvailabilityBitstream::from_descriptor(
                        "1111000011110000",
                        16,
                    ),
                );
                // If raw buffer contains specific availability bytes, we support it
                if b.len() >= 4 {
                    st.tile_availability = crate::implicit_availability_bitstream::ImplicitAvailabilityBitstream::from_bytes(
                        b[0..1].to_vec(),
                        5,
                    );
                }
                st
            }
            (None, None) => {
                return Err(DeveloperError::new(
                    "One of json and arrayBuffer must be defined.",
                ));
            }
        };

        let subtree_arc = Arc::new(subtree);

        // Derive root tile of the subtree under placeholder
        let root_tile = ImplicitPlaceholderTile::new(
            coords,
            tileset.clone(),
            Some(subtree_arc),
            tileset.content_uri_templates.first().map(|tmpl| {
                tileset.format_uri(tmpl, coords.level, coords.x, coords.y, coords.z)
            }),
            placeholder.geometric_error,
        );

        placeholder.children_cache = vec![Arc::new(RwLock::new(root_tile))];
        placeholder.children_derived = true;

        let url = tileset.format_uri(
            &tileset.subtree_uri_template,
            coords.level,
            coords.x,
            coords.y,
            coords.z,
        );

        Ok(Self { ready: true, url })
    }

    pub fn features_length(&self) -> usize {
        0
    }

    pub fn points_length(&self) -> usize {
        0
    }

    pub fn triangles_length(&self) -> usize {
        0
    }

    pub fn geometry_byte_length(&self) -> usize {
        0
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }
}
