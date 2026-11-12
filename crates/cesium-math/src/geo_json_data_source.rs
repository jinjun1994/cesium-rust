//! GeoJsonDataSource matching CesiumJS `DataSources/GeoJsonDataSource.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use serde_json::Value;

use crate::{
    create_guid, BillboardGraphics, BillboardGraphicsOptions, Cartesian3, Color,
    ColorMaterialProperty, ConstantPositionProperty, ConstantProperty, DataSourceClock,
    DeveloperError, Entity, EntityCluster, EntityCollection, Event, HeightReference,
    MaterialProperty, PolygonGraphics, PolygonGraphicsOptions, PolygonHierarchy,
    PolylineGraphics, PolylineGraphicsOptions,
};

/// Coordinate transform callback type for custom CRS handlers.
pub type CrsTransformFn = Rc<dyn Fn(&[f64]) -> Result<Cartesian3, DeveloperError>>;

/// Describe callback type for custom feature description generation.
pub type DescribeFeatureFn =
    Rc<dyn Fn(&serde_json::Map<String, Value>, &Entity) -> Option<String>>;

/// CRS link handler callback type.
pub type CrsLinkHandlerFn = Rc<dyn Fn(&str) -> Result<CrsTransformFn, DeveloperError>>;

/// Global/default styling options for GeoJsonDataSource.
#[derive(Clone, Debug)]
pub struct GeoJsonDefaults {
    pub marker_size: f64,
    pub marker_symbol: Option<String>,
    pub marker_color: Color,
    pub stroke: Color,
    pub stroke_width: f64,
    pub fill: Color,
    pub clamp_to_ground: bool,
}

impl Default for GeoJsonDefaults {
    fn default() -> Self {
        Self {
            marker_size: 48.0,
            marker_symbol: None,
            marker_color: Color::from_css_color_string("#4169e1").unwrap_or(Color::BLUE),
            stroke: Color::BLACK,
            stroke_width: 2.0,
            fill: Color::new(1.0, 1.0, 0.0, 100.0 / 255.0),
            clamp_to_ground: false,
        }
    }
}

thread_local! {
    static GEOJSON_DEFAULTS: RefCell<GeoJsonDefaults> = RefCell::new(GeoJsonDefaults::default());
}

/// Options for loading GeoJSON or TopoJSON into a GeoJsonDataSource.
#[derive(Clone, Default)]
pub struct GeoJsonLoadOptions {
    pub source_uri: Option<String>,
    pub describe: Option<DescribeFeatureFn>,
    pub marker_size: Option<f64>,
    pub marker_symbol: Option<String>,
    pub marker_color: Option<Color>,
    pub stroke: Option<Color>,
    pub stroke_width: Option<f64>,
    pub fill: Option<Color>,
    pub clamp_to_ground: Option<bool>,
    pub credit: Option<String>,
    pub crs_link_handler: Option<CrsLinkHandlerFn>,
}

/// A DataSource which processes both GeoJSON and TopoJSON data.
pub struct GeoJsonDataSource {
    id: String,
    name: String,
    clock: Option<DataSourceClock>,
    entities: EntityCollection,
    is_loading: bool,
    clustering: EntityCluster,
    credit: Option<String>,
    pub changed_event: Event<()>,
    pub error_event: Event<String>,
    pub loading_event: Event<bool>,
}

impl Default for GeoJsonDataSource {
    fn default() -> Self {
        Self::new(None)
    }
}

impl GeoJsonDataSource {
    /// Creates a new GeoJsonDataSource with an optional name.
    pub fn new(name: Option<&str>) -> Self {
        Self {
            id: create_guid(),
            name: name.unwrap_or("").to_string(),
            clock: None,
            entities: EntityCollection::new(),
            is_loading: false,
            clustering: EntityCluster::default(),
            credit: None,
            changed_event: Event::new(),
            error_event: Event::new(),
            loading_event: Event::new(),
        }
    }

