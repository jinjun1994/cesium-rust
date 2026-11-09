use std::collections::HashMap;
use serde_json::{json, Value};

use cesium_math::{
    Cartesian3, Cartographic, DerivedResourceOptions, Ellipsoid, GeoRectangle,
    Resource, WebMercatorProjection,
};

use crate::i3s_data_provider::I3SDataProvider;
use crate::i3s_decoder::I3SDecodeResult;
use crate::i3s_field::{I3SAttributeStorageInfo, I3SField};
use crate::i3s_layer::{I3SAttributeFilter, I3SGeometryBufferInfo};

/// Geoid dataset representation for vertical datum offset sampling.
#[derive(Clone, Debug, PartialEq)]
pub struct GeoidData {
    pub buffer: Vec<f32>,
    pub height: usize,
    pub width: usize,
    pub native_extent: GeoRectangle,
    pub offset: f64,
    pub scale: f64,
    pub projection_type: String,
}

impl GeoidData {
    /// Sample the geoid height at the projected coordinate (sample_x, sample_y).
    pub fn sample_geoid(&self, sample_x: f64, sample_y: f64) -> f64 {
        let extent = &self.native_extent;
        let width_f = self.width as f64;
        let height_f = self.height as f64;

        if (extent.east - extent.west).abs() < 1e-12 || (extent.north - extent.south).abs() < 1e-12 {
            return self.offset;
        }

        let mut x = ((sample_x - extent.west) / (extent.east - extent.west)) * (width_f - 1.0);
        let mut y = ((sample_y - extent.south) / (extent.north - extent.south)) * (height_f - 1.0);

        let xi = x.floor() as usize;
        let yi = y.floor() as usize;

        x -= xi as f64;
        y -= yi as f64;

        let x_next = if xi + 1 < self.width { xi + 1 } else { xi };
        let y_next = if yi + 1 < self.height { yi + 1 } else { yi };

        let yi_inv = self.height - 1 - yi;
        let y_next_inv = self.height - 1 - y_next;

        let sample_map = |u: usize, v: usize| -> f64 {
            let addr = u + v * self.width;
            if addr < self.buffer.len() {
                self.buffer[addr] as f64
            } else {
                0.0
            }
        };

        let h00 = sample_map(xi, yi_inv);
        let h10 = sample_map(x_next, yi_inv);
        let h01 = sample_map(xi, y_next_inv);
        let h11 = sample_map(x_next, y_next_inv);

        let a = h00 * (1.0 - x) + h10 * x;
        let b = h01 * (1.0 - x) + h11 * x;
        let final_h = a * (1.0 - y) + b * y;

        final_h * self.scale + self.offset
    }
}

/// Custom attributes used for triangle picking.
#[derive(Clone, Debug, PartialEq)]
pub struct I3SCustomAttributes {
    pub cartesian_center: Cartesian3,
    pub feature_index: Option<Vec<u32>>,
    pub positions: Vec<f32>,
    pub indices: Option<Vec<u32>>,
    pub parent_rotation: [f64; 9],
}

/// Result of closest point search on geometry triangles.
#[derive(Clone, Debug, PartialEq)]
pub struct I3SClosestPointResult {
    pub index: i32,
    pub distance: f64,
    pub distance_squared: f64,
    pub queried_position: Option<Cartesian3>,
    pub closest_position: Option<Cartesian3>,
}

/// An I3S Feature associated with a node.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SFeature {
    pub resource: Option<Resource>,
    pub data: Option<Value>,
}

impl I3SFeature {
    /// Create a new I3S feature from a node and URI.
    pub fn new(
        node_resource: Option<&Resource>,
        layer_resource: Option<&Resource>,
        mesh_attribute_resource: Option<&str>,
        uri: &str,
    ) -> Self {
        let resource = if let (Some(l_res), Some(attr_res)) = (layer_resource, mesh_attribute_resource) {
            let rel_url = format!("nodes/{}/{}", attr_res, uri);
            Some(l_res.get_derived_resource(DerivedResourceOptions {
                url: Some(rel_url),
                preserve_query_parameters: true,
                ..Default::default()
            }))
        } else if let Some(n_res) = node_resource {
            Some(n_res.get_derived_resource(DerivedResourceOptions {
                url: Some(uri.to_string()),
                preserve_query_parameters: true,
                ..Default::default()
            }))
        } else {
            Some(Resource::from_url(uri))
        };

        Self {
            resource,
            data: None,
        }
    }

