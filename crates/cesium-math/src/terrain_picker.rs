use crate::axis_aligned_bounding_box::AxisAlignedBoundingBox;
use crate::cartesian::Cartesian3;
use crate::core_math::TWO_PI;
use crate::matrices::Matrix4;
use crate::projection::MapProjection;
use crate::ray::Ray;
use crate::scene_mode::SceneMode;
use crate::terrain_encoding::TerrainEncoding;
use glam::DVec3;

#[allow(dead_code)]
pub const MAXIMUM_TERRAIN_PICKER_LEVEL: u32 = 3;

/// A node in the terrain picker quadtree.
#[derive(Debug, Clone)]
pub struct TerrainPickerNode {
    pub x: u32,
    pub y: u32,
    pub level: u32,
    pub aabb: AxisAlignedBoundingBox,
    pub intersecting_triangles: Vec<u32>,
    pub children: Vec<TerrainPickerNode>,
    pub building_children: bool,
}

impl Default for TerrainPickerNode {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

impl TerrainPickerNode {
    pub fn new(x: u32, y: u32, level: u32) -> Self {
        let aabb = create_aabb_for_node(x, y, level);
        Self {
            x,
            y,
            level,
            aabb,
            intersecting_triangles: Vec::new(),
            children: Vec::new(),
            building_children: false,
        }
    }

    pub fn add_child(&mut self, child_idx: u32) {
        assert!(child_idx < 4, "Child index must be 0-3");
        let child_x = self.x * 2 + (child_idx & 1);
        let child_y = self.y * 2 + ((child_idx >> 1) & 1);
        let child_node = TerrainPickerNode::new(child_x, child_y, self.level + 1);
        if (child_idx as usize) < self.children.len() {
            self.children[child_idx as usize] = child_node;
        } else {
            while self.children.len() < (child_idx as usize) {
                self.children.push(TerrainPickerNode::default());
            }
            self.children.push(child_node);
        }
    }

    pub fn building_children(&self) -> bool {
        self.building_children
    }
}

fn create_aabb_for_node(x: u32, y: u32, level: u32) -> AxisAlignedBoundingBox {
    let size_at_level = 1.0 / (2.0f64.powi(level as i32));
    let aabb_min = DVec3::new(
        (x as f64) * size_at_level - 0.5,
        (y as f64) * size_at_level - 0.5,
        -0.5,
    );
    let aabb_max = DVec3::new(
        ((x + 1) as f64) * size_at_level - 0.5,
        ((y + 1) as f64) * size_at_level - 0.5,
        0.5,
    );
    AxisAlignedBoundingBox::from_corners(aabb_min, aabb_max)
}

/// Handles arbitrary ray intersections with a terrain mesh using a quadtree spatial structure.
#[derive(Debug, Clone)]
pub struct TerrainPicker {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub encoding: TerrainEncoding,
    pub inverse_transform: Matrix4,
    pub needs_rebuild: bool,
    pub root_node: TerrainPickerNode,
}

impl TerrainPicker {
    pub fn new(vertices: Vec<f32>, indices: Vec<u32>, encoding: TerrainEncoding) -> Self {
        Self {
            vertices,
            indices,
            encoding,
            inverse_transform: Matrix4::IDENTITY,
            needs_rebuild: true,
            root_node: TerrainPickerNode::default(),
        }
    }

    pub fn needs_rebuild(&self) -> bool {
        self.needs_rebuild
    }

    pub fn set_needs_rebuild(&mut self, value: bool) {
        self.needs_rebuild = value;
    }

    pub fn reset(&mut self, tile_transform: &Matrix4) {
        self.inverse_transform = tile_transform.inverse().unwrap_or(Matrix4::IDENTITY);
        self.needs_rebuild = false;

        let triangle_count = self.indices.len() / 3;
        let mut intersecting_triangles = Vec::with_capacity(triangle_count);
        for i in 0..triangle_count {
            intersecting_triangles.push(i as u32);
        }
        self.root_node.intersecting_triangles = intersecting_triangles;
        self.root_node.children.clear();
    }

