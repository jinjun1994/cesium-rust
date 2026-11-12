//! CzmlDataSource matching CesiumJS `DataSources/CzmlDataSource.js` (Apache-2.0).

use std::rc::Rc;

use serde_json::Value;

use crate::{
    create_guid, ArcType, BillboardGraphics, BillboardGraphicsOptions, BoxGraphics,
    BoxGraphicsOptions, Cartesian2, Cartesian3, ClockRange, ClockStep, Color,
    ColorMaterialProperty, ConstantPositionProperty, ConstantProperty, DataSourceClock,
    DeveloperError, DistanceDisplayCondition, DynProperty, EllipseGraphics, EllipseGraphicsOptions,
    Ellipsoid, Entity, EntityCluster, EntityCollection, Event, HeightReference, HorizontalOrigin,
    JulianDate, LabelGraphics, LabelGraphicsOptions, LabelStyle, MaterialProperty, NearFarScalar,
    PathGraphics, PathGraphicsOptions, PointGraphics, PointGraphicsOptions, PolygonGraphics,
    PolygonGraphicsOptions, PolygonHierarchy, PolylineArrowMaterialProperty,
    PolylineDashMaterialProperty, PolylineGlowMaterialProperty, PolylineGraphics,
    PolylineGraphicsOptions, PolylineOutlineMaterialProperty, Quaternion, ReferenceFrame,
    SampledPositionProperty, SampledProperty, TimeInterval, TimeIntervalCollection,
    TimeIntervalCollectionProperty, VerticalOrigin,
};

/// Options for loading CZML data into a CzmlDataSource.
#[derive(Clone, Default)]
pub struct CzmlLoadOptions {
    pub source_uri: Option<String>,
    pub credit: Option<String>,
    pub ellipsoid: Option<Ellipsoid>,
}

/// A DataSource which processes CZML (Cesium Language) packets.
pub struct CzmlDataSource {
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

impl Default for CzmlDataSource {
    fn default() -> Self {
        Self::new(None)
    }
}

impl CzmlDataSource {
    /// Creates a new CzmlDataSource with an optional name.
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

    fn set_loading(&mut self, loading: bool) {
        if self.is_loading != loading {
            self.is_loading = loading;
            self.loading_event.raise_event(&loading);
        }
    }

    /// Loads CZML from a JSON string, replacing existing entities.
    pub fn load_str(
        &mut self,
        json_str: &str,
        options: Option<CzmlLoadOptions>,
    ) -> Result<(), DeveloperError> {
        self.set_loading(true);
        let parsed: Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("Failed to parse CZML JSON: {}", e);
                self.error_event.raise_event(&msg);
                self.set_loading(false);
                return Err(DeveloperError::new(msg));
            }
        };
        self.load_value(&parsed, options)
    }

    /// Loads CZML from a serde_json Value, replacing existing entities.
    pub fn load_value(
        &mut self,
        data: &Value,
        options: Option<CzmlLoadOptions>,
    ) -> Result<(), DeveloperError> {
        self.set_loading(true);
        self.entities.remove_all();
        self.clock = None;
        let res = self.process_value_internal(data, options, true);
        if let Err(ref e) = res {
            self.error_event.raise_event(&e.message);
        }
        self.set_loading(false);
        res
    }

    /// Processes CZML from a JSON string without clearing existing entities.
    pub fn process_str(
        &mut self,
        json_str: &str,
        options: Option<CzmlLoadOptions>,
    ) -> Result<(), DeveloperError> {
        self.set_loading(true);
        let parsed: Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("Failed to parse CZML JSON: {}", e);
                self.error_event.raise_event(&msg);
                self.set_loading(false);
                return Err(DeveloperError::new(msg));
            }
        };
        self.process_value(&parsed, options)
    }

    /// Processes CZML from a serde_json Value without clearing existing entities.
    pub fn process_value(
        &mut self,
        data: &Value,
        options: Option<CzmlLoadOptions>,
    ) -> Result<(), DeveloperError> {
        self.set_loading(true);
        let res = self.process_value_internal(data, options, false);
        if let Err(ref e) = res {
            self.error_event.raise_event(&e.message);
        }
        self.set_loading(false);
        res
    }

    fn process_value_internal(
        &mut self,
        data: &Value,
        options: Option<CzmlLoadOptions>,
        is_load: bool,
    ) -> Result<(), DeveloperError> {
        let opts = options.unwrap_or_default();
        if let Some(ref credit) = opts.credit {
            self.credit = Some(credit.clone());
        }
        let ellipsoid = opts.ellipsoid.unwrap_or(Ellipsoid::WGS84);

        let packets: Vec<&Value> = match data {
            Value::Array(arr) => arr.iter().collect(),
            Value::Object(_) => vec![data],
            _ => {
                return Err(DeveloperError::new(
                    "CZML data must be a JSON array or packet object.",
                ));
            }
        };

        self.entities.suspend_events();

        let mut metadata_changed = is_load;
        for packet in packets {
            let obj = match packet.as_object() {
                Some(o) => o,
                None => {
                    let _ = self.entities.resume_events();
                    return Err(DeveloperError::new("CZML packet must be a JSON object."));
                }
            };

            let id = obj.get("id").and_then(|v| v.as_str());
            if id == Some("document") {
                if let Some(Value::String(new_name)) = obj.get("name") {
                    if self.name != *new_name {
                        self.name = new_name.clone();
                        metadata_changed = true;
                    }
                }
                if let Some(clock_val) = obj.get("clock") {
                    if let Some(new_clock) = parse_clock_packet(clock_val)? {
                        if self.clock.as_ref() != Some(&new_clock) {
                            self.clock = Some(new_clock);
                            metadata_changed = true;
                        }
                    }
                }
                continue;
            }

            if obj.get("delete").and_then(|v| v.as_bool()) == Some(true) {
                if let Some(eid) = id {
                    self.entities.remove_by_id(eid);
                }
                continue;
            }

            let entity_id = id.map(|s| s.to_string()).unwrap_or_else(create_guid);
            let entity = self.entities.get_or_create_entity(&entity_id)?;
            apply_packet_to_entity(entity, obj, &ellipsoid)?;
        }

        let _ = self.entities.resume_events();

        // Fallback: if clock is still None, compute availability from entities
        if self.clock.is_none() {
            let avail = self.entities.compute_availability();
            let min_iso = crate::Iso8601::minimum_value();
            let max_iso = crate::Iso8601::maximum_value();
            if !avail.start.equals(&min_iso) && !avail.stop.equals(&max_iso) {
                let mut clk = DataSourceClock::new();
                clk.start_time = Some(avail.start);
                clk.stop_time = Some(avail.stop);
                clk.current_time = Some(avail.start);
                self.clock = Some(clk);
                metadata_changed = true;
            }
        }

        if metadata_changed {
            self.changed_event.raise_event(&());
        }

        Ok(())
    }
}

