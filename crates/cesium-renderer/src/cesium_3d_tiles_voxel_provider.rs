//! Cesium3DTilesVoxelProvider matching CesiumJS `Scene/Cesium3DTilesVoxelProvider.js`.

use cesium_math::{
    Cartesian3, DeveloperError, Ellipsoid, Matrix3, Matrix4, OrientedBoundingBox, Quaternion,
    RuntimeError,
};
use serde_json::Value;

use crate::metadata_component_type::MetadataComponentType;
use crate::metadata_type::MetadataType;
use crate::voxel_primitive::VoxelProvider;
use crate::voxel_shape_type::VoxelShapeType;

/// Initialization options for `Cesium3DTilesVoxelProvider`.
#[derive(Clone, Debug)]
pub struct Cesium3DTilesVoxelProviderOptions {
    pub class_name: String,
    pub names: Vec<String>,
    pub types: Vec<MetadataType>,
    pub component_types: Vec<MetadataComponentType>,
    pub shape: VoxelShapeType,
    pub dimensions: Cartesian3,
    pub padding_before: Option<Cartesian3>,
    pub padding_after: Option<Cartesian3>,
    pub global_transform: Option<Matrix4>,
    pub shape_transform: Option<Matrix4>,
    pub min_bounds: Option<Cartesian3>,
    pub max_bounds: Option<Cartesian3>,
    pub minimum_values: Option<Vec<Vec<f64>>>,
    pub maximum_values: Option<Vec<Vec<f64>>>,
    pub maximum_tile_count: Option<usize>,
}

/// Voxel tile content returned by `Cesium3DTilesVoxelProvider::request_data`.
#[derive(Clone, Debug)]
pub struct Cesium3DTilesVoxelContent {
    pub metadata: Vec<Vec<f32>>,
    pub ready: bool,
}

impl Cesium3DTilesVoxelContent {
    pub fn update(&mut self) -> bool {
        self.ready = true;
        self.ready
    }
}

/// A `VoxelProvider` that fetches voxel metadata & geometry configuration from a 3D Tiles voxel tileset.
#[derive(Clone, Debug)]
pub struct Cesium3DTilesVoxelProvider {
    class_name: String,
    names: Vec<String>,
    types: Vec<MetadataType>,
    component_types: Vec<MetadataComponentType>,
    shape: VoxelShapeType,
    dimensions: Cartesian3,
    padding_before: Cartesian3,
    padding_after: Cartesian3,
    global_transform: Matrix4,
    shape_transform: Matrix4,
    min_bounds: Cartesian3,
    max_bounds: Cartesian3,
    minimum_values: Option<Vec<Vec<f64>>>,
    maximum_values: Option<Vec<Vec<f64>>>,
    maximum_tile_count: Option<usize>,
    available_levels: Option<usize>,
}

impl Cesium3DTilesVoxelProvider {
    /// Creates a new `Cesium3DTilesVoxelProvider` from explicit options.
    pub fn new(options: Cesium3DTilesVoxelProviderOptions) -> Result<Self, DeveloperError> {
        if options.class_name.is_empty() {
            return Err(DeveloperError::new("className must be a non-empty string."));
        }
        if options.names.len() != options.types.len()
            || options.names.len() != options.component_types.len()
        {
            return Err(DeveloperError::new(
                "names, types, and componentTypes must have matching lengths.",
            ));
        }
        let shape = options.shape;
        Ok(Self {
            class_name: options.class_name,
            names: options.names,
            types: options.types,
            component_types: options.component_types,
            shape,
            dimensions: options.dimensions,
            padding_before: options.padding_before.unwrap_or(Cartesian3::ZERO),
            padding_after: options.padding_after.unwrap_or(Cartesian3::ZERO),
            global_transform: options.global_transform.unwrap_or(Matrix4::IDENTITY),
            shape_transform: options.shape_transform.unwrap_or(Matrix4::IDENTITY),
            min_bounds: options
                .min_bounds
                .unwrap_or_else(|| VoxelShapeType::get_min_bounds(shape)),
            max_bounds: options
                .max_bounds
                .unwrap_or_else(|| VoxelShapeType::get_max_bounds(shape)),
            minimum_values: options.minimum_values,
            maximum_values: options.maximum_values,
            maximum_tile_count: options.maximum_tile_count,
            available_levels: None,
        })
    }

