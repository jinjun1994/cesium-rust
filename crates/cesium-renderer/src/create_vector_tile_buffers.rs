//! Vector Tile Buffer generation from ModelComponents matching CesiumJS
//! `Scene/Model/createVectorTileBuffersFromModelComponents.js` (Apache-2.0).

use cesium_math::{Cartesian3, ComponentDatatype, IndexDatatype, Matrix4, PrimitiveType};

use crate::model_components::{
    Attribute, Components, FeatureId, FeatureIdAttribute, Node, Polygon, Primitive, Vector,
};
use crate::model_reader::ModelReader;
use crate::model_utility::ModelUtility;

/// Trait to get flyweight views from a `BufferCollection`.
pub trait BufferCollectionGet<T> {
    fn get(&self, index: usize, result: &mut T);
}

// ----------------------------------------------------------------------------
// BufferPoint & BufferPointCollection
// ----------------------------------------------------------------------------

/// Flyweight view onto a point in a [`BufferPointCollection`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BufferPoint {
    pub index: usize,
    pub position: Cartesian3,
    pub feature_id: u32,
    pub show: bool,
}

impl BufferPoint {
    #[inline]
    pub fn get_position(&self, result: Option<&mut Cartesian3>) -> Cartesian3 {
        let pos = self.position;
        if let Some(res) = result {
            *res = pos;
        }
        pos
    }

    #[inline]
    pub fn set_position(&mut self, position: Cartesian3) {
        self.position = position;
    }

    #[inline]
    pub fn feature_id(&self) -> u32 {
        self.feature_id
    }

    #[inline]
    pub fn show(&self) -> bool {
        self.show
    }
}

/// Options for adding a point to a [`BufferPointCollection`].
#[derive(Debug, Clone, Default)]
pub struct BufferPointOptions {
    pub position: Option<Cartesian3>,
    pub feature_id: Option<u32>,
    pub show: Option<bool>,
}

/// Collection of points held in continuous storage.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BufferPointCollection {
    pub positions: Vec<Cartesian3>,
    pub feature_ids: Vec<u32>,
    pub shows: Vec<bool>,
    pub primitive_count_max: usize,
    pub position_normalized: bool,
    pub position_datatype: Option<ComponentDatatype>,
}

impl BufferPointCollection {
    #[inline]
    pub fn primitive_count(&self) -> usize {
        self.positions.len()
    }

    #[inline]
    pub fn get(&self, index: usize, result: &mut BufferPoint) {
        result.index = index;
        result.position = self.positions[index];
        result.feature_id = self.feature_ids[index];
        result.show = self.shows[index];
    }

    pub fn add(&mut self, options: BufferPointOptions, result: &mut BufferPoint) {
        let index = self.positions.len();
        let pos = options.position.unwrap_or(Cartesian3::ZERO);
        let fid = options.feature_id.unwrap_or(index as u32);
        let show = options.show.unwrap_or(true);

        self.positions.push(pos);
        self.feature_ids.push(fid);
        self.shows.push(show);

        result.index = index;
        result.position = pos;
        result.feature_id = fid;
        result.show = show;
    }
}

// ----------------------------------------------------------------------------
// BufferPolyline & BufferPolylineCollection
// ----------------------------------------------------------------------------

/// Flyweight view onto a polyline in a [`BufferPolylineCollection`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BufferPolyline {
    pub index: usize,
    pub positions: Vec<f32>,
    pub feature_id: u32,
    pub show: bool,
}

impl BufferPolyline {
    #[inline]
    pub fn get_positions(&self) -> &[f32] {
        &self.positions
    }

    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.positions.len() / 3
    }

    #[inline]
    pub fn feature_id(&self) -> u32 {
        self.feature_id
    }

    #[inline]
    pub fn show(&self) -> bool {
        self.show
    }
}

/// Options for adding a polyline to a [`BufferPolylineCollection`].
#[derive(Debug, Clone, Default)]
pub struct BufferPolylineOptions {
    pub positions: Vec<f32>,
    pub feature_id: Option<u32>,
    pub show: Option<bool>,
}

/// Collection of polylines held in continuous storage.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BufferPolylineCollection {
    pub positions: Vec<f32>,
    pub offsets: Vec<usize>,
    pub counts: Vec<usize>,
    pub feature_ids: Vec<u32>,
    pub shows: Vec<bool>,
    pub primitive_count_max: usize,
    pub vertex_count_max: usize,
    pub position_normalized: bool,
    pub position_datatype: Option<ComponentDatatype>,
}

