use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use glam::DVec3;

use crate::core_math::{negative_pi_to_pi, EPSILON3, EPSILON5, TWO_PI};
use crate::{
    BoundingSphere, Cartesian2, Cartesian3, Cartographic, Color, Ellipsoid, GeoRectangle,
    GeographicTilingScheme, TilingScheme,
};

static NEXT_COLLECTION_ID: AtomicUsize = AtomicUsize::new(1);
static NEXT_PRIMITIVE_PICK_ID: AtomicUsize = AtomicUsize::new(100);

const GRID_TARGET_SEGMENTS_PER_CELL: usize = 16;
const GRID_NEIGHBOR_PADDING_SCALE: f64 = 0.35;
pub const MAXIMUM_GROUND_WIDTH_ANGLE: f64 = 0.1;

/// Units for line stroke width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WidthUnits {
    #[default]
    Pixels,
    Meters,
}

/// Surface clamping / height reference classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeightReference {
    #[default]
    None = 0,
    ClampToTerrain = 1,
    RelativeToTerrain = 2,
    ClampTo3DTiles = 3,
    RelativeTo3DTiles = 4,
    ClampToGround = 5,
    RelativeToGround = 6,
}

impl HeightReference {
    pub const NONE: Self = Self::None;
    pub const CLAMP_TO_TERRAIN: Self = Self::ClampToTerrain;
    pub const RELATIVE_TO_TERRAIN: Self = Self::RelativeToTerrain;
    pub const CLAMP_TO_3D_TILE: Self = Self::ClampTo3DTiles;
    pub const CLAMP_TO_3D_TILES: Self = Self::ClampTo3DTiles;
    pub const RELATIVE_TO_3D_TILE: Self = Self::RelativeTo3DTiles;
    pub const CLAMP_TO_GROUND: Self = Self::ClampToGround;
    pub const RELATIVE_TO_GROUND: Self = Self::RelativeToGround;

    pub fn is_clamp(self) -> bool {
        matches!(
            self,
            Self::ClampToTerrain | Self::ClampTo3DTiles | Self::ClampToGround
        )
    }

    pub fn targets_surface(self, target: HeightReference) -> bool {
        self == Self::ClampToGround || self == target
    }
}

/// A line segment projected into a tile's UV coordinate space [0, 1]^2.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorSegment {
    pub start_uv: Cartesian2,
    pub end_uv: Cartesian2,
    pub width: f64,
    pub width_units: WidthUnits,
    pub color: Color,
    pub pick_color: Option<Color>,
}

/// A polygon ring projected into a tile's UV space.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorRing {
    pub points_uv: Vec<Cartesian2>,
    pub is_hole: bool,
    pub color: Color,
    pub pick_color: Option<Color>,
}

/// Baked vector data intersecting a terrain or 3D tile.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorTileData {
    pub show: bool,
    pub has_polylines: bool,
    pub has_polygons: bool,
    pub has_pixel_widths: bool,
    pub has_meter_widths: bool,
    pub rectangle: GeoRectangle,
    pub meters_per_uv: Cartesian2,
    pub minimum_tile_screen_pixels: f64,
    pub primitive_count: usize,
    pub widths: Vec<f32>,
    pub colors: Vec<[u8; 4]>,
    pub pick_colors: Vec<[u8; 4]>,
    pub pick_color_texture_defined: bool,
    pub polyline_segment_texels: Vec<f32>,
    pub polyline_grid_cell_indices: Vec<u32>,
    pub polygon_edge_texels: Vec<f32>,
    pub polygon_grid_cell_indices: Vec<u32>,
    pub polygon_edge_primitive_indices_texels: Vec<f32>,
    pub collection_versions: HashMap<usize, u64>,
}

impl Default for VectorTileData {
    fn default() -> Self {
        Self {
            show: false,
            has_polylines: false,
            has_polygons: false,
            has_pixel_widths: false,
            has_meter_widths: false,
            rectangle: GeoRectangle::MAXIMUM,
            meters_per_uv: Cartesian2::new(1.0, 1.0),
            minimum_tile_screen_pixels: 256.0,
            primitive_count: 0,
            widths: Vec::new(),
            colors: Vec::new(),
            pick_colors: Vec::new(),
            pick_color_texture_defined: false,
            polyline_segment_texels: Vec::new(),
            polyline_grid_cell_indices: Vec::new(),
            polygon_edge_texels: Vec::new(),
            polygon_grid_cell_indices: Vec::new(),
            polygon_edge_primitive_indices_texels: Vec::new(),
            collection_versions: HashMap::new(),
        }
    }
}

/// Material properties for a polyline.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferPolylineMaterial {
    pub width: f64,
    pub color: Color,
}

impl Default for BufferPolylineMaterial {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: Color::WHITE,
        }
    }
}

impl BufferPolylineMaterial {
    pub fn new(width: f64, color: Color) -> Self {
        Self { width, color }
    }
}

/// A polyline primitive inside a collection.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferPolyline {
    pub positions: Vec<Cartesian3>,
    pub material: BufferPolylineMaterial,
    pub show: bool,
    pub pick_id: u32,
}

impl Default for BufferPolyline {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            material: BufferPolylineMaterial::default(),
            show: true,
            pick_id: NEXT_PRIMITIVE_PICK_ID.fetch_add(1, Ordering::Relaxed) as u32,
        }
    }
}

impl BufferPolyline {
    pub fn new(positions: Vec<Cartesian3>, material: BufferPolylineMaterial) -> Self {
        Self {
            positions,
            material,
            show: true,
            pick_id: NEXT_PRIMITIVE_PICK_ID.fetch_add(1, Ordering::Relaxed) as u32,
        }
    }

    pub fn set_positions(&mut self, positions: Vec<Cartesian3>) {
        self.positions = positions;
    }

    pub fn set_material(&mut self, material: BufferPolylineMaterial) {
        self.material = material;
    }
}

/// Material properties for a polygon.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferPolygonMaterial {
    pub color: Color,
}

impl Default for BufferPolygonMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
        }
    }
}

/// A polygon primitive inside a collection.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferPolygon {
    pub positions: Vec<Cartesian3>,
    pub holes: Vec<u32>,
    pub material: BufferPolygonMaterial,
    pub show: bool,
    pub pick_id: u32,
}