fn parse_iso_interval(s: &str) -> Result<TimeInterval<()>, DeveloperError> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 2 {
        return Err(DeveloperError::new(format!(
            "Invalid ISO8601 interval string: {}",
            s
        )));
    }
    let start = JulianDate::from_iso8601(parts[0])
        .map_err(|_| DeveloperError::new(format!("Invalid start date: {}", parts[0])))?;
    let stop = JulianDate::from_iso8601(parts[1])
        .map_err(|_| DeveloperError::new(format!("Invalid stop date: {}", parts[1])))?;
    Ok(TimeInterval::new(start, stop, true, true))
}

fn parse_clock_packet(val: &Value) -> Result<Option<DataSourceClock>, DeveloperError> {
    let obj = match val.as_object() {
        Some(o) => o,
        None => return Ok(None),
    };
    let mut clock = DataSourceClock::default();

    if let Some(Value::String(interval_str)) = obj.get("interval") {
        let interval = parse_iso_interval(interval_str)?;
        clock.start_time = Some(interval.start);
        clock.stop_time = Some(interval.stop);
    }

    if let Some(Value::String(curr_str)) = obj.get("currentTime") {
        let curr = JulianDate::from_iso8601(curr_str)
            .map_err(|_| DeveloperError::new(format!("Invalid currentTime: {}", curr_str)))?;
        clock.current_time = Some(curr);
    }

    if let Some(mult) = obj.get("multiplier").and_then(|v| v.as_f64()) {
        clock.multiplier = Some(mult);
    }

    if let Some(Value::String(range_str)) = obj.get("range") {
        clock.clock_range = Some(match range_str.as_str() {
            "UNBOUNDED" => ClockRange::Unbounded,
            "CLAMPED" => ClockRange::Clamped,
            "LOOP_STOP" => ClockRange::LoopStop,
            other => return Err(DeveloperError::new(format!("Unknown clock range: {}", other))),
        });
    }

    if let Some(Value::String(step_str)) = obj.get("step") {
        clock.clock_step = Some(match step_str.as_str() {
            "SYSTEM_CLOCK" => ClockStep::SystemClock,
            "SYSTEM_CLOCK_MULTIPLIER" => ClockStep::SystemClockMultiplier,
            "TICK_DEPENDENT" => ClockStep::TickDependent,
            other => return Err(DeveloperError::new(format!("Unknown clock step: {}", other))),
        });
    }

    Ok(Some(clock))
}