impl BufferPolylineCollection {
    #[inline]
    pub fn primitive_count(&self) -> usize {
        self.offsets.len()
    }

    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.positions.len() / 3
    }

    pub fn get(&self, index: usize, result: &mut BufferPolyline) {
        let offset = self.offsets[index];
        let count = self.counts[index];
        let start = offset * 3;
        let end = start + count * 3;

        result.index = index;
        result.positions = self.positions[start..end].to_vec();
        result.feature_id = self.feature_ids[index];
        result.show = self.shows[index];
    }

    pub fn add(&mut self, options: BufferPolylineOptions, result: &mut BufferPolyline) {
        let index = self.offsets.len();
        let vertex_offset = self.positions.len() / 3;
        let vertex_count = options.positions.len() / 3;
        let fid = options.feature_id.unwrap_or(index as u32);
        let show = options.show.unwrap_or(true);

        self.positions.extend_from_slice(&options.positions);
        self.offsets.push(vertex_offset);
        self.counts.push(vertex_count);
        self.feature_ids.push(fid);
        self.shows.push(show);

        result.index = index;
        result.positions = options.positions;
        result.feature_id = fid;
        result.show = show;
    }
}

// ----------------------------------------------------------------------------
// BufferPolygon & BufferPolygonCollection
// ----------------------------------------------------------------------------

/// Flyweight view onto a polygon in a [`BufferPolygonCollection`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BufferPolygon {
    pub index: usize,
    pub positions: Vec<f32>,
    pub triangles: Vec<u32>,
    pub holes: Vec<u32>,
    pub feature_id: u32,
    pub show: bool,
}

impl BufferPolygon {
    #[inline]
    pub fn get_positions(&self) -> &[f32] {
        &self.positions
    }

    #[inline]
    pub fn get_triangles(&self) -> &[u32] {
        &self.triangles
    }

    #[inline]
    pub fn get_holes(&self) -> &[u32] {
        &self.holes
    }

    #[inline]
    pub fn hole_count(&self) -> usize {
        self.holes.len()
    }

    #[inline]
    pub fn triangle_count(&self) -> usize {
        self.triangles.len() / 3
    }

    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.positions.len() / 3
    }

    #[inline]
    pub fn feature_id(&self) -> u32 {
        self.feature_id
    }

    #[inline]
    pub fn show(&self) -> bool {
        self.show
    }
}

/// Options for adding a polygon to a [`BufferPolygonCollection`].
#[derive(Debug, Clone, Default)]
pub struct BufferPolygonOptions {
    pub positions: Vec<f32>,
    pub triangles: Vec<u32>,
    pub holes: Vec<u32>,
    pub feature_id: Option<u32>,
    pub show: Option<bool>,
}

/// Collection of polygons held in continuous storage.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BufferPolygonCollection {
    pub positions: Vec<f32>,
    pub triangles: Vec<u32>,
    pub holes: Vec<u32>,
    pub pos_offsets: Vec<usize>,
    pub pos_counts: Vec<usize>,
    pub tri_offsets: Vec<usize>,
    pub tri_counts: Vec<usize>,
    pub hole_offsets: Vec<usize>,
    pub hole_counts: Vec<usize>,
    pub feature_ids: Vec<u32>,
    pub shows: Vec<bool>,
    pub primitive_count_max: usize,
    pub vertex_count_max: usize,
    pub hole_count_max: usize,
    pub triangle_count_max: usize,
    pub position_normalized: bool,
    pub position_datatype: Option<ComponentDatatype>,
}

impl BufferPolygonCollection {
    #[inline]
    pub fn primitive_count(&self) -> usize {
        self.pos_offsets.len()
    }

    #[inline]
    pub fn triangle_count(&self) -> usize {
        self.triangles.len() / 3
    }

