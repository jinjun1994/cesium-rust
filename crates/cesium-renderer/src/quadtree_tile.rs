//! QuadtreeTile matching CesiumJS `Scene/QuadtreeTile.js` (Apache-2.0).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::{Rc, Weak};

use cesium_math::{
    Cartesian3, Cartographic, DeveloperError, GeoRectangle, GeographicTilingScheme,
    TilingScheme, WebMercatorTilingScheme,
};

use crate::quadtree_tile_load_state::QuadtreeTileLoadState;

/// Selection result matching CesiumJS `Scene/TileSelectionResult.js`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum TileSelectionResult {
    #[default]
    None,
    Culled,
    Rendered,
    Refined,
    CulledButNeeded,
}

/// A simple Least Recently Used (LRU) cache implementation matching CesiumJS QuadtreeTile LRUCache.
#[derive(Clone, Debug)]
pub struct LRUCache<K: Clone + std::hash::Hash + Eq, V: Clone> {
    max_size: usize,
    map: HashMap<K, V>,
    order: VecDeque<K>,
}

impl<K: Clone + std::hash::Hash + Eq, V: Clone> LRUCache<K, V> {
    /// Creates a new LRUCache with the specified maximum size.
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    /// Gets a value from the cache, moving it to the most recently used position.
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(val) = self.map.get(key) {
            let val = val.clone();
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                self.order.remove(pos);
            }
            self.order.push_back(key.clone());
            Some(val)
        } else {
            None
        }
    }

    /// Inserts or updates a value in the cache.
    pub fn set(&mut self, key: K, value: V) {
        if self.map.contains_key(&key) {
            if let Some(pos) = self.order.iter().position(|k| k == &key) {
                self.order.remove(pos);
            }
        } else if self.map.len() >= self.max_size {
            if let Some(oldest) = self.order.pop_front() {
                self.map.remove(&oldest);
            }
        }
        self.map.insert(key.clone(), value);
        self.order.push_back(key);
    }

    /// Returns the number of items in the cache.
    pub fn size(&self) -> usize {
        self.map.len()
    }

    /// Clears the cache.
    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }
}

const MAX_CACHE_ENTRIES: usize = 1000;
const TILE_SIZE: f64 = 256.0;

/// Wrapper enum for supported quadtree tiling schemes.
#[derive(Clone, Debug, PartialEq)]
pub enum AnyTilingScheme {
    Geographic(GeographicTilingScheme),
    WebMercator(WebMercatorTilingScheme),
}

impl AnyTilingScheme {
    pub fn number_of_x_tiles_at_level(&self, level: u32) -> u32 {
        match self {
            Self::Geographic(s) => s.number_of_x_tiles_at_level(level),
            Self::WebMercator(s) => s.number_of_x_tiles_at_level(level),
        }
    }

    pub fn number_of_y_tiles_at_level(&self, level: u32) -> u32 {
        match self {
            Self::Geographic(s) => s.number_of_y_tiles_at_level(level),
            Self::WebMercator(s) => s.number_of_y_tiles_at_level(level),
        }
    }

    pub fn tile_xy_to_rectangle(&self, x: u32, y: u32, level: u32) -> Option<GeoRectangle> {
        match self {
            Self::Geographic(s) => s.tile_rectangle(x, y, level),
            Self::WebMercator(s) => s.tile_rectangle(x, y, level),
        }
    }
}

/// Custom data entry attached to QuadtreeTile.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomDataEntry {
    pub id: u64,
    pub position_cartographic: Cartographic,
}

impl std::hash::Hash for CustomDataEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for CustomDataEntry {}

/// QuadtreeTile options.
pub struct QuadtreeTileOptions {
    pub x: u32,
    pub y: u32,
    pub level: u32,
    pub tiling_scheme: AnyTilingScheme,
    pub parent: Option<Weak<RefCell<QuadtreeTile>>>,
}

/// A single tile in a `QuadtreePrimitive`.
pub struct QuadtreeTile {
    pub x: u32,
    pub y: u32,
    pub level: u32,
    pub tiling_scheme: AnyTilingScheme,
    pub rectangle: GeoRectangle,
    pub parent: Option<Weak<RefCell<QuadtreeTile>>>,

