//! KmlDataSource matching CesiumJS `DataSources/KmlDataSource.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;

use serde_json::Value;

use crate::xml::{parse_xml, XmlNode};
use crate::{
    create_guid, BillboardGraphics, Cartesian2, Cartesian3, Color, ColorMaterialProperty,
    ConstantPositionProperty, ConstantProperty, DataSourceClock, DeveloperError, Ellipsoid,
    Entity, EntityCluster, EntityCollection, Event, HeightReference, HorizontalOrigin, Iso8601,
    JulianDate, LabelGraphics, PathGraphics, PolygonGraphics, PolygonHierarchy, PolylineGraphics,
    Rectangle, RectangleGraphics, SampledPositionProperty, TimeInterval, TimeIntervalCollection,
    VerticalOrigin,
};

#[derive(Debug, Clone, Default)]
pub struct KmlLoadOptions {
    pub clamp_to_ground: bool,
    pub source_uri: Option<String>,
    pub ellipsoid: Option<Ellipsoid>,
}

#[derive(Debug, Clone, Default)]
struct KmlIconStyle {
    color: Option<Color>,
    scale: Option<f64>,
    heading: Option<f64>,
    icon_href: Option<String>,
    has_icon_tag: bool,
    hot_spot_x: Option<f64>,
    hot_spot_y: Option<f64>,
}

#[derive(Debug, Clone, Default)]
struct KmlLabelStyle {
    color: Option<Color>,
    scale: Option<f64>,
}

#[derive(Debug, Clone, Default)]
struct KmlLineStyle {
    color: Option<Color>,
    width: Option<f64>,
}

#[derive(Debug, Clone, Default)]
struct KmlPolyStyle {
    color: Option<Color>,
    fill: Option<bool>,
    outline: Option<bool>,
}

#[derive(Debug, Clone, Default)]
struct KmlStyle {
    icon: Option<KmlIconStyle>,
    label: Option<KmlLabelStyle>,
    line: Option<KmlLineStyle>,
    poly: Option<KmlPolyStyle>,
}

pub struct KmlDataSource {
    name: Option<String>,
    clock: Option<DataSourceClock>,
    entities: EntityCollection,
    is_loading: bool,
    clustering: EntityCluster,
    credit: Option<String>,
    pub changed_event: Event<()>,
    pub error_event: Event<String>,
    pub loading_event: Event<bool>,
    pub unsupported_node_event: Event<String>,
    pub refresh_event: Event<String>,
}

impl Default for KmlDataSource {
    fn default() -> Self {
        Self::new(None)
    }
}

