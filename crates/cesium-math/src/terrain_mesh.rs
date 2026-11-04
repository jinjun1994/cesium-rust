use crate::cartesian::Cartesian3;
use crate::ellipsoid::Ellipsoid;
use crate::fixed_frame_transforms::FixedFrameTransforms;
use crate::matrices::Matrix4;
use crate::oriented_bounding_box::OrientedBoundingBox;
use crate::projection::MapProjection;
use crate::ray::Ray;
use crate::rectangle::GeoRectangle;
use crate::scene_mode::SceneMode;
use crate::terrain_encoding::TerrainEncoding;
use crate::terrain_picker::TerrainPicker;
use crate::vertical_exaggeration::VerticalExaggeration;
use crate::{Cartographic, Sphere};

/// A mesh plus related metadata for a single tile of terrain.
#[derive(Debug, Clone)]
pub struct TerrainMesh {
    pub center: Cartesian3,
    pub vertices: Vec<f32>,
    pub stride: usize,
    pub indices: Vec<u32>,
    pub index_count_without_skirts: usize,
    pub vertex_count_without_skirts: usize,
    pub minimum_height: f64,
    pub maximum_height: f64,
    pub rectangle: GeoRectangle,
    pub bounding_sphere3d: Sphere,
    pub occludee_point_in_scaled_space: Cartesian3,
    pub oriented_bounding_box: Option<OrientedBoundingBox>,
    pub encoding: TerrainEncoding,
    pub west_indices_south_to_north: Vec<u32>,
    pub south_indices_east_to_west: Vec<u32>,
    pub east_indices_north_to_south: Vec<u32>,
    pub north_indices_west_to_east: Vec<u32>,
    pub transform: Matrix4,
    pub last_pick_scene_mode: Option<SceneMode>,
    pub terrain_picker: TerrainPicker,
}

impl TerrainMesh {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        center: Cartesian3,
        vertices: Vec<f32>,
        indices: Vec<u32>,
        index_count_without_skirts: usize,
        vertex_count_without_skirts: usize,
        minimum_height: f64,
        maximum_height: f64,
        rectangle: GeoRectangle,
        bounding_sphere3d: Sphere,
        occludee_point_in_scaled_space: Cartesian3,
        vertex_stride: Option<usize>,
        oriented_bounding_box: Option<OrientedBoundingBox>,
        encoding: TerrainEncoding,
        west_indices_south_to_north: Vec<u32>,
        south_indices_east_to_west: Vec<u32>,
        east_indices_north_to_south: Vec<u32>,
        north_indices_west_to_east: Vec<u32>,
    ) -> Self {
        let stride = vertex_stride.unwrap_or(6);
        let terrain_picker = TerrainPicker::new(vertices.clone(), indices.clone(), encoding.clone());

        Self {
            center,
            vertices,
            stride,
            indices,
            index_count_without_skirts,
            vertex_count_without_skirts,
            minimum_height,
            maximum_height,
            rectangle,
            bounding_sphere3d,
            occludee_point_in_scaled_space,
            oriented_bounding_box,
            encoding,
            west_indices_south_to_north,
            south_indices_east_to_west,
            east_indices_north_to_south,
            north_indices_west_to_east,
            transform: Matrix4::IDENTITY,
            last_pick_scene_mode: None,
            terrain_picker,
        }
    }

    /// Gets the terrain tile's model-to-world transform matrix for the given scene mode and projection.
    pub fn get_transform<P: MapProjection>(&mut self, mode: SceneMode, projection: &P) -> Matrix4 {
        if self.last_pick_scene_mode == Some(mode) {
            return self.transform;
        }

        self.terrain_picker.set_needs_rebuild(true);

        self.transform = if mode == SceneMode::Scene3D {
            compute_transform(self)
        } else {
            compute_transform_2d(self, projection)
        };

        self.transform
    }

    /// Gives the point on this terrain tile where the given ray intersects.
    pub fn pick<P: MapProjection>(
        &mut self,
        ray: &Ray,
        cull_back_faces: bool,
        mode: SceneMode,
        projection: &P,
    ) -> Option<Cartesian3> {
        let transform = self.get_transform(mode, projection);
        let intersection = self
            .terrain_picker
            .ray_intersect(ray, &transform, cull_back_faces, mode, projection);
        self.last_pick_scene_mode = Some(mode);
        intersection
    }

    /// Updates the terrain mesh to account for changes in vertical exaggeration.
    pub fn update_exaggeration(
        &mut self,
        exaggeration: f64,
        exaggeration_relative_height: f64,
    ) {
        self.encoding.exaggeration = exaggeration;
        self.encoding.exaggeration_relative_height = exaggeration_relative_height;
        self.terrain_picker.encoding = self.encoding.clone();
        self.terrain_picker.vertices = self.vertices.clone();
        self.terrain_picker.set_needs_rebuild(true);
        self.last_pick_scene_mode = None;
    }

    /// Updates the terrain mesh to account for changes in scene mode.
    pub fn update_scene_mode(&mut self, _mode: SceneMode) {
        self.terrain_picker.set_needs_rebuild(true);
        self.last_pick_scene_mode = None;
    }
}