fn apply_packet_to_entity(
    entity: &mut Entity,
    obj: &serde_json::Map<String, Value>,
    ellipsoid: &Ellipsoid,
) -> Result<(), DeveloperError> {
    if let Some(Value::String(name)) = obj.get("name") {
        entity.name = Some(name.clone());
    }
    if let Some(show) = obj.get("show").and_then(|v| v.as_bool()) {
        entity.show = show;
    }

    if let Some(avail_val) = obj.get("availability") {
        let mut collection = TimeIntervalCollection::new();
        match avail_val {
            Value::String(s) => {
                collection.add_interval(parse_iso_interval(s)?);
            }
            Value::Array(arr) => {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        collection.add_interval(parse_iso_interval(s)?);
                    }
                }
            }
            _ => {}
        }
        entity.availability = Some(collection);
    }

    if let Some(desc_val) = obj.get("description") {
        if let Some(s) = desc_val.as_str() {
            entity.description = Some(ConstantProperty::from_value(s.to_string()));
        } else if let Some(s) = desc_val.get("string").and_then(|v| v.as_str()) {
            entity.description = Some(ConstantProperty::from_value(s.to_string()));
        }
    }

    if let Some(pos_val) = obj.get("position") {
        entity.position = Some(parse_position_property(pos_val, ellipsoid)?);
    }

    if let Some(ori_val) = obj.get("orientation") {
        if let Some(q) = parse_quaternion(ori_val) {
            entity.orientation = Some(ConstantProperty::from_value(q));
        }
    }

    if let Some(bb_val) = obj.get("billboard") {
        entity.billboard = Some(parse_billboard(bb_val)?);
    }

    if let Some(lbl_val) = obj.get("label") {
        entity.label = Some(parse_label(lbl_val)?);
    }

    if let Some(pt_val) = obj.get("point") {
        entity.point = Some(parse_point(pt_val)?);
    }

    if let Some(pl_val) = obj.get("polyline") {
        entity.polyline = Some(parse_polyline(pl_val, ellipsoid)?);
    }

    if let Some(pg_val) = obj.get("polygon") {
        entity.polygon = Some(parse_polygon(pg_val, ellipsoid)?);
    }

    if let Some(el_val) = obj.get("ellipse") {
        entity.ellipse = Some(parse_ellipse(el_val)?);
    }

    if let Some(box_val) = obj.get("box") {
        entity.box_graphics = Some(parse_box(box_val)?);
    }

    if let Some(path_val) = obj.get("path") {
        entity.path = Some(parse_path(path_val)?);
    }

    if let Some(Value::Object(props)) = obj.get("properties") {
        for (k, v) in props {
            entity
                .properties
                .add_property(k, Some(Rc::new(ConstantProperty::from_value(v.clone()))));
        }
    }

    Ok(())
}

fn parse_epoch(obj: &serde_json::Map<String, Value>) -> Result<JulianDate, DeveloperError> {
    if let Some(Value::String(epoch_str)) = obj.get("epoch") {
        JulianDate::from_iso8601(epoch_str)
            .map_err(|_| DeveloperError::new(format!("Invalid epoch: {}", epoch_str)))
    } else {
        Ok(JulianDate::default())
    }
}

fn parse_position_property(
    val: &Value,
    ellipsoid: &Ellipsoid,
) -> Result<ConstantPositionProperty, DeveloperError> {
    let obj = val
        .as_object()
        .ok_or_else(|| DeveloperError::new("Position packet must be an object."))?;

    let frame = match obj.get("referenceFrame").and_then(|v| v.as_str()) {
        Some("INERTIAL") => ReferenceFrame::Inertial,
        _ => ReferenceFrame::Fixed,
    };

    let epoch = parse_epoch(obj)?;

    if let Some(Value::Array(arr)) = obj.get("cartesian") {
        if arr.len() == 3 && arr[0].is_number() && arr[1].is_number() && arr[2].is_number() {
            let pos = Cartesian3::new(
                arr[0].as_f64().unwrap(),
                arr[1].as_f64().unwrap(),
                arr[2].as_f64().unwrap(),
            );
            return Ok(ConstantPositionProperty::new(Some(pos), Some(frame)));
        }
        return parse_sampled_position_array(arr, epoch, frame, |x, y, z| {
            Ok(Cartesian3::new(x, y, z))
        });
    }

    if let Some(Value::Array(arr)) = obj.get("cartographicDegrees") {
        if arr.len() == 3 && arr[0].is_number() && arr[1].is_number() && arr[2].is_number() {
            let pos = Cartesian3::from_degrees(
                arr[0].as_f64().unwrap(),
                arr[1].as_f64().unwrap(),
                Some(arr[2].as_f64().unwrap()),
                Some(ellipsoid),
            )
            .map_err(|_| DeveloperError::new("Invalid cartographicDegrees"))?;
            return Ok(ConstantPositionProperty::new(Some(pos), Some(frame)));
        }
        return parse_sampled_position_array(arr, epoch, frame, |lon, lat, h| {
            Cartesian3::from_degrees(lon, lat, Some(h), Some(ellipsoid))
                .map_err(|_| DeveloperError::new("Invalid cartographicDegrees"))
        });
    }

    if let Some(Value::Array(arr)) = obj.get("cartographicRadians") {
        if arr.len() == 3 && arr[0].is_number() && arr[1].is_number() && arr[2].is_number() {
            let pos = Cartesian3::from_radians(
                arr[0].as_f64().unwrap(),
                arr[1].as_f64().unwrap(),
                Some(arr[2].as_f64().unwrap()),
                Some(ellipsoid),
            )
            .map_err(|_| DeveloperError::new("Invalid cartographicRadians"))?;
            return Ok(ConstantPositionProperty::new(Some(pos), Some(frame)));
        }
        return parse_sampled_position_array(arr, epoch, frame, |lon, lat, h| {
            Cartesian3::from_radians(lon, lat, Some(h), Some(ellipsoid))
                .map_err(|_| DeveloperError::new("Invalid cartographicRadians"))
        });
    }

    Ok(ConstantPositionProperty::new(None, Some(frame)))
}