    /// Parses a `Cesium3DTilesVoxelProvider` from a 3D Tiles voxel `tileset.json` and optional external schema JSON.
    pub fn from_tileset_json(
        tileset_json: &Value,
        external_schema: Option<&Value>,
    ) -> Result<Self, RuntimeError> {
        let root = tileset_json
            .get("root")
            .ok_or_else(|| RuntimeError::new("Tileset must have a root tile"))?;

        let content = root
            .get("content")
            .ok_or_else(|| RuntimeError::new("Root must have content"))?;

        let voxel = content
            .get("extensions")
            .and_then(|e| e.get("3DTILES_content_voxels"))
            .ok_or_else(|| {
                RuntimeError::new("Root tile content must have 3DTILES_content_voxels extension")
            })?;

        let implicit_tiling = root
            .get("extensions")
            .and_then(|e| e.get("3DTILES_implicit_tiling"))
            .or_else(|| root.get("implicitTiling"))
            .ok_or_else(|| RuntimeError::new("Root tile must have implicit tiling"))?;

        let metadata_json = tileset_json
            .get("extensions")
            .and_then(|e| e.get("3DTILES_metadata"))
            .unwrap_or(tileset_json);

        let schema = external_schema
            .or_else(|| metadata_json.get("schema"))
            .or_else(|| tileset_json.get("schema"))
            .ok_or_else(|| RuntimeError::new("Tileset must have a metadata schema"))?;

        let class_name = voxel
            .get("class")
            .and_then(Value::as_str)
            .ok_or_else(|| RuntimeError::new("Voxel extension must specify class"))?
            .to_string();

        let class_properties = schema
            .get("classes")
            .and_then(|c| c.get(&class_name))
            .and_then(|c| c.get("properties"))
            .and_then(Value::as_object)
            .ok_or_else(|| {
                RuntimeError::new(format!("Schema missing voxel class '{class_name}' properties"))
            })?;

        let class_statistics = metadata_json
            .get("statistics")
            .and_then(|s| s.get("classes"))
            .and_then(|c| c.get(&class_name))
            .and_then(|c| c.get("properties"));

        let mut names = Vec::with_capacity(class_properties.len());
        let mut types = Vec::with_capacity(class_properties.len());
        let mut component_types = Vec::with_capacity(class_properties.len());
        let mut min_vals = Vec::with_capacity(class_properties.len());
        let mut max_vals = Vec::with_capacity(class_properties.len());
        let mut has_min = false;

        for (id, prop) in class_properties {
            let prop_type = parse_metadata_type(
                prop.get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("SCALAR"),
            )?;
            let comp_type = parse_component_type(
                prop.get("componentType")
                    .and_then(Value::as_str)
                    .unwrap_or("FLOAT32"),
            )?;
            let component_count = prop_type.get_component_count();

            let prop_stats = class_statistics.and_then(|p| p.get(id));
            if let Some(min_v) = prop_stats.and_then(|s| s.get("min")) {
                has_min = true;
                min_vals.push(copy_json_number_array(min_v, component_count));
            } else {
                min_vals.push(Vec::new());
            }
            if let Some(max_v) = prop_stats.and_then(|s| s.get("max")) {
                max_vals.push(copy_json_number_array(max_v, component_count));
            } else {
                max_vals.push(Vec::new());
            }

            names.push(id.clone());
            types.push(prop_type);
            component_types.push(comp_type);
        }

        let bounding_volume = root
            .get("boundingVolume")
            .ok_or_else(|| RuntimeError::new("Root tile must have a boundingVolume"))?;
        let (shape, min_bounds, max_bounds, shape_transform) = get_shape(bounding_volume)?;

        let global_transform = root
            .get("transform")
            .and_then(Value::as_array)
            .filter(|a| a.len() == 16)
            .map(|a| unpack_matrix4(a))
            .unwrap_or(Matrix4::IDENTITY);

        let dimensions = voxel
            .get("dimensions")
            .and_then(Value::as_array)
            .filter(|a| a.len() >= 3)
            .map(|a| unpack_cartesian3(a))
            .ok_or_else(|| RuntimeError::new("Voxel extension must specify dimensions"))?;

        let padding_before = voxel
            .get("padding")
            .and_then(|p| p.get("before"))
            .and_then(Value::as_array)
            .filter(|a| a.len() >= 3)
            .map(|a| unpack_cartesian3(a));
        let padding_after = voxel
            .get("padding")
            .and_then(|p| p.get("after"))
            .and_then(Value::as_array)
            .filter(|a| a.len() >= 3)
            .map(|a| unpack_cartesian3(a));

        let available_levels = implicit_tiling
            .get("availableLevels")
            .and_then(Value::as_u64)
            .map(|v| v as usize);

        let mut provider = Self::new(Cesium3DTilesVoxelProviderOptions {
            class_name,
            names,
            types,
            component_types,
            shape,
            dimensions,
            padding_before,
            padding_after,
            global_transform: Some(global_transform),
            shape_transform: Some(shape_transform),
            min_bounds: Some(min_bounds),
            max_bounds: Some(max_bounds),
            minimum_values: if has_min { Some(min_vals) } else { None },
            maximum_values: if has_min { Some(max_vals) } else { None },
            maximum_tile_count: None,
        })
        .map_err(|e| RuntimeError::new(e.to_string()))?;

        provider.available_levels = available_levels;
        Ok(provider)
    }