fn compute_transform(mesh: &TerrainMesh) -> Matrix4 {
    let exaggeration = mesh.encoding.exaggeration;
    let exaggeration_relative_height = mesh.encoding.exaggeration_relative_height;

    let exaggerated_min_height = VerticalExaggeration::get_height(
        mesh.minimum_height,
        exaggeration,
        exaggeration_relative_height,
    );

    let exaggerated_max_height = VerticalExaggeration::get_height(
        mesh.maximum_height,
        exaggeration,
        exaggeration_relative_height,
    );

    let obb = if let Some(ref existing) = mesh.oriented_bounding_box {
        *existing
    } else {
        OrientedBoundingBox::from_rectangle(
            mesh.rectangle,
            exaggerated_min_height,
            exaggerated_max_height,
            Ellipsoid::default(),
        )
        .unwrap_or_else(|_| OrientedBoundingBox::new(glam::DVec3::ZERO, glam::DMat3::IDENTITY))
    };

    let dmat = obb.transformation();
    let mut mat = Matrix4::from_column_major_array(dmat.to_cols_array());

    let mut scale = mat.get_scale();
    if scale.z <= 1e-16 {
        scale.z = 1.0;
        mat.set_scale(scale);
    }

    mat
}

fn compute_transform_2d<P: MapProjection>(
    mesh: &TerrainMesh,
    projection: &P,
) -> Matrix4 {
    let exaggeration = mesh.encoding.exaggeration;
    let exaggeration_relative_height = mesh.encoding.exaggeration_relative_height;

    let exaggerated_min_height = VerticalExaggeration::get_height(
        mesh.minimum_height,
        exaggeration,
        exaggeration_relative_height,
    );

    let exaggerated_max_height = VerticalExaggeration::get_height(
        mesh.maximum_height,
        exaggeration,
        exaggeration_relative_height,
    );

    let sw_carto = Cartographic::from_radians(mesh.rectangle.west, mesh.rectangle.south);
    let ne_carto = Cartographic::from_radians(mesh.rectangle.east, mesh.rectangle.north);

    let southwest = projection.project(sw_carto).unwrap_or(glam::DVec3::ZERO);
    let northeast = projection.project(ne_carto).unwrap_or(glam::DVec3::ZERO);

    let height_range = exaggerated_max_height - exaggerated_min_height;
    let scale = Cartesian3::new(
        northeast.x - southwest.x,
        northeast.y - southwest.y,
        if height_range > 0.0 { height_range } else { 1.0 },
    );

    let center = Cartesian3::new(
        southwest.x + scale.x * 0.5,
        southwest.y + scale.y * 0.5,
        exaggerated_min_height + scale.z * 0.5,
    );

    let mut mat = Matrix4::from_translation(center);
    mat.set_scale(scale);
    FixedFrameTransforms::SWIZZLE_3D_TO_2D_MATRIX.multiply(&mat)
}