fn parse_sampled_position_array<F>(
    arr: &[Value],
    epoch: JulianDate,
    frame: ReferenceFrame,
    convert: F,
) -> Result<ConstantPositionProperty, DeveloperError>
where
    F: Fn(f64, f64, f64) -> Result<Cartesian3, DeveloperError>,
{
    let mut sampled = SampledPositionProperty::new(Some(frame));
    let mut i = 0;
    while i + 3 < arr.len() {
        let time = if let Some(t_sec) = arr[i].as_f64() {
            epoch.add_seconds(t_sec)
        } else if let Some(iso) = arr[i].as_str() {
            JulianDate::from_iso8601(iso)
                .map_err(|_| DeveloperError::new(format!("Invalid sample time: {}", iso)))?
        } else {
            return Err(DeveloperError::new("Invalid sample time format."));
        };

        let x = arr[i + 1]
            .as_f64()
            .ok_or_else(|| DeveloperError::new("Sample coordinate must be numeric."))?;
        let y = arr[i + 2]
            .as_f64()
            .ok_or_else(|| DeveloperError::new("Sample coordinate must be numeric."))?;
        let z = arr[i + 3]
            .as_f64()
            .ok_or_else(|| DeveloperError::new("Sample coordinate must be numeric."))?;

        let pos = convert(x, y, z)?;
        sampled.add_sample(time, pos);
        i += 4;
    }

    Ok(ConstantPositionProperty::from_dynamic(Rc::new(sampled)))
}

fn parse_color(val: &Value) -> Option<Color> {
    if let Some(arr) = val.get("rgba").and_then(|v| v.as_array()) {
        if arr.len() == 4 {
            return Some(Color::new(
                arr[0].as_f64()? as f32 / 255.0,
                arr[1].as_f64()? as f32 / 255.0,
                arr[2].as_f64()? as f32 / 255.0,
                arr[3].as_f64()? as f32 / 255.0,
            ));
        }
    }
    if let Some(arr) = val.get("rgbaf").and_then(|v| v.as_array()) {
        if arr.len() == 4 {
            return Some(Color::new(
                arr[0].as_f64()? as f32,
                arr[1].as_f64()? as f32,
                arr[2].as_f64()? as f32,
                arr[3].as_f64()? as f32,
            ));
        }
    }
    None
}

fn parse_color_property(val: &Value) -> Result<Option<Rc<dyn DynProperty<Color>>>, DeveloperError> {
    if let Value::Array(intervals) = val {
        let mut prop = TimeIntervalCollectionProperty::new();
        for item in intervals {
            if let Some(interval_str) = item.get("interval").and_then(|v| v.as_str()) {
                let ti = parse_iso_interval(interval_str)?;
                if let Some(c) = parse_color(item) {
                    prop.intervals.add_interval(TimeInterval::with_data(
                        ti.start,
                        ti.stop,
                        ti.is_start_included,
                        ti.is_stop_included,
                        c,
                    ));
                }
            }
        }
        return Ok(Some(Rc::new(prop)));
    }

    if let Some(c) = parse_color(val) {
        return Ok(Some(Rc::new(ConstantProperty::from_value(c))));
    }

    if let Some(obj) = val.as_object() {
        if let Some(Value::Array(rgba)) = obj.get("rgba") {
            if rgba.len() > 4 {
                let epoch = parse_epoch(obj)?;
                let mut sampled = SampledProperty::<Color>::new();
                let mut i = 0;
                while i + 4 < rgba.len() {
                    let t = if let Some(sec) = rgba[i].as_f64() {
                        epoch.add_seconds(sec)
                    } else if let Some(iso) = rgba[i].as_str() {
                        JulianDate::from_iso8601(iso).map_err(|_| {
                            DeveloperError::new(format!("Invalid sample time: {}", iso))
                        })?
                    } else {
                        return Err(DeveloperError::new("Invalid rgba sample time."));
                    };
                    let c = Color::new(
                        rgba[i + 1].as_f64().unwrap_or(0.0) as f32 / 255.0,
                        rgba[i + 2].as_f64().unwrap_or(0.0) as f32 / 255.0,
                        rgba[i + 3].as_f64().unwrap_or(0.0) as f32 / 255.0,
                        rgba[i + 4].as_f64().unwrap_or(255.0) as f32 / 255.0,
                    );
                    sampled.add_sample(t, c);
                    i += 5;
                }
                return Ok(Some(Rc::new(sampled)));
            }
        }
    }

    Ok(None)
}

fn parse_cartesian2(val: &Value) -> Option<Cartesian2> {
    let arr = val.get("cartesian2").and_then(|v| v.as_array())?;
    if arr.len() == 2 {
        Some(Cartesian2::new(arr[0].as_f64()?, arr[1].as_f64()?))
    } else {
        None
    }
}

fn parse_cartesian3(val: &Value) -> Option<Cartesian3> {
    let arr = val.get("cartesian").and_then(|v| v.as_array())?;
    if arr.len() == 3 {
        Some(Cartesian3::new(
            arr[0].as_f64()?,
            arr[1].as_f64()?,
            arr[2].as_f64()?,
        ))
    } else {
        None
    }
}

fn parse_quaternion(val: &Value) -> Option<Quaternion> {
    let arr = val.get("unitQuaternion").and_then(|v| v.as_array())?;
    if arr.len() == 4 {
        Some(Quaternion::new(
            arr[0].as_f64()?,
            arr[1].as_f64()?,
            arr[2].as_f64()?,
            arr[3].as_f64()?,
        ))
    } else {
        None
    }
}