    #[inline]
    pub fn hole_count(&self) -> usize {
        self.holes.len()
    }

    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.positions.len() / 3
    }

    pub fn get(&self, index: usize, result: &mut BufferPolygon) {
        let p_start = self.pos_offsets[index] * 3;
        let p_end = p_start + self.pos_counts[index] * 3;

        let t_start = self.tri_offsets[index] * 3;
        let t_end = t_start + self.tri_counts[index] * 3;

        let h_start = self.hole_offsets[index];
        let h_end = h_start + self.hole_counts[index];

        result.index = index;
        result.positions = self.positions[p_start..p_end].to_vec();
        result.triangles = self.triangles[t_start..t_end].to_vec();
        result.holes = self.holes[h_start..h_end].to_vec();
        result.feature_id = self.feature_ids[index];
        result.show = self.shows[index];
    }

    pub fn add(&mut self, options: BufferPolygonOptions, result: &mut BufferPolygon) {
        let index = self.pos_offsets.len();

        let pos_offset = self.positions.len() / 3;
        let pos_count = options.positions.len() / 3;

        let tri_offset = self.triangles.len() / 3;
        let tri_count = options.triangles.len() / 3;

        let hole_offset = self.holes.len();
        let hole_count = options.holes.len();

        let fid = options.feature_id.unwrap_or(index as u32);
        let show = options.show.unwrap_or(true);

        self.positions.extend_from_slice(&options.positions);
        self.triangles.extend_from_slice(&options.triangles);
        self.holes.extend_from_slice(&options.holes);

        self.pos_offsets.push(pos_offset);
        self.pos_counts.push(pos_count);
        self.tri_offsets.push(tri_offset);
        self.tri_counts.push(tri_count);
        self.hole_offsets.push(hole_offset);
        self.hole_counts.push(hole_count);
        self.feature_ids.push(fid);
        self.shows.push(show);

        result.index = index;
        result.positions = options.positions;
        result.triangles = options.triangles;
        result.holes = options.holes;
        result.feature_id = fid;
        result.show = show;
    }
}

// ----------------------------------------------------------------------------
// BufferCollection Enum & Helpers
// ----------------------------------------------------------------------------

/// Any vector buffer primitive collection.
#[derive(Debug, Clone, PartialEq)]
pub enum BufferCollection {
    Point(BufferPointCollection),
    Polyline(BufferPolylineCollection),
    Polygon(BufferPolygonCollection),
}

impl BufferCollection {
    #[inline]
    pub fn primitive_count(&self) -> usize {
        match self {
            Self::Point(c) => c.primitive_count(),
            Self::Polyline(c) => c.primitive_count(),
            Self::Polygon(c) => c.primitive_count(),
        }
    }

    #[inline]
    pub fn triangle_count(&self) -> usize {
        match self {
            Self::Polygon(c) => c.triangle_count(),
            _ => 0,
        }
    }

    #[inline]
    pub fn hole_count(&self) -> usize {
        match self {
            Self::Polygon(c) => c.hole_count(),
            _ => 0,
        }
    }

    #[inline]
    pub fn vertex_count(&self) -> usize {
        match self {
            Self::Point(c) => c.primitive_count(),
            Self::Polyline(c) => c.vertex_count(),
            Self::Polygon(c) => c.vertex_count(),
        }
    }

    #[inline]
    pub fn as_point(&self) -> Option<&BufferPointCollection> {
        match self {
            Self::Point(c) => Some(c),
            _ => None,
        }
    }

    #[inline]
    pub fn as_polyline(&self) -> Option<&BufferPolylineCollection> {
        match self {
            Self::Polyline(c) => Some(c),
            _ => None,
        }
    }

    #[inline]
    pub fn as_polygon(&self) -> Option<&BufferPolygonCollection> {
        match self {
            Self::Polygon(c) => Some(c),
            _ => None,
        }
    }
}

impl BufferCollectionGet<BufferPoint> for BufferCollection {
    fn get(&self, index: usize, result: &mut BufferPoint) {
        if let Self::Point(c) = self {
            c.get(index, result);
        }
    }
}

impl BufferCollectionGet<BufferPolyline> for BufferCollection {
    fn get(&self, index: usize, result: &mut BufferPolyline) {
        if let Self::Polyline(c) = self {
            c.get(index, result);
        }
    }
}

impl BufferCollectionGet<BufferPolygon> for BufferCollection {
    fn get(&self, index: usize, result: &mut BufferPolygon) {
        if let Self::Polygon(c) = self {
            c.get(index, result);
        }
    }
}

// ----------------------------------------------------------------------------
// Content & Result structs
// ----------------------------------------------------------------------------