impl Default for BufferPolygon {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            holes: Vec::new(),
            material: BufferPolygonMaterial::default(),
            show: true,
            pick_id: NEXT_PRIMITIVE_PICK_ID.fetch_add(1, Ordering::Relaxed) as u32,
        }
    }
}

impl BufferPolygon {
    pub fn new(
        positions: Vec<Cartesian3>,
        holes: Vec<u32>,
        material: BufferPolygonMaterial,
    ) -> Self {
        Self {
            positions,
            holes,
            material,
            show: true,
            pick_id: NEXT_PRIMITIVE_PICK_ID.fetch_add(1, Ordering::Relaxed) as u32,
        }
    }
}

/// Options for creating a BufferPolylineCollection.
#[derive(Debug, Clone, Default)]
pub struct BufferPolylineCollectionOptions {
    pub primitive_count_max: Option<usize>,
    pub vertex_count_max: Option<usize>,
    pub height_reference: Option<HeightReference>,
    pub width_units: Option<WidthUnits>,
    pub allow_picking: Option<bool>,
    pub bounding_volume: Option<BoundingSphere>,
}

#[derive(Debug)]
struct PolylineCollectionInner {
    id: usize,
    polylines: Vec<BufferPolyline>,
    height_reference: HeightReference,
    width_units: WidthUnits,
    allow_picking: bool,
    bounding_volume: Option<BoundingSphere>,
    version: u64,
    dirty_count: usize,
}

/// A collection of polylines draped onto terrain or 3D Tiles.
#[derive(Debug, Clone)]
pub struct BufferPolylineCollection {
    inner: Arc<RwLock<PolylineCollectionInner>>,
}

impl BufferPolylineCollection {
    pub fn new(options: BufferPolylineCollectionOptions) -> Self {
        let id = NEXT_COLLECTION_ID.fetch_add(1, Ordering::Relaxed);
        let inner = PolylineCollectionInner {
            id,
            polylines: Vec::new(),
            height_reference: options
                .height_reference
                .unwrap_or(HeightReference::ClampToTerrain),
            width_units: options.width_units.unwrap_or(WidthUnits::Pixels),
            allow_picking: options.allow_picking.unwrap_or(false),
            bounding_volume: options.bounding_volume,
            version: 1,
            dirty_count: 1,
        };
        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub fn id(&self) -> usize {
        self.inner.read().unwrap().id
    }

    pub fn height_reference(&self) -> HeightReference {
        self.inner.read().unwrap().height_reference
    }

    pub fn width_units(&self) -> WidthUnits {
        self.inner.read().unwrap().width_units
    }

    pub fn allow_picking(&self) -> bool {
        self.inner.read().unwrap().allow_picking
    }

    pub fn version(&self) -> u64 {
        self.inner.read().unwrap().version
    }

    pub fn dirty_count(&self) -> usize {
        self.inner.read().unwrap().dirty_count
    }

    pub fn primitive_count(&self) -> usize {
        self.inner.read().unwrap().polylines.len()
    }

    pub fn add(&self, polyline: BufferPolyline) {
        let mut inner = self.inner.write().unwrap();
        inner.polylines.push(polyline);
        inner.version += 1;
        inner.dirty_count += 1;
    }

    pub fn get(&self, index: usize) -> Option<BufferPolyline> {
        self.inner.read().unwrap().polylines.get(index).cloned()
    }

    pub fn set_polyline_positions(&self, index: usize, positions: Vec<Cartesian3>) {
        let mut inner = self.inner.write().unwrap();
        if let Some(p) = inner.polylines.get_mut(index) {
            p.set_positions(positions);
            inner.version += 1;
            inner.dirty_count += 1;
        }
    }

    pub fn set_polyline_material(&self, index: usize, material: BufferPolylineMaterial) {
        let mut inner = self.inner.write().unwrap();
        if let Some(p) = inner.polylines.get_mut(index) {
            p.set_material(material);
            inner.version += 1;
            inner.dirty_count += 1;
        }
    }

    pub fn bounding_volume(&self, ellipsoid: &Ellipsoid) -> BoundingSphere {
        let inner = self.inner.read().unwrap();
        if let Some(bv) = inner.bounding_volume {
            return bv;
        }
        let mut all_positions = Vec::new();
        for p in &inner.polylines {
            all_positions.extend_from_slice(&p.positions);
        }
        if all_positions.is_empty() {
            BoundingSphere::new(DVec3::ZERO, 0.0)
        } else {
            let pts: Vec<DVec3> = all_positions.iter().map(|&p| DVec3::from(p)).collect();
            BoundingSphere::from_points(&pts).unwrap_or_else(|_| {
                let carto = ellipsoid
                    .cartesian_to_cartographic(pts[0])
                    .unwrap_or(Cartographic::ZERO);
                let center = ellipsoid.cartographic_to_cartesian(carto).unwrap_or(DVec3::ZERO);
                BoundingSphere::new(center, 10.0)
            })
        }
    }

    pub fn make_clean(&self) {
        let mut inner = self.inner.write().unwrap();
        if inner.dirty_count > 0 {
            inner.version += 1;
            inner.dirty_count = 0;
        }
    }
}

/// Options for creating a BufferPolygonCollection.
#[derive(Debug, Clone, Default)]
pub struct BufferPolygonCollectionOptions {
    pub primitive_count_max: Option<usize>,
    pub vertex_count_max: Option<usize>,
    pub hole_count_max: Option<usize>,
    pub triangle_count_max: Option<usize>,
    pub height_reference: Option<HeightReference>,
    pub allow_picking: Option<bool>,
    pub bounding_volume: Option<BoundingSphere>,
}

#[derive(Debug)]
struct PolygonCollectionInner {
    id: usize,
    polygons: Vec<BufferPolygon>,
    height_reference: HeightReference,
    allow_picking: bool,
    bounding_volume: Option<BoundingSphere>,
    version: u64,
    dirty_count: usize,
}

/// A collection of polygons draped onto terrain or 3D Tiles.
#[derive(Debug, Clone)]
pub struct BufferPolygonCollection {
    inner: Arc<RwLock<PolygonCollectionInner>>,
}

impl BufferPolygonCollection {
    pub fn new(options: BufferPolygonCollectionOptions) -> Self {
        let id = NEXT_COLLECTION_ID.fetch_add(1, Ordering::Relaxed);
        let inner = PolygonCollectionInner {
            id,
            polygons: Vec::new(),
            height_reference: options
                .height_reference
                .unwrap_or(HeightReference::ClampToTerrain),
            allow_picking: options.allow_picking.unwrap_or(false),
            bounding_volume: options.bounding_volume,
            version: 1,
            dirty_count: 1,
        };
        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub fn id(&self) -> usize {
        self.inner.read().unwrap().id
    }

    pub fn height_reference(&self) -> HeightReference {
        self.inner.read().unwrap().height_reference
    }

    pub fn allow_picking(&self) -> bool {
        self.inner.read().unwrap().allow_picking
    }

    pub fn version(&self) -> u64 {
        self.inner.read().unwrap().version
    }

    pub fn dirty_count(&self) -> usize {
        self.inner.read().unwrap().dirty_count
    }

    pub fn primitive_count(&self) -> usize {
        self.inner.read().unwrap().polygons.len()
    }

    pub fn add(&self, polygon: BufferPolygon) {
        let mut inner = self.inner.write().unwrap();
        inner.polygons.push(polygon);
        inner.version += 1;
        inner.dirty_count += 1;
    }

    pub fn get(&self, index: usize) -> Option<BufferPolygon> {
        self.inner.read().unwrap().polygons.get(index).cloned()
    }

    pub fn bounding_volume(&self, ellipsoid: &Ellipsoid) -> BoundingSphere {
        let inner = self.inner.read().unwrap();
        if let Some(bv) = inner.bounding_volume {
            return bv;
        }
        let mut all_positions = Vec::new();
        for p in &inner.polygons {
            all_positions.extend_from_slice(&p.positions);
        }
        if all_positions.is_empty() {
            BoundingSphere::new(DVec3::ZERO, 0.0)
        } else {
            let pts: Vec<DVec3> = all_positions.iter().map(|&p| DVec3::from(p)).collect();
            BoundingSphere::from_points(&pts).unwrap_or_else(|_| {
                let carto = ellipsoid
                    .cartesian_to_cartographic(pts[0])
                    .unwrap_or(Cartographic::ZERO);
                let center = ellipsoid.cartographic_to_cartesian(carto).unwrap_or(DVec3::ZERO);
                BoundingSphere::new(center, 10.0)
            })
        }
    }

    pub fn make_clean(&self) {
        let mut inner = self.inner.write().unwrap();
        if inner.dirty_count > 0 {
            inner.version += 1;
            inner.dirty_count = 0;
        }
    }
}

/// Generic container for collection reference.
#[derive(Debug, Clone)]
pub enum VectorCollectionRef {
    Polyline(BufferPolylineCollection),
    Polygon(BufferPolygonCollection),
}

impl VectorCollectionRef {
    pub fn id(&self) -> usize {
        match self {
            Self::Polyline(c) => c.id(),
            Self::Polygon(c) => c.id(),
        }
    }

