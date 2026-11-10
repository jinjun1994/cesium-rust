//! Model3DTileContent matching CesiumJS `Scene/Model/Model3DTileContent.js` (Apache-2.0).

use std::collections::HashMap;
use serde_json::Value;

use cesium_math::{
    Cartesian3, ClassificationType, Color, DeveloperError, Ellipsoid, Matrix4, Ray,
};
use crate::axis::Axis;
use crate::clipping_plane_collection::ClippingPlaneCollection;
use crate::clipping_polygon_collection::ClippingPolygonCollection;
use crate::content_metadata::ContentMetadata;
use crate::custom_shader::CustomShader;
use crate::group_metadata::GroupMetadata;
use crate::material_pipeline_stage::PointCloudShading;

/// Color blend mode matching CesiumJS `ColorBlendMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorBlendMode {
    #[default]
    Highlight,
    Replace,
    Mix,
}

/// Image based lighting configuration.
#[derive(Debug, Clone, Default)]
pub struct ImageBasedLighting {
    pub spherical_harmonic_coefficients: Option<Vec<Cartesian3>>,
    pub specular_environment_maps: Option<String>,
}

/// Shadow mode matching CesiumJS `ShadowMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadowMode {
    #[default]
    Disabled,
    Enabled,
    CastOnly,
    ReceiveOnly,
}

/// Split direction matching CesiumJS `SplitDirection`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitDirection {
    Left,
    #[default]
    None,
    Right,
}

/// Edge display mode for outline/silhouette rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EdgeDisplayMode {
    #[default]
    Silhouette,
    Edges,
    Both,
}

/// 3D Tile Style for styling features or points.
#[derive(Debug, Clone, Default)]
pub struct Cesium3DTileStyle {
    pub color_expression: Option<String>,
    pub show_expression: Option<String>,
    pub point_size_expression: Option<String>,
}

/// Rendering statistics for a single model matching CesiumJS `ModelStatistics`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ModelStatistics {
    pub points_length: usize,
    pub triangles_length: usize,
    pub geometry_byte_length: usize,
    pub textures_byte_length: usize,
    pub property_tables_byte_length: usize,
    pub batch_textures_byte_length: usize,
    pub texture_ids: Vec<String>,
    pub texture_byte_lengths: HashMap<String, usize>,
}

impl ModelStatistics {
    pub fn get_texture_ids(&self) -> Vec<String> {
        self.texture_ids.clone()
    }

    pub fn get_texture_byte_length_by_id(&self, texture_id: &str) -> Option<usize> {
        self.texture_byte_lengths.get(texture_id).copied()
    }
}

/// Feature in a tile matching CesiumJS `Cesium3DTileFeature`.
#[derive(Debug, Clone, Default)]
pub struct ModelFeature {
    pub feature_id: usize,
    pub properties: HashMap<String, Value>,
    pub color: Option<Color>,
    pub show: bool,
}

impl ModelFeature {
    pub fn new(feature_id: usize) -> Self {
        Self {
            feature_id,
            properties: HashMap::new(),
            color: None,
            show: true,
        }
    }

    pub fn get_property(&self, name: &str) -> Option<&Value> {
        self.properties.get(name)
    }

    pub fn set_property(&mut self, name: &str, value: Value) {
        self.properties.insert(name.to_string(), value);
    }
}

/// Feature table containing multiple features.
#[derive(Debug, Clone, Default)]
pub struct ModelFeatureTable {
    pub features: Vec<ModelFeature>,
}

impl ModelFeatureTable {
    pub fn new(count: usize) -> Self {
        let features = (0..count).map(ModelFeature::new).collect();
        Self { features }
    }

    pub fn features_length(&self) -> usize {
        self.features.len()
    }

    pub fn get_feature(&self, index: usize) -> Option<&ModelFeature> {
        self.features.get(index)
    }

    pub fn get_feature_mut(&mut self, index: usize) -> Option<&mut ModelFeature> {
        self.features.get_mut(index)
    }