    pub southwest_child: RefCell<Option<Rc<RefCell<QuadtreeTile>>>>,
    pub southeast_child: RefCell<Option<Rc<RefCell<QuadtreeTile>>>>,
    pub northwest_child: RefCell<Option<Rc<RefCell<QuadtreeTile>>>>,
    pub northeast_child: RefCell<Option<Rc<RefCell<QuadtreeTile>>>>,

    pub state: QuadtreeTileLoadState,
    pub renderable: bool,
    pub upsampled_from_parent: bool,

    pub custom_data: RefCell<HashSet<CustomDataEntry>>,
    pub added_custom_data: RefCell<Vec<CustomDataEntry>>,
    pub removed_custom_data: RefCell<Vec<CustomDataEntry>>,

    pub position_cache: RefCell<LRUCache<String, Cartesian3>>,
    pub last_selection_result: TileSelectionResult,
    pub last_selection_result_frame: Option<u64>,
    pub distance: f64,
    pub load_priority: f64,
}

impl QuadtreeTile {
    /// Creates a new `QuadtreeTile`.
    pub fn new(options: QuadtreeTileOptions) -> Result<Rc<RefCell<Self>>, DeveloperError> {
        let rectangle = options
            .tiling_scheme
            .tile_xy_to_rectangle(options.x, options.y, options.level)
            .ok_or_else(|| {
                DeveloperError::new(format!(
                    "Invalid tile coordinates: x={}, y={}, level={}",
                    options.x, options.y, options.level
                ))
            })?;

        Ok(Rc::new(RefCell::new(Self {
            x: options.x,
            y: options.y,
            level: options.level,
            tiling_scheme: options.tiling_scheme,
            rectangle,
            parent: options.parent,
            southwest_child: RefCell::new(None),
            southeast_child: RefCell::new(None),
            northwest_child: RefCell::new(None),
            northeast_child: RefCell::new(None),
            state: QuadtreeTileLoadState::Start,
            renderable: false,
            upsampled_from_parent: false,
            custom_data: RefCell::new(HashSet::new()),
            added_custom_data: RefCell::new(Vec::new()),
            removed_custom_data: RefCell::new(Vec::new()),
            position_cache: RefCell::new(LRUCache::new(MAX_CACHE_ENTRIES)),
            last_selection_result: TileSelectionResult::None,
            last_selection_result_frame: None,
            distance: 0.0,
            load_priority: 0.0,
        })))
    }

    /// Creates level zero tiles for a given tiling scheme.
    pub fn create_level_zero_tiles(
        tiling_scheme: AnyTilingScheme,
    ) -> Result<Vec<Rc<RefCell<Self>>>, DeveloperError> {
        let x_tiles = tiling_scheme.number_of_x_tiles_at_level(0);
        let y_tiles = tiling_scheme.number_of_y_tiles_at_level(0);
        let mut result = Vec::with_capacity((x_tiles * y_tiles) as usize);

        for y in 0..y_tiles {
            for x in 0..x_tiles {
                let tile = Self::new(QuadtreeTileOptions {
                    x,
                    y,
                    level: 0,
                    tiling_scheme: tiling_scheme.clone(),
                    parent: None,
                })?;
                result.push(tile);
            }
        }

        Ok(result)
    }

    /// Southwest child tile.
    pub fn southwest_child(this: &Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        let mut child = this.borrow().southwest_child.borrow().clone();
        if child.is_none() {
            let (x, y, level, tiling_scheme) = {
                let b = this.borrow();
                (b.x * 2, b.y * 2 + 1, b.level + 1, b.tiling_scheme.clone())
            };
            let new_child = Self::new(QuadtreeTileOptions {
                x,
                y,
                level,
                tiling_scheme,
                parent: Some(Rc::downgrade(this)),
            })
            .expect("Valid southwest child");
            *this.borrow().southwest_child.borrow_mut() = Some(new_child.clone());
            child = Some(new_child);
        }
        child.unwrap()
    }