    pub fn class_name(&self) -> &str {
        &self.class_name
    }

    pub fn shape(&self) -> VoxelShapeType {
        self.shape
    }

    pub fn padding_before(&self) -> Cartesian3 {
        self.padding_before
    }

    pub fn padding_after(&self) -> Cartesian3 {
        self.padding_after
    }

    pub fn global_transform(&self) -> Matrix4 {
        self.global_transform
    }

    pub fn shape_transform(&self) -> Matrix4 {
        self.shape_transform
    }

    pub fn min_bounds(&self) -> Cartesian3 {
        self.min_bounds
    }

    pub fn max_bounds(&self) -> Cartesian3 {
        self.max_bounds
    }

    pub fn dimensions(&self) -> Cartesian3 {
        self.dimensions
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn types(&self) -> &[MetadataType] {
        &self.types
    }

    pub fn component_types(&self) -> &[MetadataComponentType] {
        &self.component_types
    }

    pub fn minimum_values(&self) -> Option<&[Vec<f64>]> {
        self.minimum_values.as_deref()
    }

    pub fn maximum_values(&self) -> Option<&[Vec<f64>]> {
        self.maximum_values.as_deref()
    }

    pub fn maximum_tile_count(&self) -> Option<usize> {
        self.maximum_tile_count
    }

    pub fn available_levels(&self) -> Option<usize> {
        self.available_levels
    }

    /// Requests voxel tile data for the given tile coordinate `(level, x, y, z)`.
    /// If `attribute_buffers` is `None`, generates properly sized attribute buffers
    /// initialized to each attribute's minimum value (or `0.0`).
    pub fn request_data(
        &self,
        level: usize,
        _x: usize,
        _y: usize,
        _z: usize,
        attribute_buffers: Option<&[Vec<f32>]>,
    ) -> Result<Cesium3DTilesVoxelContent, RuntimeError> {
        if let Some(levels) = self.available_levels {
            if level >= levels {
                return Err(RuntimeError::new(format!(
                    "Requested level {level} exceeds availableLevels {levels}."
                )));
            }
        }
        let padded_x = self.dimensions.x + self.padding_before.x + self.padding_after.x;
        let padded_y = self.dimensions.y + self.padding_before.y + self.padding_after.y;
        let padded_z = self.dimensions.z + self.padding_before.z + self.padding_after.z;
        let voxel_count = (padded_x * padded_y * padded_z).max(0.0) as usize;

        if let Some(buffers) = attribute_buffers {
            if buffers.len() != self.names.len() {
                return Err(RuntimeError::new(
                    "attribute_buffers length must match provider attribute count.",
                ));
            }
            return Ok(Cesium3DTilesVoxelContent {
                metadata: buffers.to_vec(),
                ready: true,
            });
        }

        let mut metadata = Vec::with_capacity(self.names.len());
        for (i, ty) in self.types.iter().enumerate() {
            let component_count = ty.get_component_count();
            let expected_len = voxel_count * component_count;
            let fill = self
                .minimum_values
                .as_ref()
                .and_then(|m| m.get(i))
                .and_then(|v| v.first())
                .copied()
                .unwrap_or(0.0) as f32;
            metadata.push(vec![fill; expected_len]);
        }

        Ok(Cesium3DTilesVoxelContent {
            metadata,
            ready: true,
        })
    }
}

impl VoxelProvider for Cesium3DTilesVoxelProvider {
    fn shape_type(&self) -> VoxelShapeType {
        self.shape
    }

    fn dimensions(&self) -> Cartesian3 {
        self.dimensions
    }

    fn tile_count(&self) -> usize {
        self.maximum_tile_count.unwrap_or(1)
    }

    fn minimum_values(&self) -> Vec<f64> {
        self.minimum_values
            .as_ref()
            .and_then(|v| v.first())
            .cloned()
            .unwrap_or_default()
    }