/// Tileset options for vector tile parsing.
#[derive(Debug, Clone, Default)]
pub struct VectorTileset {
    pub feature_id_label: Option<String>,
}

/// Content interface for vector glTF 3D Tile content.
#[derive(Debug, Clone, Default)]
pub struct VectorGltf3DTileContent {
    pub tileset: VectorTileset,
}

/// Result of creating vector tile buffers from model components.
#[derive(Debug, Clone, Default)]
pub struct VectorTileResult {
    pub collections: Vec<BufferCollection>,
    pub collection_local_matrices: Vec<Matrix4>,
    pub collection_feature_table_ids: Vec<Option<usize>>,
}

// ----------------------------------------------------------------------------
// Main implementation
// ----------------------------------------------------------------------------

/// Creates vector buffers from model components.
pub fn create_vector_tile_buffers_from_model_components(
    content: &VectorGltf3DTileContent,
    components: &Components,
) -> VectorTileResult {
    let mut result = VectorTileResult::default();

    // In CesiumJS: const rootNodes = components.scene.nodes;
    if let Some(scene) = &components.scene {
        for &node_idx in &scene.nodes {
            if let Some(node) = components.nodes.get(node_idx) {
                append_node_to_buffers(
                    content,
                    components,
                    node,
                    &Matrix4::IDENTITY,
                    &mut result,
                );
            }
        }
    } else {
        // Fallback: if no scene, traverse root components.nodes
        for node in &components.nodes {
            append_node_to_buffers(
                content,
                components,
                node,
                &Matrix4::IDENTITY,
                &mut result,
            );
        }
    }

    result
}

fn append_node_to_buffers(
    content: &VectorGltf3DTileContent,
    components: &Components,
    node: &Node,
    parent_transform: &Matrix4,
    result: &mut VectorTileResult,
) {
    let local_transform = ModelUtility::get_node_transform(
        node.matrix,
        node.translation,
        node.rotation,
        node.scale,
    );
    let node_transform = parent_transform.multiply(&local_transform);

    for primitive in &node.primitives {
        // Skip non-vector primitives
        if primitive.vector.is_none() && primitive.polygon.is_none() {
            continue;
        }

        let prim_type = match primitive.primitive_type {
            Some(t) => t,
            None => continue,
        };

        let stats = gather_primitive_stats(primitive);

        let pos_attr = match get_attribute_by_semantic(
            &primitive.attributes,
            "POSITION",
            None,
        ) {
            Some(a) => a,
            None => continue,
        };

        let position_normalized = pos_attr.normalized;
        let position_datatype = pos_attr.component_datatype;

        let collection = match prim_type {
            PrimitiveType::Points => BufferCollection::Point(BufferPointCollection {
                primitive_count_max: stats.point_primitive_count,
                position_normalized,
                position_datatype,
                ..Default::default()
            }),
            PrimitiveType::LineStrip => {
                BufferCollection::Polyline(BufferPolylineCollection {
                    primitive_count_max: stats.polyline_primitive_count,
                    vertex_count_max: stats.polyline_vertex_count,
                    position_normalized,
                    position_datatype,
                    ..Default::default()
                })
            }
            PrimitiveType::Triangles | PrimitiveType::LineLoop => {
                BufferCollection::Polygon(BufferPolygonCollection {
                    primitive_count_max: stats.polygon_primitive_count,
                    vertex_count_max: stats.polygon_vertex_count,
                    hole_count_max: stats.polygon_hole_count,
                    triangle_count_max: stats.polygon_triangle_count,
                    position_normalized,
                    position_datatype,
                    ..Default::default()
                })
            }
            _ => continue,
        };

        let feature_id_component = find_feature_id_attribute(content, primitive);
        let property_table_id = feature_id_component.and_then(|f| f.property_table_id);

        let mut collection = collection;
        append_primitive_to_buffers(
            content,
            primitive,
            &mut collection,
        );

        result.collections.push(collection);
        result.collection_local_matrices.push(node_transform);
        result.collection_feature_table_ids.push(property_table_id);
    }

    for &child_idx in &node.children {
        if let Some(child) = components.nodes.get(child_idx) {
            append_node_to_buffers(content, components, child, &node_transform, result);
        }
    }
}

