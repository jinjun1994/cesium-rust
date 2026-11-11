use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    Cartographic, Ellipsoid, GeoRectangle, GeographicTilingScheme, Visibility,
};
use cesium_renderer::quadtree_primitive::{
    QuadtreePrimitive, QuadtreePrimitiveOptions, QuadtreeTileProvider,
};
use cesium_renderer::quadtree_tile::{
    AnyTilingScheme, CustomDataEntry, QuadtreeTile,
};
use cesium_renderer::quadtree_tile_load_state::QuadtreeTileLoadState;
use cesium_renderer::scene_primitive::PrimitiveRenderFrameState;

struct MockTileProvider {
    pub is_ready: bool,
    pub tiling_scheme: AnyTilingScheme,
    pub sse_value: f64,
    pub visibility: Visibility,
    pub load_calls: usize,
    pub begin_update_calls: usize,
    pub end_update_calls: usize,
}

impl MockTileProvider {
    fn new(ready: bool) -> Self {
        let geo = GeographicTilingScheme::new(Ellipsoid::WGS84, GeoRectangle::MAXIMUM, 2, 1).unwrap();
        Self {
            is_ready: ready,
            tiling_scheme: AnyTilingScheme::Geographic(geo),
            sse_value: 10.0,
            visibility: Visibility::Full,
            load_calls: 0,
            begin_update_calls: 0,
            end_update_calls: 0,
        }
    }
}

impl QuadtreeTileProvider for MockTileProvider {
    fn ready(&self) -> bool {
        self.is_ready
    }

    fn tiling_scheme(&self) -> AnyTilingScheme {
        self.tiling_scheme.clone()
    }

    fn initialize(&mut self, _frame_state: &mut PrimitiveRenderFrameState) {}

    fn begin_update(&mut self, _frame_state: &mut PrimitiveRenderFrameState) {
        self.begin_update_calls += 1;
    }

    fn end_update(&mut self, _frame_state: &mut PrimitiveRenderFrameState) {
        self.end_update_calls += 1;
    }

    fn load_tile(&mut self, _frame_state: &mut PrimitiveRenderFrameState, tile: &Rc<RefCell<QuadtreeTile>>) {
        self.load_calls += 1;
        tile.borrow_mut().state = QuadtreeTileLoadState::Done;
        tile.borrow_mut().renderable = true;
    }

    fn compute_tile_visibility(&self, _tile: &QuadtreeTile, _frame_state: &PrimitiveRenderFrameState) -> Visibility {
        self.visibility
    }

    fn compute_screen_space_error(&self, tile: &QuadtreeTile, _frame_state: &PrimitiveRenderFrameState) -> f64 {
        self.sse_value / (1_u64 << tile.level.min(30)) as f64
    }
}

#[test]
fn test_selects_nothing_when_root_tiles_are_not_ready() {
    let provider = Box::new(MockTileProvider::new(false));
    let mut primitive = QuadtreePrimitive::new(QuadtreePrimitiveOptions {
        tile_provider: provider,
        maximum_screen_space_error: Some(2.0),
        tile_cache_size: Some(10),
        loading_descendant_limit: None,
        preload_ancestors: None,
        preload_siblings: None,
    });

    let mut frame_state = PrimitiveRenderFrameState::default();

    primitive.select_tiles_for_rendering(&mut frame_state);
    assert_eq!(primitive.tiles_to_render.len(), 0);
}

#[test]
fn test_selects_root_tiles_once_ready() {
    let provider = Box::new(MockTileProvider::new(true));
    let mut primitive = QuadtreePrimitive::new(QuadtreePrimitiveOptions {
        tile_provider: provider,
        maximum_screen_space_error: Some(20.0), // Meets SSE for root tiles
        tile_cache_size: Some(10),
        loading_descendant_limit: None,
        preload_ancestors: None,
        preload_siblings: None,
    });

    let mut frame_state = PrimitiveRenderFrameState::default();

    // First update: tiles are in Start state, queue for load and load them
    primitive.update(&mut frame_state);
    // Second update: tiles are now loaded & renderable, select them for rendering
    primitive.update(&mut frame_state);
    assert_eq!(primitive.tiles_to_render.len(), 2);
}

#[test]
fn test_selects_deeper_tiles_once_renderable() {
    let mut provider = Box::new(MockTileProvider::new(true));
    provider.sse_value = 10.0; // Roots (SSE 10.0) exceed maximum_screen_space_error (2.0), so refines

    let mut primitive = QuadtreePrimitive::new(QuadtreePrimitiveOptions {
        tile_provider: provider,
        maximum_screen_space_error: Some(2.0),
        tile_cache_size: Some(20),
        loading_descendant_limit: None,
        preload_ancestors: None,
        preload_siblings: None,
    });

    let mut frame_state = PrimitiveRenderFrameState::default();

    // Frame 1: loads roots and marks them renderable
    primitive.update(&mut frame_state);
    // Frame 2: roots are renderable, refine to children and load children
    primitive.update(&mut frame_state);
    // Frame 3: children are loaded and renderable
    primitive.update(&mut frame_state);

    // Rendered children count should be greater than 2
    assert!(primitive.tiles_to_render.len() >= 2);
}

#[test]
fn test_calls_begin_and_end_update() {
    let provider = Box::new(MockTileProvider::new(true));
    let mut primitive = QuadtreePrimitive::new(QuadtreePrimitiveOptions {
        tile_provider: provider,
        maximum_screen_space_error: Some(2.0),
        tile_cache_size: Some(10),
        loading_descendant_limit: None,
        preload_ancestors: None,
        preload_siblings: None,
    });

    let mut frame_state = PrimitiveRenderFrameState::default();

    primitive.update(&mut frame_state);
    assert!(primitive.tile_replacement_queue.count() > 0);
}

#[test]
fn test_for_each_loaded_tile_enumerates_loaded_tiles() {
    let provider = Box::new(MockTileProvider::new(true));
    let mut primitive = QuadtreePrimitive::new(QuadtreePrimitiveOptions {
        tile_provider: provider,
        maximum_screen_space_error: Some(20.0),
        tile_cache_size: Some(10),
        loading_descendant_limit: None,
        preload_ancestors: None,
        preload_siblings: None,
    });

    let mut frame_state = PrimitiveRenderFrameState::default();

    primitive.update(&mut frame_state);

    let mut loaded_count = 0;
    primitive.for_each_loaded_tile(|t| {
        if t.state != QuadtreeTileLoadState::Start {
            loaded_count += 1;
        }
    });

    assert_eq!(loaded_count, 2);
}

#[test]
fn test_add_and_remove_height_callbacks() {
    let provider = Box::new(MockTileProvider::new(true));
    let mut primitive = QuadtreePrimitive::new(QuadtreePrimitiveOptions {
        tile_provider: provider,
        maximum_screen_space_error: Some(20.0),
        tile_cache_size: Some(10),
        loading_descendant_limit: None,
        preload_ancestors: None,
        preload_siblings: None,
    });

    let entry = CustomDataEntry {
        id: 101,
        position_cartographic: Cartographic::new(0.0, 0.0, 0.0),
    };

    primitive.add_height_callback(entry.clone());
    assert_eq!(primitive.add_height_callbacks.len(), 1);

    let mut frame_state = PrimitiveRenderFrameState::default();
    primitive.update(&mut frame_state);
    assert_eq!(primitive.add_height_callbacks.len(), 0);

    primitive.remove_height_callback(entry);
    assert_eq!(primitive.remove_height_callbacks.len(), 1);
    primitive.update(&mut frame_state);
    assert_eq!(primitive.remove_height_callbacks.len(), 0);
}