fn parse_near_far_scalar(val: &Value) -> Option<NearFarScalar> {
    let arr = val.get("nearFarScalar").and_then(|v| v.as_array())?;
    if arr.len() == 4 {
        Some(NearFarScalar::new(
            arr[0].as_f64()?,
            arr[1].as_f64()?,
            arr[2].as_f64()?,
            arr[3].as_f64()?,
        ))
    } else {
        None
    }
}

fn parse_distance_display_condition(val: &Value) -> Option<DistanceDisplayCondition> {
    let arr = val
        .get("distanceDisplayCondition")
        .and_then(|v| v.as_array())?;
    if arr.len() == 2 {
        Some(DistanceDisplayCondition::new(
            arr[0].as_f64()?,
            arr[1].as_f64()?,
        ))
    } else {
        None
    }
}

fn parse_number(val: &Value) -> Option<f64> {
    val.as_f64().or_else(|| val.get("number").and_then(|v| v.as_f64()))
}

fn parse_boolean(val: &Value) -> Option<bool> {
    val.as_bool()
        .or_else(|| val.get("boolean").and_then(|v| v.as_bool()))
}

fn parse_uri(val: &Value) -> Option<String> {
    val.as_str()
        .map(|s| s.to_string())
        .or_else(|| val.get("uri").and_then(|v| v.as_str()).map(|s| s.to_string()))
}

fn parse_material(val: &Value) -> Result<Option<Rc<dyn MaterialProperty>>, DeveloperError> {
    let obj = match val.as_object() {
        Some(o) => o,
        None => return Ok(None),
    };

    if let Some(sc) = obj.get("solidColor") {
        if let Some(c_val) = sc.get("color") {
            if let Some(c_prop) = parse_color_property(c_val)? {
                return Ok(Some(Rc::new(ColorMaterialProperty::from_dynamic_color(
                    c_prop,
                ))));
            }
        }
    }

    if let Some(po) = obj.get("polylineOutline") {
        let color = po
            .get("color")
            .and_then(parse_color)
            .map(ConstantProperty::from_value);
        let outline_color = po
            .get("outlineColor")
            .and_then(parse_color)
            .map(ConstantProperty::from_value);
        let outline_width = po
            .get("outlineWidth")
            .and_then(parse_number)
            .map(ConstantProperty::from_value);
        return Ok(Some(Rc::new(PolylineOutlineMaterialProperty::new(
            color,
            outline_color,
            outline_width,
        ))));
    }

    if let Some(pg) = obj.get("polylineGlow") {
        let color = pg
            .get("color")
            .and_then(parse_color)
            .map(ConstantProperty::from_value);
        let glow_power = pg
            .get("glowPower")
            .and_then(parse_number)
            .map(ConstantProperty::from_value);
        let taper_power = pg
            .get("taperPower")
            .and_then(parse_number)
            .map(ConstantProperty::from_value);
        return Ok(Some(Rc::new(PolylineGlowMaterialProperty::new(
            color,
            glow_power,
            taper_power,
        ))));
    }

    if let Some(pa) = obj.get("polylineArrow") {
        let color = pa
            .get("color")
            .and_then(parse_color)
            .map(ConstantProperty::from_value);
        return Ok(Some(Rc::new(PolylineArrowMaterialProperty::new(color))));
    }

    if let Some(pd) = obj.get("polylineDash") {
        let color = pd
            .get("color")
            .and_then(parse_color)
            .map(ConstantProperty::from_value);
        let gap_color = pd
            .get("gapColor")
            .and_then(parse_color)
            .map(ConstantProperty::from_value);
        let dash_length = pd
            .get("dashLength")
            .and_then(parse_number)
            .map(ConstantProperty::from_value);
        let dash_pattern = pd
            .get("dashPattern")
            .and_then(|v| v.as_u64())
            .map(|n| ConstantProperty::from_value(n as f64));
        return Ok(Some(Rc::new(PolylineDashMaterialProperty::new(
            color,
            gap_color,
            dash_length,
            dash_pattern,
        ))));
    }

    Ok(None)
}

fn parse_positions_array(val: &Value, ellipsoid: &Ellipsoid) -> Result<Vec<Cartesian3>, DeveloperError> {
    let obj = val
        .as_object()
        .ok_or_else(|| DeveloperError::new("Positions must be an object."))?;

    if let Some(Value::Array(arr)) = obj.get("cartesian") {
        let mut pts = Vec::with_capacity(arr.len() / 3);
        let mut i = 0;
        while i + 2 < arr.len() {
            pts.push(Cartesian3::new(
                arr[i].as_f64().unwrap_or(0.0),
                arr[i + 1].as_f64().unwrap_or(0.0),
                arr[i + 2].as_f64().unwrap_or(0.0),
            ));
            i += 3;
        }
        return Ok(pts);
    }

    if let Some(Value::Array(arr)) = obj.get("cartographicDegrees") {
        let mut pts = Vec::with_capacity(arr.len() / 3);
        let mut i = 0;
        while i + 2 < arr.len() {
            pts.push(
                Cartesian3::from_degrees(
                    arr[i].as_f64().unwrap_or(0.0),
                    arr[i + 1].as_f64().unwrap_or(0.0),
                    Some(arr[i + 2].as_f64().unwrap_or(0.0)),
                    Some(ellipsoid),
                )
                .map_err(|_| DeveloperError::new("Invalid cartographicDegrees"))?,
            );
            i += 3;
        }
        return Ok(pts);
    }

    if let Some(Value::Array(arr)) = obj.get("cartographicRadians") {
        let mut pts = Vec::with_capacity(arr.len() / 3);
        let mut i = 0;
        while i + 2 < arr.len() {
            pts.push(
                Cartesian3::from_radians(
                    arr[i].as_f64().unwrap_or(0.0),
                    arr[i + 1].as_f64().unwrap_or(0.0),
                    Some(arr[i + 2].as_f64().unwrap_or(0.0)),
                    Some(ellipsoid),
                )
                .map_err(|_| DeveloperError::new("Invalid cartographicRadians"))?,
            );
            i += 3;
        }
        return Ok(pts);
    }

    Ok(Vec::new())
}

