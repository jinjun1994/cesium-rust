use std::f64::consts::PI;

use crate::Cartographic;
use crate::rectangle::GeoRectangle;
use crate::tiling::{GeographicTilingScheme, WebMercatorTilingScheme};

/// Supported tiling schemes for tile availability.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TileScheme {
    Geographic(GeographicTilingScheme),
    WebMercator(WebMercatorTilingScheme),
}

impl TileScheme {
    pub fn tile_rectangle(&self, x: u32, y: u32, level: u32) -> Option<GeoRectangle> {
        match self {
            Self::Geographic(g) => g.tile_rectangle(x, y, level),
            Self::WebMercator(w) => w.tile_rectangle(x, y, level),
        }
    }

    pub fn number_of_x_tiles_at_level(&self, level: u32) -> u32 {
        match self {
            Self::Geographic(g) => g.number_of_x_tiles_at_level(level),
            Self::WebMercator(w) => w.number_of_x_tiles_at_level(level),
        }
    }

    pub fn number_of_y_tiles_at_level(&self, level: u32) -> u32 {
        match self {
            Self::Geographic(g) => g.number_of_y_tiles_at_level(level),
            Self::WebMercator(w) => w.number_of_y_tiles_at_level(level),
        }
    }
}

impl From<GeographicTilingScheme> for TileScheme {
    fn from(g: GeographicTilingScheme) -> Self {
        Self::Geographic(g)
    }
}

