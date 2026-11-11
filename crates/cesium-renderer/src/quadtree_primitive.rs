//! QuadtreePrimitive matching CesiumJS `Scene/QuadtreePrimitive.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{GeoRectangle, Visibility};

use crate::quadtree_tile::{
    AnyTilingScheme, CustomDataEntry, QuadtreeTile, TileSelectionResult,
};
use crate::quadtree_tile_load_state::QuadtreeTileLoadState;
use crate::scene_primitive::PrimitiveRenderFrameState;

/// Trait for tile providers used with `QuadtreePrimitive`.
pub trait QuadtreeTileProvider {
    fn ready(&self) -> bool;
    fn tiling_scheme(&self) -> AnyTilingScheme;
    fn initialize(&mut self, frame_state: &mut PrimitiveRenderFrameState);
    fn begin_update(&mut self, frame_state: &mut PrimitiveRenderFrameState);
    fn end_update(&mut self, frame_state: &mut PrimitiveRenderFrameState);
    fn load_tile(&mut self, frame_state: &mut PrimitiveRenderFrameState, tile: &Rc<RefCell<QuadtreeTile>>);
    fn compute_tile_visibility(&self, tile: &QuadtreeTile, frame_state: &PrimitiveRenderFrameState) -> Visibility;
    fn compute_screen_space_error(&self, tile: &QuadtreeTile, frame_state: &PrimitiveRenderFrameState) -> f64;
    fn can_refine(&self, tile: &QuadtreeTile) -> bool {
        tile.renderable
    }
    fn compute_tile_load_priority(&self, tile: &QuadtreeTile, _frame_state: &PrimitiveRenderFrameState) -> f64 {
        tile.distance
    }
    fn can_render_without_losing_detail(&self, tile: &QuadtreeTile) -> bool {
        tile.renderable
    }
    fn update_heights(&mut self, _tile: &Rc<RefCell<QuadtreeTile>>) {}
}

pub use crate::tile_replacement_queue::TileReplacementQueue;

/// Options for `QuadtreePrimitive`.
pub struct QuadtreePrimitiveOptions {
    pub tile_provider: Box<dyn QuadtreeTileProvider>,
    pub maximum_screen_space_error: Option<f64>,
    pub tile_cache_size: Option<usize>,
    pub loading_descendant_limit: Option<usize>,
    pub preload_ancestors: Option<bool>,
    pub preload_siblings: Option<bool>,
}

/// Renders massive hierarchical surface datasets utilizing LOD and culling.
pub struct QuadtreePrimitive {
    pub tile_provider: Box<dyn QuadtreeTileProvider>,
    pub maximum_screen_space_error: f64,
    pub tile_cache_size: usize,
    pub loading_descendant_limit: usize,
    pub preload_ancestors: bool,
    pub preload_siblings: bool,

    level_zero_tiles: Option<Vec<Rc<RefCell<QuadtreeTile>>>>,
    pub tiles_to_render: Vec<Rc<RefCell<QuadtreeTile>>>,
    pub tile_load_queue_high: Vec<Rc<RefCell<QuadtreeTile>>>,
    pub tile_load_queue_medium: Vec<Rc<RefCell<QuadtreeTile>>>,
    pub tile_load_queue_low: Vec<Rc<RefCell<QuadtreeTile>>>,
    pub tile_replacement_queue: TileReplacementQueue,

    pub add_height_callbacks: Vec<CustomDataEntry>,
    pub remove_height_callbacks: Vec<CustomDataEntry>,
    pub tile_to_update_heights: Vec<Rc<RefCell<QuadtreeTile>>>,

    pub current_frame_number: u64,
    last_selection_frame_number: u64,
    pub tile_load_progress: usize,
}