    pub fn default_marker_size() -> f64 {
        GEOJSON_DEFAULTS.with(|d| d.borrow().marker_size)
    }
    pub fn set_default_marker_size(val: f64) {
        GEOJSON_DEFAULTS.with(|d| d.borrow_mut().marker_size = val);
    }
    pub fn default_marker_symbol() -> Option<String> {
        GEOJSON_DEFAULTS.with(|d| d.borrow().marker_symbol.clone())
    }
    pub fn set_default_marker_symbol(val: Option<String>) {
        GEOJSON_DEFAULTS.with(|d| d.borrow_mut().marker_symbol = val);
    }
    pub fn default_marker_color() -> Color {
        GEOJSON_DEFAULTS.with(|d| d.borrow().marker_color)
    }
    pub fn set_default_marker_color(val: Color) {
        GEOJSON_DEFAULTS.with(|d| d.borrow_mut().marker_color = val);
    }
    pub fn default_stroke() -> Color {
        GEOJSON_DEFAULTS.with(|d| d.borrow().stroke)
    }
    pub fn set_default_stroke(val: Color) {
        GEOJSON_DEFAULTS.with(|d| d.borrow_mut().stroke = val);
    }
    pub fn default_stroke_width() -> f64 {
        GEOJSON_DEFAULTS.with(|d| d.borrow().stroke_width)
    }
    pub fn set_default_stroke_width(val: f64) {
        GEOJSON_DEFAULTS.with(|d| d.borrow_mut().stroke_width = val);
    }
    pub fn default_fill() -> Color {
        GEOJSON_DEFAULTS.with(|d| d.borrow().fill)
    }
    pub fn set_default_fill(val: Color) {
        GEOJSON_DEFAULTS.with(|d| d.borrow_mut().fill = val);
    }
    pub fn default_clamp_to_ground() -> bool {
        GEOJSON_DEFAULTS.with(|d| d.borrow().clamp_to_ground)
    }
    pub fn set_default_clamp_to_ground(val: bool) {
        GEOJSON_DEFAULTS.with(|d| d.borrow_mut().clamp_to_ground = val);
    }
    pub fn reset_defaults() {
        GEOJSON_DEFAULTS.with(|d| *d.borrow_mut() = GeoJsonDefaults::default());
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        let s = name.into();
        if self.name != s {
            self.name = s;
            self.changed_event.raise_event(&());
        }
    }

    pub fn clock(&self) -> Option<&DataSourceClock> {
        self.clock.as_ref()
    }

    pub fn set_clock(&mut self, clock: Option<DataSourceClock>) {
        if self.clock != clock {
            self.clock = clock;
            self.changed_event.raise_event(&());
        }
    }

    pub fn entities(&self) -> &EntityCollection {
        &self.entities
    }

    pub fn entities_mut(&mut self) -> &mut EntityCollection {
        &mut self.entities
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    pub fn show(&self) -> bool {
        self.entities.show()
    }

    pub fn set_show(&mut self, show: bool) {
        self.entities.set_show(show);
    }

    pub fn credit(&self) -> Option<&str> {
        self.credit.as_deref()
    }

    pub fn clustering(&self) -> &EntityCluster {
        &self.clustering
    }

    pub fn clustering_mut(&mut self) -> &mut EntityCluster {
        &mut self.clustering
    }

    /// Loads GeoJSON/TopoJSON from a JSON string, replacing existing entities.
    pub fn load_str(
        &mut self,
        json_str: &str,
        options: Option<GeoJsonLoadOptions>,
    ) -> Result<(), DeveloperError> {
        self.set_loading(true);
        let parsed: Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("Failed to parse GeoJSON: {}", e);
                self.error_event.raise_event(&msg);
                self.set_loading(false);
                return Err(DeveloperError::new(msg));
            }
        };