fn parse_billboard(val: &Value) -> Result<BillboardGraphics, DeveloperError> {
    let mut opts = BillboardGraphicsOptions::default();

    if let Some(img) = val.get("image").and_then(parse_uri) {
        opts.image = Some(Rc::new(ConstantProperty::from_value(img)));
    }
    if let Some(show) = val.get("show").and_then(parse_boolean) {
        opts.show = Some(Rc::new(ConstantProperty::from_value(show)));
    }
    if let Some(scale) = val.get("scale").and_then(parse_number) {
        opts.scale = Some(Rc::new(ConstantProperty::from_value(scale)));
    }
    if let Some(c_val) = val.get("color") {
        opts.color = parse_color_property(c_val)?;
    }
    if let Some(po) = val.get("pixelOffset").and_then(parse_cartesian2) {
        opts.pixel_offset = Some(Rc::new(ConstantProperty::from_value(po)));
    }
    if let Some(eo) = val.get("eyeOffset").and_then(parse_cartesian3) {
        opts.eye_offset = Some(Rc::new(ConstantProperty::from_value(eo)));
    }
    if let Some(ho) = val.get("horizontalOrigin").and_then(|v| v.as_str()) {
        let origin = match ho {
            "LEFT" => HorizontalOrigin::Left,
            "RIGHT" => HorizontalOrigin::Right,
            _ => HorizontalOrigin::Center,
        };
        opts.horizontal_origin = Some(Rc::new(ConstantProperty::from_value(origin)));
    }
    if let Some(vo) = val.get("verticalOrigin").and_then(|v| v.as_str()) {
        let origin = match vo {
            "BOTTOM" => VerticalOrigin::Bottom,
            "BASELINE" => VerticalOrigin::Baseline,
            "TOP" => VerticalOrigin::Top,
            _ => VerticalOrigin::Center,
        };
        opts.vertical_origin = Some(Rc::new(ConstantProperty::from_value(origin)));
    }
    if let Some(hr) = val.get("heightReference").and_then(|v| v.as_str()) {
        let href = match hr {
            "CLAMP_TO_GROUND" => HeightReference::ClampToGround,
            "RELATIVE_TO_GROUND" => HeightReference::RelativeToGround,
            _ => HeightReference::None,
        };
        opts.height_reference = Some(Rc::new(ConstantProperty::from_value(href)));
    }
    if let Some(w) = val.get("width").and_then(parse_number) {
        opts.width = Some(Rc::new(ConstantProperty::from_value(w)));
    }
    if let Some(h) = val.get("height").and_then(parse_number) {
        opts.height = Some(Rc::new(ConstantProperty::from_value(h)));
    }
    if let Some(rot) = val.get("rotation").and_then(parse_number) {
        opts.rotation = Some(Rc::new(ConstantProperty::from_value(rot)));
    }
    if let Some(aa) = val.get("alignedAxis").and_then(parse_cartesian3) {
        opts.aligned_axis = Some(Rc::new(ConstantProperty::from_value(aa)));
    }
    if let Some(sbd) = val.get("scaleByDistance").and_then(parse_near_far_scalar) {
        opts.scale_by_distance = Some(Rc::new(ConstantProperty::from_value(sbd)));
    }
    if let Some(tbd) = val
        .get("translucencyByDistance")
        .and_then(parse_near_far_scalar)
    {
        opts.translucency_by_distance = Some(Rc::new(ConstantProperty::from_value(tbd)));
    }
    if let Some(ddc) = val
        .get("distanceDisplayCondition")
        .and_then(parse_distance_display_condition)
    {
        opts.distance_display_condition = Some(Rc::new(ConstantProperty::from_value(ddc)));
    }

    Ok(BillboardGraphics::from_options(opts))
}

