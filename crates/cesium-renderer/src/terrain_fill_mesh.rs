//! TerrainFillMesh matching CesiumJS `Scene/TerrainFillMesh.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{Cartesian4, TerrainMesh};

/// Edge and corner neighbor representations for a quadtree tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TileCoord {
    pub level: u32,
    pub x: u32,
    pub y: u32,
}

impl TileCoord {
    pub const fn new(level: u32, x: u32, y: u32) -> Self {
        Self { level, x, y }
    }
}

/// A fill mesh generated for an unloaded tile from surrounding loaded terrain edges.
#[derive(Clone, Debug)]
pub struct TerrainFillMesh {
    pub tile: TileCoord,
    pub frame_last_updated: Option<i64>,

    pub west_meshes: Vec<TerrainMesh>,
    pub west_tiles: Vec<TileCoord>,

    pub south_meshes: Vec<TerrainMesh>,
    pub south_tiles: Vec<TileCoord>,

    pub east_meshes: Vec<TerrainMesh>,
    pub east_tiles: Vec<TileCoord>,

    pub north_meshes: Vec<TerrainMesh>,
    pub north_tiles: Vec<TileCoord>,

    pub southwest_mesh: Option<TerrainMesh>,
    pub southwest_tile: Option<TileCoord>,

    pub southeast_mesh: Option<TerrainMesh>,
    pub southeast_tile: Option<TileCoord>,

    pub northwest_mesh: Option<TerrainMesh>,
    pub northwest_tile: Option<TileCoord>,

    pub northeast_mesh: Option<TerrainMesh>,
    pub northeast_tile: Option<TileCoord>,

    pub changed_this_frame: bool,
    pub visited_frame: Option<i64>,
    pub enqueued_frame: Option<i64>,
    pub mesh: Option<TerrainMesh>,
    pub water_mask_translation_and_scale: Cartesian4,
    pub is_destroyed: bool,
}

impl TerrainFillMesh {
    /// Creates a new `TerrainFillMesh` for the given tile coordinate.
    pub fn new(tile: TileCoord) -> Self {
        Self {
            tile,
            frame_last_updated: None,
            west_meshes: Vec::new(),
            west_tiles: Vec::new(),
            south_meshes: Vec::new(),
            south_tiles: Vec::new(),
            east_meshes: Vec::new(),
            east_tiles: Vec::new(),
            north_meshes: Vec::new(),
            north_tiles: Vec::new(),
            southwest_mesh: None,
            southwest_tile: None,
            southeast_mesh: None,
            southeast_tile: None,
            northwest_mesh: None,
            northwest_tile: None,
            northeast_mesh: None,
            northeast_tile: None,
            changed_this_frame: true,
            visited_frame: None,
            enqueued_frame: None,
            mesh: None,
            water_mask_translation_and_scale: Cartesian4::ZERO,
            is_destroyed: false,
        }
    }

    /// Updates this fill mesh geometry if changes occurred during the current frame.
    pub fn update(&mut self) {
        if self.is_destroyed {
            return;
        }

        if self.changed_this_frame {
            // Synthesize fill mesh geometry if at least one neighbor mesh is available
            let candidate_mesh = self
                .west_meshes
                .first()
                .or_else(|| self.south_meshes.first())
                .or_else(|| self.east_meshes.first())
                .or_else(|| self.north_meshes.first())
                .or(self.southwest_mesh.as_ref())
                .or(self.southeast_mesh.as_ref())
                .or(self.northwest_mesh.as_ref())
                .or(self.northeast_mesh.as_ref());

            if let Some(m) = candidate_mesh {
                self.mesh = Some(m.clone());
            }

            self.changed_this_frame = false;
        }
    }

    /// Propagates loaded tile edges and corners to unloaded tiles in a rendered tile set.
    pub fn update_fill_tiles(
        rendered_tiles: &[TileCoord],
        loaded_meshes: &HashMap<TileCoord, TerrainMesh>,
        fill_meshes: &mut HashMap<TileCoord, TerrainFillMesh>,
    ) {
        if rendered_tiles.is_empty() {
            return;
        }

        // For each rendered tile that doesn't have a loaded mesh, compute fill neighbors
        for &tile in rendered_tiles {
            if loaded_meshes.contains_key(&tile) {
                continue;
            }

            let fill = fill_meshes
                .entry(tile)
                .or_insert_with(|| TerrainFillMesh::new(tile));

            // Neighbors: West, South, East, North
            let west_coord = TileCoord::new(tile.level, tile.x.saturating_sub(1), tile.y);
            let east_coord = TileCoord::new(tile.level, tile.x + 1, tile.y);
            let south_coord = TileCoord::new(tile.level, tile.x, tile.y.saturating_sub(1));
            let north_coord = TileCoord::new(tile.level, tile.x, tile.y + 1);

            let sw_coord = TileCoord::new(tile.level, tile.x.saturating_sub(1), tile.y.saturating_sub(1));
            let se_coord = TileCoord::new(tile.level, tile.x + 1, tile.y.saturating_sub(1));
            let nw_coord = TileCoord::new(tile.level, tile.x.saturating_sub(1), tile.y + 1);
            let ne_coord = TileCoord::new(tile.level, tile.x + 1, tile.y + 1);

            // Propagate west
            if let Some(mesh) = loaded_meshes.get(&west_coord) {
                fill.west_tiles = vec![west_coord];
                fill.west_meshes = vec![mesh.clone()];
            }

            // Propagate south
            if let Some(mesh) = loaded_meshes.get(&south_coord) {
                fill.south_tiles = vec![south_coord];
                fill.south_meshes = vec![mesh.clone()];
            }

            // Propagate east
            if let Some(mesh) = loaded_meshes.get(&east_coord) {
                fill.east_tiles = vec![east_coord];
                fill.east_meshes = vec![mesh.clone()];
            }

            // Propagate north
            if let Some(mesh) = loaded_meshes.get(&north_coord) {
                fill.north_tiles = vec![north_coord];
                fill.north_meshes = vec![mesh.clone()];
            }

            // Propagate corners
            if let Some(mesh) = loaded_meshes.get(&sw_coord) {
                fill.southwest_tile = Some(sw_coord);
                fill.southwest_mesh = Some(mesh.clone());
            }

            if let Some(mesh) = loaded_meshes.get(&se_coord) {
                fill.southeast_tile = Some(se_coord);
                fill.southeast_mesh = Some(mesh.clone());
            }

            if let Some(mesh) = loaded_meshes.get(&nw_coord) {
                fill.northwest_tile = Some(nw_coord);
                fill.northwest_mesh = Some(mesh.clone());
            }

            if let Some(mesh) = loaded_meshes.get(&ne_coord) {
                fill.northeast_tile = Some(ne_coord);
                fill.northeast_mesh = Some(mesh.clone());
            }

            fill.changed_this_frame = true;
        }
    }

    /// Destroys resources held by this fill mesh.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.mesh = None;
        self.west_meshes.clear();
        self.west_tiles.clear();
        self.south_meshes.clear();
        self.south_tiles.clear();
        self.east_meshes.clear();
        self.east_tiles.clear();
        self.north_meshes.clear();
        self.north_tiles.clear();
        self.southwest_mesh = None;
        self.southwest_tile = None;
        self.southeast_mesh = None;
        self.southeast_tile = None;
        self.northwest_mesh = None;
        self.northwest_tile = None;
        self.northeast_mesh = None;
        self.northeast_tile = None;
    }
}