    /// Southeast child tile.
    pub fn southeast_child(this: &Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        let mut child = this.borrow().southeast_child.borrow().clone();
        if child.is_none() {
            let (x, y, level, tiling_scheme) = {
                let b = this.borrow();
                (b.x * 2 + 1, b.y * 2 + 1, b.level + 1, b.tiling_scheme.clone())
            };
            let new_child = Self::new(QuadtreeTileOptions {
                x,
                y,
                level,
                tiling_scheme,
                parent: Some(Rc::downgrade(this)),
            })
            .expect("Valid southeast child");
            *this.borrow().southeast_child.borrow_mut() = Some(new_child.clone());
            child = Some(new_child);
        }
        child.unwrap()
    }

    /// Northwest child tile.
    pub fn northwest_child(this: &Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        let mut child = this.borrow().northwest_child.borrow().clone();
        if child.is_none() {
            let (x, y, level, tiling_scheme) = {
                let b = this.borrow();
                (b.x * 2, b.y * 2, b.level + 1, b.tiling_scheme.clone())
            };
            let new_child = Self::new(QuadtreeTileOptions {
                x,
                y,
                level,
                tiling_scheme,
                parent: Some(Rc::downgrade(this)),
            })
            .expect("Valid northwest child");
            *this.borrow().northwest_child.borrow_mut() = Some(new_child.clone());
            child = Some(new_child);
        }
        child.unwrap()
    }

    /// Northeast child tile.
    pub fn northeast_child(this: &Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        let mut child = this.borrow().northeast_child.borrow().clone();
        if child.is_none() {
            let (x, y, level, tiling_scheme) = {
                let b = this.borrow();
                (b.x * 2 + 1, b.y * 2, b.level + 1, b.tiling_scheme.clone())
            };
            let new_child = Self::new(QuadtreeTileOptions {
                x,
                y,
                level,
                tiling_scheme,
                parent: Some(Rc::downgrade(this)),
            })
            .expect("Valid northeast child");
            *this.borrow().northeast_child.borrow_mut() = Some(new_child.clone());
            child = Some(new_child);
        }
        child.unwrap()
    }

    /// Returns all four children in order: [NW, NE, SW, SE].
    pub fn children(this: &Rc<RefCell<Self>>) -> [Rc<RefCell<Self>>; 4] {
        [
            Self::northwest_child(this),
            Self::northeast_child(this),
            Self::southwest_child(this),
            Self::southeast_child(this),
        ]
    }

    /// Generates a unique cache key for a given cartographic position.
    pub fn get_cache_key(&self, cartographic: Cartographic, maximum_screen_space_error: f64) -> String {
        let max_error = (self.rectangle.width() / TILE_SIZE) * maximum_screen_space_error;
        let lon_grid = (cartographic.longitude / max_error).floor() * max_error;
        let lat_grid = (cartographic.latitude / max_error).floor() * max_error;
        format!("{:.10},{:.10}", lon_grid, lat_grid)
    }

    /// Gets a cached position for the specified cartographic coordinate.
    pub fn get_position_cache_entry(
        &self,
        cartographic: Cartographic,
        maximum_screen_space_error: f64,
    ) -> Option<Cartesian3> {
        let key = self.get_cache_key(cartographic, maximum_screen_space_error);
        self.position_cache.borrow_mut().get(&key)
    }

    /// Sets a position in the cache for this tile.
    pub fn set_position_cache_entry(
        &self,
        cartographic: Cartographic,
        maximum_screen_space_error: f64,
        value: Cartesian3,
    ) {
        let key = self.get_cache_key(cartographic, maximum_screen_space_error);
        self.position_cache.borrow_mut().set(key, value);
    }

    /// Clears the position cache for this tile.
    pub fn clear_position_cache(&self) {
        self.position_cache.borrow_mut().clear();
    }