fn get_attribute_by_semantic<'a>(
    attributes: &'a [Attribute],
    semantic: &str,
    set_index: Option<usize>,
) -> Option<&'a Attribute> {
    attributes.iter().find(|a| {
        if a.semantic.as_deref() == Some(semantic) {
            if let Some(set_idx) = set_index {
                a.set_index.unwrap_or(0) == set_idx
            } else {
                true
            }
        } else {
            false
        }
    })
}

struct PrimitiveStats {
    point_primitive_count: usize,
    polyline_primitive_count: usize,
    polyline_vertex_count: usize,
    polygon_primitive_count: usize,
    polygon_vertex_count: usize,
    polygon_hole_count: usize,
    polygon_triangle_count: usize,
}

fn gather_primitive_stats(primitive: &Primitive) -> PrimitiveStats {
    let mut stats = PrimitiveStats {
        point_primitive_count: 0,
        polyline_primitive_count: 0,
        polyline_vertex_count: 0,
        polygon_primitive_count: 0,
        polygon_vertex_count: 0,
        polygon_hole_count: 0,
        polygon_triangle_count: 0,
    };

    let vector = primitive.vector.as_ref();
    let polygon = primitive.polygon.as_ref();
    let prim_type = primitive.primitive_type.unwrap_or(PrimitiveType::Triangles);

    let pos_count = get_attribute_by_semantic(&primitive.attributes, "POSITION", None)
        .map(|a| a.count)
        .unwrap_or(0);

    let indices = read_indices_as_u32(primitive.indices.as_ref());
    let restart_index = primitive
        .indices
        .as_ref()
        .map(get_indices_restart_index)
        .unwrap_or(u32::MAX);

    match prim_type {
        PrimitiveType::Points => {
            stats.point_primitive_count = pos_count;
        }
        PrimitiveType::LineStrip => {
            let strip_count = if let Some(v) = vector {
                v.count
            } else {
                get_primitive_count(&indices, restart_index)
            };
            stats.polyline_primitive_count = strip_count;
            stats.polyline_vertex_count = indices.len().saturating_sub(strip_count) + 1;
        }
        PrimitiveType::Triangles | PrimitiveType::LineLoop => {
            if let Some(v) = vector {
                stats.polygon_primitive_count = v.count;
                stats.polygon_vertex_count = pos_count;
                stats.polygon_triangle_count = indices.len() / 3;
                stats.polygon_hole_count = v.polygon_hole_counts.iter().map(|&c| c as usize).sum();
            } else if let Some(p) = polygon {
                let loop_count = get_primitive_count(&p.loop_indices, u32::MAX);
                stats.polygon_primitive_count = p.count;
                stats.polygon_vertex_count = p.loop_indices.len().saturating_sub(loop_count) + 1;
                stats.polygon_triangle_count = p.triangle_indices.as_ref().map(|t| t.len()).unwrap_or(0);
                stats.polygon_hole_count = loop_count.saturating_sub(p.count);
            }
        }
        _ => {}
    }

    stats
}

fn get_primitive_count(indices: &[u32], restart: u32) -> usize {
    let mut count = 0;
    for (i, &idx) in indices.iter().enumerate() {
        if idx == restart && i + 1 < indices.len() {
            count += 1;
        }
    }
    count + 1
}

fn get_indices_restart_index(indices: &crate::model_components::Indices) -> u32 {
    match indices.index_datatype {
        Some(IndexDatatype::UnsignedByte) => 0xff,
        Some(IndexDatatype::UnsignedShort) => 0xffff,
        _ => 0xffffffff,
    }
}

fn read_indices_as_u32(indices: Option<&crate::model_components::Indices>) -> Vec<u32> {
    let indices = match indices {
        Some(i) => i,
        None => return Vec::new(),
    };

    let bytes = match &indices.typed_array {
        Some(b) => b,
        None => return Vec::new(),
    };

    let count = indices.count;
    let mut result = Vec::with_capacity(count);

    let datatype = indices.index_datatype.unwrap_or_else(|| {
        if bytes.len() == count {
            IndexDatatype::UnsignedByte
        } else if bytes.len() == count * 2 {
            IndexDatatype::UnsignedShort
        } else {
            IndexDatatype::UnsignedInt
        }
    });

    match datatype {
        IndexDatatype::UnsignedByte => {
            for &b in bytes.iter().take(count) {
                result.push(b as u32);
            }
        }
        IndexDatatype::UnsignedShort => {
            for chunk in bytes.as_chunks::<2>().0.iter().take(count) {
                let val = u16::from_le_bytes(*chunk);
                result.push(val as u32);
            }
        }
        IndexDatatype::UnsignedInt => {
            for chunk in bytes.as_chunks::<4>().0.iter().take(count) {
                let val = u32::from_le_bytes(*chunk);
                result.push(val);
            }
        }
    }

    result
}