fn parse_label(val: &Value) -> Result<LabelGraphics, DeveloperError> {
    let mut opts = LabelGraphicsOptions::default();

    if let Some(text) = val
        .get("text")
        .and_then(|v| v.as_str().or_else(|| v.get("string").and_then(|s| s.as_str())))
    {
        opts.text = Some(Rc::new(ConstantProperty::from_value(text.to_string())));
    }
    if let Some(font) = val.get("font").and_then(|v| v.as_str()) {
        opts.font = Some(Rc::new(ConstantProperty::from_value(font.to_string())));
    }
    if let Some(style_str) = val.get("style").and_then(|v| v.as_str()) {
        let s = match style_str {
            "OUTLINE" => LabelStyle::Outline,
            "FILL_AND_OUTLINE" => LabelStyle::FillAndOutline,
            _ => LabelStyle::Fill,
        };
        opts.style = Some(Rc::new(ConstantProperty::from_value(s)));
    }
    if let Some(fc) = val.get("fillColor") {
        opts.fill_color = parse_color_property(fc)?;
    }
    if let Some(oc) = val.get("outlineColor") {
        opts.outline_color = parse_color_property(oc)?;
    }
    if let Some(ow) = val.get("outlineWidth").and_then(parse_number) {
        opts.outline_width = Some(Rc::new(ConstantProperty::from_value(ow)));
    }
    if let Some(show) = val.get("show").and_then(parse_boolean) {
        opts.show = Some(Rc::new(ConstantProperty::from_value(show)));
    }
    if let Some(scale) = val.get("scale").and_then(parse_number) {
        opts.scale = Some(Rc::new(ConstantProperty::from_value(scale)));
    }
    if let Some(po) = val.get("pixelOffset").and_then(parse_cartesian2) {
        opts.pixel_offset = Some(Rc::new(ConstantProperty::from_value(po)));
    }

    Ok(LabelGraphics::from_options(opts))
}

fn parse_point(val: &Value) -> Result<PointGraphics, DeveloperError> {
    let mut opts = PointGraphicsOptions::default();

    if let Some(c) = val.get("color") {
        opts.color = parse_color_property(c)?;
    }
    if let Some(ps) = val.get("pixelSize").and_then(parse_number) {
        opts.pixel_size = Some(Rc::new(ConstantProperty::from_value(ps)));
    }
    if let Some(oc) = val.get("outlineColor") {
        opts.outline_color = parse_color_property(oc)?;
    }
    if let Some(ow) = val.get("outlineWidth").and_then(parse_number) {
        opts.outline_width = Some(Rc::new(ConstantProperty::from_value(ow)));
    }
    if let Some(show) = val.get("show").and_then(parse_boolean) {
        opts.show = Some(Rc::new(ConstantProperty::from_value(show)));
    }

    Ok(PointGraphics::from_options(opts))
}

fn parse_polyline(
    val: &Value,
    ellipsoid: &Ellipsoid,
) -> Result<PolylineGraphics, DeveloperError> {
    let mut opts = PolylineGraphicsOptions::default();

    if let Some(pos_val) = val.get("positions") {
        let positions = parse_positions_array(pos_val, ellipsoid)?;
        opts.positions = Some(Rc::new(ConstantProperty::from_value(positions)));
    }
    if let Some(w) = val.get("width").and_then(parse_number) {
        opts.width = Some(Rc::new(ConstantProperty::from_value(w)));
    }
    if let Some(mat_val) = val.get("material") {
        opts.material = parse_material(mat_val)?;
    }
    if let Some(ctg) = val.get("clampToGround").and_then(parse_boolean) {
        opts.clamp_to_ground = Some(Rc::new(ConstantProperty::from_value(ctg)));
    }
    if let Some(show) = val.get("show").and_then(parse_boolean) {
        opts.show = Some(Rc::new(ConstantProperty::from_value(show)));
    }
    if let Some(at) = val.get("arcType").and_then(|v| v.as_str()) {
        let arc = match at {
            "NONE" => ArcType::None,
            "RHUMB" => ArcType::Rhumb,
            _ => ArcType::Geodesic,
        };
        opts.arc_type = Some(Rc::new(ConstantProperty::from_value(arc)));
    }

    Ok(PolylineGraphics::from_options(opts))
}

fn parse_polygon(val: &Value, ellipsoid: &Ellipsoid) -> Result<PolygonGraphics, DeveloperError> {
    let mut opts = PolygonGraphicsOptions::default();

    if let Some(pos_val) = val.get("positions") {
        let positions = parse_positions_array(pos_val, ellipsoid)?;
        let mut hierarchy = PolygonHierarchy::new(positions);

        if let Some(holes_val) = val.get("holes").and_then(|v| v.as_object()) {
            for key in ["cartesian", "cartographicDegrees", "cartographicRadians"] {
                if let Some(Value::Array(hole_arr)) = holes_val.get(key) {
                    if hole_arr.first().is_some_and(|v| v.is_array()) {
                        for sub in hole_arr {
                            let obj = serde_json::json!({ key: sub });
                            let pts = parse_positions_array(&obj, ellipsoid)?;
                            if !pts.is_empty() {
                                hierarchy.holes.push(PolygonHierarchy::new(pts));
                            }
                        }
                    } else {
                        let obj = serde_json::json!({ key: hole_arr });
                        let pts = parse_positions_array(&obj, ellipsoid)?;
                        if !pts.is_empty() {
                            hierarchy.holes.push(PolygonHierarchy::new(pts));
                        }
                    }
                    break;
                }
            }
        }

        opts.hierarchy = Some(Rc::new(ConstantProperty::from_value(hierarchy)));
    }
    if let Some(mat_val) = val.get("material") {
        opts.material = parse_material(mat_val)?;
    }
    if let Some(fill) = val.get("fill").and_then(parse_boolean) {
        opts.fill = Some(Rc::new(ConstantProperty::from_value(fill)));
    }
    if let Some(outline) = val.get("outline").and_then(parse_boolean) {
        opts.outline = Some(Rc::new(ConstantProperty::from_value(outline)));
    }
    if let Some(oc) = val.get("outlineColor") {
        opts.outline_color = parse_color_property(oc)?;
    }
    if let Some(ow) = val.get("outlineWidth").and_then(parse_number) {
        opts.outline_width = Some(Rc::new(ConstantProperty::from_value(ow)));
    }
    if let Some(h) = val.get("height").and_then(parse_number) {
        opts.height = Some(Rc::new(ConstantProperty::from_value(h)));
    }
    if let Some(eh) = val.get("extrudedHeight").and_then(parse_number) {
        opts.extruded_height = Some(Rc::new(ConstantProperty::from_value(eh)));
    }
    if let Some(pph) = val.get("perPositionHeight").and_then(parse_boolean) {
        opts.per_position_height = Some(Rc::new(ConstantProperty::from_value(pph)));
    }
    if let Some(show) = val.get("show").and_then(parse_boolean) {
        opts.show = Some(Rc::new(ConstantProperty::from_value(show)));
    }

    Ok(PolygonGraphics::from_options(opts))
}