    /// Load feature JSON data.
    pub fn load(&mut self, data: Value) {
        self.data = Some(data);
    }
}

/// An I3S Geometry associated with a node.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SGeometry {
    pub resource: Option<Resource>,
    pub data: Option<Vec<u8>>,
    pub custom_attributes: Option<I3SCustomAttributes>,
    pub geometry_definitions: Option<Value>,
    pub geometry_buffer_info: Option<I3SGeometryBufferInfo>,
}

fn same_side(p1: Cartesian3, p2: Cartesian3, a: Cartesian3, b: Cartesian3) -> bool {
    let ab = b - a;
    let cp1 = ab.cross(&(p1 - a));
    let cp2 = ab.cross(&(p2 - a));
    cp1.dot(&cp2) >= 0.0
}

impl I3SGeometry {
    /// Create a new I3SGeometry.
    pub fn new(resource: Option<Resource>) -> Self {
        Self {
            resource,
            data: None,
            custom_attributes: None,
            geometry_definitions: None,
            geometry_buffer_info: None,
        }
    }

    /// Load raw geometry binary data.
    pub fn load(&mut self, data: Vec<u8>) {
        self.data = Some(data);
    }

    /// Find a triangle touching the point [px, py, pz], then return the vertex closest to the search point.
    pub fn get_closest_point_index_on_triangle(&self, px: f64, py: f64, pz: f64) -> I3SClosestPointResult {
        let attrs = match &self.custom_attributes {
            Some(a) if !a.positions.is_empty() => a,
            _ => {
                return I3SClosestPointResult {
                    index: -1,
                    distance: f64::INFINITY,
                    distance_squared: f64::INFINITY,
                    queried_position: None,
                    closest_position: None,
                };
            }
        };

        // Convert queried position to local
        let mut position = Cartesian3::new(
            px - attrs.cartesian_center.x,
            py - attrs.cartesian_center.y,
            pz - attrs.cartesian_center.z,
        );

        let m = &attrs.parent_rotation;
        let px_rot = m[0] * position.x + m[3] * position.y + m[6] * position.z;
        let py_rot = m[1] * position.x + m[4] * position.y + m[7] * position.z;
        let pz_rot = m[2] * position.x + m[5] * position.y + m[8] * position.z;
        position = Cartesian3::new(px_rot, py_rot, pz_rot);

        let mut best_tri_dist = f64::MAX;
        let mut best_tri: Option<usize> = None;
        let mut best_dist_sq = f64::MAX;
        let mut best_index = -1;
        let mut best_pt = Cartesian3::ZERO;

        let positions = &attrs.positions;
        let indices = attrs.indices.as_deref();

        let tri_count = if let Some(idx) = indices {
            idx.len()
        } else {
            positions.len() / 3
        };

        for tri_index in 0..tri_count {
            let (i0, i1, i2) = if let Some(idx) = indices {
                if tri_index + 2 >= idx.len() {
                    continue;
                }
                (idx[tri_index] as usize, idx[tri_index + 1] as usize, idx[tri_index + 2] as usize)
            } else {
                (tri_index * 3, tri_index * 3 + 1, tri_index * 3 + 2)
            };

            if i0 * 3 + 2 >= positions.len() || i1 * 3 + 2 >= positions.len() || i2 * 3 + 2 >= positions.len() {
                continue;
            }

            let v0 = Cartesian3::new(
                positions[i0 * 3] as f64,
                positions[i0 * 3 + 1] as f64,
                positions[i0 * 3 + 2] as f64,
            );
            let v1 = Cartesian3::new(
                positions[i1 * 3] as f64,
                positions[i1 * 3 + 1] as f64,
                positions[i1 * 3 + 2] as f64,
            );
            let v2 = Cartesian3::new(
                positions[i2 * 3] as f64,
                positions[i2 * 3 + 1] as f64,
                positions[i2 * 3 + 2] as f64,
            );

            if !same_side(position, v0, v1, v2)
                || !same_side(position, v1, v0, v2)
                || !same_side(position, v2, v0, v1)
            {
                continue;
            }

            let v0v1 = v1 - v0;
            let v0v2 = v2 - v0;
            let cross_prod = v0v1.cross(&v0v2);
            let mag = cross_prod.magnitude();
            if mag == 0.0 {
                continue;
            }
            let normal = cross_prod / mag;

            let v0p = position - v0;
            let normal_dist = v0p.dot(&normal).abs();
            if normal_dist < best_tri_dist {
                best_tri_dist = normal_dist;
                best_tri = Some(tri_index);

                let d0 = (position - v0).magnitude_squared();
                let d1 = (position - v1).magnitude_squared();
                let d2 = (position - v2).magnitude_squared();

                if d0 < d1 && d0 < d2 {
                    best_index = i0 as i32;
                    best_pt = v0;
                    best_dist_sq = d0;
                } else if d1 < d2 {
                    best_index = i1 as i32;
                    best_pt = v1;
                    best_dist_sq = d1;
                } else {
                    best_index = i2 as i32;
                    best_pt = v2;
                    best_dist_sq = d2;
                }
            }
        }

        if best_tri.is_some() {
            I3SClosestPointResult {
                index: best_index,
                distance: best_dist_sq.sqrt(),
                distance_squared: best_dist_sq,
                queried_position: Some(position),
                closest_position: Some(best_pt),
            }
        } else {
            I3SClosestPointResult {
                index: -1,
                distance: f64::INFINITY,
                distance_squared: f64::INFINITY,
                queried_position: None,
                closest_position: None,
            }
        }
    }