        self.load_value(&parsed, options)
    }

    /// Loads GeoJSON/TopoJSON from a serde_json Value, replacing existing entities.
    pub fn load_value(
        &mut self,
        data: &Value,
        options: Option<GeoJsonLoadOptions>,
    ) -> Result<(), DeveloperError> {
        self.set_loading(true);
        self.entities.remove_all();
        let res = self.process_value_internal(data, options);
        if let Err(ref e) = res {
            self.error_event.raise_event(&e.message);
        }
        self.set_loading(false);
        res
    }

    /// Processes GeoJSON/TopoJSON from a serde_json Value without clearing existing entities.
    pub fn process_value(
        &mut self,
        data: &Value,
        options: Option<GeoJsonLoadOptions>,
    ) -> Result<(), DeveloperError> {
        self.set_loading(true);
        let res = self.process_value_internal(data, options);
        if let Err(ref e) = res {
            self.error_event.raise_event(&e.message);
        }
        self.set_loading(false);
        res
    }

    fn set_loading(&mut self, loading: bool) {
        if self.is_loading != loading {
            self.is_loading = loading;
            self.loading_event.raise_event(&loading);
        }
    }

    fn process_value_internal(
        &mut self,
        data: &Value,
        options: Option<GeoJsonLoadOptions>,
    ) -> Result<(), DeveloperError> {
        let obj = data
            .as_object()
            .ok_or_else(|| DeveloperError::new("GeoJSON data must be a JSON object."))?;

        let opts = options.unwrap_or_default();
        if let Some(ref credit) = opts.credit {
            self.credit = Some(credit.clone());
        }

        let crs_fn = resolve_crs(obj, opts.crs_link_handler.as_ref())?;

        let type_str = obj
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DeveloperError::new("GeoJSON object is missing \"type\" property."))?;

        let defaults = GEOJSON_DEFAULTS.with(|d| d.borrow().clone());
        let style = ResolvedStyle {
            marker_size: opts.marker_size.unwrap_or(defaults.marker_size),
            marker_symbol: opts.marker_symbol.or(defaults.marker_symbol),
            marker_color: opts.marker_color.unwrap_or(defaults.marker_color),
            stroke: opts.stroke.unwrap_or(defaults.stroke),
            stroke_width: opts.stroke_width.unwrap_or(defaults.stroke_width),
            fill: opts.fill.unwrap_or(defaults.fill),
            clamp_to_ground: opts.clamp_to_ground.unwrap_or(defaults.clamp_to_ground),
            describe: opts.describe,
        };

        self.entities.suspend_events();
        let res = match type_str {
            "Topology" => self.process_topology(obj, &crs_fn, &style),
            "FeatureCollection" => {
                let features = obj
                    .get("features")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| DeveloperError::new("FeatureCollection missing features array."))?;
                for feat in features {
                    self.process_feature(feat, &crs_fn, &style)?;
                }
                Ok(())
            }
            "Feature" => self.process_feature(data, &crs_fn, &style),
            "Point" | "MultiPoint" | "LineString" | "MultiLineString" | "Polygon"
            | "MultiPolygon" | "GeometryCollection" => {
                self.process_geometry(data, None, None, &crs_fn, &style)
            }
            other => Err(DeveloperError::new(format!(
                "Unsupported GeoJSON type: {}",
                other
            ))),
        };
        let _ = self.entities.resume_events();

        res
    }

    fn process_topology(
        &mut self,
        obj: &serde_json::Map<String, Value>,
        crs_fn: &CrsTransformFn,
        style: &ResolvedStyle,
    ) -> Result<(), DeveloperError> {
        let objects = obj
            .get("objects")
            .and_then(|v| v.as_object())
            .ok_or_else(|| DeveloperError::new("TopoJSON missing objects."))?;

        let raw_arcs = obj
            .get("arcs")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let (scale, translate) = if let Some(transform) = obj.get("transform").and_then(|v| v.as_object()) {
            let s = transform
                .get("scale")
                .and_then(|v| v.as_array())
                .map(|a| (a[0].as_f64().unwrap_or(1.0), a[1].as_f64().unwrap_or(1.0)))
                .unwrap_or((1.0, 1.0));
            let t = transform
                .get("translate")
                .and_then(|v| v.as_array())
                .map(|a| (a[0].as_f64().unwrap_or(0.0), a[1].as_f64().unwrap_or(0.0)))
                .unwrap_or((0.0, 0.0));
            (Some(s), Some(t))
        } else {
            (None, None)
        };

        let mut decoded_arcs: Vec<Vec<[f64; 2]>> = Vec::with_capacity(raw_arcs.len());
        for arc in &raw_arcs {
            let pts = arc.as_array().ok_or_else(|| DeveloperError::new("Invalid TopoJSON arc"))?;
            let mut decoded_pts = Vec::with_capacity(pts.len());
            let mut x = 0.0;
            let mut y = 0.0;
            for pt in pts {
                let coords = pt.as_array().ok_or_else(|| DeveloperError::new("Invalid arc point"))?;
                let dx = coords[0].as_f64().unwrap_or(0.0);
                let dy = coords[1].as_f64().unwrap_or(0.0);
                if let (Some(s), Some(t)) = (scale, translate) {
                    x += dx;
                    y += dy;
                    decoded_pts.push([x * s.0 + t.0, y * s.1 + t.1]);
                } else {
                    decoded_pts.push([dx, dy]);
                }
            }
            decoded_arcs.push(decoded_pts);
        }

        for (_name, topo_obj) in objects {
            let geojson_val = topo_object_to_geojson(topo_obj, &decoded_arcs)?;
            match geojson_val.get("type").and_then(|v| v.as_str()) {
                Some("FeatureCollection") => {
                    if let Some(features) = geojson_val.get("features").and_then(|v| v.as_array()) {
                        for f in features {
                            self.process_feature(f, crs_fn, style)?;
                        }
                    }
                }
                Some("Feature") => {
                    self.process_feature(&geojson_val, crs_fn, style)?;
                }
                _ => {
                    self.process_geometry(&geojson_val, None, None, crs_fn, style)?;
                }
            }
        }
        Ok(())
    }

    fn process_feature(
        &mut self,
        feature: &Value,
        crs_fn: &CrsTransformFn,
        style: &ResolvedStyle,
    ) -> Result<(), DeveloperError> {
        let obj = feature
            .as_object()
            .ok_or_else(|| DeveloperError::new("GeoJSON Feature must be an object."))?;

        let feature_id = match obj.get("id") {
            Some(Value::String(s)) => Some(s.clone()),
            Some(Value::Number(n)) => Some(n.to_string()),
            Some(Value::Null) | None => None,
            Some(_) => None,
        };

        let props = obj.get("properties").and_then(|v| v.as_object());

        let geom = obj
            .get("geometry")
            .ok_or_else(|| DeveloperError::new("Feature is missing geometry property."))?;

        if geom.is_null() {
            let mut entity = self.create_object(feature_id.as_deref(), props, style);
            let _ = self.entities.add(entity.clone());
            // Ensure entity is preserved
            entity.show = true;
            return Ok(());
        }

        self.process_geometry(geom, feature_id.as_deref(), props, crs_fn, style)
    }

    fn create_object(
        &self,
        feature_id: Option<&str>,
        props: Option<&serde_json::Map<String, Value>>,
        style: &ResolvedStyle,
    ) -> Entity {
        let mut id = feature_id.map(|s| s.to_string()).unwrap_or_else(create_guid);
        if self.entities.get_by_id(&id).is_some() {
            id = create_guid();
        }

        let mut entity = Entity::new();
        entity.id = id;

        if let Some(p) = props {
            // Name from title or name
            if let Some(Value::String(title)) = p.get("title") {
                entity.name = Some(title.clone());
            } else if let Some(Value::String(name)) = p.get("name") {
                entity.name = Some(name.clone());
            }

            // Properties bag
            for (k, v) in p {
                entity
                    .properties
                    .add_property(k, Some(Rc::new(ConstantProperty::from_value(v.clone()))));
            }

            // Description
            if let Some(desc_val) = p.get("description") {
                if let Value::String(s) = desc_val {
                    entity.description = Some(ConstantProperty::from_value(s.clone()));
                }
                // if null, leave description None
            } else if let Some(ref desc_fn) = style.describe {
                if let Some(s) = desc_fn(p, &entity) {
                    entity.description = Some(ConstantProperty::from_value(s));
                }
            } else if let Some(html) = default_describe(p) {
                entity.description = Some(ConstantProperty::from_value(html));
            }
        }

        entity
    }

    fn process_geometry(
        &mut self,
        geom: &Value,
        feature_id: Option<&str>,
        props: Option<&serde_json::Map<String, Value>>,
        crs_fn: &CrsTransformFn,
        style: &ResolvedStyle,
    ) -> Result<(), DeveloperError> {
        let obj = geom
            .as_object()
            .ok_or_else(|| DeveloperError::new("GeoJSON geometry must be an object."))?;

        let geom_type = obj
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DeveloperError::new("GeoJSON geometry missing \"type\"."))?;

        match geom_type {
            "Point" => {
                let coords = obj
                    .get("coordinates")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| DeveloperError::new("Point missing coordinates."))?;
                let pos = coords_to_cartesian(coords, crs_fn)?;
                let mut entity = self.create_object(feature_id, props, style);
                entity.position = Some(ConstantPositionProperty::new(Some(pos), None));
                entity.billboard = Some(build_billboard_from_style(props, style));
                let _ = self.entities.add(entity);
                Ok(())
            }
            "MultiPoint" => {
                let coords_arr = obj
                    .get("coordinates")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| DeveloperError::new("MultiPoint missing coordinates."))?;
                for pt in coords_arr {
                    let c = pt
                        .as_array()
                        .ok_or_else(|| DeveloperError::new("Invalid MultiPoint coordinate."))?;
                    let pos = coords_to_cartesian(c, crs_fn)?;
                    let mut entity = self.create_object(None, props, style);
                    entity.position = Some(ConstantPositionProperty::new(Some(pos), None));
                    entity.billboard = Some(build_billboard_from_style(props, style));
                    let _ = self.entities.add(entity);
                }
                Ok(())
            }
            "LineString" => {
                let coords_arr = obj
                    .get("coordinates")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| DeveloperError::new("LineString missing coordinates."))?;
                let positions = coords_array_to_cartesian(coords_arr, crs_fn)?;
                let mut entity = self.create_object(feature_id, props, style);
                entity.polyline = Some(build_polyline_from_style(positions, props, style));
                let _ = self.entities.add(entity);
                Ok(())
            }
            "MultiLineString" => {
                let lines = obj
                    .get("coordinates")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| DeveloperError::new("MultiLineString missing coordinates."))?;
                for line in lines {
                    let c_arr = line
                        .as_array()
                        .ok_or_else(|| DeveloperError::new("Invalid MultiLineString line."))?;
                    let positions = coords_array_to_cartesian(c_arr, crs_fn)?;
                    let mut entity = self.create_object(None, props, style);
                    entity.polyline = Some(build_polyline_from_style(positions, props, style));
                    let _ = self.entities.add(entity);
                }
                Ok(())
            }
            "Polygon" => {
                let rings = obj
                    .get("coordinates")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| DeveloperError::new("Polygon missing coordinates."))?;
                let (hierarchy, has_height) = rings_to_polygon_hierarchy(rings, crs_fn)?;
                let mut entity = self.create_object(feature_id, props, style);
                entity.polygon = Some(build_polygon_from_style(
                    hierarchy,
                    has_height,
                    props,
                    style,
                ));
                let _ = self.entities.add(entity);
                Ok(())
            }
            "MultiPolygon" => {
                let polys = obj
                    .get("coordinates")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| DeveloperError::new("MultiPolygon missing coordinates."))?;
                for poly in polys {
                    let rings = poly
                        .as_array()
                        .ok_or_else(|| DeveloperError::new("Invalid MultiPolygon."))?;
                    let (hierarchy, has_height) = rings_to_polygon_hierarchy(rings, crs_fn)?;
                    let mut entity = self.create_object(None, props, style);
                    entity.polygon = Some(build_polygon_from_style(
                        hierarchy,
                        has_height,
                        props,
                        style,
                    ));
                    let _ = self.entities.add(entity);
                }
                Ok(())
            }
            "GeometryCollection" => {
                let geoms = obj
                    .get("geometries")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| DeveloperError::new("GeometryCollection missing geometries."))?;
                for g in geoms {
                    self.process_geometry(g, None, props, crs_fn, style)?;
                }
                Ok(())
            }
            other => Err(DeveloperError::new(format!(
                "Unknown GeoJSON geometry type: {}",
                other
            ))),
        }
    }
}