impl From<WebMercatorTilingScheme> for TileScheme {
    fn from(w: WebMercatorTilingScheme) -> Self {
        Self::WebMercator(w)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectangleWithLevel {
    pub level: u32,
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}

impl RectangleWithLevel {
    pub fn new(level: u32, west: f64, south: f64, east: f64, north: f64) -> Self {
        Self {
            level,
            west,
            south,
            east,
            north,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuadtreeNode {
    pub parent: Option<usize>,
    pub level: u32,
    pub x: u32,
    pub y: u32,
    pub extent: GeoRectangle,
    pub rectangles: Vec<RectangleWithLevel>,
    pub nw: Option<usize>,
    pub ne: Option<usize>,
    pub sw: Option<usize>,
    pub se: Option<usize>,
}

/// Reports the availability of tiles in a tiling scheme.
#[derive(Clone, Debug, PartialEq)]
pub struct TileAvailability {
    pub scheme: TileScheme,
    pub maximum_level: u32,
    pub nodes: Vec<QuadtreeNode>,
    pub root_nodes: Vec<usize>,
}

impl TileAvailability {
    /// Creates a new `TileAvailability` tracker.
    pub fn new(scheme: impl Into<TileScheme>, maximum_level: u32) -> Self {
        Self {
            scheme: scheme.into(),
            maximum_level,
            nodes: Vec::new(),
            root_nodes: Vec::new(),
        }
    }

    fn alloc_node(
        &mut self,
        parent: Option<usize>,
        level: u32,
        x: u32,
        y: u32,
    ) -> usize {
        let extent = self
            .scheme
            .tile_rectangle(x, y, level)
            .unwrap_or(GeoRectangle::MAXIMUM);
        let id = self.nodes.len();
        self.nodes.push(QuadtreeNode {
            parent,
            level,
            x,
            y,
            extent,
            rectangles: Vec::new(),
            nw: None,
            ne: None,
            sw: None,
            se: None,
        });
        id
    }

    fn get_or_create_child(&mut self, parent_idx: usize, quad: usize) -> usize {
        let (level, x, y) = {
            let p = &self.nodes[parent_idx];
            match quad {
                0 => (p.level + 1, p.x * 2, p.y * 2),         // NW
                1 => (p.level + 1, p.x * 2 + 1, p.y * 2),     // NE
                2 => (p.level + 1, p.x * 2, p.y * 2 + 1),     // SW
                _ => (p.level + 1, p.x * 2 + 1, p.y * 2 + 1), // SE
            }
        };

        let child_idx = self.alloc_node(Some(parent_idx), level, x, y);
        let p = &mut self.nodes[parent_idx];
        match quad {
            0 => p.nw = Some(child_idx),
            1 => p.ne = Some(child_idx),
            2 => p.sw = Some(child_idx),
            _ => p.se = Some(child_idx),
        }
        child_idx
    }

    /// Marks a rectangular range of tiles in a particular level as being available.
    pub fn add_available_tile_range(
        &mut self,
        level: u32,
        start_x: u32,
        start_y: u32,
        end_x: u32,
        end_y: u32,
    ) {
        if level == 0 {
            for y in start_y..=end_y {
                for x in start_x..=end_x {
                    let mut exists = false;
                    for &root_idx in &self.root_nodes {
                        let node = &self.nodes[root_idx];
                        if node.level == 0 && node.x == x && node.y == y {
                            exists = true;
                            break;
                        }
                    }
                    if !exists {
                        let root_id = self.alloc_node(None, 0, x, y);
                        self.root_nodes.push(root_id);
                    }
                }
            }
        }

        let start_rect = self
            .scheme
            .tile_rectangle(start_x, start_y, level)
            .unwrap_or(GeoRectangle::MAXIMUM);
        let end_rect = self
            .scheme
            .tile_rectangle(end_x, end_y, level)
            .unwrap_or(GeoRectangle::MAXIMUM);

        let west = start_rect.west;
        let north = start_rect.north;
        let east = end_rect.east;
        let south = end_rect.south;

        let rectangle_with_level = RectangleWithLevel::new(level, west, south, east, north);

        let root_count = self.root_nodes.len();
        for i in 0..root_count {
            let root_idx = self.root_nodes[i];
            let root_extent = self.nodes[root_idx].extent;
            if rectangles_overlap(
                root_extent.west,
                root_extent.south,
                root_extent.east,
                root_extent.north,
                rectangle_with_level.west,
                rectangle_with_level.south,
                rectangle_with_level.east,
                rectangle_with_level.north,
            ) {
                self.put_rectangle_in_quadtree(root_idx, rectangle_with_level);
            }
        }
    }

    fn put_rectangle_in_quadtree(&mut self, mut node_idx: usize, rect: RectangleWithLevel) {
        let max_depth = self.maximum_level;

        while self.nodes[node_idx].level < max_depth {
            // Check NW
            let nw_child = self.nodes[node_idx].nw.unwrap_or_else(|| self.get_or_create_child(node_idx, 0));
            if rectangle_fully_contains_rectangle(
                self.nodes[nw_child].extent.west,
                self.nodes[nw_child].extent.south,
                self.nodes[nw_child].extent.east,
                self.nodes[nw_child].extent.north,
                rect.west,
                rect.south,
                rect.east,
                rect.north,
            ) {
                node_idx = nw_child;
                continue;
            }

            // Check NE
            let ne_child = self.nodes[node_idx].ne.unwrap_or_else(|| self.get_or_create_child(node_idx, 1));
            if rectangle_fully_contains_rectangle(
                self.nodes[ne_child].extent.west,
                self.nodes[ne_child].extent.south,
                self.nodes[ne_child].extent.east,
                self.nodes[ne_child].extent.north,
                rect.west,
                rect.south,
                rect.east,
                rect.north,
            ) {
                node_idx = ne_child;
                continue;
            }

            // Check SW
            let sw_child = self.nodes[node_idx].sw.unwrap_or_else(|| self.get_or_create_child(node_idx, 2));
            if rectangle_fully_contains_rectangle(
                self.nodes[sw_child].extent.west,
                self.nodes[sw_child].extent.south,
                self.nodes[sw_child].extent.east,
                self.nodes[sw_child].extent.north,
                rect.west,
                rect.south,
                rect.east,
                rect.north,
            ) {
                node_idx = sw_child;
                continue;
            }

            // Check SE
            let se_child = self.nodes[node_idx].se.unwrap_or_else(|| self.get_or_create_child(node_idx, 3));
            if rectangle_fully_contains_rectangle(
                self.nodes[se_child].extent.west,
                self.nodes[se_child].extent.south,
                self.nodes[se_child].extent.east,
                self.nodes[se_child].extent.north,
                rect.west,
                rect.south,
                rect.east,
                rect.north,
            ) {
                node_idx = se_child;
                continue;
            }

            break;
        }

        let node = &mut self.nodes[node_idx];
        if node.rectangles.is_empty() || node.rectangles.last().unwrap().level <= rect.level {
            node.rectangles.push(rect);
        } else {
            let insert_pos = match node.rectangles.binary_search_by_key(&rect.level, |r| r.level) {
                Ok(idx) => idx,
                Err(idx) => idx,
            };
            node.rectangles.insert(insert_pos, rect);
        }
    }

    /// Determines the level of the most detailed tile covering the position.
    /// Returns `-1` if the position is outside the tiling scheme.
    pub fn compute_maximum_level_at_position(&self, position: Cartographic) -> i32 {
        let mut target_root = None;
        for &root_idx in &self.root_nodes {
            let extent = self.nodes[root_idx].extent;
            if rectangle_contains_position(
                extent.west,
                extent.south,
                extent.east,
                extent.north,
                position.longitude,
                position.latitude,
            ) {
                target_root = Some(root_idx);
                break;
            }
        }

        let root_idx = match target_root {
            Some(idx) => idx,
            None => return -1,
        };

        self.find_max_level_from_node(None, root_idx, position) as i32
    }

    fn find_max_level_from_node(
        &self,
        stop_node: Option<usize>,
        mut node_idx: usize,
        position: Cartographic,
    ) -> u32 {
        let mut max_level = 0;

        let mut found = false;
        while !found {
            let nw = self.nodes[node_idx].nw.filter(|&child| {
                let ext = self.nodes[child].extent;
                rectangle_contains_position(ext.west, ext.south, ext.east, ext.north, position.longitude, position.latitude)
            });
            let ne = self.nodes[node_idx].ne.filter(|&child| {
                let ext = self.nodes[child].extent;
                rectangle_contains_position(ext.west, ext.south, ext.east, ext.north, position.longitude, position.latitude)
            });
            let sw = self.nodes[node_idx].sw.filter(|&child| {
                let ext = self.nodes[child].extent;
                rectangle_contains_position(ext.west, ext.south, ext.east, ext.north, position.longitude, position.latitude)
            });
            let se = self.nodes[node_idx].se.filter(|&child| {
                let ext = self.nodes[child].extent;
                rectangle_contains_position(ext.west, ext.south, ext.east, ext.north, position.longitude, position.latitude)
            });

            let count = nw.is_some() as usize + ne.is_some() as usize + sw.is_some() as usize + se.is_some() as usize;
            if count > 1 {
                if let Some(child) = nw {
                    max_level = max_level.max(self.find_max_level_from_node(Some(node_idx), child, position));
                }
                if let Some(child) = ne {
                    max_level = max_level.max(self.find_max_level_from_node(Some(node_idx), child, position));
                }
                if let Some(child) = sw {
                    max_level = max_level.max(self.find_max_level_from_node(Some(node_idx), child, position));
                }
                if let Some(child) = se {
                    max_level = max_level.max(self.find_max_level_from_node(Some(node_idx), child, position));
                }
                break;
            } else if let Some(child) = nw.or(ne).or(sw).or(se) {
                node_idx = child;
            } else {
                found = true;
            }
        }

        let mut curr = Some(node_idx);
        while curr != stop_node {
            let c_idx = match curr {
                Some(idx) => idx,
                None => break,
            };
            let node = &self.nodes[c_idx];
            for r in node.rectangles.iter().rev() {
                if r.level <= max_level {
                    break;
                }
                if rectangle_contains_position(r.west, r.south, r.east, r.north, position.longitude, position.latitude) {
                    max_level = r.level;
                }
            }
            curr = node.parent;
        }

        max_level
    }

    /// Finds the most detailed level that is available everywhere within a given rectangle.
    pub fn compute_best_available_level_over_rectangle(&self, rectangle: GeoRectangle) -> u32 {
        let mut rectangles = Vec::new();
        if rectangle.east < rectangle.west {
            rectangles.push(GeoRectangle::new(-PI, rectangle.south, rectangle.east, rectangle.north));
            rectangles.push(GeoRectangle::new(rectangle.west, rectangle.south, PI, rectangle.north));
        } else {
            rectangles.push(rectangle);
        }

        let mut remaining_to_cover_by_level: Vec<Option<Vec<GeoRectangle>>> =
            vec![None; (self.maximum_level + 1) as usize];

        for &root_idx in &self.root_nodes {
            self.update_coverage_with_node(&mut remaining_to_cover_by_level, root_idx, &rectangles);
        }

        for (level, remaining) in remaining_to_cover_by_level.iter().enumerate().rev() {
            if let Some(rem) = remaining {
                if rem.is_empty() {
                    return level as u32;
                }
            }
        }

        0
    }

    fn update_coverage_with_node(
        &self,
        remaining_to_cover_by_level: &mut [Option<Vec<GeoRectangle>>],
        node_idx: usize,
        rectangles_to_cover: &[GeoRectangle],
    ) {
        let node = &self.nodes[node_idx];
        let mut any_overlap = false;
        for r in rectangles_to_cover {
            if rectangles_overlap(
                node.extent.west,
                node.extent.south,
                node.extent.east,
                node.extent.north,
                r.west,
                r.south,
                r.east,
                r.north,
            ) {
                any_overlap = true;
                break;
            }
        }

        if !any_overlap {
            return;
        }

        for rect in &node.rectangles {
            let level = rect.level as usize;
            if level < remaining_to_cover_by_level.len() {
                if remaining_to_cover_by_level[level].is_none() {
                    remaining_to_cover_by_level[level] = Some(rectangles_to_cover.to_vec());
                }
                if let Some(ref mut cur_rem) = remaining_to_cover_by_level[level] {
                    *cur_rem = subtract_rectangle(cur_rem, rect);
                }
            }
        }

        if let Some(child) = node.nw {
            self.update_coverage_with_node(remaining_to_cover_by_level, child, rectangles_to_cover);
        }
        if let Some(child) = node.ne {
            self.update_coverage_with_node(remaining_to_cover_by_level, child, rectangles_to_cover);
        }
        if let Some(child) = node.sw {
            self.update_coverage_with_node(remaining_to_cover_by_level, child, rectangles_to_cover);
        }
        if let Some(child) = node.se {
            self.update_coverage_with_node(remaining_to_cover_by_level, child, rectangles_to_cover);
        }
    }

    /// Determines if a particular tile is available.
    pub fn is_tile_available(&self, level: u32, x: u32, y: u32) -> bool {
        let rect = match self.scheme.tile_rectangle(x, y, level) {
            Some(r) => r,
            None => return false,
        };
        let center = Cartographic::new(
            rect.west + rect.width() * 0.5,
            rect.south + rect.height() * 0.5,
            0.0,
        );
        self.compute_maximum_level_at_position(center) >= level as i32
    }

    /// Computes a bit mask indicating which of a tile's four children exist:
    /// SW (1), SE (2), NW (4), NE (8).
    pub fn compute_child_mask_for_tile(&self, level: u32, x: u32, y: u32) -> u8 {
        let child_level = level + 1;
        if child_level >= self.maximum_level {
            return 0;
        }

        let mut mask = 0;
        if self.is_tile_available(child_level, 2 * x, 2 * y + 1) {
            mask |= 1;
        }
        if self.is_tile_available(child_level, 2 * x + 1, 2 * y + 1) {
            mask |= 2;
        }
        if self.is_tile_available(child_level, 2 * x, 2 * y) {
            mask |= 4;
        }
        if self.is_tile_available(child_level, 2 * x + 1, 2 * y) {
            mask |= 8;
        }

        mask
    }
}

#[allow(clippy::too_many_arguments)]
pub fn rectangles_overlap(
    r1_west: f64,
    r1_south: f64,
    r1_east: f64,
    r1_north: f64,
    r2_west: f64,
    r2_south: f64,
    r2_east: f64,
    r2_north: f64,
) -> bool {
    let west = r1_west.max(r2_west);
    let south = r1_south.max(r2_south);
    let east = r1_east.min(r2_east);
    let north = r1_north.min(r2_north);
    south < north && west < east
}

#[allow(clippy::too_many_arguments)]
pub fn rectangle_fully_contains_rectangle(
    container_west: f64,
    container_south: f64,
    container_east: f64,
    container_north: f64,
    r_west: f64,
    r_south: f64,
    r_east: f64,
    r_north: f64,
) -> bool {
    r_west >= container_west
        && r_east <= container_east
        && r_south >= container_south
        && r_north <= container_north
}

pub fn rectangle_contains_position(
    r_west: f64,
    r_south: f64,
    r_east: f64,
    r_north: f64,
    longitude: f64,
    latitude: f64,
) -> bool {
    longitude >= r_west && longitude <= r_east && latitude >= r_south && latitude <= r_north
}

fn subtract_rectangle(
    rectangle_list: &[GeoRectangle],
    rectangle_to_subtract: &RectangleWithLevel,
) -> Vec<GeoRectangle> {
    let mut result = Vec::new();
    for rectangle in rectangle_list {
        if !rectangles_overlap(
            rectangle.west,
            rectangle.south,
            rectangle.east,
            rectangle.north,
            rectangle_to_subtract.west,
            rectangle_to_subtract.south,
            rectangle_to_subtract.east,
            rectangle_to_subtract.north,
        ) {
            result.push(*rectangle);
        } else {
            if rectangle.west < rectangle_to_subtract.west {
                result.push(GeoRectangle::new(
                    rectangle.west,
                    rectangle.south,
                    rectangle_to_subtract.west,
                    rectangle.north,
                ));
            }
            if rectangle.east > rectangle_to_subtract.east {
                result.push(GeoRectangle::new(
                    rectangle_to_subtract.east,
                    rectangle.south,
                    rectangle.east,
                    rectangle.north,
                ));
            }
            if rectangle.south < rectangle_to_subtract.south {
                result.push(GeoRectangle::new(
                    rectangle_to_subtract.west.max(rectangle.west),
                    rectangle.south,
                    rectangle_to_subtract.east.min(rectangle.east),
                    rectangle_to_subtract.south,
                ));
            }
            if rectangle.north > rectangle_to_subtract.north {
                result.push(GeoRectangle::new(
                    rectangle_to_subtract.west.max(rectangle.west),
                    rectangle_to_subtract.north,
                    rectangle_to_subtract.east.min(rectangle.east),
                    rectangle.north,
                ));
            }
        }
    }
    result
}