    pub fn height_reference(&self) -> HeightReference {
        match self {
            Self::Polyline(c) => c.height_reference(),
            Self::Polygon(c) => c.height_reference(),
        }
    }

    pub fn bounding_volume(&self, ellipsoid: &Ellipsoid) -> BoundingSphere {
        match self {
            Self::Polyline(c) => c.bounding_volume(ellipsoid),
            Self::Polygon(c) => c.bounding_volume(ellipsoid),
        }
    }

    pub fn is_dirty(&self, cached_version: Option<u64>) -> bool {
        match self {
            Self::Polyline(c) => {
                c.dirty_count() > 0 || cached_version.is_none_or(|v| v != c.version())
            }
            Self::Polygon(c) => {
                c.dirty_count() > 0 || cached_version.is_none_or(|v| v != c.version())
            }
        }
    }

    pub fn make_clean(&self) {
        match self {
            Self::Polyline(c) => c.make_clean(),
            Self::Polygon(c) => c.make_clean(),
        }
    }
}

/// Snapshot of a vector collection extracted in a single pass.
#[derive(Debug, Clone)]
pub struct VectorCollectionData {
    pub version: u64,
    pub rectangle: GeoRectangle,
    pub positions: Vec<Cartographic>,
    pub widths: Vec<f32>,
    pub maximum_width: f64,
    pub width_in_meters: bool,
    pub colors: Vec<[u8; 4]>,
    pub pick_colors: Vec<[u8; 4]>,
}

/// Pipeline for vector data baking.
pub struct VectorPipeline;

impl VectorPipeline {
    pub fn pack_polyline_collection_data(
        collection: &BufferPolylineCollection,
        ellipsoid: &Ellipsoid,
    ) -> VectorCollectionData {
        let primitive_count = collection.primitive_count();
        let bounding_volume = collection.bounding_volume(ellipsoid);
        let rectangle = GeoRectangle::from_bounding_sphere(bounding_volume, *ellipsoid)
            .unwrap_or(GeoRectangle::MAXIMUM);

        let inner = collection.inner.read().unwrap();
        let width_in_meters = inner.width_units == WidthUnits::Meters;
        let maximum_ground_width = if width_in_meters {
            MAXIMUM_GROUND_WIDTH_ANGLE * ellipsoid.maximum_radius()
        } else {
            f64::INFINITY
        };

        let mut positions = Vec::new();
        let mut widths = Vec::with_capacity(primitive_count);
        let mut colors = Vec::with_capacity(primitive_count);
        let mut pick_colors = Vec::with_capacity(primitive_count);
        let mut maximum_width_magnitude = 0.0_f64;

        for p in &inner.polylines {
            for pos in &p.positions {
                let carto = ellipsoid
                    .cartesian_to_cartographic(DVec3::from(*pos))
                    .unwrap_or(Cartographic::ZERO);
                positions.push(carto);
            }

            let mut width = p.material.width;
            if width_in_meters && width > maximum_ground_width {
                width = maximum_ground_width;
            }

            maximum_width_magnitude = maximum_width_magnitude.max(width.abs());
            widths.push(if width_in_meters { -width as f32 } else { width as f32 });

            let c = p.material.color;
            colors.push(c.to_bytes());

            if inner.allow_picking {
                let pick_c = Color::from_rgba(p.pick_id);
                pick_colors.push(pick_c.to_bytes());
            } else {
                pick_colors.push([0, 0, 0, 0]);
            }
        }

        let signed_max_width = if width_in_meters {
            -maximum_width_magnitude
        } else {
            maximum_width_magnitude
        };

        VectorCollectionData {
            version: inner.version,
            rectangle,
            positions,
            widths,
            maximum_width: signed_max_width,
            width_in_meters,
            colors,
            pick_colors,
        }
    }