struct ResolvedStyle {
    marker_size: f64,
    marker_symbol: Option<String>,
    marker_color: Color,
    stroke: Color,
    stroke_width: f64,
    fill: Color,
    clamp_to_ground: bool,
    describe: Option<DescribeFeatureFn>,
}

fn default_describe(props: &serde_json::Map<String, Value>) -> Option<String> {
    let mut rows = String::new();
    for (k, v) in props {
        if v.is_null() {
            continue;
        }
        let val_str = match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => v.to_string(),
        };
        rows.push_str(&format!("<tr><th>{}</th><td>{}</td></tr>", k, val_str));
    }
    if rows.is_empty() {
        None
    } else {
        Some(format!("<table class=\"cesium-infoBox-defaultTable\"><tbody>{}</tbody></table>", rows))
    }
}

fn resolve_crs(
    obj: &serde_json::Map<String, Value>,
    link_handler: Option<&CrsLinkHandlerFn>,
) -> Result<CrsTransformFn, DeveloperError> {
    let default_crs: CrsTransformFn = Rc::new(|coords: &[f64]| {
        if coords.len() < 2 {
            return Err(DeveloperError::new("Coordinate must have at least 2 elements."));
        }
        let height = coords.get(2).copied();
        Cartesian3::from_degrees(coords[0], coords[1], height, None)
            .map_err(|_| DeveloperError::new("Invalid coordinate degrees."))
    });

    let crs_val = match obj.get("crs") {
        None => return Ok(default_crs),
        Some(Value::Null) => return Err(DeveloperError::new("CRS cannot be null.")),
        Some(v) => v,
    };

    let crs_obj = crs_val
        .as_object()
        .ok_or_else(|| DeveloperError::new("CRS must be an object."))?;

    let crs_type = crs_obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DeveloperError::new("CRS is missing \"type\" property."))?;

    let props = crs_obj
        .get("properties")
        .and_then(|v| v.as_object())
        .ok_or_else(|| DeveloperError::new("CRS is missing \"properties\" object."))?;

    match crs_type {
        "name" => {
            let name = props
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| DeveloperError::new("Named CRS missing \"name\" property."))?;
            match name {
                "EPSG:4326"
                | "urn:ogc:def:crs:OGC:1.3:CRS84"
                | "urn:ogc:def:crs:EPSG::4326" => Ok(default_crs),
                other => Err(DeveloperError::new(format!("Unknown CRS name: {}", other))),
            }
        }
        "EPSG" => {
            let code = props
                .get("code")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| DeveloperError::new("EPSG CRS missing \"code\" property."))?;
            if code == 4326 {
                Ok(default_crs)
            } else {
                Err(DeveloperError::new(format!("Unsupported EPSG code: {}", code)))
            }
        }
        "link" => {
            let href = props
                .get("href")
                .and_then(|v| v.as_str())
                .ok_or_else(|| DeveloperError::new("Link CRS missing \"href\" property."))?;
            if let Some(handler) = link_handler {
                handler(href)
            } else {
                Err(DeveloperError::new(format!(
                    "Unknown CRS link: {}",
                    href
                )))
            }
        }
        other => Err(DeveloperError::new(format!("Unknown CRS type: {}", other))),
    }
}