    /// Propagates custom data additions and removals down to the appropriate child tiles.
    pub fn update_custom_data(this: &Rc<RefCell<Self>>) {
        let added: Vec<CustomDataEntry> = this.borrow().added_custom_data.borrow_mut().drain(..).collect();
        let removed: Vec<CustomDataEntry> = this.borrow().removed_custom_data.borrow_mut().drain(..).collect();

        if added.is_empty() && removed.is_empty() {
            return;
        }

        for data in added {
            this.borrow().custom_data.borrow_mut().insert(data.clone());
            let child = Self::child_tile_at_position(this, data.position_cartographic);
            child.borrow().added_custom_data.borrow_mut().push(data);
        }

        for data in removed {
            this.borrow().custom_data.borrow_mut().remove(&data);
            let child = Self::child_tile_at_position(this, data.position_cartographic);
            child.borrow().removed_custom_data.borrow_mut().push(data);
        }
    }

    fn child_tile_at_position(
        this: &Rc<RefCell<Self>>,
        position: Cartographic,
    ) -> Rc<RefCell<Self>> {
        let nw = Self::northwest_child(this);
        let nw_rect = nw.borrow().rectangle;
        let split_lon = nw_rect.east;
        let split_lat = nw_rect.south;

        let x = if position.longitude >= split_lon { 1 } else { 0 };
        let y = if position.latitude < split_lat { 1 } else { 0 };

        match y * 2 + x {
            0 => nw,
            1 => Self::northeast_child(this),
            2 => Self::southwest_child(this),
            _ => Self::southeast_child(this),
        }
    }

    fn find_level_zero_tile(
        level_zero_tiles: &[Rc<RefCell<Self>>],
        tiling_scheme: &AnyTilingScheme,
        mut x: i64,
        y: i64,
    ) -> Option<Rc<RefCell<Self>>> {
        let x_tiles = tiling_scheme.number_of_x_tiles_at_level(0) as i64;
        let y_tiles = tiling_scheme.number_of_y_tiles_at_level(0) as i64;

        if x < 0 {
            x += x_tiles;
        } else if x >= x_tiles {
            x -= x_tiles;
        }

        if y < 0 || y >= y_tiles {
            return None;
        }

        level_zero_tiles
            .iter()
            .find(|tile| {
                let b = tile.borrow();
                b.x as i64 == x && b.y as i64 == y
            })
            .cloned()
    }

    /// Finds adjacent tile to the west.
    pub fn find_tile_to_west(
        this: &Rc<RefCell<Self>>,
        level_zero_tiles: &[Rc<RefCell<Self>>],
    ) -> Option<Rc<RefCell<Self>>> {
        let parent_weak = this.borrow().parent.clone();
        let Some(parent_rc) = parent_weak.and_then(|w| w.upgrade()) else {
            let (x, y, scheme) = {
                let b = this.borrow();
                (b.x as i64, b.y as i64, b.tiling_scheme.clone())
            };
            return Self::find_level_zero_tile(level_zero_tiles, &scheme, x - 1, y);
        };

        let se = Self::southeast_child(&parent_rc);
        let ne = Self::northeast_child(&parent_rc);
        let sw = Self::southwest_child(&parent_rc);
        let nw = Self::northwest_child(&parent_rc);

        if Rc::ptr_eq(&se, this) {
            return Some(sw);
        } else if Rc::ptr_eq(&ne, this) {
            return Some(nw);
        }

        let west_of_parent = Self::find_tile_to_west(&parent_rc, level_zero_tiles)?;
        if Rc::ptr_eq(&sw, this) {
            Some(Self::southeast_child(&west_of_parent))
        } else {
            Some(Self::northeast_child(&west_of_parent))
        }
    }