fn read_positions_as_f32(primitive: &Primitive) -> Vec<f32> {
    let pos_attr = match get_attribute_by_semantic(&primitive.attributes, "POSITION", None) {
        Some(a) => a,
        None => return Vec::new(),
    };

    if let Ok(floats) = ModelReader::read_attribute_as_typed_array(pos_attr) {
        return floats;
    }

    if let Some(bytes) = &pos_attr.typed_array {
        let count = pos_attr.count * 3;
        let mut floats = Vec::with_capacity(count);
        for chunk in bytes.as_chunks::<4>().0.iter().take(count) {
            floats.push(f32::from_le_bytes(*chunk));
        }
        return floats;
    }

    Vec::new()
}

fn find_feature_id_attribute<'a>(
    content: &VectorGltf3DTileContent,
    primitive: &'a Primitive,
) -> Option<&'a FeatureIdAttribute> {
    let target_label = content.tileset.feature_id_label.as_deref();

    for fid in &primitive.feature_ids {
        if let FeatureId::Attribute(attr) = fid {
            if let Some(target) = target_label {
                if attr.label.as_deref() == Some(target)
                    || attr.positional_label.as_deref() == Some(target)
                {
                    return Some(attr);
                }
            } else {
                return Some(attr);
            }
        }
    }

    // If no label matched but an attribute exists, fallback to first
    for fid in &primitive.feature_ids {
        if let FeatureId::Attribute(attr) = fid {
            return Some(attr);
        }
    }

    None
}

fn create_feature_id_reader(
    content: &VectorGltf3DTileContent,
    primitive: &Primitive,
) -> Box<dyn Fn(usize) -> Option<u32>> {
    let fid_attr_info = find_feature_id_attribute(content, primitive);
    let fid_attr_info = match fid_attr_info {
        Some(info) => info.clone(),
        None => return Box::new(|_| None),
    };

    let set_index = fid_attr_info.set_index;
    let attr = get_attribute_by_semantic(&primitive.attributes, "FEATURE_ID", set_index);
    let attr = match attr {
        Some(a) => a,
        None => return Box::new(|_| None),
    };

    let feature_ids = if let Some(bytes) = &attr.typed_array {
        let count = attr.count;
        let mut ids = Vec::with_capacity(count);
        let datatype = attr.component_datatype.unwrap_or_else(|| {
            if bytes.len() == count {
                ComponentDatatype::UnsignedByte
            } else if bytes.len() == count * 2 {
                ComponentDatatype::UnsignedShort
            } else {
                ComponentDatatype::UnsignedInt
            }
        });

        match datatype {
            ComponentDatatype::UnsignedByte | ComponentDatatype::Byte => {
                for &b in bytes.iter().take(count) {
                    ids.push(b as u32);
                }
            }
            ComponentDatatype::UnsignedShort | ComponentDatatype::Short => {
                for chunk in bytes.as_chunks::<2>().0.iter().take(count) {
                    let val = u16::from_le_bytes(*chunk);
                    ids.push(val as u32);
                }
            }
            _ => {
                for chunk in bytes.as_chunks::<4>().0.iter().take(count) {
                    let val = u32::from_le_bytes(*chunk);
                    ids.push(val);
                }
            }
        }
        ids
    } else {
        Vec::new()
    };

    let null_id = fid_attr_info.null_feature_id;

    Box::new(move |vertex_offset: usize| {
        if let Some(&fid) = feature_ids.get(vertex_offset) {
            if let Some(null_val) = null_id {
                if fid as i64 == null_val {
                    return None;
                }
            }
            Some(fid)
        } else {
            None
        }
    })
}

