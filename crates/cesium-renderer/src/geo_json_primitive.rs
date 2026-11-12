//! GeoJsonPrimitive matching CesiumJS `Scene/GeoJsonPrimitive.js`.

use cesium_math::{Cartesian3, Cartographic, DeveloperError, Ellipsoid, RuntimeError};
use serde_json::Value;

use crate::create_vector_tile_buffers::{
    BufferPoint, BufferPointCollection, BufferPointOptions, BufferPolygon, BufferPolygonCollection,
    BufferPolygonOptions, BufferPolyline, BufferPolylineCollection, BufferPolylineOptions,
};

/// Builds `BufferPointCollection`, `BufferPolylineCollection`, and `BufferPolygonCollection`
/// directly from GeoJSON features or geometries.
#[derive(Debug, Clone)]
pub struct GeoJsonPrimitive {
    ids: Vec<Option<Value>>,
    properties: Vec<Value>,
    pub points: Option<BufferPointCollection>,
    pub polylines: Option<BufferPolylineCollection>,
    pub polygons: Option<BufferPolygonCollection>,
}

impl GeoJsonPrimitive {
    /// Rejects without a URL (`DeveloperError`); reserved for async URL loading.
    pub fn from_url(url: Option<&str>) -> Result<Self, DeveloperError> {
        let u = url.ok_or_else(|| DeveloperError::new("url is required."))?;
        if u.is_empty() {
            return Err(DeveloperError::new("url is required."));
        }
        Err(DeveloperError::new(
            "GeoJsonPrimitive::from_url requires an async HTTP client; use from_geo_json with fetched JSON.",
        ))
    }

    /// Parses a GeoJSON `FeatureCollection`, `Feature`, or top-level `Geometry`.
    pub fn from_geo_json(geo_json: &Value) -> Result<Self, RuntimeError> {
        let obj = geo_json
            .as_object()
            .ok_or_else(|| RuntimeError::new("GeoJSON root must be an object"))?;
        let geo_type = obj
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| RuntimeError::new("GeoJSON object must have a 'type' field"))?;

        let mut builder = GeoJsonBuilder::default();

        match geo_type {
            "FeatureCollection" => {
                let features = obj
                    .get("features")
                    .and_then(Value::as_array)
                    .ok_or_else(|| {
                        RuntimeError::new("FeatureCollection must have a 'features' array")
                    })?;
                for feature in features {
                    builder.process_feature(feature)?;
                }
            }
            "Feature" => {
                builder.process_feature(geo_json)?;
            }
            "Point" | "MultiPoint" | "LineString" | "MultiLineString" | "Polygon"
            | "MultiPolygon" | "GeometryCollection" => {
                let feature_id = builder.ids.len() as u32;
                builder.ids.push(None);
                builder.properties.push(Value::Null);
                builder.process_geometry(geo_json, feature_id)?;
            }
            other => {
                return Err(RuntimeError::new(format!(
                    "Unsupported GeoJSON type: {other}"
                )));
            }
        }

        Ok(Self {
            ids: builder.ids,
            properties: builder.properties,
            points: builder.points,
            polylines: builder.polylines,
            polygons: builder.polygons,
        })
    }

    pub fn feature_count(&self) -> usize {
        self.ids.len()
    }

    pub fn ids(&self) -> &[Option<Value>] {
        &self.ids
    }

    pub fn get_id(&self, index: usize) -> Option<&Value> {
        self.ids.get(index).and_then(|v| v.as_ref())
    }

    pub fn get_properties(&self, index: usize) -> Option<&Value> {
        self.properties.get(index)
    }
}

#[derive(Default)]
struct GeoJsonBuilder {
    ids: Vec<Option<Value>>,
    properties: Vec<Value>,
    points: Option<BufferPointCollection>,
    polylines: Option<BufferPolylineCollection>,
    polygons: Option<BufferPolygonCollection>,
}

impl GeoJsonBuilder {
    fn process_feature(&mut self, feature: &Value) -> Result<(), RuntimeError> {
        let obj = feature
            .as_object()
            .ok_or_else(|| RuntimeError::new("Feature must be an object"))?;
        let feature_id = self.ids.len() as u32;
        self.ids.push(obj.get("id").cloned());
        self.properties
            .push(obj.get("properties").cloned().unwrap_or(Value::Null));

        if let Some(geom) = obj.get("geometry") {
            if !geom.is_null() {
                self.process_geometry(geom, feature_id)?;
            }
        }
        Ok(())
    }

    fn process_geometry(&mut self, geom: &Value, feature_id: u32) -> Result<(), RuntimeError> {
        let obj = geom
            .as_object()
            .ok_or_else(|| RuntimeError::new("Geometry must be an object"))?;
        let geom_type = obj
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| RuntimeError::new("Geometry must have a 'type' field"))?;