    pub fn pack_polygon_collection_data(
        collection: &BufferPolygonCollection,
        ellipsoid: &Ellipsoid,
    ) -> VectorCollectionData {
        let primitive_count = collection.primitive_count();
        let bounding_volume = collection.bounding_volume(ellipsoid);
        let rectangle = GeoRectangle::from_bounding_sphere(bounding_volume, *ellipsoid)
            .unwrap_or(GeoRectangle::MAXIMUM);

        let inner = collection.inner.read().unwrap();
        let mut positions = Vec::new();
        let mut widths = Vec::with_capacity(primitive_count);
        let mut colors = Vec::with_capacity(primitive_count);
        let mut pick_colors = Vec::with_capacity(primitive_count);

        for p in &inner.polygons {
            for pos in &p.positions {
                let carto = ellipsoid
                    .cartesian_to_cartographic(DVec3::from(*pos))
                    .unwrap_or(Cartographic::ZERO);
                positions.push(carto);
            }
            widths.push(0.0);
            colors.push(p.material.color.to_bytes());
            if inner.allow_picking {
                let pick_c = Color::from_rgba(p.pick_id);
                pick_colors.push(pick_c.to_bytes());
            } else {
                pick_colors.push([0, 0, 0, 0]);
            }
        }

        VectorCollectionData {
            version: inner.version,
            rectangle,
            positions,
            widths,
            maximum_width: 0.0,
            width_in_meters: false,
            colors,
            pick_colors,
        }
    }

    pub fn maximum_half_width_to_tile_uv(maximum_width: f64, tile_data: &VectorTileData) -> f64 {
        if maximum_width == 0.0 {
            0.0
        } else {
            half_width_to_tile_uv(maximum_width, tile_data)
        }
    }
}

fn half_width_to_tile_uv(width: f64, tile_data: &VectorTileData) -> f64 {
    if width >= 0.0 {
        ((width + 1.0) * 0.5) / tile_data.minimum_tile_screen_pixels
    } else {
        let meters_per_uv = tile_data.meters_per_uv;
        (-width * 0.5) / meters_per_uv.x.min(meters_per_uv.y).max(1.0e-6)
    }
}

fn project_segment_to_tile_uv(
    a: &mut Cartesian2,
    b: &mut Cartesian2,
    rectangle: &GeoRectangle,
    width: f64,
) {
    let mut lon_a = a.x;
    let mut lon_b = b.x;

    lon_b = lon_a + negative_pi_to_pi(lon_b - lon_a);
    let center = rectangle.west + width * 0.5;
    let shift = center + negative_pi_to_pi(lon_a - center) - lon_a;
    lon_a += shift;
    lon_b += shift;

    let height = rectangle.height();
    a.x = (lon_a - rectangle.west) / width;
    a.y = (a.y - rectangle.south) / height;
    b.x = (lon_b - rectangle.west) / width;
    b.y = (b.y - rectangle.south) / height;
}

fn clip_segment_to_tile(a: &mut Cartesian2, b: &mut Cartesian2, margin: f64) -> bool {
    let lo = -margin;
    let hi = 1.0 + margin;

    let mut t_min = 0.0_f64;
    let mut t_max = 1.0_f64;

    let ax = a.x;
    let ay = a.y;
    let bx = b.x;
    let by = b.y;

    let dx = bx - ax;
    let dy = by - ay;
    let p = [-dx, dx, -dy, dy];
    let q = [ax - lo, hi - ax, ay - lo, hi - ay];

    for i in 0..4 {
        if p[i].abs() < 1.0e-12 {
            if q[i] < 0.0 {
                return false;
            }
            continue;
        }
        let t = q[i] / p[i];
        if p[i] < 0.0 {
            t_min = t_min.max(t);
        } else {
            t_max = t_max.min(t);
        }

        if t_min > t_max {
            return false;
        }
    }

    a.x = ax + t_min * dx;
    a.y = ay + t_min * dy;
    b.x = ax + t_max * dx;
    b.y = ay + t_max * dy;

    true
}

fn clip_ring_to_rect(
    ring: &[f64],
    vertex_count: usize,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
) -> Vec<f64> {
    let mut input = ring[..vertex_count * 2].to_vec();
    let mut input_count = vertex_count;

    for plane in 0..4 {
        let axis = plane >> 1;
        let sign = if (plane & 1) == 0 { 1.0 } else { -1.0 };
        let limit = match plane {
            0 => min_x,
            1 => max_x,
            2 => min_y,
            _ => max_y,
        };

        let mut output = Vec::new();
        let mut output_count = 0;
        let prev_x = input[(input_count - 1) * 2];
        let prev_y = input[(input_count - 1) * 2 + 1];
        let mut prev_inside = sign * (if axis == 0 { prev_x } else { prev_y } - limit) >= 0.0;
        let mut p_x = prev_x;
        let mut p_y = prev_y;

        for i in 0..input_count {
            let x = input[i * 2];
            let y = input[i * 2 + 1];
            let inside = sign * (if axis == 0 { x } else { y } - limit) >= 0.0;

            if inside != prev_inside {
                let t = if axis == 0 {
                    (limit - p_x) / (x - p_x)
                } else {
                    (limit - p_y) / (y - p_y)
                };
                let out_x = if axis == 0 { limit } else { p_x + t * (x - p_x) };
                let out_y = if axis == 0 { p_y + t * (y - p_y) } else { limit };
                output.push(out_x);
                output.push(out_y);
                output_count += 1;
            }
            if inside {
                output.push(x);
                output.push(y);
                output_count += 1;
            }

            p_x = x;
            p_y = y;
            prev_inside = inside;
        }

        if output_count == 0 {
            return Vec::new();
        }

        input = output;
        input_count = output_count;
    }

    input
}

fn next_power_of_two_size(count: usize) -> (usize, usize) {
    let width = (count as u32).next_power_of_two().max(1) as usize;
    let height = 1;
    (width, height)
}

/// A cartographic dirty region.
#[derive(Debug, Clone)]
pub struct VectorDirtyRegion {
    pub rectangle: GeoRectangle,
    pub maximum_width: f64,
}

/// Options for constructing VectorProvider.
pub struct VectorProviderOptions<T: TilingScheme = GeographicTilingScheme> {
    pub tiling_scheme: T,
    pub antialias: Option<bool>,
    pub minimum_tile_screen_pixels: Option<f64>,
}

/// Coordinates baking and updating of vector geometry onto 3D Tiles and terrain tiles.
pub struct VectorProvider<T: TilingScheme = GeographicTilingScheme> {
    pub tiling_scheme: T,
    pub antialias: bool,
    pub minimum_tile_screen_pixels: f64,