impl KmlDataSource {
    pub fn new(name: Option<&str>) -> Self {
        Self {
            name: name.map(|s| s.to_string()),
            clock: None,
            entities: EntityCollection::new(),
            is_loading: false,
            clustering: EntityCluster::default(),
            credit: None,
            changed_event: Event::new(),
            error_event: Event::new(),
            loading_event: Event::new(),
            unsupported_node_event: Event::new(),
            refresh_event: Event::new(),
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn set_name(&mut self, name: &str) {
        if self.name.as_deref() != Some(name) {
            self.name = Some(name.to_string());
            self.changed_event.raise_event(&());
        }
    }

    pub fn clock(&self) -> Option<&DataSourceClock> {
        self.clock.as_ref()
    }

    pub fn set_clock(&mut self, clock: Option<DataSourceClock>) {
        self.clock = clock;
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

    pub fn set_show(&mut self, value: bool) {
        self.entities.set_show(value);
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

    pub fn update(&mut self, _time: &JulianDate) -> bool {
        true
    }

    pub fn load_str(
        &mut self,
        xml: &str,
        options: Option<KmlLoadOptions>,
    ) -> Result<(), DeveloperError> {
        if xml.trim().is_empty() {
            let err = DeveloperError::new("KML data is required.");
            self.error_event.raise_event(&err.to_string());
            return Err(err);
        }

        self.set_loading(true);
        let res = self.load_str_inner(xml, options.unwrap_or_default());
        if let Err(ref e) = res {
            self.error_event.raise_event(&e.to_string());
        } else {
            self.changed_event.raise_event(&());
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

    fn load_str_inner(
        &mut self,
        xml: &str,
        options: KmlLoadOptions,
    ) -> Result<(), DeveloperError> {
        let root = parse_xml(xml)?;
        if root.local_name() != "kml" {
            return Err(DeveloperError::new("Expected <kml> root element."));
        }

        self.entities.suspend_events();
        self.entities.remove_all();
        self.clock = None;

        let mut styles: HashMap<String, KmlStyle> = HashMap::new();
        let mut style_maps: HashMap<String, String> = HashMap::new();
        collect_styles_recursive(&root, &mut styles, &mut style_maps);

        // Resolve style maps into styles
        for (map_id, target_url) in &style_maps {
            let key = target_url.strip_prefix('#').unwrap_or(target_url);
            if let Some(resolved) = styles.get(key).cloned() {
                styles.insert(map_id.clone(), resolved);
            }
        }

        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);

        // Find root feature node(s)
        let mut doc_name: Option<String> = None;
        for child in &root.children {
            match child.local_name() {
                "Document" | "Folder" | "Placemark" | "GroundOverlay" | "ScreenOverlay"
                | "NetworkLink" => {
                    if doc_name.is_none() {
                        doc_name = child.child_text("name");
                    }
                    self.process_node(child, &styles, &options, &ellipsoid, true)?;
                }
                _ => {}
            }
        }

        if let Some(name) = doc_name {
            if !name.is_empty() {
                self.name = Some(name);
            }
        } else if let Some(ref uri) = options.source_uri {
            let fallback = uri.rsplit('/').next().unwrap_or(uri);
            if !fallback.is_empty() {
                self.name = Some(fallback.to_string());
            }
        }

        let _ = self.entities.resume_events();

        // Compute DataSourceClock from entity availability
        let avail = self.entities.compute_availability();
        let min_iso = Iso8601::minimum_value();
        let max_iso = Iso8601::maximum_value();
        if !avail.start.equals(&min_iso) && !avail.stop.equals(&max_iso) {
            let mut clk = DataSourceClock::new();
            clk.start_time = Some(avail.start);
            clk.stop_time = Some(avail.stop);
            clk.current_time = Some(avail.start);
            self.clock = Some(clk);
        }

        Ok(())
    }

    fn process_node(
        &mut self,
        node: &XmlNode,
        styles: &HashMap<String, KmlStyle>,
        options: &KmlLoadOptions,
        ellipsoid: &Ellipsoid,
        parent_show: bool,
    ) -> Result<(), DeveloperError> {
        let local = node.local_name();
        let node_show = parent_show && parse_visibility(node).unwrap_or(true);

        match local {
            "Document" | "Folder" => {
                for child in &node.children {
                    match child.local_name() {
                        "Document" | "Folder" | "Placemark" | "GroundOverlay"
                        | "ScreenOverlay" | "NetworkLink" => {
                            self.process_node(child, styles, options, ellipsoid, node_show)?;
                        }
                        "Style" | "StyleMap" | "name" | "description" | "visibility"
                        | "open" | "Snippet" | "address" | "phoneNumber" | "ExtendedData"
                        | "TimeSpan" | "TimeStamp" | "LookAt" | "Camera" | "Region" => {}
                        other => {
                            self.unsupported_node_event.raise_event(&other.to_string());
                        }
                    }
                }
            }
            "Placemark" => {
                let entity =
                    parse_placemark(node, styles, options, ellipsoid, node_show, &self.entities)?;
                let _ = self.entities.add(entity);
            }
            "GroundOverlay" => {
                let entity =
                    parse_ground_overlay(node, options, ellipsoid, node_show, &self.entities)?;
                let _ = self.entities.add(entity);
            }
            "ScreenOverlay" => {
                // ScreenOverlay creates an entity tracking screen overlay metadata
                let mut entity = create_feature_entity(node, node_show, &self.entities);
                if entity.name.is_none() {
                    entity.name = Some("ScreenOverlay".to_string());
                }
                let _ = self.entities.add(entity);
            }
            "NetworkLink" => {
                if let Some(link) = node.child("Link").or_else(|| node.child("Url")) {
                    if let Some(href) = link.child_text("href") {
                        self.refresh_event.raise_event(&href);
                    }
                }
            }
            other => {
                self.unsupported_node_event.raise_event(&other.to_string());
            }
        }
        Ok(())
    }
}

fn collect_styles_recursive(
    node: &XmlNode,
    styles: &mut HashMap<String, KmlStyle>,
    style_maps: &mut HashMap<String, String>,
) {
    match node.local_name() {
        "Style" => {
            if let Some(id) = node.attr("id") {
                styles.insert(id.to_string(), parse_style(node));
            }
        }
        "StyleMap" => {
            if let Some(id) = node.attr("id") {
                for pair in node.children_named("Pair") {
                    if pair.child_text("key").as_deref() == Some("normal") {
                        if let Some(url) = pair.child_text("styleUrl") {
                            style_maps.insert(id.to_string(), url);
                            break;
                        }
                    }
                }
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect_styles_recursive(child, styles, style_maps);
    }
}

/// Parses KML `aabbggrr` hex color string into Cesium `Color`.
pub(crate) fn parse_kml_color(s: &str) -> Option<Color> {
    let hex = s.trim().trim_start_matches('#');
    if hex.len() != 8 {
        return None;
    }
    let a = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
    let r = u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0;
    Some(Color::new(r, g, b, a))
}

fn parse_kml_bool(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    }
}

fn parse_visibility(node: &XmlNode) -> Option<bool> {
    node.child_text("visibility").and_then(|s| parse_kml_bool(&s))
}

fn parse_style(node: &XmlNode) -> KmlStyle {
    let icon = node.child("IconStyle").map(|is| {
        let icon_node = is.child("Icon");
        let has_icon_tag = icon_node.is_some();
        let icon_href = icon_node.and_then(|ic| ic.child_text("href")).filter(|h| !h.is_empty());
        let hot_spot = is.child("hotSpot");
        KmlIconStyle {
            color: is.child_text("color").and_then(|s| parse_kml_color(&s)),
            scale: is.child_text("scale").and_then(|s| s.parse::<f64>().ok()),
            heading: is.child_text("heading").and_then(|s| s.parse::<f64>().ok()),
            icon_href,
            has_icon_tag,
            hot_spot_x: hot_spot.and_then(|h| h.attr("x")).and_then(|s| s.parse().ok()),
            hot_spot_y: hot_spot.and_then(|h| h.attr("y")).and_then(|s| s.parse().ok()),
        }
    });

    let label = node.child("LabelStyle").map(|ls| KmlLabelStyle {
        color: ls.child_text("color").and_then(|s| parse_kml_color(&s)),
        scale: ls.child_text("scale").and_then(|s| s.parse::<f64>().ok()),
    });

    let line = node.child("LineStyle").map(|ls| KmlLineStyle {
        color: ls.child_text("color").and_then(|s| parse_kml_color(&s)),
        width: ls.child_text("width").and_then(|s| s.parse::<f64>().ok()),
    });

    let poly = node.child("PolyStyle").map(|ps| KmlPolyStyle {
        color: ps.child_text("color").and_then(|s| parse_kml_color(&s)),
        fill: ps.child_text("fill").and_then(|s| parse_kml_bool(&s)),
        outline: ps.child_text("outline").and_then(|s| parse_kml_bool(&s)),
    });

    KmlStyle {
        icon,
        label,
        line,
        poly,
    }
}

fn resolve_style(node: &XmlNode, styles: &HashMap<String, KmlStyle>) -> KmlStyle {
    let mut result = KmlStyle::default();
    if let Some(url) = node.child_text("styleUrl") {
        let key = url.trim().strip_prefix('#').unwrap_or(url.trim());
        if let Some(s) = styles.get(key) {
            result = s.clone();
        }
    }
    if let Some(inline) = node.child("Style") {
        let inl = parse_style(inline);
        if inl.icon.is_some() {
            result.icon = inl.icon;
        }
        if inl.label.is_some() {
            result.label = inl.label;
        }
        if inl.line.is_some() {
            result.line = inl.line;
        }
        if inl.poly.is_some() {
            result.poly = inl.poly;
        }
    }
    result
}

fn create_feature_entity(
    node: &XmlNode,
    show: bool,
    existing: &EntityCollection,
) -> Entity {
    let raw_id = node.attr("id").filter(|s| !s.is_empty());
    let id = match raw_id {
        Some(id_str) => {
            if existing.get_by_id(id_str).is_some() {
                format!("{}_{}", id_str, create_guid())
            } else {
                id_str.to_string()
            }
        }
        None => create_guid(),
    };

    let mut entity = Entity::with_id(&id);
    entity.show = show;
    entity.name = node.child_text("name").filter(|s| !s.is_empty());

    if let Some(desc) = build_feature_description(node) {
        entity.description = Some(ConstantProperty::from_value(desc));
    }

    // Availability from TimeSpan or TimeStamp
    if let Some(ts) = node.child("TimeSpan") {
        let begin_opt = ts
            .child_text("begin")
            .filter(|s| !s.is_empty())
            .and_then(|s| JulianDate::from_iso8601(&s).ok());
        let end_opt = ts
            .child_text("end")
            .filter(|s| !s.is_empty())
            .and_then(|s| JulianDate::from_iso8601(&s).ok());
        if begin_opt.is_some() || end_opt.is_some() {
            let mut start = begin_opt.unwrap_or_else(Iso8601::minimum_value);
            let mut stop = end_opt.unwrap_or_else(Iso8601::maximum_value);
            if stop.compare(&start) == std::cmp::Ordering::Less {
                std::mem::swap(&mut start, &mut stop);
            }
            let mut avail = TimeIntervalCollection::new();
            avail.add_interval(TimeInterval::new(start, stop, true, true));
            entity.availability = Some(avail);
        }
    } else if let Some(ts) = node.child("TimeStamp") {
        if let Some(when_str) = ts.child_text("when").filter(|s| !s.is_empty()) {
            if let Ok(when) = JulianDate::from_iso8601(&when_str) {
                let mut avail = TimeIntervalCollection::new();
                avail.add_interval(TimeInterval::new(when, when, true, true));
                entity.availability = Some(avail);
            }
        }
    }

    // ExtendedData <Data name="..."><value>...</value></Data>
    if let Some(ext) = node.child("ExtendedData") {
        for data in ext.children_named("Data") {
            if let Some(key) = data.attr("name") {
                let val_str = data.child_text("value").unwrap_or_default();
                entity.properties.add_property(
                    key,
                    Some(Rc::new(ConstantProperty::from_value(Value::String(val_str)))),
                );
            }
        }
    }

    entity
}

fn build_feature_description(node: &XmlNode) -> Option<String> {
    if let Some(desc) = node.child_text("description").filter(|s| !s.is_empty()) {
        return Some(desc);
    }
    let mut parts = Vec::new();
    if let Some(snippet) = node.child_text("Snippet").filter(|s| !s.is_empty()) {
        parts.push(snippet);
    }
    if let Some(addr) = node.child_text("address").filter(|s| !s.is_empty()) {
        parts.push(format!("Address: {}", addr));
    }
    if let Some(phone) = node.child_text("phoneNumber").filter(|s| !s.is_empty()) {
        parts.push(format!("Phone: {}", phone));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("<br/>"))
    }
}

fn parse_coordinates_list(text: &str, ellipsoid: &Ellipsoid) -> Vec<Cartesian3> {
    let mut pts = Vec::new();
    for token in text.split_whitespace() {
        let parts: Vec<&str> = token.split(',').collect();
        if parts.len() >= 2 {
            if let (Ok(lon), Ok(lat)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                let alt = parts.get(2).and_then(|s| s.parse::<f64>().ok());
                if let Ok(pos) = Cartesian3::from_degrees(lon, lat, alt, Some(ellipsoid)) {
                    pts.push(pos);
                }
            }
        }
    }
    pts
}

fn parse_coordinates_raw(text: &str) -> Vec<(f64, f64, Option<f64>)> {
    let mut pts = Vec::new();
    for token in text.split_whitespace() {
        let parts: Vec<&str> = token.split(',').collect();
        if parts.len() >= 2 {
            if let (Ok(lon), Ok(lat)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                let alt = parts.get(2).and_then(|s| s.parse::<f64>().ok());
                pts.push((lon, lat, alt));
            }
        }
    }
    pts
}

fn parse_placemark(
    node: &XmlNode,
    styles: &HashMap<String, KmlStyle>,
    options: &KmlLoadOptions,
    ellipsoid: &Ellipsoid,
    show: bool,
    existing: &EntityCollection,
) -> Result<Entity, DeveloperError> {
    let mut entity = create_feature_entity(node, show, existing);
    let style = resolve_style(node, styles);

    if let Some(geom) = node.child("Point") {
        apply_point(geom, &mut entity, &style, options, ellipsoid);
    } else if let Some(geom) = node.child("LineString") {
        apply_line_string(geom, &mut entity, &style, options, ellipsoid);
    } else if let Some(geom) = node.child("Polygon") {
        apply_polygon(geom, &mut entity, &style, options, ellipsoid);
    } else if let Some(geom) = node.child("Track") {
        apply_gx_track(geom, &mut entity, &style, options, ellipsoid);
    } else if let Some(geom) = node.child("MultiTrack") {
        if let Some(first_track) = geom.child("Track") {
            apply_gx_track(first_track, &mut entity, &style, options, ellipsoid);
        }
    } else if let Some(mg) = node.child("MultiGeometry") {
        for child in &mg.children {
            match child.local_name() {
                "Point" => apply_point(child, &mut entity, &style, options, ellipsoid),
                "LineString" => apply_line_string(child, &mut entity, &style, options, ellipsoid),
                "Polygon" => apply_polygon(child, &mut entity, &style, options, ellipsoid),
                "Track" => apply_gx_track(child, &mut entity, &style, options, ellipsoid),
                _ => {}
            }
        }
    }

    Ok(entity)
}

fn apply_point(
    node: &XmlNode,
    entity: &mut Entity,
    style: &KmlStyle,
    options: &KmlLoadOptions,
    ellipsoid: &Ellipsoid,
) {
    let coords_text = node.child_text("coordinates").unwrap_or_default();
    let pts = parse_coordinates_list(&coords_text, ellipsoid);
    if let Some(&pos) = pts.first() {
        entity.position = Some(ConstantPositionProperty::new(Some(pos), None));
    }

    let alt_mode = node
        .child_text("altitudeMode")
        .unwrap_or_else(|| "clampToGround".to_string());
    let clamp = options.clamp_to_ground || alt_mode == "clampToGround" || alt_mode == "clampToSeaFloor";

    let mut bb = BillboardGraphics::new();
    bb.vertical_origin = Some(Rc::new(ConstantProperty::from_value(
        VerticalOrigin::Bottom,
    )));
    bb.horizontal_origin = Some(Rc::new(ConstantProperty::from_value(
        HorizontalOrigin::Center,
    )));
    if clamp {
        bb.height_reference = Some(Rc::new(ConstantProperty::from_value(
            HeightReference::ClampToGround,
        )));
    } else if alt_mode == "relativeToGround" || alt_mode == "relativeToSeaFloor" {
        bb.height_reference = Some(Rc::new(ConstantProperty::from_value(
            HeightReference::RelativeToGround,
        )));
    }

    if let Some(ref icon) = style.icon {
        if let Some(href) = &icon.icon_href {
            bb.image = Some(Rc::new(ConstantProperty::from_value(href.clone())));
        } else if !icon.has_icon_tag {
            bb.image = Some(Rc::new(ConstantProperty::from_value("pin".to_string())));
        }
        if let Some(c) = icon.color {
            bb.color = Some(Rc::new(ConstantProperty::from_value(c)));
        }
        if let Some(s) = icon.scale {
            bb.scale = Some(Rc::new(ConstantProperty::from_value(s)));
        }
        if let Some(h) = icon.heading {
            bb.rotation = Some(Rc::new(ConstantProperty::from_value((-h).to_radians())));
        }
        if let (Some(x), Some(y)) = (icon.hot_spot_x, icon.hot_spot_y) {
            bb.pixel_offset = Some(Rc::new(ConstantProperty::from_value(Cartesian2::new(x, y))));
        }
    } else {
        bb.image = Some(Rc::new(ConstantProperty::from_value("pin".to_string())));
    }
    entity.billboard = Some(bb);

    if let Some(ref name) = entity.name {
        let mut lbl = LabelGraphics::new();
        lbl.text = Some(Rc::new(ConstantProperty::from_value(name.clone())));
        if let Some(ref ls) = style.label {
            if let Some(c) = ls.color {
                lbl.fill_color = Some(Rc::new(ConstantProperty::from_value(c)));
            }
            if let Some(s) = ls.scale {
                lbl.scale = Some(Rc::new(ConstantProperty::from_value(s)));
            }
        }
        entity.label = Some(lbl);
    }
}

fn apply_line_string(
    node: &XmlNode,
    entity: &mut Entity,
    style: &KmlStyle,
    options: &KmlLoadOptions,
    ellipsoid: &Ellipsoid,
) {
    let coords_text = node.child_text("coordinates").unwrap_or_default();
    let pts = parse_coordinates_list(&coords_text, ellipsoid);
    let alt_mode = node
        .child_text("altitudeMode")
        .unwrap_or_else(|| "clampToGround".to_string());
    let clamp = options.clamp_to_ground || alt_mode == "clampToGround" || alt_mode == "clampToSeaFloor";

    let mut pl = PolylineGraphics::new();
    pl.positions = Some(Rc::new(ConstantProperty::from_value(pts)));
    let width = style.line.as_ref().and_then(|l| l.width).unwrap_or(1.0);
    pl.width = Some(Rc::new(ConstantProperty::from_value(width)));
    let color = style.line.as_ref().and_then(|l| l.color).unwrap_or(Color::WHITE);
    pl.material = Some(Rc::new(ColorMaterialProperty::from_color(color)));
    if clamp {
        pl.clamp_to_ground = Some(Rc::new(ConstantProperty::from_value(true)));
    }
    entity.polyline = Some(pl);
}

fn apply_polygon(
    node: &XmlNode,
    entity: &mut Entity,
    style: &KmlStyle,
    options: &KmlLoadOptions,
    ellipsoid: &Ellipsoid,
) {
    let outer_text = node
        .child("outerBoundaryIs")
        .and_then(|ob| ob.child("LinearRing"))
        .and_then(|lr| lr.child_text("coordinates"))
        .unwrap_or_default();
    let raw_outer = parse_coordinates_raw(&outer_text);
    let outer_pts = parse_coordinates_list(&outer_text, ellipsoid);

    let mut hierarchy = PolygonHierarchy::new(outer_pts);
    for inner in node.children_named("innerBoundaryIs") {
        if let Some(coords) = inner
            .child("LinearRing")
            .and_then(|lr| lr.child_text("coordinates"))
        {
            let hole_pts = parse_coordinates_list(&coords, ellipsoid);
            if !hole_pts.is_empty() {
                hierarchy.holes.push(PolygonHierarchy::new(hole_pts));
            }
        }
    }

    let alt_mode = node
        .child_text("altitudeMode")
        .unwrap_or_else(|| "clampToGround".to_string());
    let extrude = node
        .child_text("extrude")
        .and_then(|s| parse_kml_bool(&s))
        .unwrap_or(false);
    let clamp = options.clamp_to_ground || alt_mode == "clampToGround" || alt_mode == "clampToSeaFloor";

    let mut pg = PolygonGraphics::new();
    pg.hierarchy = Some(Rc::new(ConstantProperty::from_value(hierarchy)));

    let fill = style.poly.as_ref().and_then(|p| p.fill).unwrap_or(true);
    let outline = style.poly.as_ref().and_then(|p| p.outline).unwrap_or(true);
    let poly_color = style.poly.as_ref().and_then(|p| p.color).unwrap_or(Color::WHITE);
    let outline_color = style.line.as_ref().and_then(|l| l.color).unwrap_or(Color::WHITE);

    pg.fill = Some(Rc::new(ConstantProperty::from_value(fill)));
    pg.outline = Some(Rc::new(ConstantProperty::from_value(outline)));
    pg.material = Some(Rc::new(ColorMaterialProperty::from_color(poly_color)));
    pg.outline_color = Some(Rc::new(ConstantProperty::from_value(outline_color)));

    if !clamp {
        let has_alt = raw_outer.iter().any(|(_, _, a)| a.is_some());
        if has_alt {
            pg.per_position_height = Some(Rc::new(ConstantProperty::from_value(true)));
        } else if !extrude {
            pg.height = Some(Rc::new(ConstantProperty::from_value(0.0)));
        }
        if extrude {
            pg.extruded_height = Some(Rc::new(ConstantProperty::from_value(0.0)));
        }
    }
    entity.polygon = Some(pg);
}

fn apply_gx_track(
    node: &XmlNode,
    entity: &mut Entity,
    style: &KmlStyle,
    options: &KmlLoadOptions,
    ellipsoid: &Ellipsoid,
) {
    let mut times = Vec::new();
    for w in node.children_named("when") {
        let text = w.all_text();
        if let Ok(jd) = JulianDate::from_iso8601(&text) {
            times.push(jd);
        }
    }

    let mut coords = Vec::new();
    for c in node.children_named("coord") {
        let text = c.all_text();
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() >= 2 {
            if let (Ok(lon), Ok(lat)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                let alt = parts.get(2).and_then(|s| s.parse::<f64>().ok());
                if let Ok(pos) = Cartesian3::from_degrees(lon, lat, alt, Some(ellipsoid)) {
                    coords.push(pos);
                }
            }
        }
    }

    let count = times.len().min(coords.len());
    if count > 0 {
        let mut sampled = SampledPositionProperty::new(None);
        for i in 0..count {
            sampled.add_sample(times[i], coords[i]);
        }
        entity.position = Some(ConstantPositionProperty::from_dynamic(Rc::new(sampled)));

        let mut avail = TimeIntervalCollection::new();
        avail.add_interval(TimeInterval::new(times[0], times[count - 1], true, true));
        entity.availability = Some(avail);

        let mut path = PathGraphics::new();
        let width = style.line.as_ref().and_then(|l| l.width).unwrap_or(1.0);
        path.width = Some(Rc::new(ConstantProperty::from_value(width)));
        let color = style.line.as_ref().and_then(|l| l.color).unwrap_or(Color::WHITE);
        path.material = Some(Rc::new(ColorMaterialProperty::from_color(color)));
        entity.path = Some(path);
    } else if !coords.is_empty() {
        let mut pl = PolylineGraphics::new();
        pl.positions = Some(Rc::new(ConstantProperty::from_value(coords)));
        if options.clamp_to_ground {
            pl.clamp_to_ground = Some(Rc::new(ConstantProperty::from_value(true)));
        }
        entity.polyline = Some(pl);
    }
}

fn parse_ground_overlay(
    node: &XmlNode,
    options: &KmlLoadOptions,
    ellipsoid: &Ellipsoid,
    show: bool,
    existing: &EntityCollection,
) -> Result<Entity, DeveloperError> {
    let mut entity = create_feature_entity(node, show, existing);
    let color = node
        .child_text("color")
        .and_then(|s| parse_kml_color(&s))
        .unwrap_or(Color::WHITE);
    let icon_href = node
        .child("Icon")
        .and_then(|ic| ic.child_text("href"))
        .filter(|s| !s.is_empty());

    if let Some(latlonquad) = node.child("LatLonQuad") {
        let coords_text = latlonquad.child_text("coordinates").unwrap_or_default();
        let pts = parse_coordinates_list(&coords_text, ellipsoid);
        let mut pg = PolygonGraphics::new();
        pg.hierarchy = Some(Rc::new(ConstantProperty::from_value(PolygonHierarchy::new(pts))));
        pg.material = Some(Rc::new(ColorMaterialProperty::from_color(color)));
        if icon_href.is_some() {
            // Note: image material URI recorded on polygon
            pg.fill = Some(Rc::new(ConstantProperty::from_value(true)));
        }
        entity.polygon = Some(pg);
        return Ok(entity);
    }

    if let Some(box_node) = node.child("LatLonBox") {
        let north = box_node
            .child_text("north")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let south = box_node
            .child_text("south")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let east = box_node
            .child_text("east")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let west = box_node
            .child_text("west")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let rotation = box_node
            .child_text("rotation")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let rect = Rectangle::from_degrees(west, south, east, north);
        let mut rg = RectangleGraphics::new();
        rg.coordinates = Some(Rc::new(ConstantProperty::from_value(rect)));
        rg.material = Some(Rc::new(ColorMaterialProperty::from_color(color)));
        if rotation.abs() > 0.0 {
            rg.rotation = Some(Rc::new(ConstantProperty::from_value(rotation.to_radians())));
        }
        if !options.clamp_to_ground {
            if let Some(alt) = node
                .child_text("altitude")
                .and_then(|s| s.parse::<f64>().ok())
            {
                if node.child_text("altitudeMode").as_deref() == Some("absolute") {
                    rg.height = Some(Rc::new(ConstantProperty::from_value(alt)));
                }
            }
        }
        entity.rectangle = Some(rg);
    }

    Ok(entity)
}