        match geom_type {
            "Point" => {
                if let Some(coords) = obj.get("coordinates").and_then(Value::as_array) {
                    self.add_point(coords, feature_id)?;
                }
            }
            "MultiPoint" => {
                if let Some(arr) = obj.get("coordinates").and_then(Value::as_array) {
                    for pt in arr {
                        if let Some(coords) = pt.as_array() {
                            self.add_point(coords, feature_id)?;
                        }
                    }
                }
            }
            "LineString" => {
                if let Some(arr) = obj.get("coordinates").and_then(Value::as_array) {
                    self.add_polyline(arr, feature_id)?;
                }
            }
            "MultiLineString" => {
                if let Some(lines) = obj.get("coordinates").and_then(Value::as_array) {
                    for line in lines {
                        if let Some(arr) = line.as_array() {
                            self.add_polyline(arr, feature_id)?;
                        }
                    }
                }
            }
            "Polygon" => {
                if let Some(rings) = obj.get("coordinates").and_then(Value::as_array) {
                    self.add_polygon(rings, feature_id)?;
                }
            }
            "MultiPolygon" => {
                if let Some(polys) = obj.get("coordinates").and_then(Value::as_array) {
                    for poly in polys {
                        if let Some(rings) = poly.as_array() {
                            self.add_polygon(rings, feature_id)?;
                        }
                    }
                }
            }
            "GeometryCollection" => {
                if let Some(geoms) = obj.get("geometries").and_then(Value::as_array) {
                    for g in geoms {
                        self.process_geometry(g, feature_id)?;
                    }
                }
            }
            other => {
                return Err(RuntimeError::new(format!(
                    "Unsupported GeoJSON geometry type: {other}"
                )));
            }
        }
        Ok(())
    }

    fn add_point(&mut self, coords: &[Value], feature_id: u32) -> Result<(), RuntimeError> {
        let pos = coord_to_cartesian(coords)?;
        let col = self
            .points
            .get_or_insert_with(BufferPointCollection::default);
        let mut scratch = BufferPoint::default();
        col.add(
            BufferPointOptions {
                position: Some(pos),
                feature_id: Some(feature_id),
                show: Some(true),
                material: None,
            },
            &mut scratch,
        );
        Ok(())
    }

    fn add_polyline(&mut self, coords: &[Value], feature_id: u32) -> Result<(), RuntimeError> {
        let mut positions = Vec::with_capacity(coords.len() * 3);
        for c in coords {
            let arr = c
                .as_array()
                .ok_or_else(|| RuntimeError::new("Polyline coordinate must be an array"))?;
            let p = coord_to_cartesian(arr)?;
            positions.push(p.x as f32);
            positions.push(p.y as f32);
            positions.push(p.z as f32);
        }
        let col = self
            .polylines
            .get_or_insert_with(BufferPolylineCollection::default);
        let mut scratch = BufferPolyline::default();
        col.add(
            BufferPolylineOptions {
                positions,
                feature_id: Some(feature_id),
                show: Some(true),
                material: None,
            },
            &mut scratch,
        );
        Ok(())
    }

    fn add_polygon(&mut self, rings: &[Value], feature_id: u32) -> Result<(), RuntimeError> {
        if rings.is_empty() {
            return Ok(());
        }
        let mut positions = Vec::new();
        let mut holes = Vec::new();
        let mut outer_vertex_count = 0usize;

        for (ring_idx, ring_val) in rings.iter().enumerate() {
            let ring = ring_val
                .as_array()
                .ok_or_else(|| RuntimeError::new("Polygon ring must be an array"))?;
            if ring.is_empty() {
                continue;
            }
            // Drop duplicate closing vertex if present
            let mut ring_len = ring.len();
            if ring_len > 1 && ring[0] == ring[ring_len - 1] {
                ring_len -= 1;
            }
            if ring_idx > 0 {
                holes.push((positions.len() / 3) as u32);
            } else {
                outer_vertex_count = ring_len;
            }
            for pt in &ring[..ring_len] {
                let arr = pt
                    .as_array()
                    .ok_or_else(|| RuntimeError::new("Polygon vertex must be an array"))?;
                let p = coord_to_cartesian(arr)?;
                positions.push(p.x as f32);
                positions.push(p.y as f32);
                positions.push(p.z as f32);
            }
        }

        let mut triangles = Vec::new();
        if outer_vertex_count >= 3 {
            for i in 1..(outer_vertex_count - 1) {
                triangles.push(0);
                triangles.push(i as u32);
                triangles.push((i + 1) as u32);
            }
        }

        let col = self
            .polygons
            .get_or_insert_with(BufferPolygonCollection::default);
        let mut scratch = BufferPolygon::default();
        col.add(
            BufferPolygonOptions {
                positions,
                triangles,
                holes,
                feature_id: Some(feature_id),
                show: Some(true),
                material: None,
            },
            &mut scratch,
        );
        Ok(())
    }
}

fn coord_to_cartesian(coords: &[Value]) -> Result<Cartesian3, RuntimeError> {
    if coords.len() < 2 {
        return Err(RuntimeError::new(
            "GeoJSON position must have at least 2 coordinates",
        ));
    }
    let lon_deg = coords[0].as_f64().unwrap_or(0.0);
    let lat_deg = coords[1].as_f64().unwrap_or(0.0);
    let height = coords.get(2).and_then(Value::as_f64).unwrap_or(0.0);
    let pos = Ellipsoid::WGS84
        .cartographic_to_cartesian(Cartographic::new(
            lon_deg.to_radians(),
            lat_deg.to_radians(),
            height,
        ))
        .ok_or_else(|| RuntimeError::new("Invalid cartographic coordinate in GeoJSON"))?;
    Ok(Cartesian3::new(pos.x, pos.y, pos.z))
}