fn append_primitive_to_buffers(
    content: &VectorGltf3DTileContent,
    primitive: &Primitive,
    collection: &mut BufferCollection,
) {
    let positions = read_positions_as_f32(primitive);
    let indices = read_indices_as_u32(primitive.indices.as_ref());
    let get_feature = create_feature_id_reader(content, primitive);

    match collection {
        BufferCollection::Point(c) => {
            append_buffer_points(c, &positions, &indices, &get_feature);
        }
        BufferCollection::Polyline(c) => {
            let restart = primitive
                .indices
                .as_ref()
                .map(get_indices_restart_index)
                .unwrap_or(u32::MAX);
            append_buffer_polylines(c, &positions, &indices, restart, &get_feature);
        }
        BufferCollection::Polygon(c) => {
            if let Some(vector) = &primitive.vector {
                append_buffer_polygons_deprecated(c, &positions, &indices, vector, &get_feature);
            } else if let Some(polygon) = &primitive.polygon {
                append_buffer_polygons(c, &positions, polygon, &get_feature);
            }
        }
    }
}

fn append_buffer_points(
    collection: &mut BufferPointCollection,
    positions: &[f32],
    indices: &[u32],
    get_feature: &dyn Fn(usize) -> Option<u32>,
) {
    let vertex_count = positions.len() / 3;
    let count = if !indices.is_empty() {
        indices.len()
    } else {
        vertex_count
    };

    let mut scratch = BufferPoint::default();
    for i in 0..count {
        let vertex_offset = if !indices.is_empty() {
            indices[i] as usize
        } else {
            i
        };
        let p_idx = vertex_offset * 3;
        let pos = if p_idx + 2 < positions.len() {
            Cartesian3::new(
                positions[p_idx] as f64,
                positions[p_idx + 1] as f64,
                positions[p_idx + 2] as f64,
            )
        } else {
            Cartesian3::ZERO
        };

        let feature_id = get_feature(vertex_offset);
        collection.add(
            BufferPointOptions {
                position: Some(pos),
                feature_id,
                show: Some(true),
            },
            &mut scratch,
        );
    }
}

fn copy_array_by_indices(
    array: &[f32],
    indices: &[u32],
    stride: usize,
    restart_index: u32,
    result_indices: Option<&mut [u32]>,
) -> Vec<f32> {
    let count = indices.iter().filter(|&&idx| idx != restart_index).count();
    let mut result = Vec::with_capacity(count * stride);
    let mut dst_index = 0u32;

    let mut result_indices = result_indices;

    for &src_index in indices {
        if let Some(ref mut mapping) = result_indices {
            if let Some(slot) = mapping.get_mut(src_index as usize) {
                *slot = if src_index == restart_index {
                    restart_index
                } else {
                    dst_index
                };
            }
        }

        if src_index == restart_index {
            continue;
        }

        let start = (src_index as usize) * stride;
        let end = start + stride;
        if end <= array.len() {
            result.extend_from_slice(&array[start..end]);
        } else {
            result.resize(result.len() + stride, 0.0);
        }
        dst_index += 1;
    }

    result
}

fn append_buffer_polylines(
    collection: &mut BufferPolylineCollection,
    positions: &[f32],
    indices: &[u32],
    restart_index: u32,
    get_feature: &dyn Fn(usize) -> Option<u32>,
) {
    let mut line_index_start = 0;
    let mut scratch = BufferPolyline::default();

    for i in 0..indices.len() {
        let index = indices[i];
        let mut line_index_count = 0;

        if index == restart_index {
            line_index_count = i - line_index_start;
        } else if i + 1 == indices.len() {
            line_index_count = i + 1 - line_index_start;
        }

        if line_index_count == 0 {
            if index == restart_index {
                line_index_start = i + 1;
            }
            continue;
        }

        let line_index_end = line_index_start + line_index_count;
        let line_indices = &indices[line_index_start..line_index_end];
        let line_positions = copy_array_by_indices(positions, line_indices, 3, restart_index, None);

        let feature_id = get_feature(indices[line_index_start] as usize);

        collection.add(
            BufferPolylineOptions {
                positions: line_positions,
                feature_id,
                show: Some(true),
            },
            &mut scratch,
        );

        line_index_start = i + 1;
    }
}