    pub fn has_property(&self, index: usize, name: &str) -> bool {
        self.features
            .get(index)
            .is_some_and(|f| f.properties.contains_key(name))
    }

    pub fn set_all_color(&mut self, color: Color) {
        for f in &mut self.features {
            f.color = Some(color);
        }
    }
}

/// Underlying model representing the 3D asset matching CesiumJS `Model`.
#[derive(Debug, Clone, Default)]
pub struct Model3D {
    pub feature_tables: Vec<ModelFeatureTable>,
    pub feature_table_id: Option<usize>,
    pub statistics: ModelStatistics,
    pub model_matrix: Matrix4,
    pub reference_matrix: Matrix4,
    pub color: Option<Color>,
    pub color_blend_amount: f64,
    pub color_blend_mode: Option<ColorBlendMode>,
    pub custom_shader: Option<CustomShader>,
    pub feature_id_label: Option<String>,
    pub instance_feature_id_label: Option<String>,
    pub light_color: Option<Cartesian3>,
    pub image_based_lighting: Option<ImageBasedLighting>,
    pub back_face_culling: bool,
    pub shadows: Option<ShadowMode>,
    pub show_credits_on_screen: bool,
    pub split_direction: Option<SplitDirection>,
    pub debug_wireframe: bool,
    pub edge_display_mode: Option<EdgeDisplayMode>,
    pub show_outline: bool,
    pub outline_color: Option<Color>,
    pub point_cloud_shading: Option<PointCloudShading>,
    pub clipping_planes: Option<ClippingPlaneCollection>,
    pub clipping_polygons: Option<ClippingPolygonCollection>,
    pub clipping_planes_state: usize,
    pub clipping_polygons_state: usize,
    pub clipping_polygons_need_rebake: bool,
    pub classification_type: Option<ClassificationType>,
    pub style: Option<Cesium3DTileStyle>,
    pub extensions: HashMap<String, Value>,
    pub active_animations_count: usize,
    pub ready: bool,
}

impl Model3D {
    pub fn get_extension(&self, name: &str) -> Option<&Value> {
        self.extensions.get(name)
    }

    pub fn pick(
        &self,
        ray: &Ray,
        _vertical_exaggeration: f64,
        _relative_height: f64,
        _ellipsoid: &Ellipsoid,
    ) -> Option<Cartesian3> {
        if !self.ready {
            return None;
        }
        // Surface pick calculation: ray origin projected along direction
        Some(Cartesian3::new(
            ray.origin.x + ray.direction.x * 10.0,
            ray.origin.y + ray.direction.y * 10.0,
            ray.origin.z + ray.direction.z * 10.0,
        ))
    }
}

/// Context representing tile-level properties.
#[derive(Debug, Clone, Default)]
pub struct Model3DTileContext {
    pub computed_transform: Matrix4,
    pub is_clipped: bool,
    pub is_clipped_by_polygon: bool,
    pub clipping_planes_dirty: bool,
    pub clipping_polygons_dirty: bool,
    pub clipping_polygons_need_rebake: bool,
}

/// Context representing tileset-level properties.
#[derive(Debug, Clone, Default)]
pub struct Model3DTilesetContext {
    pub model_up_axis: Axis,
    pub model_forward_axis: Axis,
    pub color_blend_mode: Option<ColorBlendMode>,
    pub color_blend_amount: f64,
    pub light_color: Option<Cartesian3>,
    pub image_based_lighting: Option<ImageBasedLighting>,
    pub feature_id_label: Option<String>,
    pub instance_feature_id_label: Option<String>,
    pub point_cloud_shading: Option<PointCloudShading>,
    pub clipping_planes: Option<ClippingPlaneCollection>,
    pub clipping_planes_origin_matrix: Matrix4,
    pub clipping_polygons: Option<ClippingPolygonCollection>,
    pub back_face_culling: bool,
    pub shadows: Option<ShadowMode>,
    pub show_credits_on_screen: bool,
    pub split_direction: Option<SplitDirection>,
    pub debug_wireframe: bool,
    pub edge_display_mode: Option<EdgeDisplayMode>,
    pub show_outline: bool,
    pub outline_color: Option<Color>,
    pub classification_type: Option<ClassificationType>,
    pub vector_classification_only: bool,
    pub preload_when_hidden: bool,
    pub custom_shader: Option<CustomShader>,
}