fn parse_ellipse(val: &Value) -> Result<EllipseGraphics, DeveloperError> {
    let mut opts = EllipseGraphicsOptions::default();

    if let Some(sma) = val.get("semiMajorAxis").and_then(parse_number) {
        opts.semi_major_axis = Some(Rc::new(ConstantProperty::from_value(sma)));
    }
    if let Some(smi) = val.get("semiMinorAxis").and_then(parse_number) {
        opts.semi_minor_axis = Some(Rc::new(ConstantProperty::from_value(smi)));
    }
    if let Some(h) = val.get("height").and_then(parse_number) {
        opts.height = Some(Rc::new(ConstantProperty::from_value(h)));
    }
    if let Some(eh) = val.get("extrudedHeight").and_then(parse_number) {
        opts.extruded_height = Some(Rc::new(ConstantProperty::from_value(eh)));
    }
    if let Some(rot) = val.get("rotation").and_then(parse_number) {
        opts.rotation = Some(Rc::new(ConstantProperty::from_value(rot)));
    }
    if let Some(mat_val) = val.get("material") {
        opts.material = parse_material(mat_val)?;
    }
    if let Some(fill) = val.get("fill").and_then(parse_boolean) {
        opts.fill = Some(Rc::new(ConstantProperty::from_value(fill)));
    }
    if let Some(outline) = val.get("outline").and_then(parse_boolean) {
        opts.outline = Some(Rc::new(ConstantProperty::from_value(outline)));
    }
    if let Some(oc) = val.get("outlineColor") {
        opts.outline_color = parse_color_property(oc)?;
    }
    if let Some(show) = val.get("show").and_then(parse_boolean) {
        opts.show = Some(Rc::new(ConstantProperty::from_value(show)));
    }

    Ok(EllipseGraphics::from_options(opts))
}

fn parse_box(val: &Value) -> Result<BoxGraphics, DeveloperError> {
    let mut opts = BoxGraphicsOptions::default();

    if let Some(dim) = val.get("dimensions").and_then(parse_cartesian3) {
        opts.dimensions = Some(Rc::new(ConstantProperty::from_value(dim)));
    }
    if let Some(mat_val) = val.get("material") {
        opts.material = parse_material(mat_val)?;
    }
    if let Some(fill) = val.get("fill").and_then(parse_boolean) {
        opts.fill = Some(Rc::new(ConstantProperty::from_value(fill)));
    }
    if let Some(outline) = val.get("outline").and_then(parse_boolean) {
        opts.outline = Some(Rc::new(ConstantProperty::from_value(outline)));
    }
    if let Some(oc) = val.get("outlineColor") {
        opts.outline_color = parse_color_property(oc)?;
    }
    if let Some(show) = val.get("show").and_then(parse_boolean) {
        opts.show = Some(Rc::new(ConstantProperty::from_value(show)));
    }

    Ok(BoxGraphics::from_options(opts))
}

fn parse_path(val: &Value) -> Result<PathGraphics, DeveloperError> {
    let mut opts = PathGraphicsOptions::default();

    if let Some(mat_val) = val.get("material") {
        opts.material = parse_material(mat_val)?;
    }
    if let Some(w) = val.get("width").and_then(parse_number) {
        opts.width = Some(Rc::new(ConstantProperty::from_value(w)));
    }
    if let Some(res) = val.get("resolution").and_then(parse_number) {
        opts.resolution = Some(Rc::new(ConstantProperty::from_value(res)));
    }
    if let Some(lt) = val.get("leadTime").and_then(parse_number) {
        opts.lead_time = Some(Rc::new(ConstantProperty::from_value(lt)));
    }
    if let Some(tt) = val.get("trailTime").and_then(parse_number) {
        opts.trail_time = Some(Rc::new(ConstantProperty::from_value(tt)));
    }
    if let Some(show) = val.get("show").and_then(parse_boolean) {
        opts.show = Some(Rc::new(ConstantProperty::from_value(show)));
    }

    Ok(PathGraphics::from_options(opts))
}