fn coords_to_cartesian(
    coords: &[Value],
    crs_fn: &CrsTransformFn,
) -> Result<Cartesian3, DeveloperError> {
    let mut nums = Vec::with_capacity(coords.len());
    for v in coords {
        nums.push(
            v.as_f64()
                .ok_or_else(|| DeveloperError::new("Coordinate component must be a number."))?,
        );
    }
    crs_fn(&nums)
}

fn coords_array_to_cartesian(
    coords_arr: &[Value],
    crs_fn: &CrsTransformFn,
) -> Result<Vec<Cartesian3>, DeveloperError> {
    let mut result = Vec::with_capacity(coords_arr.len());
    for item in coords_arr {
        let arr = item
            .as_array()
            .ok_or_else(|| DeveloperError::new("Coordinate item must be an array."))?;
        result.push(coords_to_cartesian(arr, crs_fn)?);
    }
    Ok(result)
}

fn rings_to_polygon_hierarchy(
    rings: &[Value],
    crs_fn: &CrsTransformFn,
) -> Result<(PolygonHierarchy, bool), DeveloperError> {
    if rings.is_empty() {
        return Err(DeveloperError::new("Polygon must have at least one ring."));
    }
    let mut has_height = false;

    let outer_arr = rings[0]
        .as_array()
        .ok_or_else(|| DeveloperError::new("Polygon ring must be an array."))?;
    for pt in outer_arr {
        if let Some(a) = pt.as_array() {
            if a.len() >= 3 {
                has_height = true;
            }
        }
    }
    let outer_positions = coords_array_to_cartesian(outer_arr, crs_fn)?;
    let mut hierarchy = PolygonHierarchy::new(outer_positions);

    for hole_val in &rings[1..] {
        let hole_arr = hole_val
            .as_array()
            .ok_or_else(|| DeveloperError::new("Polygon hole must be an array."))?;
        let hole_positions = coords_array_to_cartesian(hole_arr, crs_fn)?;
        hierarchy.holes.push(PolygonHierarchy::new(hole_positions));
    }

    Ok((hierarchy, has_height))
}

