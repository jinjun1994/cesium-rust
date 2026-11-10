//! GeoJsonLoader for parsing GeoJSON content in 3D Tiles (MAXAR_content_geojson).
//! Matches CesiumJS `Scene/Model/GeoJsonLoader.js`.

use crate::buffer::{Buffer, BufferOptions, BufferUsage, IndexBufferOptions};
use crate::gltf_structural_metadata_loader::{MetadataSchema, PropertyTable, StructuralMetadata};
use crate::model_components::{
    Attribute, Components, FeatureId, FeatureIdAttribute, Indices, Material, Node, Primitive,
    Scene,
};
use cesium_math::{
    AttributeType, Cartesian3, ComponentDatatype, DeveloperError, Ellipsoid,
    FixedFrameTransforms, IndexDatatype, PrimitiveType,
};
use std::collections::HashMap;

/// Options for configuring `GeoJsonLoader`.
#[derive(Clone, Debug, Default)]
pub struct GeoJsonLoaderOptions {
    pub geo_json: serde_json::Value,
}

/// Loader for GeoJSON format content into `ModelComponents`.
pub struct GeoJsonLoader {
    geo_json: Option<serde_json::Value>,
    components: Option<Components>,
    pub structural_metadata: Option<StructuralMetadata>,
    is_destroyed: bool,
}

impl GeoJsonLoader {
    /// Creates a new `GeoJsonLoader`.
    pub fn new(options: GeoJsonLoaderOptions) -> Result<Self, DeveloperError> {
        if options.geo_json.is_null() {
            return Err(DeveloperError::new("options.geoJson is required"));
        }
        Ok(Self {
            geo_json: Some(options.geo_json),
            components: None,
            structural_metadata: None,
            is_destroyed: false,
        })
    }

    /// Loads the GeoJSON.
    pub fn load(&mut self) -> Result<(), DeveloperError> {
        Ok(())
    }

    /// Processes and parses the GeoJSON into `Components`.
    pub fn process(&mut self) -> Result<bool, DeveloperError> {
        if self.is_destroyed {
            return Ok(true);
        }

        if self.components.is_some() {
            return Ok(true);
        }

        let Some(ref geo_json) = self.geo_json else {
            return Ok(true);
        };

        let (components, metadata) = parse_geojson(geo_json)?;
        self.components = Some(components);
        self.structural_metadata = Some(metadata);
        self.geo_json = None;

        Ok(true)
    }

    /// Gets the loaded components.
    pub fn components(&self) -> Option<&Components> {
        self.components.as_ref()
    }

    /// Destroys this loader and releases components.
    pub fn destroy(&mut self) {
        self.components = None;
        self.structural_metadata = None;
        self.geo_json = None;
        self.is_destroyed = true;
    }

    /// Checks if the loader has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}

struct ParsedFeature {
    lines: Vec<Vec<Cartesian3>>,
    points: Vec<Cartesian3>,
    properties: Option<serde_json::Map<String, serde_json::Value>>,
}

fn parse_position(coords: &[serde_json::Value]) -> Cartesian3 {
    let x = coords.first().and_then(|v| v.as_f64()).unwrap_or(0.0);
    let y = coords.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let z = coords.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0);
    Cartesian3::new(x, y, z)
}

fn parse_coordinates_as_points(val: &serde_json::Value) -> Vec<Cartesian3> {
    let mut points = Vec::new();
    if let Some(arr) = val.as_array() {
        if arr.first().is_some_and(|v| v.is_array()) {
            for sub in arr {
                if let Some(sub_arr) = sub.as_array() {
                    points.push(parse_position(sub_arr));
                }
            }
        } else {
            points.push(parse_position(arr));
        }
    }
    points
}