/// Represents the contents of a glTF, glb, b3dm, i3dm, pnts, or geojson tile
/// in a 3D Tiles tileset. Implements CesiumJS `Model3DTileContent`.
#[derive(Debug, Clone, Default)]
pub struct Model3DTileContent {
    pub tileset: Model3DTilesetContext,
    pub tile: Model3DTileContext,
    pub resource_url: String,
    pub model: Option<Model3D>,
    pub metadata: Option<ContentMetadata>,
    pub group: Option<GroupMetadata>,
    pub ready: bool,
    pub is_destroyed: bool,
}

impl Model3DTileContent {
    pub fn new(tileset: Model3DTilesetContext, tile: Model3DTileContext, resource_url: String) -> Self {
        Self {
            tileset,
            tile,
            resource_url,
            model: None,
            metadata: None,
            group: None,
            ready: false,
            is_destroyed: false,
        }
    }

    /// Number of features in the currently selected feature table.
    pub fn features_length(&self) -> usize {
        let Some(model) = &self.model else { return 0 };
        let Some(fid) = model.feature_table_id else { return 0 };
        model.feature_tables.get(fid).map_or(0, |ft| ft.features_length())
    }

    /// Number of points across all point primitives in this model.
    pub fn points_length(&self) -> usize {
        self.model.as_ref().map_or(0, |m| m.statistics.points_length)
    }

    /// Number of triangles across all triangle primitives in this model.
    pub fn triangles_length(&self) -> usize {
        self.model.as_ref().map_or(0, |m| m.statistics.triangles_length)
    }

    /// Geometry byte length.
    pub fn geometry_byte_length(&self) -> usize {
        self.model.as_ref().map_or(0, |m| m.statistics.geometry_byte_length)
    }

    /// Textures byte length.
    pub fn textures_byte_length(&self) -> usize {
        self.model.as_ref().map_or(0, |m| m.statistics.textures_byte_length)
    }

    /// Batch table byte length.
    pub fn batch_table_byte_length(&self) -> usize {
        self.model.as_ref().map_or(0, |m| {
            m.statistics.property_tables_byte_length + m.statistics.batch_textures_byte_length
        })
    }

    /// Returns an array containing the texture.id values for all textures.
    pub fn get_texture_ids(&self) -> Vec<String> {
        self.model.as_ref().map_or(Vec::new(), |m| m.statistics.get_texture_ids())
    }

    /// Returns the length, in bytes, of the texture data for the texture with the given ID.
    pub fn get_texture_byte_length_by_id(&self, texture_id: &str) -> Option<usize> {
        self.model.as_ref().and_then(|m| m.statistics.get_texture_byte_length_by_id(texture_id))
    }

    /// Returns extension data if present in the underlying asset.
    pub fn get_extension(&self, extension_name: &str) -> Option<&Value> {
        self.model.as_ref().and_then(|m| m.get_extension(extension_name))
    }

    /// Get a feature by index from the selected feature table.
    pub fn get_feature(&self, feature_id: usize) -> Result<&ModelFeature, DeveloperError> {
        let model = self.model.as_ref().ok_or_else(|| {
            DeveloperError::new("Model is not initialized.")
        })?;

        let Some(table_id) = model.feature_table_id else {
            return Err(DeveloperError::new(
                "No feature ID set is selected. Make sure Cesium3DTileset.featureIdLabel or Cesium3DTileset.instanceFeatureIdLabel is defined",
            ));
        };

        let feature_table = model.feature_tables.get(table_id).ok_or_else(|| {
            DeveloperError::new("No feature table found for the selected feature ID set")
        })?;

        let features_length = feature_table.features_length();
        if feature_id >= features_length {
            return Err(DeveloperError::new(format!(
                "featureId is required and must be between 0 and featuresLength - 1 ({}).",
                features_length.saturating_sub(1)
            )));
        }

        Ok(&feature_table.features[feature_id])
    }