    height_reference_by_collection: HashMap<usize, (VectorCollectionRef, HeightReference)>,
    collection_data_cache: HashMap<usize, VectorCollectionData>,
    marked_this_frame: HashSet<usize>,
    marked_frame_number: Option<u64>,
    dirty_regions: Vec<VectorDirtyRegion>,
}

impl Default for VectorProvider<GeographicTilingScheme> {
    fn default() -> Self {
        Self::new(VectorProviderOptions {
            tiling_scheme: GeographicTilingScheme::default(),
            antialias: None,
            minimum_tile_screen_pixels: None,
        })
    }
}

impl<T: TilingScheme> VectorProvider<T> {
    pub fn new(options: VectorProviderOptions<T>) -> Self {
        Self {
            tiling_scheme: options.tiling_scheme,
            antialias: options.antialias.unwrap_or(true),
            minimum_tile_screen_pixels: options.minimum_tile_screen_pixels.unwrap_or(256.0),
            height_reference_by_collection: HashMap::new(),
            collection_data_cache: HashMap::new(),
            marked_this_frame: HashSet::new(),
            marked_frame_number: None,
            dirty_regions: Vec::new(),
        }
    }

    pub fn dirty_regions(&self) -> &[VectorDirtyRegion] {
        &self.dirty_regions
    }

    pub fn mark_for_frame_polyline(
        &mut self,
        collection: &BufferPolylineCollection,
        frame_number: u64,
        height_reference: HeightReference,
    ) {
        self.mark_for_frame(
            VectorCollectionRef::Polyline(collection.clone()),
            frame_number,
            height_reference,
        );
    }

    pub fn mark_for_frame_polygon(
        &mut self,
        collection: &BufferPolygonCollection,
        frame_number: u64,
        height_reference: HeightReference,
    ) {
        self.mark_for_frame(
            VectorCollectionRef::Polygon(collection.clone()),
            frame_number,
            height_reference,
        );
    }

    pub fn mark_for_frame(
        &mut self,
        collection: VectorCollectionRef,
        frame_number: u64,
        height_reference: HeightReference,
    ) {
        let id = collection.id();
        self.begin_frame(frame_number);
        self.marked_this_frame.insert(id);

        if !self.height_reference_by_collection.contains_key(&id) {
            self.mark_collection_region_dirty(&collection);
        }

        self.height_reference_by_collection
            .insert(id, (collection, height_reference));
    }

    fn begin_frame(&mut self, frame_number: u64) {
        if self.marked_frame_number == Some(frame_number) {
            return;
        }

        let mut to_remove = Vec::new();
        for (id, (coll, _)) in &self.height_reference_by_collection {
            if !self.marked_this_frame.contains(id) {
                to_remove.push((*id, coll.clone()));
            }
        }

        for (id, coll) in to_remove {
            self.height_reference_by_collection.remove(&id);
            self.mark_collection_region_dirty(&coll);
        }

        self.marked_this_frame.clear();
        self.marked_frame_number = Some(frame_number);
    }

    fn mark_collection_region_dirty(&mut self, collection: &VectorCollectionRef) {
        let ellipsoid = self.tiling_scheme.ellipsoid();
        let rect = match collection {
            VectorCollectionRef::Polyline(c) => {
                let bv = c.bounding_volume(&ellipsoid);
                GeoRectangle::from_bounding_sphere(bv, ellipsoid).unwrap_or(GeoRectangle::MAXIMUM)
            }
            VectorCollectionRef::Polygon(c) => {
                let bv = c.bounding_volume(&ellipsoid);
                GeoRectangle::from_bounding_sphere(bv, ellipsoid).unwrap_or(GeoRectangle::MAXIMUM)
            }
        };

        let max_w = self
            .collection_data_cache
            .get(&collection.id())
            .map(|d| d.maximum_width)
            .unwrap_or(0.0);

        self.dirty_regions.push(VectorDirtyRegion {
            rectangle: rect,
            maximum_width: max_w,
        });
    }

    pub fn has_polyline(&self, collection: &BufferPolylineCollection) -> bool {
        self.height_reference_by_collection.contains_key(&collection.id())
    }

    pub fn has_polygon(&self, collection: &BufferPolygonCollection) -> bool {
        self.height_reference_by_collection.contains_key(&collection.id())
    }

    pub fn has(&self, id: usize) -> bool {
        self.height_reference_by_collection.contains_key(&id)
    }

    pub fn remove_polyline(&mut self, collection: &BufferPolylineCollection) {
        self.remove(collection.id());
    }

    pub fn remove_polygon(&mut self, collection: &BufferPolygonCollection) {
        self.remove(collection.id());
    }

    pub fn remove(&mut self, id: usize) {
        if let Some((coll, _)) = self.height_reference_by_collection.remove(&id) {
            self.marked_this_frame.remove(&id);
            self.mark_collection_region_dirty(&coll);
        }
    }

    pub fn make_clean(&mut self) {
        self.dirty_regions.clear();
    }

    pub fn update(&mut self, frame_number: Option<u64>) {
        if let Some(fn_num) = frame_number {
            self.begin_frame(fn_num);
        }

        let ellipsoid = self.tiling_scheme.ellipsoid();
        let ids: Vec<usize> = self.height_reference_by_collection.keys().copied().collect();

        for id in ids {
            let (collection, _) = self.height_reference_by_collection.get(&id).unwrap().clone();
            let cached_version = self.collection_data_cache.get(&id).map(|c| c.version);
            let is_dirty = collection.is_dirty(cached_version);

            if !is_dirty {
                continue;
            }

            if let Some(cache) = self.collection_data_cache.get(&id) {
                self.dirty_regions.push(VectorDirtyRegion {
                    rectangle: cache.rectangle,
                    maximum_width: cache.maximum_width,
                });
            }

            self.get_collection_data_cached(&collection, &ellipsoid);
            self.mark_collection_region_dirty(&collection);
        }
    }