fn build_billboard_from_style(
    props: Option<&serde_json::Map<String, Value>>,
    style: &ResolvedStyle,
) -> BillboardGraphics {
    let mut marker_color = style.marker_color;
    let mut marker_size = style.marker_size;
    let mut marker_symbol = style.marker_symbol.clone();

    if let Some(p) = props {
        if let Some(Value::String(color_str)) = p.get("marker-color") {
            if let Ok(c) = Color::from_css_color_string(color_str) {
                marker_color = c;
            }
        }
        if let Some(Value::String(size_str)) = p.get("marker-size") {
            marker_size = match size_str.as_str() {
                "small" => 24.0,
                "medium" => 48.0,
                "large" => 64.0,
                _ => style.marker_size,
            };
        }
        if let Some(Value::String(sym)) = p.get("marker-symbol") {
            marker_symbol = Some(sym.clone());
        }
    }

    let _ = marker_symbol;
    let image_uri = format!("maki://{}", marker_size as u32);

    let height_reference = if style.clamp_to_ground {
        Some(Rc::new(ConstantProperty::from_value(
            HeightReference::ClampToGround,
        )) as Rc<dyn crate::DynProperty<HeightReference>>)
    } else {
        None
    };

    BillboardGraphics::from_options(BillboardGraphicsOptions {
        image: Some(Rc::new(ConstantProperty::from_value(image_uri))),
        color: Some(Rc::new(ConstantProperty::from_value(marker_color))),
        width: Some(Rc::new(ConstantProperty::from_value(marker_size))),
        height: Some(Rc::new(ConstantProperty::from_value(marker_size))),
        height_reference,
        ..Default::default()
    })
}