fn parse_coordinates_as_lines(geom_type: &str, val: &serde_json::Value) -> Vec<Vec<Cartesian3>> {
    let mut lines = Vec::new();
    match geom_type {
        "LineString" => {
            if let Some(arr) = val.as_array() {
                let mut line = Vec::with_capacity(arr.len());
                for pt in arr {
                    if let Some(pt_arr) = pt.as_array() {
                        line.push(parse_position(pt_arr));
                    }
                }
                lines.push(line);
            }
        }
        "MultiLineString" | "Polygon" => {
            if let Some(arr) = val.as_array() {
                for ring in arr {
                    if let Some(ring_arr) = ring.as_array() {
                        let mut line = Vec::with_capacity(ring_arr.len());
                        for pt in ring_arr {
                            if let Some(pt_arr) = pt.as_array() {
                                line.push(parse_position(pt_arr));
                            }
                        }
                        lines.push(line);
                    }
                }
            }
        }
        "MultiPolygon" => {
            if let Some(arr) = val.as_array() {
                for poly in arr {
                    if let Some(poly_arr) = poly.as_array() {
                        for ring in poly_arr {
                            if let Some(ring_arr) = ring.as_array() {
                                let mut line = Vec::with_capacity(ring_arr.len());
                                for pt in ring_arr {
                                    if let Some(pt_arr) = pt.as_array() {
                                        line.push(parse_position(pt_arr));
                                    }
                                }
                                lines.push(line);
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    lines
}

fn parse_feature(feat: &serde_json::Value, features: &mut Vec<ParsedFeature>) {
    let Some(geom) = feat.get("geometry") else {
        return;
    };
    let geom_type = geom.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let coords = geom.get("coordinates");

    let Some(coords_val) = coords else {
        return;
    };

    let properties = feat.get("properties").and_then(|v| v.as_object()).cloned();

    let is_point_type = geom_type == "Point" || geom_type == "MultiPoint";
    if is_point_type {
        let points = parse_coordinates_as_points(coords_val);
        features.push(ParsedFeature {
            lines: Vec::new(),
            points,
            properties,
        });
    } else {
        let lines = parse_coordinates_as_lines(geom_type, coords_val);
        features.push(ParsedFeature {
            lines,
            points: Vec::new(),
            properties,
        });
    }
}

fn parse_geojson(
    json: &serde_json::Value,
) -> Result<(Components, StructuralMetadata), DeveloperError> {
    let obj_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let mut parsed_features = Vec::new();

    match obj_type {
        "FeatureCollection" => {
            if let Some(features_arr) = json.get("features").and_then(|v| v.as_array()) {
                for feat in features_arr {
                    parse_feature(feat, &mut parsed_features);
                }
            }
        }
        "Feature" => {
            parse_feature(json, &mut parsed_features);
        }
        _ => {
            return Err(DeveloperError::new(
                "Top-level GeoJSON type must be FeatureCollection or Feature",
            ));
        }
    }

    if parsed_features.is_empty() {
        return Err(DeveloperError::new("No valid features found in GeoJSON"));
    }

    // Determine whether primitive is Points or Lines
    let is_points = parsed_features.iter().any(|f| !f.points.is_empty());
    let primitive_type = if is_points {
        PrimitiveType::Points
    } else {
        PrimitiveType::Lines
    };

    // Calculate center for Local ENU frame
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_z = 0.0;
    let mut total_pts = 0;

    for f in &parsed_features {
        for pt in &f.points {
            sum_x += pt.x;
            sum_y += pt.y;
            sum_z += pt.z;
            total_pts += 1;
        }
        for line in &f.lines {
            for pt in line {
                sum_x += pt.x;
                sum_y += pt.y;
                sum_z += pt.z;
                total_pts += 1;
            }
        }
    }

    if total_pts == 0 {
        return Err(DeveloperError::new("GeoJSON contains no vertices"));
    }

    let avg_lon = sum_x / (total_pts as f64);
    let avg_lat = sum_y / (total_pts as f64);
    let avg_h = sum_z / (total_pts as f64);

    let center_cartesian = Cartesian3::from_degrees(avg_lon, avg_lat, Some(avg_h), Some(&Ellipsoid::WGS84))
        .map_err(|e| DeveloperError::new(e.to_string()))?;
    let enu_to_fixed = FixedFrameTransforms::east_north_up_to_fixed_frame(
        center_cartesian,
        Some(Ellipsoid::WGS84),
    )?;
    let fixed_to_enu = enu_to_fixed.inverse().map_err(|e| DeveloperError::new(e.to_string()))?;

    // Collect vertices and indices
    let mut vertex_positions = Vec::new();
    let mut feature_id_values = Vec::new();
    let mut indices = Vec::new();

    let mut local_min = Cartesian3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
    let mut local_max = Cartesian3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

    let mut current_vertex_idx = 0u32;

    for (feat_idx, feat) in parsed_features.iter().enumerate() {
        if is_points {
            for pt in &feat.points {
                let global = Cartesian3::from_degrees(pt.x, pt.y, Some(pt.z), Some(&Ellipsoid::WGS84))
                    .map_err(|e| DeveloperError::new(e.to_string()))?;
                let local = fixed_to_enu.multiply_by_point(global);

                local_min.x = local_min.x.min(local.x);
                local_min.y = local_min.y.min(local.y);
                local_min.z = local_min.z.min(local.z);

                local_max.x = local_max.x.max(local.x);
                local_max.y = local_max.y.max(local.y);
                local_max.z = local_max.z.max(local.z);

                vertex_positions.push(local.x as f32);
                vertex_positions.push(local.y as f32);
                vertex_positions.push(local.z as f32);

                feature_id_values.push(feat_idx as f32);
                current_vertex_idx += 1;
            }
        } else {
            for line in &feat.lines {
                let line_len = line.len();

                for (k, pt) in line.iter().enumerate() {
                    let global = Cartesian3::from_degrees(pt.x, pt.y, Some(pt.z), Some(&Ellipsoid::WGS84))
                    .map_err(|e| DeveloperError::new(e.to_string()))?;
                    let local = fixed_to_enu.multiply_by_point(global);

                    local_min.x = local_min.x.min(local.x);
                    local_min.y = local_min.y.min(local.y);
                    local_min.z = local_min.z.min(local.z);

                    local_max.x = local_max.x.max(local.x);
                    local_max.y = local_max.y.max(local.y);
                    local_max.z = local_max.z.max(local.z);

                    vertex_positions.push(local.x as f32);
                    vertex_positions.push(local.y as f32);
                    vertex_positions.push(local.z as f32);

                    feature_id_values.push(feat_idx as f32);

                    if k + 1 < line_len {
                        indices.push(current_vertex_idx as u16);
                        indices.push((current_vertex_idx + 1) as u16);
                    }

                    current_vertex_idx += 1;
                }
            }
        }
    }

    let vertex_count = vertex_positions.len() / 3;

    // Build buffers
    let pos_bytes: Vec<u8> = vertex_positions
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();
    let pos_buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 0,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: Some(pos_bytes),
    })?;

    let fid_bytes: Vec<u8> = feature_id_values
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();
    let fid_buffer = Buffer::create_vertex_buffer(BufferOptions {
        size_in_bytes: 0,
        usage: Some(BufferUsage::StaticDraw),
        typed_array: Some(fid_bytes),
    })?;

    let pos_attr = Attribute {
        name: Some("POSITION".to_string()),
        semantic: Some("POSITION".to_string()),
        set_index: None,
        component_datatype: Some(ComponentDatatype::Float),
        attribute_type: Some(AttributeType::Vec3),
        normalized: false,
        min: Some(vec![local_min.x, local_min.y, local_min.z]),
        max: Some(vec![local_max.x, local_max.y, local_max.z]),
        constant: None,
        quantization: None,
        typed_array: None,
        buffer: Some(pos_buffer),
        byte_offset: 0,
        byte_stride: Some(12),
        count: vertex_count,
    };

    let fid_attr = Attribute {
        name: Some("FEATURE_ID_0".to_string()),
        semantic: Some("_FEATURE_ID_0".to_string()),
        set_index: Some(0),
        component_datatype: Some(ComponentDatatype::Float),
        attribute_type: Some(AttributeType::Scalar),
        normalized: false,
        min: None,
        max: None,
        constant: None,
        quantization: None,
        typed_array: None,
        buffer: Some(fid_buffer),
        byte_offset: 0,
        byte_stride: Some(4),
        count: vertex_count,
    };

    let indices_component = if !indices.is_empty() {
        let idx_bytes: Vec<u8> = indices.iter().flat_map(|i| i.to_le_bytes()).collect();
        let idx_buffer = Buffer::create_index_buffer(IndexBufferOptions {
            size_in_bytes: 0,
            usage: Some(BufferUsage::StaticDraw),
            index_datatype: IndexDatatype::UnsignedShort,
            typed_array: Some(idx_bytes),
        })?;
        Some(Indices {
            count: indices.len(),
            index_datatype: Some(IndexDatatype::UnsignedShort),
            buffer: Some(idx_buffer),
            typed_array: None,
        })
    } else {
        None
    };

    let feature_count = parsed_features.len();
    let feature_id_accessor = FeatureId::Attribute(FeatureIdAttribute {
        feature_count,
        null_feature_id: None,
        property_table_id: Some(0),
        set_index: Some(0),
        label: None,
        positional_label: Some("featureId_0".to_string()),
    });

    let mat = Material {
        unlit: true,
        ..Default::default()
    };

    let primitive = Primitive {
        attributes: vec![pos_attr, fid_attr],
        morph_targets: Vec::new(),
        indices: indices_component,
        material: Some(mat),
        primitive_type: Some(primitive_type),
        feature_ids: vec![feature_id_accessor],
        outline_coordinates: None,
        property_attribute_ids: Vec::new(),
        property_texture_ids: Vec::new(),
        ..Default::default()
    };

    // Extract property table
    let mut properties: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    for f in &parsed_features {
        if let Some(ref props) = f.properties {
            for (k, v) in props {
                properties.entry(k.clone()).or_default().push(v.clone());
            }
        }
    }

    let property_table = PropertyTable {
        id: 0,
        name: Some("GeoJsonProperties".to_string()),
        class_name: "GeoJsonClass".to_string(),
        count: feature_count,
        properties,
    };

    let structural_metadata = StructuralMetadata {
        schema: MetadataSchema::default(),
        property_tables: vec![property_table],
        property_textures: Vec::new(),
    };

    let root_node = Node {
        primitives: vec![primitive],
        ..Default::default()
    };

    let scene = Scene {
        nodes: vec![0],
    };

    let components = Components {
        scene: Some(scene),
        nodes: vec![root_node],
        transform: enu_to_fixed,
        ..Default::default()
    };

    Ok((components, structural_metadata))
}
