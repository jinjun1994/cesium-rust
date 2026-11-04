use crate::{
    BoundingSphere, Cartographic, Ellipsoid, GeoRectangle, GeographicTilingScheme,
};
use std::collections::HashMap;
use std::sync::OnceLock;

static TERRAIN_HEIGHTS: OnceLock<HashMap<String, [f64; 2]>> = OnceLock::new();

fn get_terrain_heights_map() -> &'static HashMap<String, [f64; 2]> {
    TERRAIN_HEIGHTS.get_or_init(|| {
        let json_str = include_str!("../assets/approximateTerrainHeights.json");
        serde_json::from_str(json_str).unwrap_or_default()
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerrainHeights {
    pub minimum_terrain_height: f64,
    pub maximum_terrain_height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileXYLevel {
    pub x: u32,
    pub y: u32,
    pub level: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApproximateTerrainHeights;

impl ApproximateTerrainHeights {
    pub const TERRAIN_HEIGHTS_MAX_LEVEL: u32 = 6;
    pub const DEFAULT_MAX_TERRAIN_HEIGHT: f64 = 9000.0;
    pub const DEFAULT_MIN_TERRAIN_HEIGHT: f64 = -100000.0;

    /// Returns whether the terrain heights dataset is loaded and initialized.
    pub fn is_initialized() -> bool {
        !get_terrain_heights_map().is_empty()
    }

    /// Computes the tile x, y, and level that bounds the rectangle up to max level.
    pub fn get_tile_xy_level(rectangle: &GeoRectangle) -> Option<TileXYLevel> {
        let corners = [
            Cartographic::from_radians(rectangle.east, rectangle.north),
            Cartographic::from_radians(rectangle.west, rectangle.north),
            Cartographic::from_radians(rectangle.east, rectangle.south),
            Cartographic::from_radians(rectangle.west, rectangle.south),
        ];

        let tiling_scheme = GeographicTilingScheme::default();
        let mut last_level_x = 0;
        let mut last_level_y = 0;
        let mut reached_level = 0;

        for level in 0..=Self::TERRAIN_HEIGHTS_MAX_LEVEL {
            let mut current_x = 0;
            let mut current_y = 0;
            let mut failed = false;

            for (j, corner) in corners.iter().enumerate() {
                if let Some((x, y)) = tiling_scheme.position_to_tile(*corner, level) {
                    if j == 0 {
                        current_x = x;
                        current_y = y;
                    } else if current_x != x || current_y != y {
                        failed = true;
                        break;
                    }
                } else {
                    failed = true;
                    break;
                }
            }

            if failed {
                break;
            }

            last_level_x = current_x;
            last_level_y = current_y;
            reached_level = level + 1;
        }

        if reached_level == 0 {
            None
        } else {
            let level = if reached_level > Self::TERRAIN_HEIGHTS_MAX_LEVEL {
                Self::TERRAIN_HEIGHTS_MAX_LEVEL
            } else {
                reached_level - 1
            };
            Some(TileXYLevel {
                x: last_level_x,
                y: last_level_y,
                level,
            })
        }
    }

    /// Computes the minimum and maximum terrain heights for a given rectangle.
    pub fn get_minimum_maximum_heights(
        rectangle: &GeoRectangle,
        ellipsoid: Option<Ellipsoid>,
    ) -> TerrainHeights {
        let ellipsoid = ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let mut min_terrain_height = Self::DEFAULT_MIN_TERRAIN_HEIGHT;
        let mut max_terrain_height = Self::DEFAULT_MAX_TERRAIN_HEIGHT;

        if let Some(xy_level) = Self::get_tile_xy_level(rectangle) {
            let key = format!("{}-{}-{}", xy_level.level, xy_level.x, xy_level.y);
            let map = get_terrain_heights_map();
            if let Some(heights) = map.get(&key) {
                min_terrain_height = heights[0];
                max_terrain_height = heights[1];
            }

            let ne = ellipsoid.cartographic_to_cartesian(rectangle.northeast());
            let sw = ellipsoid.cartographic_to_cartesian(rectangle.southwest());

            if let (Some(ne), Some(sw)) = (ne, sw) {
                let center = (ne + sw) * 0.5;
                if let Some(surface) = ellipsoid.scale_to_geodetic_surface(center) {
                    let distance = (center - surface).length();
                    min_terrain_height = min_terrain_height.min(-distance);
                } else {
                    min_terrain_height = Self::DEFAULT_MIN_TERRAIN_HEIGHT;
                }
            } else {
                min_terrain_height = Self::DEFAULT_MIN_TERRAIN_HEIGHT;
            }
        }

        min_terrain_height = min_terrain_height.max(Self::DEFAULT_MIN_TERRAIN_HEIGHT);

        TerrainHeights {
            minimum_terrain_height: min_terrain_height,
            maximum_terrain_height: max_terrain_height,
        }
    }

    /// Computes the bounding sphere based on the tile heights in the rectangle.
    pub fn get_bounding_sphere(
        rectangle: &GeoRectangle,
        ellipsoid: Option<Ellipsoid>,
    ) -> BoundingSphere {
        let ellipsoid = ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let mut max_terrain_height = Self::DEFAULT_MAX_TERRAIN_HEIGHT;

        if let Some(xy_level) = Self::get_tile_xy_level(rectangle) {
            let key = format!("{}-{}-{}", xy_level.level, xy_level.x, xy_level.y);
            let map = get_terrain_heights_map();
            if let Some(heights) = map.get(&key) {
                max_terrain_height = heights[1];
            }
        }

        let bs0 = BoundingSphere::from_rectangle_3d(Some(*rectangle), ellipsoid, 0.0)
            .unwrap_or_else(|_| BoundingSphere::new(glam::DVec3::ZERO, 0.0));
        let bs_max = BoundingSphere::from_rectangle_3d(Some(*rectangle), ellipsoid, max_terrain_height)
            .unwrap_or_else(|_| BoundingSphere::new(glam::DVec3::ZERO, 0.0));

        bs0.union(bs_max)
    }
}