    fn maximum_values(&self) -> Vec<f64> {
        self.maximum_values
            .as_ref()
            .and_then(|v| v.first())
            .cloned()
            .unwrap_or_default()
    }

    fn min_bounds(&self) -> Cartesian3 {
        self.min_bounds
    }

    fn max_bounds(&self) -> Cartesian3 {
        self.max_bounds
    }

    fn shape_transform(&self) -> Option<Matrix4> {
        Some(self.shape_transform)
    }

    fn global_transform(&self) -> Option<Matrix4> {
        Some(self.global_transform)
    }

    fn names(&self) -> Vec<String> {
        self.names.clone()
    }

    fn types(&self) -> Vec<MetadataType> {
        self.types.clone()
    }

    fn component_types(&self) -> Vec<MetadataComponentType> {
        self.component_types.clone()
    }

    fn min_values_per_component(&self) -> Option<Vec<Vec<f64>>> {
        self.minimum_values.clone()
    }

    fn max_values_per_component(&self) -> Option<Vec<Vec<f64>>> {
        self.maximum_values.clone()
    }
}

fn get_shape(
    bounding_volume: &Value,
) -> Result<(VoxelShapeType, Cartesian3, Cartesian3, Matrix4), RuntimeError> {
    if let Some(box_arr) = bounding_volume.get("box").and_then(Value::as_array) {
        if box_arr.len() != 12 {
            return Err(RuntimeError::new("boundingVolume.box must have 12 numbers"));
        }
        let mut raw = [0.0f64; 12];
        for (i, v) in box_arr.iter().enumerate() {
            raw[i] = v.as_f64().unwrap_or(0.0);
        }
        let obb = OrientedBoundingBox::unpack(&raw, 0)
            .map_err(|e| RuntimeError::new(e.to_string()))?;
        let ax = obb.half_axes.x_axis;
        let ay = obb.half_axes.y_axis;
        let az = obb.half_axes.z_axis;
        let sx = ax.length();
        let sy = ay.length();
        let sz = az.length();
        let scale = Cartesian3::new(sx, sy, sz);
        let nx = if sx > 0.0 { ax / sx } else { glam::DVec3::X };
        let ny = if sy > 0.0 { ay / sy } else { glam::DVec3::Y };
        let nz = if sz > 0.0 { az / sz } else { glam::DVec3::Z };
        let rotation = Matrix3::from_column_major_array([
            nx.x, nx.y, nx.z, ny.x, ny.y, ny.z, nz.x, nz.y, nz.z,
        ]);
        let center = Cartesian3::new(obb.center.x, obb.center.y, obb.center.z);
        let shape_transform = Matrix4::from_rotation_translation(&rotation, center);
        return Ok((
            VoxelShapeType::Box,
            Cartesian3::new(-scale.x, -scale.y, -scale.z),
            scale,
            shape_transform,
        ));
    }

    if let Some(region_arr) = bounding_volume.get("region").and_then(Value::as_array) {
        if region_arr.len() < 6 {
            return Err(RuntimeError::new("boundingVolume.region must have 6 numbers"));
        }
        let west = region_arr[0].as_f64().unwrap_or(0.0);
        let south = region_arr[1].as_f64().unwrap_or(0.0);
        let east = region_arr[2].as_f64().unwrap_or(0.0);
        let north = region_arr[3].as_f64().unwrap_or(0.0);
        let min_height = region_arr[4].as_f64().unwrap_or(0.0);
        let max_height = region_arr[5].as_f64().unwrap_or(0.0);
        let radii = Ellipsoid::WGS84.radii();
        let shape_transform = Matrix4::from_scale(Cartesian3::new(radii.x, radii.y, radii.z));
        return Ok((
            VoxelShapeType::Ellipsoid,
            Cartesian3::new(west, south, min_height),
            Cartesian3::new(east, north, max_height),
            shape_transform,
        ));
    }

    if let Some(cyl) = bounding_volume
        .get("extensions")
        .and_then(|e| e.get("3DTILES_bounding_volume_cylinder"))
    {
        let min_radius = cyl.get("minRadius").and_then(Value::as_f64).ok_or_else(|| {
            RuntimeError::new("3DTILES_bounding_volume_cylinder.minRadius is required")
        })?;
        let max_radius = cyl.get("maxRadius").and_then(Value::as_f64).ok_or_else(|| {
            RuntimeError::new("3DTILES_bounding_volume_cylinder.maxRadius is required")
        })?;
        let height = cyl.get("height").and_then(Value::as_f64).ok_or_else(|| {
            RuntimeError::new("3DTILES_bounding_volume_cylinder.height is required")
        })?;
        let min_angle = cyl
            .get("minAngle")
            .and_then(Value::as_f64)
            .unwrap_or(-std::f64::consts::PI);
        let max_angle = cyl
            .get("maxAngle")
            .and_then(Value::as_f64)
            .unwrap_or(std::f64::consts::PI);
        let translation = cyl
            .get("translation")
            .and_then(Value::as_array)
            .filter(|a| a.len() >= 3)
            .map(|a| unpack_cartesian3(a))
            .unwrap_or(Cartesian3::ZERO);
        let rotation = cyl
            .get("rotation")
            .and_then(Value::as_array)
            .filter(|a| a.len() >= 4)
            .map(|a| {
                Quaternion::new(
                    a[0].as_f64().unwrap_or(0.0),
                    a[1].as_f64().unwrap_or(0.0),
                    a[2].as_f64().unwrap_or(0.0),
                    a[3].as_f64().unwrap_or(1.0),
                )
            })
            .unwrap_or(Quaternion::IDENTITY);
        let min_height = -0.5 * height + translation.z;
        let max_height = 0.5 * height + translation.z;
        let shape_transform = Matrix4::from_translation_quaternion_rotation_scale(
            translation,
            rotation,
            Cartesian3::new(1.0, 1.0, 1.0),
        );
        return Ok((
            VoxelShapeType::Cylinder,
            Cartesian3::new(min_radius, min_angle, min_height),
            Cartesian3::new(max_radius, max_angle, max_height),
            shape_transform,
        ));
    }

    Err(RuntimeError::new(
        "Only box, region and 3DTILES_bounding_volume_cylinder are supported in Cesium3DTilesVoxelProvider",
    ))
}

fn unpack_cartesian3(arr: &[Value]) -> Cartesian3 {
    Cartesian3::new(
        arr[0].as_f64().unwrap_or(0.0),
        arr[1].as_f64().unwrap_or(0.0),
        arr[2].as_f64().unwrap_or(0.0),
    )
}

fn unpack_matrix4(arr: &[Value]) -> Matrix4 {
    let mut raw = [0.0f64; 16];
    for (i, v) in arr.iter().take(16).enumerate() {
        raw[i] = v.as_f64().unwrap_or(0.0);
    }
    Matrix4::from_column_major_array(raw)
}

fn copy_json_number_array(value: &Value, len: usize) -> Vec<f64> {
    match value {
        Value::Array(arr) => (0..len)
            .map(|i| arr.get(i).and_then(Value::as_f64).unwrap_or(0.0))
            .collect(),
        Value::Number(n) => {
            let mut out = vec![0.0; len];
            if !out.is_empty() {
                out[0] = n.as_f64().unwrap_or(0.0);
            }
            out
        }
        _ => vec![0.0; len],
    }
}

fn parse_metadata_type(s: &str) -> Result<MetadataType, RuntimeError> {
    match s {
        "SCALAR" => Ok(MetadataType::Scalar),
        "VEC2" => Ok(MetadataType::Vec2),
        "VEC3" => Ok(MetadataType::Vec3),
        "VEC4" => Ok(MetadataType::Vec4),
        "MAT2" => Ok(MetadataType::Mat2),
        "MAT3" => Ok(MetadataType::Mat3),
        "MAT4" => Ok(MetadataType::Mat4),
        "STRING" => Ok(MetadataType::String),
        "BOOLEAN" => Ok(MetadataType::Boolean),
        "ENUM" => Ok(MetadataType::Enum),
        other => Err(RuntimeError::new(format!(
            "Unsupported MetadataType: {other}"
        ))),
    }
}

fn parse_component_type(s: &str) -> Result<MetadataComponentType, RuntimeError> {
    match s {
        "INT8" => Ok(MetadataComponentType::Int8),
        "UINT8" => Ok(MetadataComponentType::Uint8),
        "INT16" => Ok(MetadataComponentType::Int16),
        "UINT16" => Ok(MetadataComponentType::Uint16),
        "INT32" => Ok(MetadataComponentType::Int32),
        "UINT32" => Ok(MetadataComponentType::Uint32),
        "INT64" => Ok(MetadataComponentType::Int64),
        "UINT64" => Ok(MetadataComponentType::Uint64),
        "FLOAT32" => Ok(MetadataComponentType::Float32),
        "FLOAT64" => Ok(MetadataComponentType::Float64),
        other => Err(RuntimeError::new(format!(
            "Unsupported MetadataComponentType: {other}"
        ))),
    }
}