    pub fn ray_intersect<P: MapProjection>(
        &mut self,
        ray: &Ray,
        tile_transform: &Matrix4,
        cull_back_faces: bool,
        mode: SceneMode,
        projection: &P,
    ) -> Option<Cartesian3> {
        if self.needs_rebuild {
            self.reset(tile_transform);
        }

        let origin_cart = Cartesian3::from(ray.origin);
        let dir_cart = Cartesian3::from(ray.direction);

        let origin = self.inverse_transform.multiply_by_point(origin_cart);
        let direction = self.inverse_transform.multiply_by_point_as_vector(dir_cart);
        let transformed_ray = Ray::new(origin.into(), direction.into()).unwrap_or(*ray);

        let mut intersecting_nodes = Vec::new();
        get_nodes_intersecting_ray(&self.root_node, &transformed_ray, &mut intersecting_nodes);

        find_closest_point_in_closest_node(
            self,
            &mut intersecting_nodes,
            ray,
            cull_back_faces,
            mode,
            projection,
        )
    }
}

struct IntersectingNode<'a> {
    node: &'a TerrainPickerNode,
    t_min: f64,
}

fn get_nodes_intersecting_ray<'a>(
    node: &'a TerrainPickerNode,
    ray: &Ray,
    intersecting_nodes: &mut Vec<IntersectingNode<'a>>,
) {
    if let Some(interval) = ray.intersect_box(node.aabb) {
        let is_leaf = node.children.is_empty() || node.building_children();
        if is_leaf {
            intersecting_nodes.push(IntersectingNode {
                node,
                t_min: interval.start,
            });
            return;
        }

        for child in &node.children {
            get_nodes_intersecting_ray(child, ray, intersecting_nodes);
        }
    }
}

fn find_closest_point_in_closest_node<P: MapProjection>(
    picker: &TerrainPicker,
    intersections: &mut [IntersectingNode<'_>],
    ray: &Ray,
    cull_back_faces: bool,
    mode: SceneMode,
    projection: &P,
) -> Option<Cartesian3> {
    intersections.sort_by(|a, b| a.t_min.partial_cmp(&b.t_min).unwrap_or(std::cmp::Ordering::Equal));

    let mut min_t = f64::MAX;
    for intersection in intersections.iter() {
        let tri_t = get_closest_triangle_in_node(
            picker,
            ray,
            intersection.node,
            cull_back_faces,
            mode,
            projection,
        );
        if tri_t < min_t {
            min_t = tri_t;
            break;
        }
    }

    if min_t < f64::MAX {
        let pt = ray.point_at(min_t);
        Some(Cartesian3::new(pt.x, pt.y, pt.z))
    } else {
        None
    }
}

fn get_closest_triangle_in_node<P: MapProjection>(
    picker: &TerrainPicker,
    ray: &Ray,
    node: &TerrainPickerNode,
    cull_back_faces: bool,
    mode: SceneMode,
    projection: &P,
) -> f64 {
    let mut result = f64::MAX;
    let encoding = &picker.encoding;
    let indices = &picker.indices;
    let vertices = &picker.vertices;

    for &tri_index in &node.intersecting_triangles {
        let base = tri_index as usize * 3;
        if base + 2 >= indices.len() {
            continue;
        }

        let i0 = indices[base] as usize;
        let i1 = indices[base + 1] as usize;
        let i2 = indices[base + 2] as usize;

        let v0 = get_vertex_position(encoding, mode, projection, ray, vertices, i0);
        let v1 = get_vertex_position(encoding, mode, projection, ray, vertices, i1);
        let v2 = get_vertex_position(encoding, mode, projection, ray, vertices, i2);

        if let Some(tri_t) = ray.triangle_parameter([v0.into(), v1.into(), v2.into()], cull_back_faces) {
            if tri_t >= 0.0 && tri_t < result {
                result = tri_t;
            }
        }
    }

    result
}

fn get_vertex_position<P: MapProjection>(
    encoding: &TerrainEncoding,
    mode: SceneMode,
    projection: &P,
    ray: &Ray,
    vertices: &[f32],
    index: usize,
) -> Cartesian3 {
    let position = encoding.get_exaggerated_position(vertices, index);
    if mode == SceneMode::Scene3D {
        return position;
    }

    let ellipsoid = projection.ellipsoid();
    let carto = ellipsoid
        .cartesian_to_cartographic(position.into())
        .unwrap_or(crate::Cartographic::from_radians(0.0, 0.0));
    let projected = projection.project(carto).unwrap_or(glam::DVec3::ZERO);

    // Swizzle for 2D/Columbus View: (z, x, y)
    let mut result = Cartesian3::new(projected.z, projected.x, projected.y);

    let world_width = TWO_PI * ellipsoid.maximum_radius();
    let k = ((ray.origin.y - result.y) / world_width).round();
    result.y += k * world_width;

    result
}