    fn get_collection_data_cached(
        &mut self,
        collection: &VectorCollectionRef,
        ellipsoid: &Ellipsoid,
    ) -> VectorCollectionData {
        let id = collection.id();
        let cached_version = self.collection_data_cache.get(&id).map(|c| c.version);
        let dirty = collection.is_dirty(cached_version);

        if !dirty {
            if let Some(c) = self.collection_data_cache.get(&id) {
                return c.clone();
            }
        }

        let mut data = match collection {
            VectorCollectionRef::Polyline(c) => {
                VectorPipeline::pack_polyline_collection_data(c, ellipsoid)
            }
            VectorCollectionRef::Polygon(c) => {
                VectorPipeline::pack_polygon_collection_data(c, ellipsoid)
            }
        };

        data.version += if dirty { 1 } else { 0 };
        collection.make_clean();
        self.collection_data_cache.insert(id, data.clone());
        data
    }

    pub fn is_tile_dirty(&self, x: u32, y: u32, level: u32) -> bool {
        if self.dirty_regions.is_empty() {
            return false;
        }

        let tile_rect = self
            .tiling_scheme
            .tile_rectangle(x, y, level)
            .unwrap_or(GeoRectangle::MAXIMUM);

        let ellipsoid = self.tiling_scheme.ellipsoid();
        let meters_per_uv = compute_meters_per_uv(&tile_rect, &ellipsoid);

        let tile_data = VectorTileData {
            minimum_tile_screen_pixels: self.minimum_tile_screen_pixels,
            meters_per_uv,
            ..Default::default()
        };

        for region in &self.dirty_regions {
            let painted_rect = compute_painted_rectangle(region, &tile_rect, &tile_data);
            if let Some(inter) = tile_rect.intersection(painted_rect) {
                if inter.width() > 0.0 || inter.height() > 0.0 {
                    return true;
                }
            }
        }

        false
    }

    pub fn request_tile_data(
        &mut self,
        x: u32,
        y: u32,
        level: u32,
        target_surface: HeightReference,
    ) -> Arc<VectorTileData> {
        let tile_rect = self
            .tiling_scheme
            .tile_rectangle(x, y, level)
            .unwrap_or(GeoRectangle::MAXIMUM);
        self.request_data_for_rectangle(&tile_rect, target_surface)
    }

    pub fn request_data_for_rectangle(
        &mut self,
        rectangle: &GeoRectangle,
        target_surface: HeightReference,
    ) -> Arc<VectorTileData> {
        let ellipsoid = self.tiling_scheme.ellipsoid();
        let meters_per_uv = compute_meters_per_uv(rectangle, &ellipsoid);

        let mut result = VectorTileData {
            show: true,
            has_polylines: false,
            has_polygons: false,
            has_pixel_widths: false,
            has_meter_widths: false,
            rectangle: *rectangle,
            meters_per_uv,
            minimum_tile_screen_pixels: self.minimum_tile_screen_pixels,
            primitive_count: 0,
            widths: Vec::new(),
            colors: Vec::new(),
            pick_colors: Vec::new(),
            pick_color_texture_defined: false,
            polyline_segment_texels: Vec::new(),
            polyline_grid_cell_indices: Vec::new(),
            polygon_edge_texels: Vec::new(),
            polygon_grid_cell_indices: Vec::new(),
            polygon_edge_primitive_indices_texels: Vec::new(),
            collection_versions: HashMap::new(),
        };

        let mut polyline_segments: Vec<[f64; 4]> = Vec::new();
        let mut polyline_segment_primitive_indices: Vec<usize> = Vec::new();

        let mut polygon_rings: Vec<Vec<f64>> = Vec::new();
        let mut polygon_ring_primitive_indices: Vec<usize> = Vec::new();

        let collections: Vec<(VectorCollectionRef, HeightReference)> = self
            .height_reference_by_collection
            .values()
            .cloned()
            .collect();

        for (collection, height_reference) in collections {
            if !height_reference.targets_surface(target_surface) {
                continue;
            }

            let collection_data = self.get_collection_data_cached(&collection, &ellipsoid);
            let dirty_region = VectorDirtyRegion {
                rectangle: collection_data.rectangle,
                maximum_width: collection_data.maximum_width,
            };
            let painted_rect = compute_painted_rectangle(&dirty_region, rectangle, &result);

            if rectangle.intersection(painted_rect).is_none() {
                continue;
            }

            result
                .collection_versions
                .insert(collection.id(), collection_data.version);

            match collection {
                VectorCollectionRef::Polyline(c) => {
                    let prim_count = c.primitive_count();
                    let inner = c.inner.read().unwrap();
                    let mut pos_offset = 0;

                    for (i, p) in inner.polylines.iter().enumerate() {
                        if !p.show {
                            pos_offset += p.positions.len();
                            continue;
                        }

                        let vertex_count = p.positions.len();
                        let margin = EPSILON3
                            .max(half_width_to_tile_uv(collection_data.widths[i] as f64, &result));

                        for j in 0..vertex_count.saturating_sub(1) {
                            let c1 = collection_data.positions[pos_offset + j];
                            let c2 = collection_data.positions[pos_offset + j + 1];

                            let mut a = Cartesian2::new(c1.longitude, c1.latitude);
                            let mut b = Cartesian2::new(c2.longitude, c2.latitude);

                            project_segment_to_tile_uv(&mut a, &mut b, rectangle, rectangle.width());

                            if clip_segment_to_tile(&mut a, &mut b, margin) {
                                polyline_segments.push([a.x, a.y, b.x, b.y]);
                                polyline_segment_primitive_indices.push(result.primitive_count + i);
                            }
                        }

                        pos_offset += vertex_count;
                    }

                    result.widths.extend_from_slice(&collection_data.widths);
                    result.colors.extend_from_slice(&collection_data.colors);
                    result
                        .pick_colors
                        .extend_from_slice(&collection_data.pick_colors);

                    if prim_count > 0 {
                        if collection_data.width_in_meters {
                            result.has_meter_widths = true;
                        } else {
                            result.has_pixel_widths = true;
                        }
                    }

                    result.primitive_count += prim_count;
                }
                VectorCollectionRef::Polygon(c) => {
                    let prim_count = c.primitive_count();
                    let inner = c.inner.read().unwrap();
                    let mut pos_offset = 0;

                    for (i, p) in inner.polygons.iter().enumerate() {
                        if !p.show {
                            pos_offset += p.positions.len();
                            continue;
                        }

                        let total_vertices = p.positions.len();
                        let mut ring_boundaries = vec![0];
                        for h in &p.holes {
                            ring_boundaries.push(*h as usize);
                        }
                        ring_boundaries.push(total_vertices);

                        for r in 0..ring_boundaries.len() - 1 {
                            let ring_start = ring_boundaries[r];
                            let ring_end = ring_boundaries[r + 1];
                            let ring_vertex_count = ring_end - ring_start;
                            if ring_vertex_count < 3 {
                                continue;
                            }

                            let mut ring_flat = Vec::with_capacity(ring_vertex_count * 2);
                            let width = rectangle.width();
                            let height = rectangle.height();

                            let p0 = collection_data.positions[pos_offset + ring_start];
                            let mut prev_lon = p0.longitude;
                            let center = rectangle.west + width * 0.5;
                            let shift = center + negative_pi_to_pi(prev_lon - center) - prev_lon;

                            for k in 0..ring_vertex_count {
                                let cur =
                                    collection_data.positions[pos_offset + ring_start + k];
                                let lon = prev_lon + negative_pi_to_pi(cur.longitude - prev_lon);
                                prev_lon = lon;

                                let u = (lon + shift - rectangle.west) / width;
                                let v = (cur.latitude - rectangle.south) / height;
                                ring_flat.push(u);
                                ring_flat.push(v);
                            }

                            let clipped = clip_ring_to_rect(
                                &ring_flat,
                                ring_vertex_count,
                                -EPSILON3,
                                1.0 + EPSILON3,
                                -EPSILON3,
                                1.0 + EPSILON3,
                            );

                            if clipped.len() >= 6 {
                                polygon_rings.push(clipped);
                                polygon_ring_primitive_indices.push(result.primitive_count + i);
                            }
                        }

                        pos_offset += total_vertices;
                    }

                    result.widths.extend_from_slice(&collection_data.widths);
                    result.colors.extend_from_slice(&collection_data.colors);
                    result
                        .pick_colors
                        .extend_from_slice(&collection_data.pick_colors);

                    result.primitive_count += prim_count;
                }
            }
        }

        let has_polylines = !polyline_segments.is_empty();
        let has_polygons = !polygon_rings.is_empty();
        result.has_polylines = has_polylines;
        result.has_polygons = has_polygons;

        if !has_polylines && !has_polygons {
            result.show = false;
            return Arc::new(result);
        }

        if has_polylines {
            pack_polyline_grid(
                &polyline_segments,
                &polyline_segment_primitive_indices,
                &mut result,
            );
        }

        if has_polygons {
            pack_polygon_grid(
                &polygon_rings,
                &polygon_ring_primitive_indices,
                &mut result,
            );
        }

        if result
            .pick_colors
            .iter()
            .any(|c| c[0] > 0 || c[1] > 0 || c[2] > 0 || c[3] > 0)
        {
            result.pick_color_texture_defined = true;
        }

        Arc::new(result)
    }