impl QuadtreePrimitive {
    /// Creates a new `QuadtreePrimitive`.
    pub fn new(options: QuadtreePrimitiveOptions) -> Self {
        Self {
            tile_provider: options.tile_provider,
            maximum_screen_space_error: options.maximum_screen_space_error.unwrap_or(2.0),
            tile_cache_size: options.tile_cache_size.unwrap_or(100),
            loading_descendant_limit: options.loading_descendant_limit.unwrap_or(20),
            preload_ancestors: options.preload_ancestors.unwrap_or(true),
            preload_siblings: options.preload_siblings.unwrap_or(false),
            level_zero_tiles: None,
            tiles_to_render: Vec::new(),
            tile_load_queue_high: Vec::new(),
            tile_load_queue_medium: Vec::new(),
            tile_load_queue_low: Vec::new(),
            tile_replacement_queue: TileReplacementQueue::default(),
            add_height_callbacks: Vec::new(),
            remove_height_callbacks: Vec::new(),
            tile_to_update_heights: Vec::new(),
            current_frame_number: 0,
            last_selection_frame_number: 0,
            tile_load_progress: 0,
        }
    }

    /// Selects tiles for rendering.
    pub fn select_tiles_for_rendering(&mut self, frame_state: &mut PrimitiveRenderFrameState) {
        self.tiles_to_render.clear();

        if !self.tile_provider.ready() {
            return;
        }

        if self.level_zero_tiles.is_none() {
            let scheme = self.tile_provider.tiling_scheme();
            self.level_zero_tiles = Some(QuadtreeTile::create_level_zero_tiles(scheme).unwrap());
        }

        let level_zero_tiles = self.level_zero_tiles.as_ref().unwrap().clone();

        // Dispatch height callbacks to level zero tiles
        for data in self.add_height_callbacks.drain(..) {
            for tile in &level_zero_tiles {
                let rect = tile.borrow().rectangle;
                if GeoRectangle::contains(rect, data.position_cartographic) {
                    tile.borrow().added_custom_data.borrow_mut().push(data);
                    break;
                }
            }
        }

        for data in self.remove_height_callbacks.drain(..) {
            for tile in &level_zero_tiles {
                let rect = tile.borrow().rectangle;
                if GeoRectangle::contains(rect, data.position_cartographic) {
                    tile.borrow().removed_custom_data.borrow_mut().push(data);
                    break;
                }
            }
        }

        for tile in &level_zero_tiles {
            QuadtreeTile::update_custom_data(tile);
        }

        // Traverse tiles
        for tile in &level_zero_tiles {
            self.tile_replacement_queue.mark_tile_rendered(tile);

            let visibility = self.tile_provider.compute_tile_visibility(&tile.borrow(), frame_state);
            if visibility == Visibility::None {
                tile.borrow_mut().last_selection_result = TileSelectionResult::Culled;
                continue;
            }

            if !tile.borrow().renderable {
                self.queue_tile_load(1, tile, frame_state); // high priority
            } else {
                self.visit_tile(tile, frame_state);
            }
        }

        self.last_selection_frame_number = self.current_frame_number;
    }

    fn visit_tile(
        &mut self,
        tile: &Rc<RefCell<QuadtreeTile>>,
        frame_state: &mut PrimitiveRenderFrameState,
    ) {
        self.tile_replacement_queue.mark_tile_rendered(tile);
        QuadtreeTile::update_custom_data(tile);

        let meets_sse = self.tile_provider.compute_screen_space_error(&tile.borrow(), frame_state)
            < self.maximum_screen_space_error;

        if meets_sse {
            if tile.borrow().renderable {
                self.queue_tile_load(2, tile, frame_state); // medium priority
                self.tiles_to_render.push(tile.clone());
                tile.borrow_mut().last_selection_result = TileSelectionResult::Rendered;
                tile.borrow_mut().last_selection_result_frame = Some(self.current_frame_number);
            } else {
                self.queue_tile_load(1, tile, frame_state); // high priority
            }
            return;
        }

        // Needs refinement
        let mut refined_any = false;
        if tile.borrow().level < 30 && self.tile_provider.can_refine(&tile.borrow()) {
            let children = QuadtreeTile::children(tile);
            let mut any_child_ready = false;

            for child in &children {
                let vis = self.tile_provider.compute_tile_visibility(&child.borrow(), frame_state);
                if vis != Visibility::None {
                    if child.borrow().renderable {
                        any_child_ready = true;
                        self.visit_tile(child, frame_state);
                    } else {
                        self.queue_tile_load(1, child, frame_state);
                    }
                } else {
                    child.borrow_mut().last_selection_result = TileSelectionResult::Culled;
                }
            }

            if any_child_ready {
                refined_any = true;
            }
        }

        if !refined_any && tile.borrow().renderable {
            self.tiles_to_render.push(tile.clone());
            tile.borrow_mut().last_selection_result = TileSelectionResult::Rendered;
            tile.borrow_mut().last_selection_result_frame = Some(self.current_frame_number);
        }
    }

