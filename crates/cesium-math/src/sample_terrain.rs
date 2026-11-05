//! Terrain height sampling functions matching CesiumJS `Core/sampleTerrain` and `Core/sampleTerrainMostDetailed`.

use std::collections::HashMap;

use crate::cesium_terrain_provider::{CesiumTerrainProvider, CesiumTerrainTilingScheme};
use crate::errors::{DeveloperError, RuntimeError};
use crate::tiling::TilingScheme;
use crate::Cartographic;

/// Initiates a terrain height query for a slice of Cartographic positions by requesting tiles and interpolating.
pub async fn sample_terrain(
    terrain_provider: &CesiumTerrainProvider,
    level: u32,
    positions: &mut [Cartographic],
    reject_on_tile_fail: bool,
) -> Result<(), RuntimeError> {
    sample_terrain_sync(terrain_provider, level, positions, reject_on_tile_fail)
}

/// Synchronously queries terrain heights for positions at a specified level.
pub fn sample_terrain_sync(
    terrain_provider: &CesiumTerrainProvider,
    level: u32,
    positions: &mut [Cartographic],
    reject_on_tile_fail: bool,
) -> Result<(), RuntimeError> {
    let tiling_scheme = terrain_provider.tiling_scheme();
    let mut tile_requests: HashMap<(u32, u32), Vec<usize>> = HashMap::new();

    for (i, pos) in positions.iter_mut().enumerate() {
        let xy = match tiling_scheme {
            CesiumTerrainTilingScheme::Geographic(g) => g.position_to_tile(*pos, level),
            CesiumTerrainTilingScheme::WebMercator(w) => w.position_to_tile(*pos, level),
        };

        if let Some((x, y)) = xy {
            tile_requests.entry((x, y)).or_default().push(i);
        } else if reject_on_tile_fail {
            return Err(RuntimeError::new(format!(
                "Position at index {} is out of bounds at level {}.",
                i, level
            )));
        } else {
            pos.height = f64::NAN;
        }
    }

    for ((x, y), point_indices) in tile_requests {
        let geom_result = terrain_provider.request_tile_geometry_sync(x, y, level);
        match geom_result {
            Ok(terrain_data) => {
                let rect_opt = match tiling_scheme {
                    CesiumTerrainTilingScheme::Geographic(g) => g.tile_xy_to_rectangle(x, y, level),
                    CesiumTerrainTilingScheme::WebMercator(w) => w.tile_xy_to_rectangle(x, y, level),
                };

                let Some(rect) = rect_opt else {
                    if reject_on_tile_fail {
                        return Err(RuntimeError::new(format!(
                            "Invalid tile rectangle for ({}, {}, {}).",
                            level, x, y
                        )));
                    }
                    for &idx in &point_indices {
                        positions[idx].height = f64::NAN;
                    }
                    continue;
                };

                for &idx in &point_indices {
                    let pos = &mut positions[idx];
                    if let Some(height) = terrain_data.interpolate_height(&rect, pos.longitude, pos.latitude) {
                        pos.height = height;
                    } else if reject_on_tile_fail {
                        return Err(RuntimeError::new(format!(
                            "Failed to interpolate height at position index {}.",
                            idx
                        )));
                    } else {
                        pos.height = f64::NAN;
                    }
                }
            }
            Err(e) => {
                if reject_on_tile_fail {
                    return Err(e);
                }
                for &idx in &point_indices {
                    positions[idx].height = f64::NAN;
                }
            }
        }
    }

    Ok(())
}

/// Initiates a terrain height query at the maximum available tile level for each position.
pub async fn sample_terrain_most_detailed(
    terrain_provider: &CesiumTerrainProvider,
    positions: &mut [Cartographic],
    reject_on_tile_fail: bool,
) -> Result<(), RuntimeError> {
    sample_terrain_most_detailed_sync(terrain_provider, positions, reject_on_tile_fail)
}

/// Synchronously queries terrain heights at the maximum available tile level for each position.
pub fn sample_terrain_most_detailed_sync(
    terrain_provider: &CesiumTerrainProvider,
    positions: &mut [Cartographic],
    reject_on_tile_fail: bool,
) -> Result<(), RuntimeError> {
    let availability = terrain_provider
        .availability()
        .ok_or_else(|| DeveloperError::new("sampleTerrainMostDetailed requires a terrain provider that has tile availability."))
        .map_err(|e| RuntimeError::new(format!("{:?}", e)))?;

    // Group positions by maximum available level
    let mut by_level: HashMap<u32, Vec<usize>> = HashMap::new();
    for (i, pos) in positions.iter_mut().enumerate() {
        let max_lvl = availability.compute_maximum_level_at_position(*pos);
        if max_lvl < 0 {
            if reject_on_tile_fail {
                return Err(RuntimeError::new(format!(
                    "Position at index {} has no available terrain data.",
                    i
                )));
            }
            pos.height = f64::NAN;
        } else {
            by_level.entry(max_lvl as u32).or_default().push(i);
        }
    }

    for (lvl, indices) in by_level {
        let mut sub_positions: Vec<Cartographic> = indices.iter().map(|&idx| positions[idx]).collect();
        sample_terrain_sync(terrain_provider, lvl, &mut sub_positions, reject_on_tile_fail)?;
        for (&idx, sub_pos) in indices.iter().zip(sub_positions) {
            positions[idx] = sub_pos;
        }
    }

    Ok(())
}