fn build_polyline_from_style(
    positions: Vec<Cartesian3>,
    props: Option<&serde_json::Map<String, Value>>,
    style: &ResolvedStyle,
) -> PolylineGraphics {
    let mut stroke = style.stroke;
    let mut stroke_width = style.stroke_width;

    if let Some(p) = props {
        if let Some(Value::String(s)) = p.get("stroke") {
            if let Ok(c) = Color::from_css_color_string(s) {
                stroke = c;
            }
        }
        if let Some(val) = p.get("stroke-opacity").and_then(|v| v.as_f64()) {
            stroke = Color::new(stroke.red, stroke.green, stroke.blue, val as f32);
        }
        if let Some(val) = p.get("stroke-width").and_then(|v| v.as_f64()) {
            stroke_width = val;
        }
    }

    PolylineGraphics::from_options(PolylineGraphicsOptions {
        positions: Some(Rc::new(ConstantProperty::from_value(positions))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(stroke)) as Rc<dyn MaterialProperty>),
        width: Some(Rc::new(ConstantProperty::from_value(stroke_width))),
        clamp_to_ground: Some(Rc::new(ConstantProperty::from_value(style.clamp_to_ground))),
        ..Default::default()
    })
}

fn build_polygon_from_style(
    hierarchy: PolygonHierarchy,
    has_height: bool,
    props: Option<&serde_json::Map<String, Value>>,
    style: &ResolvedStyle,
) -> PolygonGraphics {
    let mut fill = style.fill;
    let mut stroke = style.stroke;
    let mut stroke_width = style.stroke_width;

    if let Some(p) = props {
        if let Some(Value::String(s)) = p.get("fill") {
            if let Ok(c) = Color::from_css_color_string(s) {
                fill = c;
            }
        }
        if let Some(val) = p.get("fill-opacity").and_then(|v| v.as_f64()) {
            fill = Color::new(fill.red, fill.green, fill.blue, val as f32);
        }
        if let Some(Value::String(s)) = p.get("stroke") {
            if let Ok(c) = Color::from_css_color_string(s) {
                stroke = c;
            }
        }
        if let Some(val) = p.get("stroke-opacity").and_then(|v| v.as_f64()) {
            stroke = Color::new(stroke.red, stroke.green, stroke.blue, val as f32);
        }
        if let Some(val) = p.get("stroke-width").and_then(|v| v.as_f64()) {
            stroke_width = val;
        }
    }

    let per_position_height = if has_height {
        Some(Rc::new(ConstantProperty::from_value(true)) as Rc<dyn crate::DynProperty<bool>>)
    } else {
        None
    };

    let height_reference = if !has_height && style.clamp_to_ground {
        Some(Rc::new(ConstantProperty::from_value(
            HeightReference::ClampToGround,
        )) as Rc<dyn crate::DynProperty<HeightReference>>)
    } else {
        None
    };

    PolygonGraphics::from_options(PolygonGraphicsOptions {
        hierarchy: Some(Rc::new(ConstantProperty::from_value(hierarchy))),
        material: Some(Rc::new(ColorMaterialProperty::from_color(fill)) as Rc<dyn MaterialProperty>),
        outline: Some(Rc::new(ConstantProperty::from_value(true))),
        outline_color: Some(Rc::new(ConstantProperty::from_value(stroke))),
        outline_width: Some(Rc::new(ConstantProperty::from_value(stroke_width))),
        per_position_height,
        height_reference,
        ..Default::default()
    })
}