    fn queue_tile_load(
        &mut self,
        priority: u8,
        tile: &Rc<RefCell<QuadtreeTile>>,
        frame_state: &PrimitiveRenderFrameState,
    ) {
        if !tile.borrow().state.needs_loading() {
            return;
        }

        let p = self.tile_provider.compute_tile_load_priority(&tile.borrow(), frame_state);
        tile.borrow_mut().load_priority = p;

        match priority {
            1 => self.tile_load_queue_high.push(tile.clone()),
            2 => self.tile_load_queue_medium.push(tile.clone()),
            _ => self.tile_load_queue_low.push(tile.clone()),
        }
    }

    /// Primary per-frame update loop.
    pub fn update(&mut self, frame_state: &mut PrimitiveRenderFrameState) {
        self.current_frame_number += 1;
        self.tile_provider.begin_update(frame_state);
        self.select_tiles_for_rendering(frame_state);

        // Process high priority loads
        let high_tiles = std::mem::take(&mut self.tile_load_queue_high);
        for tile in high_tiles {
            self.tile_provider.load_tile(frame_state, &tile);
        }

        // Process medium priority loads
        let med_tiles = std::mem::take(&mut self.tile_load_queue_medium);
        for tile in med_tiles {
            self.tile_provider.load_tile(frame_state, &tile);
        }

        // Process low priority loads
        let low_tiles = std::mem::take(&mut self.tile_load_queue_low);
        for tile in low_tiles {
            self.tile_provider.load_tile(frame_state, &tile);
        }

        // Update heights
        self.update_heights();

        // Evict tiles if over cache size
        self.tile_replacement_queue.trim_tiles(self.tile_cache_size);

        self.tile_provider.end_update(frame_state);

        self.tile_load_progress = self.tile_load_queue_high.len()
            + self.tile_load_queue_medium.len()
            + self.tile_load_queue_low.len();
    }

    /// Invokes `callback` for each loaded tile in the tree.
    pub fn for_each_loaded_tile<F>(&self, mut callback: F)
    where
        F: FnMut(&QuadtreeTile),
    {
        if let Some(ref roots) = self.level_zero_tiles {
            for root in roots {
                self.traverse_loaded(&root.borrow(), &mut callback);
            }
        }
    }

    fn traverse_loaded<F>(&self, tile: &QuadtreeTile, callback: &mut F)
    where
        F: FnMut(&QuadtreeTile),
    {
        if tile.state != QuadtreeTileLoadState::Start {
            callback(tile);
        }

        if let Some(ref sw) = *tile.southwest_child.borrow() {
            self.traverse_loaded(&sw.borrow(), callback);
        }
        if let Some(ref se) = *tile.southeast_child.borrow() {
            self.traverse_loaded(&se.borrow(), callback);
        }
        if let Some(ref nw) = *tile.northwest_child.borrow() {
            self.traverse_loaded(&nw.borrow(), callback);
        }
        if let Some(ref ne) = *tile.northeast_child.borrow() {
            self.traverse_loaded(&ne.borrow(), callback);
        }
    }

    /// Adds a custom height sampling callback.
    pub fn add_height_callback(&mut self, entry: CustomDataEntry) {
        self.add_height_callbacks.push(entry);
    }

    /// Removes a custom height sampling callback.
    pub fn remove_height_callback(&mut self, entry: CustomDataEntry) {
        self.remove_height_callbacks.push(entry);
    }

    /// Updates heights on newly rendered or modified tiles.
    pub fn update_heights(&mut self) {
        let tiles_to_update = std::mem::take(&mut self.tile_to_update_heights);
        for tile in tiles_to_update {
            self.tile_provider.update_heights(&tile);
        }
    }
}