fn append_buffer_polygons_deprecated(
    collection: &mut BufferPolygonCollection,
    positions: &[f32],
    indices: &[u32],
    vector: &Vector,
    get_feature: &dyn Fn(usize) -> Option<u32>,
) {
    let vertex_count = positions.len() / 3;
    let polygon_count = vector.polygon_attribute_offsets.len();
    let mut scratch = BufferPolygon::default();

    for i in 0..polygon_count {
        let polygon_vertex_start = vector.polygon_attribute_offsets[i] as usize;
        let polygon_vertex_end = if i + 1 < polygon_count {
            vector.polygon_attribute_offsets[i + 1] as usize
        } else {
            vertex_count
        };

        let p_start = polygon_vertex_start * 3;
        let p_end = polygon_vertex_end * 3;
        let poly_positions = if p_end <= positions.len() {
            positions[p_start..p_end].to_vec()
        } else {
            Vec::new()
        };

        let mut holes = Vec::new();
        if let Some(&hole_count) = vector.polygon_hole_counts.get(i) {
            if hole_count > 0 {
                let h_count = hole_count as usize;
                let h_slice = &vector.polygon_hole_offsets[i..i + h_count];
                for &h in h_slice {
                    holes.push(h.saturating_sub(polygon_vertex_start as u32));
                }
            }
        }

        let tri_index_start = if i < vector.polygon_indices_offsets.len() {
            vector.polygon_indices_offsets[i] as usize
        } else {
            0
        };
        let tri_index_end = if i + 1 < polygon_count && i + 1 < vector.polygon_indices_offsets.len() {
            vector.polygon_indices_offsets[i + 1] as usize
        } else {
            indices.len()
        };

        let mut triangles = Vec::new();
        if tri_index_end <= indices.len() {
            for &t in &indices[tri_index_start..tri_index_end] {
                triangles.push(t.saturating_sub(polygon_vertex_start as u32));
            }
        }

        let feature_id = get_feature(polygon_vertex_start);

        collection.add(
            BufferPolygonOptions {
                positions: poly_positions,
                triangles,
                holes,
                feature_id,
                show: Some(true),
            },
            &mut scratch,
        );
    }
}

fn append_buffer_polygons(
    collection: &mut BufferPolygonCollection,
    positions: &[f32],
    polygon: &Polygon,
    get_feature: &dyn Fn(usize) -> Option<u32>,
) {
    let max_index = polygon.loop_indices.iter().copied().max().unwrap_or(0) as usize;
    let mapping_size = max_index.max(polygon.loop_indices.len()) + 1;
    let mut result_loop_indices = vec![0u32; mapping_size];

    let loop_restart_index = u32::MAX;
    let mut scratch = BufferPolygon::default();

    for i in 0..polygon.count {
        let is_last_polygon = i + 1 == polygon.count;

        let loop_indices_start = if i < polygon.loop_indices_offsets.len() {
            polygon.loop_indices_offsets[i] as usize
        } else {
            0
        };
        let loop_indices_end = if is_last_polygon {
            polygon.loop_indices.len()
        } else if i + 1 < polygon.loop_indices_offsets.len() {
            polygon.loop_indices_offsets[i + 1] as usize
        } else {
            polygon.loop_indices.len()
        };

        let loop_indices = &polygon.loop_indices[loop_indices_start..loop_indices_end];
        let poly_positions = copy_array_by_indices(
            positions,
            loop_indices,
            3,
            loop_restart_index,
            Some(&mut result_loop_indices),
        );

        let mut holes = Vec::new();
        for (j, &idx) in loop_indices.iter().enumerate() {
            let is_last = j + 1 == loop_indices.len();
            if idx == loop_restart_index && !is_last {
                holes.push(j as u32);
            }
        }

        let mut triangles = Vec::new();
        if let Some(tri_indices) = &polygon.triangle_indices {
            let tri_start = if i < polygon.triangle_indices_offsets.len() {
                polygon.triangle_indices_offsets[i] as usize
            } else {
                0
            };
            let tri_end = if is_last_polygon {
                tri_indices.len()
            } else if i + 1 < polygon.triangle_indices_offsets.len() {
                polygon.triangle_indices_offsets[i + 1] as usize
            } else {
                tri_indices.len()
            };

            for &t in &tri_indices[tri_start..tri_end] {
                let mapped = result_loop_indices
                    .get(t as usize)
                    .copied()
                    .unwrap_or(t);
                triangles.push(mapped);
            }
        }

        let feature_id = loop_indices.first().and_then(|&idx| get_feature(idx as usize));

        collection.add(
            BufferPolygonOptions {
                positions: poly_positions,
                triangles,
                holes,
                feature_id,
                show: Some(true),
            },
            &mut scratch,
        );
    }
}