    /// Finds adjacent tile to the east.
    pub fn find_tile_to_east(
        this: &Rc<RefCell<Self>>,
        level_zero_tiles: &[Rc<RefCell<Self>>],
    ) -> Option<Rc<RefCell<Self>>> {
        let parent_weak = this.borrow().parent.clone();
        let Some(parent_rc) = parent_weak.and_then(|w| w.upgrade()) else {
            let (x, y, scheme) = {
                let b = this.borrow();
                (b.x as i64, b.y as i64, b.tiling_scheme.clone())
            };
            return Self::find_level_zero_tile(level_zero_tiles, &scheme, x + 1, y);
        };

        let sw = Self::southwest_child(&parent_rc);
        let nw = Self::northwest_child(&parent_rc);
        let se = Self::southeast_child(&parent_rc);
        let ne = Self::northeast_child(&parent_rc);

        if Rc::ptr_eq(&sw, this) {
            return Some(se);
        } else if Rc::ptr_eq(&nw, this) {
            return Some(ne);
        }

        let east_of_parent = Self::find_tile_to_east(&parent_rc, level_zero_tiles)?;
        if Rc::ptr_eq(&se, this) {
            Some(Self::southwest_child(&east_of_parent))
        } else {
            Some(Self::northwest_child(&east_of_parent))
        }
    }

    /// Finds adjacent tile to the south.
    pub fn find_tile_to_south(
        this: &Rc<RefCell<Self>>,
        level_zero_tiles: &[Rc<RefCell<Self>>],
    ) -> Option<Rc<RefCell<Self>>> {
        let parent_weak = this.borrow().parent.clone();
        let Some(parent_rc) = parent_weak.and_then(|w| w.upgrade()) else {
            let (x, y, scheme) = {
                let b = this.borrow();
                (b.x as i64, b.y as i64, b.tiling_scheme.clone())
            };
            return Self::find_level_zero_tile(level_zero_tiles, &scheme, x, y + 1);
        };

        let nw = Self::northwest_child(&parent_rc);
        let ne = Self::northeast_child(&parent_rc);
        let sw = Self::southwest_child(&parent_rc);
        let se = Self::southeast_child(&parent_rc);

        if Rc::ptr_eq(&nw, this) {
            return Some(sw);
        } else if Rc::ptr_eq(&ne, this) {
            return Some(se);
        }

        let south_of_parent = Self::find_tile_to_south(&parent_rc, level_zero_tiles)?;
        if Rc::ptr_eq(&sw, this) {
            Some(Self::northwest_child(&south_of_parent))
        } else {
            Some(Self::northeast_child(&south_of_parent))
        }
    }

    /// Finds adjacent tile to the north.
    pub fn find_tile_to_north(
        this: &Rc<RefCell<Self>>,
        level_zero_tiles: &[Rc<RefCell<Self>>],
    ) -> Option<Rc<RefCell<Self>>> {
        let parent_weak = this.borrow().parent.clone();
        let Some(parent_rc) = parent_weak.and_then(|w| w.upgrade()) else {
            let (x, y, scheme) = {
                let b = this.borrow();
                (b.x as i64, b.y as i64, b.tiling_scheme.clone())
            };
            return Self::find_level_zero_tile(level_zero_tiles, &scheme, x, y - 1);
        };

        let sw = Self::southwest_child(&parent_rc);
        let se = Self::southeast_child(&parent_rc);
        let nw = Self::northwest_child(&parent_rc);
        let ne = Self::northeast_child(&parent_rc);

        if Rc::ptr_eq(&sw, this) {
            return Some(nw);
        } else if Rc::ptr_eq(&se, this) {
            return Some(ne);
        }

        let north_of_parent = Self::find_tile_to_north(&parent_rc, level_zero_tiles)?;
        if Rc::ptr_eq(&nw, this) {
            Some(Self::southwest_child(&north_of_parent))
        } else {
            Some(Self::southeast_child(&north_of_parent))
        }
    }

    /// Frees resources associated with this tile and returns state to Start.
    pub fn free_resources(&mut self) {
        self.clear_position_cache();
        self.state = QuadtreeTileLoadState::Start;
        self.renderable = false;
        self.upsampled_from_parent = false;

        if let Some(sw) = self.southwest_child.borrow_mut().take() {
            sw.borrow_mut().free_resources();
        }
        if let Some(se) = self.southeast_child.borrow_mut().take() {
            se.borrow_mut().free_resources();
        }
        if let Some(nw) = self.northwest_child.borrow_mut().take() {
            nw.borrow_mut().free_resources();
        }
        if let Some(ne) = self.northeast_child.borrow_mut().take() {
            ne.borrow_mut().free_resources();
        }
    }
}