    pub fn update_data_for_rectangle(
        &mut self,
        rectangle: &GeoRectangle,
        current_data: &Arc<VectorTileData>,
        target_surface: HeightReference,
    ) -> Arc<VectorTileData> {
        if !self.is_stale(current_data, rectangle, target_surface) {
            return current_data.clone();
        }

        self.request_data_for_rectangle(rectangle, target_surface)
    }

    fn is_stale(
        &mut self,
        data: &VectorTileData,
        rectangle: &GeoRectangle,
        target_surface: HeightReference,
    ) -> bool {
        if data.rectangle != *rectangle {
            return true;
        }

        let ellipsoid = self.tiling_scheme.ellipsoid();
        let baked_versions = &data.collection_versions;
        let mut baked_versions_visited = 0;

        let collections: Vec<(VectorCollectionRef, HeightReference)> = self
            .height_reference_by_collection
            .values()
            .cloned()
            .collect();

        for (collection, height_reference) in collections {
            if !height_reference.targets_surface(target_surface) {
                continue;
            }

            let coll_data = self.get_collection_data_cached(&collection, &ellipsoid);
            let region = VectorDirtyRegion {
                rectangle: coll_data.rectangle,
                maximum_width: coll_data.maximum_width,
            };
            let painted = compute_painted_rectangle(&region, rectangle, data);
            let is_intersected = rectangle.intersection(painted).is_some();
            let baked_version = baked_versions.get(&collection.id());

            if !is_intersected {
                if baked_version.is_some() {
                    return true;
                }
                continue;
            }

            if baked_version.is_none() {
                return true;
            }

            baked_versions_visited += 1;

            if collection.is_dirty(baked_version.copied()) {
                return true;
            }
        }

        baked_versions_visited != baked_versions.len()
    }
}

fn compute_meters_per_uv(rectangle: &GeoRectangle, ellipsoid: &Ellipsoid) -> Cartesian2 {
    let radius = ellipsoid.maximum_radius();
    let center_lat = (rectangle.south + rectangle.north) * 0.5;
    Cartesian2::new(
        rectangle.width() * radius * center_lat.cos(),
        rectangle.height() * radius,
    )
}

fn compute_painted_rectangle(
    region: &VectorDirtyRegion,
    tile_rectangle: &GeoRectangle,
    tile_data: &VectorTileData,
) -> GeoRectangle {
    let uv_margin = VectorPipeline::maximum_half_width_to_tile_uv(region.maximum_width, tile_data);
    let margin_lon = uv_margin * tile_rectangle.width();
    let margin_lat = uv_margin * tile_rectangle.height();

    let coll_rect = region.rectangle;
    let mut result = coll_rect;

    result.south -= margin_lat;
    result.north += margin_lat;

    if coll_rect.width() + 2.0 * margin_lon >= TWO_PI {
        result.west = -PI;
        result.east = PI;
    } else {
        result.west = negative_pi_to_pi(result.west - margin_lon);
        result.east = negative_pi_to_pi(result.east + margin_lon);
    }

    result
}

fn pack_polyline_grid(
    segments: &[[f64; 4]],
    segment_primitive_indices: &[usize],
    result: &mut VectorTileData,
) {
    let grid_size = 1.max((segments.len() as f64 / GRID_TARGET_SEGMENTS_PER_CELL as f64).sqrt().ceil() as usize);
    let mut grid: Vec<Vec<usize>> = vec![Vec::new(); grid_size * grid_size];

    let cell_padding = GRID_NEIGHBOR_PADDING_SCALE / grid_size as f64;
    let mut packed_segment_count = 0;

    let widths = result.widths.clone();
    for (i, seg) in segments.iter().enumerate() {
        let prim_idx = segment_primitive_indices[i];
        let w = widths.get(prim_idx).copied().unwrap_or(1.0);
        let padding = cell_padding.max(half_width_to_tile_uv(w as f64, result));

        let min_x = 0.0_f64.max(seg[0].min(seg[2]) - padding);
        let max_x = 1.0_f64.min(seg[0].max(seg[2]) + padding);
        let min_y = 0.0_f64.max(seg[1].min(seg[3]) - padding);
        let max_y = 1.0_f64.min(seg[1].max(seg[3]) + padding);

        let start_x = clamp_cell_index((min_x * grid_size as f64).floor() as isize, grid_size);
        let end_x = clamp_cell_index((max_x * grid_size as f64).floor() as isize, grid_size);
        let start_y = clamp_cell_index((min_y * grid_size as f64).floor() as isize, grid_size);
        let end_y = clamp_cell_index((max_y * grid_size as f64).floor() as isize, grid_size);

        for y in start_y..=end_y {
            for x in start_x..=end_x {
                grid[y * grid_size + x].push(i);
                packed_segment_count += 1;
            }
        }
    }

    let (tex_w, tex_h) = next_power_of_two_size(packed_segment_count);
    let capacity = tex_w * tex_h;

    let mut segment_texels = vec![-1.0_f32; capacity * 4];
    let mut grid_cell_indices = vec![0_u32; grid.len() + 2];
    grid_cell_indices[0] = grid_size as u32;
    grid_cell_indices[1] = grid_size as u32;

    let mut offset = 0;
    for (cell_idx, cell_segments) in grid.iter().enumerate() {
        for &seg_idx in cell_segments {
            let seg = segments[seg_idx];
            segment_texels[offset * 4] = seg[0] as f32;
            segment_texels[offset * 4 + 1] = seg[1] as f32;
            segment_texels[offset * 4 + 2] = seg[2] as f32;
            segment_texels[offset * 4 + 3] = seg[3] as f32;
            offset += 1;
        }
        grid_cell_indices[cell_idx + 2] = offset as u32;
    }

    result.polyline_segment_texels = segment_texels;
    result.polyline_grid_cell_indices = grid_cell_indices;
}

fn pack_polygon_grid(
    rings: &[Vec<f64>],
    ring_primitive_indices: &[usize],
    result: &mut VectorTileData,
) {
    let mut total_vertices = 0;
    for r in rings {
        total_vertices += r.len() / 2;
    }

    let grid_size = 1.max((total_vertices as f64 / GRID_TARGET_SEGMENTS_PER_CELL as f64).sqrt().ceil() as usize);
    let mut grid: Vec<Vec<f64>> = vec![Vec::new(); grid_size * grid_size];
    let mut packed_edge_count = 0;

    for (r_idx, ring) in rings.iter().enumerate() {
        let prim_idx = ring_primitive_indices[r_idx];
        let v_count = ring.len() / 2;

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for k in 0..v_count {
            let x = ring[k * 2];
            let y = ring[k * 2 + 1];
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }

        let start_x = clamp_cell_index((min_x * grid_size as f64).floor() as isize, grid_size);
        let end_x = clamp_cell_index((max_x * grid_size as f64).floor() as isize, grid_size);
        let start_y = clamp_cell_index((min_y * grid_size as f64).floor() as isize, grid_size);
        let end_y = clamp_cell_index((max_y * grid_size as f64).floor() as isize, grid_size);

        for y in start_y..=end_y {
            for x in start_x..=end_x {
                let cell_min_x = x as f64 / grid_size as f64 - EPSILON5;
                let cell_max_x = (x + 1) as f64 / grid_size as f64 + EPSILON5;
                let cell_min_y = y as f64 / grid_size as f64 - EPSILON5;
                let cell_max_y = (y + 1) as f64 / grid_size as f64 + EPSILON5;

                let clipped = clip_ring_to_rect(
                    ring,
                    v_count,
                    cell_min_x,
                    cell_max_x,
                    cell_min_y,
                    cell_max_y,
                );

                let clipped_count = clipped.len() / 2;
                if clipped_count < 3 {
                    continue;
                }

                let cell = &mut grid[y * grid_size + x];
                for k in 0..clipped_count {
                    let k2 = (k + 1) % clipped_count;
                    let ax = clipped[k * 2];
                    let ay = clipped[k * 2 + 1];
                    let bx = clipped[k2 * 2];
                    let by = clipped[k2 * 2 + 1];

                    // Horizontal edges never toggle parity against shader horizontal ray!
                    if (ay - by).abs() < 1.0e-12 {
                        continue;
                    }

                    cell.push(ax);
                    cell.push(ay);
                    cell.push(bx);
                    cell.push(by);
                    cell.push(prim_idx as f64);
                    packed_edge_count += 1;
                }
            }
        }
    }

    let (tex_w, tex_h) = next_power_of_two_size(packed_edge_count);
    let capacity = tex_w * tex_h;

    let mut edge_texels = vec![-1.0_f32; capacity * 4];
    let mut edge_prim_indices = vec![-1.0_f32; capacity];
    let mut grid_cell_indices = vec![0_u32; grid.len() + 2];
    grid_cell_indices[0] = grid_size as u32;
    grid_cell_indices[1] = grid_size as u32;

    let mut offset = 0;
    for (i, cell_edges) in grid.iter().enumerate() {
        for chunk in cell_edges.as_chunks::<5>().0 {
            edge_texels[offset * 4] = chunk[0] as f32;
            edge_texels[offset * 4 + 1] = chunk[1] as f32;
            edge_texels[offset * 4 + 2] = chunk[2] as f32;
            edge_texels[offset * 4 + 3] = chunk[3] as f32;
            edge_prim_indices[offset] = chunk[4] as f32;
            offset += 1;
        }
        grid_cell_indices[i + 2] = offset as u32;
    }

    result.polygon_edge_texels = edge_texels;
    result.polygon_edge_primitive_indices_texels = edge_prim_indices;
    result.polygon_grid_cell_indices = grid_cell_indices;
}

fn clamp_cell_index(index: isize, grid_size: usize) -> usize {
    if index < 0 {
        0
    } else if index >= grid_size as isize {
        grid_size - 1
    } else {
        index as usize
    }
}