    /// Generate glTF 2.0 structure from decoded mesh data and layer material definitions.
    #[allow(clippy::too_many_arguments)]
    pub fn generate_gltf(
        &self,
        nodes_in_scene: &[Value],
        nodes: &[Value],
        meshes: &[Value],
        buffers: &[Value],
        buffer_views: &[Value],
        accessors: &[Value],
        layer_data: Option<&Value>,
        node_data: Option<&Value>,
        layer_url: Option<&str>,
    ) -> Value {
        let mut gltf_material = json!({
            "pbrMetallicRoughness": {
                "baseColorFactor": [1.0, 1.0, 1.0, 1.0],
                "metallicFactor": 0.0,
                "roughnessFactor": 1.0
            }
        });

        let mut is_textured = false;
        let mut texture_path = String::new();

        if let Some(ld) = layer_data {
            let material_definitions = ld.get("materialDefinitions").and_then(|v| v.as_array());
            let material_idx = node_data
                .and_then(|n| n.get("mesh"))
                .and_then(|m| m.get("material"))
                .and_then(|mat| mat.get("definition"))
                .and_then(|d| d.as_u64())
                .unwrap_or(0) as usize;

            if let Some(mat_defs) = material_definitions {
                if material_idx < mat_defs.len() {
                    gltf_material = mat_defs[material_idx].clone();

                    if gltf_material
                        .get("pbrMetallicRoughness")
                        .and_then(|pbr| pbr.get("baseColorTexture"))
                        .is_some()
                    {
                        is_textured = true;
                        if let Some(pbr) = gltf_material.get_mut("pbrMetallicRoughness") {
                            if let Some(bct) = pbr.get_mut("baseColorTexture") {
                                bct["index"] = json!(0);
                            }
                        }

                        let mut texture_name = "0".to_string();
                        if let Some(tex_defs) = ld.get("textureSetDefinitions").and_then(|v| v.as_array()) {
                            'outer: for def in tex_defs {
                                if let Some(formats) = def.get("formats").and_then(|f| f.as_array()) {
                                    for fmt in formats {
                                        if fmt.get("format").and_then(|s| s.as_str()) == Some("jpg") {
                                            if let Some(name) = fmt.get("name").and_then(|n| n.as_str()) {
                                                texture_name = name.to_string();
                                                break 'outer;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let res_id = node_data
                            .and_then(|n| n.get("mesh"))
                            .and_then(|m| m.get("material"))
                            .and_then(|mat| mat.get("resource"))
                            .and_then(|r| r.as_i64())
                            .unwrap_or(-1);

                        if res_id >= 0 {
                            let base = layer_url.unwrap_or("");
                            let clean_base = base.trim_end_matches('/');
                            texture_path = format!("{}/nodes/{}/textures/{}", clean_base, res_id, texture_name);
                        }
                    }
                }
            }
        } else if let Some(nd) = node_data {
            if let Some(tex_data) = nd.get("textureData").and_then(|t| t.as_array()) {
                if let Some(first_tex) = tex_data.first() {
                    if let Some(href) = first_tex.get("href").and_then(|h| h.as_str()) {
                        is_textured = true;
                        let base = self.resource.as_ref().map(|r| r.url()).unwrap_or_default();
                        let clean_base = base.trim_end_matches('/');
                        texture_path = format!("{}/{}", clean_base, href);
                        gltf_material["pbrMetallicRoughness"]["baseColorTexture"] = json!({ "index": 0 });
                    }
                }
            }
        }

        if let Some(alpha_mode) = gltf_material.get("alphaMode").and_then(|a| a.as_str()) {
            gltf_material["alphaMode"] = json!(alpha_mode.to_uppercase());
        }

        let gltf_textures = if is_textured {
            json!([{ "sampler": 0, "source": 0 }])
        } else {
            json!([])
        };

        let gltf_images = if is_textured {
            json!([{ "uri": texture_path }])
        } else {
            json!([])
        };

        let gltf_samplers = if is_textured {
            json!([{
                "magFilter": 9729,
                "minFilter": 9986,
                "wrapS": 10497,
                "wrapT": 10497
            }])
        } else {
            json!([])
        };

        let mut gltf_materials = Vec::new();
        for mesh in meshes {
            if let Some(primitives) = mesh.get("primitives").and_then(|p| p.as_array()) {
                for prim in primitives {
                    if let Some(mat_idx_val) = prim.get("material").and_then(|m| m.as_u64()) {
                        let mat_idx = mat_idx_val as usize;
                        while mat_idx >= gltf_materials.len() {
                            gltf_materials.push(gltf_material.clone());
                        }
                        let prim_mat = &mut gltf_materials[mat_idx];
                        let is_trans = prim
                            .get("extra")
                            .and_then(|e| e.get("isTransparent"))
                            .and_then(|t| t.as_bool())
                            .unwrap_or(false);

                        if is_trans {
                            if prim_mat.get("alphaMode").is_none() {
                                prim_mat["alphaMode"] = json!("BLEND");
                            }
                        } else if prim_mat.get("alphaMode").and_then(|a| a.as_str()) == Some("BLEND") {
                            prim_mat["alphaMode"] = json!("OPAQUE");
                        }
                    }
                }
            }
        }

        json!({
            "scene": 0,
            "scenes": [{ "nodes": nodes_in_scene }],
            "nodes": nodes,
            "meshes": meshes,
            "buffers": buffers,
            "bufferViews": buffer_views,
            "accessors": accessors,
            "materials": gltf_materials,
            "textures": gltf_textures,
            "images": gltf_images,
            "samplers": gltf_samplers,
            "asset": { "version": "2.0" }
        })
    }
}

/// Simulated Cesium3DTile batch table for feature picking & visibility filtering.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3STileBatchTable {
    pub features: Vec<bool>,
    pub features_length: usize,
}

impl I3STileBatchTable {
    pub fn new(features_length: usize) -> Self {
        Self {
            features: vec![true; features_length],
            features_length,
        }
    }

    pub fn set_all_show(&mut self, show: bool) {
        self.features.fill(show);
    }

    pub fn set_show(&mut self, index: usize, show: bool) {
        if index < self.features.len() {
            self.features[index] = show;
        }
    }
}

/// Simulated Cesium3DTile content model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3STileContent {
    pub batch_table: Option<I3STileBatchTable>,
    pub model_ready: bool,
    pub model_show: bool,
}

/// Simulated Cesium3DTile representation for an I3SNode.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3STile {
    pub is_loading: bool,
    pub content: Option<I3STileContent>,
    pub children: Vec<I3STile>,
    pub content_resource: Option<Resource>,
    pub bounding_volume: Option<Value>,
    pub transform: Option<[f64; 16]>,
    pub geometric_error: f64,
    pub refine: String,
}

/// Node reference type: either a numeric node page index or a relative URL string.
#[derive(Clone, Debug, PartialEq)]
pub enum I3SNodeRef {
    Index(usize),
    Uri(String),
}

/// I3S Node implementation. In Cesium each I3SNode creates a Cesium3DTile.
#[derive(Clone, Debug, Default)]
pub struct I3SNode {
    pub is_root: bool,
    pub level: u32,
    pub node_index: Option<usize>,
    pub resource: Option<Resource>,
    pub data: Option<Value>,
    pub geometry_data: Vec<I3SGeometry>,
    pub feature_data: Vec<I3SFeature>,
    pub fields: HashMap<String, I3SField>,
    pub children: Vec<I3SNode>,
    pub global_transform: Option<[f64; 16]>,
    pub inverse_global_transform: Option<[f64; 16]>,
    pub inverse_rotation_matrix: Option<[f64; 9]>,
    pub tile: Option<I3STile>,
    pub symbology_data: Option<Value>,
}

impl I3SNode {
    /// Construct a root node from URL.
    pub fn new_root_from_url(layer_resource: Option<&Resource>, ref_url: &str) -> Self {
        let resource = layer_resource.map(|r| {
            let derived_rel = format!("{}/", ref_url.trim_matches('/'));
            r.get_derived_resource(DerivedResourceOptions {
                url: Some(derived_rel),
                preserve_query_parameters: true,
                ..Default::default()
            })
        });

        Self {
            is_root: true,
            level: 0,
            node_index: None,
            resource,
            ..Default::default()
        }
    }

    /// Construct a root node from node page index.
    pub fn new_root_from_index(index: usize) -> Self {
        Self {
            is_root: true,
            level: 0,
            node_index: Some(index),
            resource: None,
            ..Default::default()
        }
    }

    /// Construct a child node from parent node and reference.
    pub fn new_child(parent: &I3SNode, ref_val: I3SNodeRef) -> Self {
        let level = parent.level + 1;
        match ref_val {
            I3SNodeRef::Index(idx) => Self {
                is_root: false,
                level,
                node_index: Some(idx),
                resource: None,
                ..Default::default()
            },
            I3SNodeRef::Uri(uri) => {
                let resource = parent.resource.as_ref().map(|r| {
                    let derived_rel = format!("{}/", uri.trim_matches('/'));
                    r.get_derived_resource(DerivedResourceOptions {
                        url: Some(derived_rel),
                        preserve_query_parameters: true,
                        ..Default::default()
                    })
                });
                Self {
                    is_root: false,
                    level,
                    node_index: None,
                    resource,
                    ..Default::default()
                }
            }
        }
    }

    /// Load node data and compute bounding volumes / transformations.
    pub fn load_with_data(
        &mut self,
        data: Value,
        parent_global_transform: Option<[f64; 16]>,
        geoid_data_list: Option<&[GeoidData]>,
        lod_selection_metric_type: Option<&str>,
        layer_resource: Option<&Resource>,
    ) {
        // If node was created from index and has mesh, update resource
        if self.resource.is_none() {
            if self.is_root {
                if let Some(lr) = layer_resource {
                    self.resource = Some(lr.get_derived_resource(DerivedResourceOptions {
                        url: Some("nodes/root/".to_string()),
                        preserve_query_parameters: true,
                        ..Default::default()
                    }));
                }
            } else if let Some(mesh) = data.get("mesh") {
                if let Some(uri_index) = mesh.get("geometry").and_then(|g| g.get("resource")).and_then(|r| r.as_u64()) {
                    if let Some(lr) = layer_resource {
                        self.resource = Some(lr.get_derived_resource(DerivedResourceOptions {
                            url: Some(format!("nodes/{}/", uri_index)),
                            preserve_query_parameters: true,
                            ..Default::default()
                        }));
                    }
                }
            }
        }

        self.data = Some(data);

        // Compute 3D Tile definition and transforms
        if let Some(tile_def) = self.compute_tile_definition(
            geoid_data_list,
            lod_selection_metric_type,
            parent_global_transform,
        ) {
            let mut tile = I3STile {
                refine: tile_def.get("refine").and_then(|r| r.as_str()).unwrap_or("REPLACE").to_string(),
                bounding_volume: tile_def.get("boundingVolume").cloned(),
                geometric_error: tile_def.get("geometricError").and_then(|g| g.as_f64()).unwrap_or(0.0),
                content_resource: self.resource.clone(),
                ..Default::default()
            };
            if let Some(tf) = tile_def.get("transform").and_then(|t| t.as_array()) {
                if tf.len() == 16 {
                    let mut arr = [0.0; 16];
                    for (i, val) in tf.iter().enumerate() {
                        arr[i] = val.as_f64().unwrap_or(0.0);
                    }
                    tile.transform = Some(arr);
                }
            }
            self.tile = Some(tile);
        }
    }

    /// Compute 3D Tile definition JSON.
    fn compute_tile_definition(
        &mut self,
        geoid_data_list: Option<&[GeoidData]>,
        lod_selection_metric_type: Option<&str>,
        parent_global_transform: Option<[f64; 16]>,
    ) -> Option<Value> {
        let data = self.data.as_ref()?;
        let obb = data.get("obb");
        let mbs = data.get("mbs").and_then(|m| m.as_array());

        if obb.is_none() && mbs.is_none() {
            return None;
        }

        let mut geo_pos = if let Some(o) = obb {
            let center = o.get("center").and_then(|c| c.as_array())?;
            let lon = center.first()?.as_f64()?;
            let lat = center.get(1)?.as_f64()?;
            let height = center.get(2)?.as_f64()?;
            Cartographic::degrees(lon, lat, height).ok()?
        } else {
            let m = mbs?;
            let lon = m.first()?.as_f64()?;
            let lat = m.get(1)?.as_f64()?;
            let height = m.get(2)?.as_f64()?;
            Cartographic::degrees(lon, lat, height).ok()?
        };

        // Offset height if geoid service data is defined
        if let Some(geoid_list) = geoid_data_list {
            let mercator = WebMercatorProjection::default();
            if let Some(proj) = mercator.project(geo_pos) {
                for tile in geoid_list {
                    if proj.x > tile.native_extent.west
                        && proj.x < tile.native_extent.east
                        && proj.y > tile.native_extent.south
                        && proj.y < tile.native_extent.north
                    {
                        geo_pos.height += tile.sample_geoid(proj.x, proj.y);
                        break;
                    }
                }
            }
        }

        let ellipsoid = Ellipsoid::WGS84;
        let pos_dvec = ellipsoid.cartographic_to_cartesian(geo_pos)?;
        let position = Cartesian3::from(pos_dvec);

        let (bounding_volume, span) = if let Some(o) = obb {
            let hs = o.get("halfSize").and_then(|h| h.as_array())?;
            let h0 = hs.first()?.as_f64()?;
            let h1 = hs.get(1)?.as_f64()?;
            let h2 = hs.get(2)?.as_f64()?;
            let span = h0.max(h1).max(h2) * 2.0;
            (
                json!({
                    "box": [0.0, 0.0, 0.0, h0, 0.0, 0.0, 0.0, h1, 0.0, 0.0, 0.0, h2]
                }),
                span,
            )
        } else {
            let m = mbs?;
            let radius = m.get(3)?.as_f64()?;
            (json!({ "sphere": [0.0, 0.0, 0.0, radius] }), radius * 2.0)
        };

        let mut meters_per_pixel = f64::INFINITY;
        if let Some(lod_threshold) = data.get("lodThreshold").and_then(|t| t.as_f64()) {
            if lod_selection_metric_type == Some("maxScreenThresholdSQ") {
                let max_st = (lod_threshold / (std::f64::consts::PI * 0.25)).sqrt();
                meters_per_pixel = span / max_st;
            } else if lod_selection_metric_type == Some("maxScreenThreshold") {
                meters_per_pixel = span / lod_threshold;
            }
        } else if let Some(lod_selection) = data.get("lodSelection").and_then(|l| l.as_array()) {
            for lod in lod_selection {
                if lod.get("metricType").and_then(|m| m.as_str()) == Some("maxScreenThreshold") {
                    if let Some(max_error) = lod.get("maxError").and_then(|e| e.as_f64()) {
                        meters_per_pixel = span / max_error;
                        break;
                    }
                }
            }
        }

        if meters_per_pixel == f64::INFINITY {
            meters_per_pixel = 100000.0;
        }
        let geometric_error = meters_per_pixel * 16.0;

        let global_transform = [
            1.0, 0.0, 0.0, position.x,
            0.0, 1.0, 0.0, position.y,
            0.0, 0.0, 1.0, position.z,
            0.0, 0.0, 0.0, 1.0,
        ];

        let mut inv_gt = global_transform;
        inv_gt[3] = -position.x;
        inv_gt[7] = -position.y;
        inv_gt[11] = -position.z;

        self.global_transform = Some(global_transform);
        self.inverse_global_transform = Some(inv_gt);
        self.inverse_rotation_matrix = Some([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);

        let local_transform = if let Some(parent_gt) = parent_global_transform {
            // Relative translation
            let mut lt = global_transform;
            lt[3] = global_transform[3] - parent_gt[3];
            lt[7] = global_transform[7] - parent_gt[7];
            lt[11] = global_transform[11] - parent_gt[11];
            lt
        } else {
            global_transform
        };

        // Convert row-based representation to column-major transform array
        let transform_col_major = [
            local_transform[0], local_transform[4], local_transform[8], local_transform[12],
            local_transform[1], local_transform[5], local_transform[9], local_transform[13],
            local_transform[2], local_transform[6], local_transform[10], local_transform[14],
            local_transform[3], local_transform[7], local_transform[11], local_transform[15],
        ];

        Some(json!({
            "children": [],
            "refine": "REPLACE",
            "boundingVolume": bounding_volume,
            "transform": transform_col_major,
            "content": {
                "uri": self.resource.as_ref().map(|r| r.url())
            },
            "geometricError": geometric_error
        }))
    }

    /// Load child nodes from child descriptions.
    pub fn load_children(&mut self, child_entries: Vec<(I3SNodeRef, Value)>) {
        self.children.clear();
        for (ref_val, child_data) in child_entries {
            let mut child = I3SNode::new_child(self, ref_val);
            child.load_with_data(child_data, self.global_transform, None, None, self.resource.as_ref());
            self.children.push(child);
        }
    }

    /// Clear all geometry data.
    pub fn clear_geometry_data(&mut self) {
        self.geometry_data.clear();
    }

    /// Load geometry data.
    pub fn load_geometry_data(
        &mut self,
        layer_resource: Option<&Resource>,
        binary_data: Option<Vec<u8>>,
    ) {
        self.geometry_data.clear();
        let data = match &self.data {
            Some(d) => d,
            None => return,
        };

        if let Some(geom_data) = data.get("geometryData").and_then(|g| g.as_array()) {
            for item in geom_data {
                if let Some(href) = item.get("href").and_then(|h| h.as_str()) {
                    let resource = self.resource.as_ref().map(|r| {
                        r.get_derived_resource(DerivedResourceOptions {
                            url: Some(href.to_string()),
                            preserve_query_parameters: true,
                            ..Default::default()
                        })
                    });
                    let mut geom = I3SGeometry::new(resource);
                    if let Some(bytes) = &binary_data {
                        geom.load(bytes.clone());
                    }
                    self.geometry_data.push(geom);
                }
            }
        } else if let Some(_mesh) = data.get("mesh") {
            let res_idx = self.node_index.unwrap_or(1);
            let geometry_uri = format!("nodes/{}/geometries/1/", res_idx);
            let resource = layer_resource.map(|r| {
                r.get_derived_resource(DerivedResourceOptions {
                    url: Some(geometry_uri),
                    preserve_query_parameters: true,
                    ..Default::default()
                })
            });
            let mut geom = I3SGeometry::new(resource);
            geom.geometry_buffer_info = Some(I3SGeometryBufferInfo {
                compressed: false,
                index: 1,
                attributes: vec!["position".to_string(), "uv0".to_string()],
            });
            geom.geometry_definitions = Some(json!({ "geometryBuffers": [] }));
            if let Some(bytes) = binary_data {
                geom.load(bytes);
            }
            self.geometry_data.push(geom);
        }
    }

    /// Load feature data.
    pub fn load_feature_data(&mut self, feature_json: Option<Value>) {
        self.feature_data.clear();
        let data = match &self.data {
            Some(d) => d,
            None => return,
        };

        if let Some(feats) = data.get("featureData").and_then(|f| f.as_array()) {
            for item in feats {
                if let Some(href) = item.get("href").and_then(|h| h.as_str()) {
                    let mut feat = I3SFeature::new(self.resource.as_ref(), None, None, href);
                    if let Some(fj) = &feature_json {
                        feat.load(fj.clone());
                    }
                    self.feature_data.push(feat);
                }
            }
        }
    }

    /// Load all attribute fields defined in the layer.
    pub fn load_fields_from_storage(
        &mut self,
        layer_storage_info: &[I3SAttributeStorageInfo],
        field_buffers: &HashMap<String, Vec<u8>>,
        layer_resource: Option<&Resource>,
    ) {
        for storage in layer_storage_info {
            let mut field = I3SField::new(storage.clone());
            let node_idx = self.node_index.unwrap_or(1);
            let field_key = &storage.key;
            if let Some(lr) = layer_resource {
                let rel_url = format!("nodes/{}/attributes/{}/0", node_idx, field_key);
                let field_res = lr.get_derived_resource(DerivedResourceOptions {
                    url: Some(rel_url),
                    preserve_query_parameters: true,
                    ..Default::default()
                });
                field.resource = Some(field_res);
            }
            if let Some(buf) = field_buffers.get(&storage.name) {
                let _ = field.load_binary(buf.clone());
            }
            self.fields.insert(storage.name.clone(), field);
        }
    }

    /// Load a specific single field.
    pub fn load_field(
        &mut self,
        name: &str,
        layer_storage_info: &[I3SAttributeStorageInfo],
        buf: Option<Vec<u8>>,
        layer_resource: Option<&Resource>,
    ) {
        if self.fields.contains_key(name) {
            return;
        }

        for storage in layer_storage_info {
            if storage.name == name {
                let mut field = I3SField::new(storage.clone());
                let node_idx = self.node_index.unwrap_or(1);
                let field_key = &storage.key;
                if let Some(lr) = layer_resource {
                    let rel_url = format!("nodes/{}/attributes/{}/0", node_idx, field_key);
                    let field_res = lr.get_derived_resource(DerivedResourceOptions {
                        url: Some(rel_url),
                        preserve_query_parameters: true,
                        ..Default::default()
                    });
                    field.resource = Some(field_res);
                }
                if let Some(bytes) = buf {
                    let _ = field.load_binary(bytes);
                }
                self.fields.insert(name.to_string(), field);
                break;
            }
        }
    }

    /// Retrieve fields map for a given feature index.
    pub fn get_fields_for_feature(&self, feature_index: usize) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        for (field_name, field) in &self.fields {
            if let Some(val) = field.values.get(feature_index) {
                map.insert(field_name.clone(), val.clone());
            }
        }
        map
    }

    /// Retrieve fields for a given picked 3D position.
    pub fn get_fields_for_picked_position(&self, picked_position: Cartesian3) -> HashMap<String, Value> {
        let geom = match self.geometry_data.first() {
            Some(g) => g,
            None => return HashMap::new(),
        };

        let attrs = match &geom.custom_attributes {
            Some(a) => a,
            None => return HashMap::new(),
        };

        let feature_indices = match &attrs.feature_index {
            Some(fi) => fi,
            None => return HashMap::new(),
        };

        let result = geom.get_closest_point_index_on_triangle(
            picked_position.x,
            picked_position.y,
            picked_position.z,
        );

        if result.index < 0 || (result.index as usize) >= feature_indices.len() {
            return HashMap::new();
        }

        let feat_idx = feature_indices[result.index as usize] as usize;
        self.get_fields_for_feature(feat_idx)
    }

    /// Filter features based on attribute values.
    pub fn filter_features(&mut self, filters: &[I3SAttributeFilter]) {
        for child in &mut self.children {
            child.filter_features(filters);
        }

        if let Some(tile) = &mut self.tile {
            if let Some(content) = &mut tile.content {
                if let Some(batch_table) = &mut content.batch_table {
                    batch_table.set_all_show(true);

                    if !filters.is_empty() {
                        for feature_idx in 0..batch_table.features_length {
                            for filter in filters {
                                let field = self.fields.get(&filter.name);
                                let matches = if let Some(f) = field {
                                    if let Some(val) = f.values.get(feature_idx) {
                                        filter.values.contains(val)
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                };

                                if !matches {
                                    batch_table.set_show(feature_idx, false);
                                    break;
                                }
                            }
                        }
                    }
                }
                content.model_show = true;
            }
        }
    }

    /// Create content GLB binary from decoded mesh data.
    pub fn create_content_glb(
        &self,
        decoded_result: &I3SDecodeResult,
        layer_data: Option<&Value>,
        layer_url: Option<&str>,
    ) -> Vec<u8> {
        let geom = match self.geometry_data.first() {
            Some(g) => g,
            None => return Vec::new(),
        };

        let raw_gltf = geom.generate_gltf(
            &decoded_result.mesh_data.nodes_in_scene.iter().map(|n| json!(n)).collect::<Vec<_>>(),
            &decoded_result.mesh_data.nodes,
            &decoded_result.mesh_data.meshes,
            &decoded_result.mesh_data.buffers,
            &decoded_result.mesh_data.buffer_views,
            &decoded_result.mesh_data.accessors,
            layer_data,
            self.data.as_ref(),
            layer_url,
        );

        I3SDataProvider::binarize_gltf(&raw_gltf)
    }
}