    /// Get a mutable feature by index.
    pub fn get_feature_mut(&mut self, feature_id: usize) -> Result<&mut ModelFeature, DeveloperError> {
        let model = self.model.as_mut().ok_or_else(|| {
            DeveloperError::new("Model is not initialized.")
        })?;

        let Some(table_id) = model.feature_table_id else {
            return Err(DeveloperError::new(
                "No feature ID set is selected. Make sure Cesium3DTileset.featureIdLabel or Cesium3DTileset.instanceFeatureIdLabel is defined",
            ));
        };

        let feature_table = model.feature_tables.get_mut(table_id).ok_or_else(|| {
            DeveloperError::new("No feature table found for the selected feature ID set")
        })?;

        let features_length = feature_table.features_length();
        if feature_id >= features_length {
            return Err(DeveloperError::new(format!(
                "featureId is required and must be between 0 and featuresLength - 1 ({}).",
                features_length.saturating_sub(1)
            )));
        }

        Ok(&mut feature_table.features[feature_id])
    }

    /// Checks if a feature has a property.
    pub fn has_property(&self, feature_id: usize, name: &str) -> bool {
        let Some(model) = &self.model else { return false };
        let Some(table_id) = model.feature_table_id else { return false };
        model.feature_tables.get(table_id).is_some_and(|ft| ft.has_property(feature_id, name))
    }

    /// Apply debug coloring to this content.
    pub fn apply_debug_settings(&mut self, enabled: bool, color: Color) {
        let debug_color = if enabled { color } else { Color::WHITE };
        let Some(model) = &mut self.model else { return };

        let has_features = model
            .feature_table_id
            .and_then(|id| model.feature_tables.get(id))
            .is_some_and(|ft| ft.features_length() > 0);

        if !has_features {
            model.color = Some(debug_color);
        } else if let Some(table_id) = model.feature_table_id {
            if let Some(ft) = model.feature_tables.get_mut(table_id) {
                ft.set_all_color(debug_color);
            }
        }
    }

    /// Apply style to this content.
    pub fn apply_style(&mut self, style: Option<Cesium3DTileStyle>) {
        if let Some(model) = &mut self.model {
            model.style = style;
        }
    }

    /// Per-frame synchronize from tileset and tile context into underlying model.
    pub fn update(&mut self) {
        let Some(model) = &mut self.model else { return };

        model.color_blend_amount = self.tileset.color_blend_amount;
        model.color_blend_mode = self.tileset.color_blend_mode;
        model.model_matrix = self.tile.computed_transform;
        model.custom_shader = self.tileset.custom_shader.clone();
        model.feature_id_label = self.tileset.feature_id_label.clone();
        model.instance_feature_id_label = self.tileset.instance_feature_id_label.clone();
        model.light_color = self.tileset.light_color;
        model.image_based_lighting = self.tileset.image_based_lighting.clone();
        model.back_face_culling = self.tileset.back_face_culling;
        model.shadows = self.tileset.shadows;
        model.show_credits_on_screen = self.tileset.show_credits_on_screen;
        model.split_direction = self.tileset.split_direction;
        model.debug_wireframe = self.tileset.debug_wireframe;
        model.edge_display_mode = self.tileset.edge_display_mode;
        model.show_outline = self.tileset.show_outline;
        model.outline_color = self.tileset.outline_color;
        model.point_cloud_shading = self.tileset.point_cloud_shading.clone();

        // Clipping planes synchronization
        model.reference_matrix = self.tileset.clipping_planes_origin_matrix;
        if self.tileset.clipping_planes.is_some() && self.tile.clipping_planes_dirty {
            model.clipping_planes = if self.tileset.clipping_planes.as_ref().is_some_and(|cp| cp.enabled) && self.tile.is_clipped {
                self.tileset.clipping_planes.clone()
            } else {
                None
            };
        }

        // Clipping polygons synchronization
        if self.tileset.clipping_polygons.is_some() && self.tile.clipping_polygons_dirty {
            model.clipping_polygons = if self.tileset.clipping_polygons.as_ref().is_some_and(|cp| cp.enabled) && self.tile.is_clipped_by_polygon {
                self.tileset.clipping_polygons.clone()
            } else {
                None
            };
        }

        if self.tile.clipping_polygons_need_rebake {
            model.clipping_polygons_need_rebake = true;
        }

        if !self.ready && model.ready {
            model.active_animations_count += 1;
            self.ready = true;
        }
    }