fn decode_arc_indices(
    arc_indices: &[Value],
    decoded_arcs: &[Vec<[f64; 2]>],
) -> Result<Vec<Value>, DeveloperError> {
    let mut ring: Vec<Value> = Vec::new();
    for idx_val in arc_indices {
        let idx = idx_val
            .as_i64()
            .ok_or_else(|| DeveloperError::new("TopoJSON arc index must be integer."))?;
        let arc = if idx < 0 {
            let actual = (!idx) as usize;
            let a = decoded_arcs
                .get(actual)
                .ok_or_else(|| DeveloperError::new("TopoJSON arc index out of bounds."))?;
            let mut rev = a.clone();
            rev.reverse();
            rev
        } else {
            decoded_arcs
                .get(idx as usize)
                .ok_or_else(|| DeveloperError::new("TopoJSON arc index out of bounds."))?
                .clone()
        };

        let start = if ring.is_empty() { 0 } else { 1 };
        for pt in &arc[start..] {
            ring.push(Value::Array(vec![
                Value::from(pt[0]),
                Value::from(pt[1]),
            ]));
        }
    }
    Ok(ring)
}

fn topo_object_to_geojson(
    topo: &Value,
    decoded_arcs: &[Vec<[f64; 2]>],
) -> Result<Value, DeveloperError> {
    let obj = topo
        .as_object()
        .ok_or_else(|| DeveloperError::new("TopoJSON object must be a JSON object."))?;
    let t = obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DeveloperError::new("TopoJSON object missing type."))?;

    let mut out = serde_json::Map::new();
    match t {
        "GeometryCollection" => {
            let geoms = obj
                .get("geometries")
                .and_then(|v| v.as_array())
                .ok_or_else(|| DeveloperError::new("TopoJSON GeometryCollection missing geometries."))?;
            let mut features = Vec::with_capacity(geoms.len());
            for g in geoms {
                let geom = topo_object_to_geojson(g, decoded_arcs)?;
                let mut f = serde_json::Map::new();
                f.insert("type".to_string(), Value::String("Feature".to_string()));
                if let Some(id) = g.get("id") {
                    f.insert("id".to_string(), id.clone());
                }
                if let Some(props) = g.get("properties") {
                    f.insert("properties".to_string(), props.clone());
                }
                f.insert("geometry".to_string(), geom);
                features.push(Value::Object(f));
            }
            out.insert(
                "type".to_string(),
                Value::String("FeatureCollection".to_string()),
            );
            out.insert("features".to_string(), Value::Array(features));
            Ok(Value::Object(out))
        }
        "Point" | "MultiPoint" => Ok(topo.clone()),
        "LineString" => {
            let arcs = obj
                .get("arcs")
                .and_then(|v| v.as_array())
                .ok_or_else(|| DeveloperError::new("TopoJSON LineString missing arcs."))?;
            let coords = decode_arc_indices(arcs, decoded_arcs)?;
            out.insert("type".to_string(), Value::String("LineString".to_string()));
            out.insert("coordinates".to_string(), Value::Array(coords));
            Ok(Value::Object(out))
        }
        "Polygon" => {
            let rings = obj
                .get("arcs")
                .and_then(|v| v.as_array())
                .ok_or_else(|| DeveloperError::new("TopoJSON Polygon missing arcs."))?;
            let mut decoded_rings = Vec::with_capacity(rings.len());
            for r in rings {
                let r_arr = r
                    .as_array()
                    .ok_or_else(|| DeveloperError::new("Invalid TopoJSON ring."))?;
                decoded_rings.push(Value::Array(decode_arc_indices(r_arr, decoded_arcs)?));
            }
            out.insert("type".to_string(), Value::String("Polygon".to_string()));
            out.insert("coordinates".to_string(), Value::Array(decoded_rings));
            Ok(Value::Object(out))
        }
        "MultiPolygon" => {
            let polys = obj
                .get("arcs")
                .and_then(|v| v.as_array())
                .ok_or_else(|| DeveloperError::new("TopoJSON MultiPolygon missing arcs."))?;
            let mut decoded_polys = Vec::with_capacity(polys.len());
            for p in polys {
                let rings = p
                    .as_array()
                    .ok_or_else(|| DeveloperError::new("Invalid TopoJSON MultiPolygon."))?;
                let mut decoded_rings = Vec::with_capacity(rings.len());
                for r in rings {
                    let r_arr = r
                        .as_array()
                        .ok_or_else(|| DeveloperError::new("Invalid TopoJSON ring."))?;
                    decoded_rings.push(Value::Array(decode_arc_indices(r_arr, decoded_arcs)?));
                }
                decoded_polys.push(Value::Array(decoded_rings));
            }
            out.insert(
                "type".to_string(),
                Value::String("MultiPolygon".to_string()),
            );
            out.insert("coordinates".to_string(), Value::Array(decoded_polys));
            Ok(Value::Object(out))
        }
        other => Err(DeveloperError::new(format!(
            "Unsupported TopoJSON object type: {}",
            other
        ))),
    }
}