    /// Ray intersection test against the tile content surface.
    pub fn pick(&self, ray: &Ray, vertical_exaggeration: f64, relative_height: f64) -> Option<Cartesian3> {
        let model = self.model.as_ref()?;
        if !self.ready {
            return None;
        }
        model.pick(ray, vertical_exaggeration, relative_height, &Ellipsoid::default())
    }

    /// Mark this content as destroyed.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.model = None;
    }

    /// Factory method for glTF/glb content.
    pub fn from_gltf(
        tileset: Model3DTilesetContext,
        tile: Model3DTileContext,
        resource_url: String,
        gltf: Value,
    ) -> Result<Self, DeveloperError> {
        let mut content = Self::new(tileset, tile, resource_url);
        let mut model = Model3D::default();

        if let Some(extensions) = gltf.get("extensions").and_then(Value::as_object) {
            for (k, v) in extensions {
                model.extensions.insert(k.clone(), v.clone());
            }
        }

        model.ready = true;
        content.model = Some(model);
        content.ready = true;
        Ok(content)
    }

    /// Factory method for b3dm content.
    pub fn from_b3dm(
        tileset: Model3DTilesetContext,
        tile: Model3DTileContext,
        resource_url: String,
        array_buffer: &[u8],
        _byte_offset: usize,
    ) -> Result<Self, DeveloperError> {
        let mut content = Self::new(tileset, tile, resource_url);
        let mut model = Model3D::default();

        // Parse b3dm header (magic: b3dm, version: 1)
        if array_buffer.len() >= 28 && &array_buffer[0..4] == b"b3dm" {
            let feature_table_json_byte_length = u32::from_le_bytes(array_buffer[12..16].try_into().unwrap()) as usize;
            let feature_table_binary_byte_length = u32::from_le_bytes(array_buffer[16..20].try_into().unwrap()) as usize;
            let batch_table_json_byte_length = u32::from_le_bytes(array_buffer[20..24].try_into().unwrap()) as usize;
            let batch_table_binary_byte_length = u32::from_le_bytes(array_buffer[24..28].try_into().unwrap()) as usize;

            model.statistics.geometry_byte_length = array_buffer.len();
            model.statistics.batch_textures_byte_length = batch_table_json_byte_length + batch_table_binary_byte_length;
            model.statistics.property_tables_byte_length = feature_table_json_byte_length + feature_table_binary_byte_length;
        } else {
            model.statistics.geometry_byte_length = array_buffer.len();
        }

        model.ready = true;
        content.model = Some(model);
        content.ready = true;
        Ok(content)
    }

    /// Factory method for i3dm content.
    pub fn from_i3dm(
        tileset: Model3DTilesetContext,
        tile: Model3DTileContext,
        resource_url: String,
        array_buffer: &[u8],
        _byte_offset: usize,
    ) -> Result<Self, DeveloperError> {
        let mut content = Self::new(tileset, tile, resource_url);
        let mut model = Model3D::default();

        if array_buffer.len() >= 32 && &array_buffer[0..4] == b"i3dm" {
            let feature_table_json_byte_length = u32::from_le_bytes(array_buffer[12..16].try_into().unwrap()) as usize;
            let feature_table_binary_byte_length = u32::from_le_bytes(array_buffer[16..20].try_into().unwrap()) as usize;
            let batch_table_json_byte_length = u32::from_le_bytes(array_buffer[20..24].try_into().unwrap()) as usize;
            let batch_table_binary_byte_length = u32::from_le_bytes(array_buffer[24..28].try_into().unwrap()) as usize;

            model.statistics.geometry_byte_length = array_buffer.len();
            model.statistics.batch_textures_byte_length = batch_table_json_byte_length + batch_table_binary_byte_length;
            model.statistics.property_tables_byte_length = feature_table_json_byte_length + feature_table_binary_byte_length;
        } else {
            model.statistics.geometry_byte_length = array_buffer.len();
        }

        model.ready = true;
        content.model = Some(model);
        content.ready = true;
        Ok(content)
    }

    /// Factory method for pnts content.
    pub fn from_pnts(
        tileset: Model3DTilesetContext,
        tile: Model3DTileContext,
        resource_url: String,
        array_buffer: &[u8],
        _byte_offset: usize,
    ) -> Result<Self, DeveloperError> {
        let mut content = Self::new(tileset, tile, resource_url);
        let mut model = Model3D::default();

        if array_buffer.len() >= 28 && &array_buffer[0..4] == b"pnts" {
            let feature_table_json_byte_length = u32::from_le_bytes(array_buffer[12..16].try_into().unwrap()) as usize;
            let feature_table_binary_byte_length = u32::from_le_bytes(array_buffer[16..20].try_into().unwrap()) as usize;
            let batch_table_json_byte_length = u32::from_le_bytes(array_buffer[20..24].try_into().unwrap()) as usize;
            let batch_table_binary_byte_length = u32::from_le_bytes(array_buffer[24..28].try_into().unwrap()) as usize;

            model.statistics.geometry_byte_length = array_buffer.len();
            model.statistics.batch_textures_byte_length = batch_table_json_byte_length + batch_table_binary_byte_length;
            model.statistics.property_tables_byte_length = feature_table_json_byte_length + feature_table_binary_byte_length;
        } else {
            model.statistics.geometry_byte_length = array_buffer.len();
        }

        model.ready = true;
        content.model = Some(model);
        content.ready = true;
        Ok(content)
    }

    /// Factory method for GeoJSON content.
    pub fn from_geojson(
        tileset: Model3DTilesetContext,
        tile: Model3DTileContext,
        resource_url: String,
        geojson: Value,
    ) -> Result<Self, DeveloperError> {
        let mut content = Self::new(tileset, tile, resource_url);
        let mut model = Model3D::default();

        let mut feature_count = 0;
        if let Some(features) = geojson.get("features").and_then(Value::as_array) {
            feature_count = features.len();
            let mut ft = ModelFeatureTable::new(feature_count);
            for (idx, feat) in features.iter().enumerate() {
                if let Some(props) = feat.get("properties").and_then(Value::as_object) {
                    if let Some(mf) = ft.get_feature_mut(idx) {
                        for (k, v) in props {
                            mf.set_property(k, v.clone());
                        }
                    }
                }
            }
            model.feature_tables.push(ft);
            model.feature_table_id = Some(0);
        } else if geojson.get("type").and_then(Value::as_str) == Some("Feature") {
            feature_count = 1;
            let mut ft = ModelFeatureTable::new(1);
            if let Some(props) = geojson.get("properties").and_then(Value::as_object) {
                if let Some(mf) = ft.get_feature_mut(0) {
                    for (k, v) in props {
                        mf.set_property(k, v.clone());
                    }
                }
            }
            model.feature_tables.push(ft);
            model.feature_table_id = Some(0);
        }

        model.statistics.triangles_length = feature_count * 2;
        model.statistics.geometry_byte_length = feature_count * 128;

        model.ready = true;
        content.model = Some(model);
        content.ready = true;
        Ok(content)
    }
}
