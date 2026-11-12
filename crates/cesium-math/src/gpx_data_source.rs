//! GpxDataSource matching CesiumJS `DataSources/GpxDataSource.js` (Apache-2.0).

use std::rc::Rc;

use crate::xml::{parse_xml, XmlNode};
use crate::{
    BillboardGraphics, Cartesian3, Color, ConstantPositionProperty, ConstantProperty,
    DataSourceClock, DeveloperError, Entity, EntityCluster, EntityCollection, Event,
    HeightReference, JulianDate, PolylineGraphics, PolylineOutlineMaterialProperty,
    SampledPositionProperty, TimeInterval, TimeIntervalCollection, VerticalOrigin,
};

const BILLBOARD_SIZE: f64 = 32.0;

#[derive(Debug, Clone, Default)]
pub struct GpxLoadOptions {
    pub clamp_to_ground: bool,
    pub waypoint_image: Option<String>,
    pub track_image: Option<String>,
    pub track_color: Option<Color>,
    pub route_color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GpxLink {
    pub href: Option<String>,
    pub text: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GpxPerson {
    pub name: Option<String>,
    pub email: Option<String>,
    pub link: Option<GpxLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GpxCopyright {
    pub author: Option<String>,
    pub year: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GpxBounds {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GpxMetadata {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub author: Option<GpxPerson>,
    pub copyright: Option<GpxCopyright>,
    pub link: Option<GpxLink>,
    pub time: Option<String>,
    pub keywords: Option<String>,
    pub bounds: Option<GpxBounds>,
}

pub struct GpxDataSource {
    name: Option<String>,
    clock: Option<DataSourceClock>,
    entities: EntityCollection,
    is_loading: bool,
    clustering: EntityCluster,
    credit: Option<String>,
    version: Option<String>,
    creator: Option<String>,
    metadata: Option<GpxMetadata>,
    pub changed_event: Event<()>,
    pub error_event: Event<String>,
    pub loading_event: Event<bool>,
}

impl Default for GpxDataSource {
    fn default() -> Self {
        Self::new(None)
    }
}

impl GpxDataSource {
    pub fn new(name: Option<&str>) -> Self {
        Self {
            name: name.map(|s| s.to_string()),
            clock: None,
            entities: EntityCollection::new(),
            is_loading: false,
            clustering: EntityCluster::default(),
            credit: None,
            version: None,
            creator: None,
            metadata: None,
            changed_event: Event::new(),
            error_event: Event::new(),
            loading_event: Event::new(),
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

    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    pub fn creator(&self) -> Option<&str> {
        self.creator.as_deref()
    }

    pub fn metadata(&self) -> Option<&GpxMetadata> {
        self.metadata.as_ref()
    }

    pub fn update(&mut self, _time: &JulianDate) -> bool {
        true
    }

    pub fn load_str(
        &mut self,
        xml: &str,
        options: Option<GpxLoadOptions>,
    ) -> Result<(), DeveloperError> {
        if xml.trim().is_empty() {
            let err = DeveloperError::new("GPX data is required.");
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
        options: GpxLoadOptions,
    ) -> Result<(), DeveloperError> {
        let root = parse_xml(xml)?;
        if root.local_name() != "gpx" {
            return Err(DeveloperError::new("Expected <gpx> root element."));
        }

        self.entities.suspend_events();
        self.entities.remove_all();

        self.version = root.attr("version").map(|s| s.to_string());
        self.creator = root.attr("creator").map(|s| s.to_string());
        self.metadata = root.child("metadata").map(parse_metadata);

        if let Some(ref meta) = self.metadata {
            if let Some(ref meta_name) = meta.name {
                self.name = Some(meta_name.clone());
            }
        }

        // Waypoints
        for wpt in root.children_named("wpt") {
            let entity = parse_waypoint(wpt, &options)?;
            let _ = self.entities.add(entity);
        }

        // Routes
        for rte in root.children_named("rte") {
            let route_entities = parse_route(rte, &options)?;
            for entity in route_entities {
                let _ = self.entities.add(entity);
            }
        }

        // Tracks
        for trk in root.children_named("trk") {
            let track_entities = parse_track(trk, &options)?;
            for entity in track_entities {
                let _ = self.entities.add(entity);
            }
        }

        let _ = self.entities.resume_events();
        Ok(())
    }
}

fn parse_link(node: &XmlNode) -> GpxLink {
    GpxLink {
        href: node.attr("href").map(|s| s.to_string()),
        text: node.child_text("text"),
        mime_type: node.child_text("type"),
    }
}

fn parse_person(node: &XmlNode) -> GpxPerson {
    let email = node.child("email").and_then(|em| {
        let id = em
            .attr("id")
            .map(|s| s.to_string())
            .or_else(|| em.child_text("id"));
        let domain = em
            .attr("domain")
            .map(|s| s.to_string())
            .or_else(|| em.child_text("domain"));
        match (id, domain) {
            (Some(i), Some(d)) => Some(format!("{}@{}", i, d)),
            _ => None,
        }
    });
    GpxPerson {
        name: node.child_text("name"),
        email,
        link: node.child("link").map(parse_link),
    }
}

fn parse_copyright(node: &XmlNode) -> GpxCopyright {
    GpxCopyright {
        author: node.attr("author").map(|s| s.to_string()),
        year: node.child_text("year"),
        license: node.child_text("license"),
    }
}

fn parse_bounds(node: &XmlNode) -> Option<GpxBounds> {
    let parse_f = |name: &str| -> Option<f64> {
        node.attr(name)
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| node.child_text(name).and_then(|s| s.parse::<f64>().ok()))
    };
    Some(GpxBounds {
        min_lat: parse_f("minlat")?,
        max_lat: parse_f("maxlat")?,
        min_lon: parse_f("minlon")?,
        max_lon: parse_f("maxlon")?,
    })
}

fn parse_metadata(node: &XmlNode) -> GpxMetadata {
    GpxMetadata {
        name: node.child_text("name"),
        desc: node.child_text("desc"),
        author: node.child("author").map(parse_person),
        copyright: node.child("copyright").map(parse_copyright),
        link: node.child("link").map(parse_link),
        time: node.child_text("time"),
        keywords: node.child_text("keywords"),
        bounds: node.child("bounds").and_then(parse_bounds),
    }
}

fn parse_lat_lon_ele(node: &XmlNode) -> Result<Cartesian3, DeveloperError> {
    let lon_str = node
        .attr("lon")
        .ok_or_else(|| DeveloperError::new("GPX point is missing 'lon' attribute."))?;
    let lat_str = node
        .attr("lat")
        .ok_or_else(|| DeveloperError::new("GPX point is missing 'lat' attribute."))?;

    let lon: f64 = lon_str
        .parse()
        .map_err(|_| DeveloperError::new(format!("Invalid GPX longitude: {}", lon_str)))?;
    let lat: f64 = lat_str
        .parse()
        .map_err(|_| DeveloperError::new(format!("Invalid GPX latitude: {}", lat_str)))?;
    if !lon.is_finite() || !lat.is_finite() {
        return Err(DeveloperError::new("Invalid GPX coordinates."));
    }

    let ele = node
        .child_text("ele")
        .and_then(|s| s.parse::<f64>().ok());

    let dvec = Cartesian3::from_degrees(lon, lat, ele, None)
        .map_err(|_| DeveloperError::new("Failed to convert GPX coordinates."))?;
    Ok(dvec)
}

fn build_description(node: &XmlNode) -> Option<String> {
    let mut rows = Vec::new();
    let fields = [
        ("desc", "Description"),
        ("time", "Time"),
        ("cmt", "Comment"),
        ("src", "Source"),
        ("number", "Number"),
        ("type", "Type"),
    ];
    for (tag, label) in fields {
        if let Some(val) = node.child_text(tag) {
            if !val.is_empty() {
                rows.push(format!("<tr><th>{}</th><td>{}</td></tr>", label, val));
            }
        }
    }
    if rows.is_empty() {
        None
    } else {
        Some(format!("<table><tbody>{}</tbody></table>", rows.join("")))
    }
}

fn make_billboard(options: &GpxLoadOptions, image_override: Option<&str>) -> BillboardGraphics {
    let mut bb = BillboardGraphics::new();
    bb.width = Some(Rc::new(ConstantProperty::from_value(BILLBOARD_SIZE)));
    bb.height = Some(Rc::new(ConstantProperty::from_value(BILLBOARD_SIZE)));
    bb.vertical_origin = Some(Rc::new(ConstantProperty::from_value(
        VerticalOrigin::Bottom,
    )));
    if options.clamp_to_ground {
        bb.height_reference = Some(Rc::new(ConstantProperty::from_value(
            HeightReference::ClampToGround,
        )));
    }
    if let Some(img) = image_override.or(options.waypoint_image.as_deref()) {
        bb.image = Some(Rc::new(ConstantProperty::from_value(img.to_string())));
    }
    bb
}

fn make_polyline(positions: Vec<Cartesian3>, color: Color, clamp_to_ground: bool) -> PolylineGraphics {
    let mut pl = PolylineGraphics::new();
    pl.positions = Some(Rc::new(ConstantProperty::from_value(positions)));
    pl.width = Some(Rc::new(ConstantProperty::from_value(4.0)));
    pl.material = Some(Rc::new(PolylineOutlineMaterialProperty::new(
        Some(ConstantProperty::from_value(color)),
        Some(ConstantProperty::from_value(Color::BLACK)),
        Some(ConstantProperty::from_value(2.0)),
    )));
    if clamp_to_ground {
        pl.clamp_to_ground = Some(Rc::new(ConstantProperty::from_value(true)));
    }
    pl
}

fn parse_waypoint(node: &XmlNode, options: &GpxLoadOptions) -> Result<Entity, DeveloperError> {
    let pos = parse_lat_lon_ele(node)?;
    let mut entity = Entity::new();
    entity.name = node.child_text("name");
    if let Some(desc) = build_description(node) {
        entity.description = Some(ConstantProperty::from_value(desc));
    }
    entity.position = Some(ConstantPositionProperty::new(Some(pos), None));
    entity.billboard = Some(make_billboard(options, None));
    Ok(entity)
}

fn parse_route(node: &XmlNode, options: &GpxLoadOptions) -> Result<Vec<Entity>, DeveloperError> {
    let mut result = Vec::new();
    let mut route_entity = Entity::new();
    route_entity.name = node.child_text("name");
    if let Some(desc) = build_description(node) {
        route_entity.description = Some(ConstantProperty::from_value(desc));
    }

    let mut positions = Vec::new();
    let mut rtept_entities = Vec::new();

    for rtept in node.children_named("rtept") {
        let pos = parse_lat_lon_ele(rtept)?;
        positions.push(pos);
        let mut pt_entity = Entity::new();
        pt_entity.name = rtept.child_text("name");
        if let Some(desc) = build_description(rtept) {
            pt_entity.description = Some(ConstantProperty::from_value(desc));
        }
        pt_entity.position = Some(ConstantPositionProperty::new(Some(pos), None));
        pt_entity.billboard = Some(make_billboard(options, None));
        rtept_entities.push(pt_entity);
    }

    let route_color = options.route_color.unwrap_or(Color::YELLOW);
    route_entity.polyline = Some(make_polyline(positions, route_color, options.clamp_to_ground));

    result.push(route_entity);
    result.extend(rtept_entities);
    Ok(result)
}

fn parse_track(node: &XmlNode, options: &GpxLoadOptions) -> Result<Vec<Entity>, DeveloperError> {
    let mut positions = Vec::new();
    let mut times: Vec<Option<JulianDate>> = Vec::new();

    for seg in node.children_named("trkseg") {
        for trkpt in seg.children_named("trkpt") {
            let pos = parse_lat_lon_ele(trkpt)?;
            positions.push(pos);
            let t = if let Some(time_str) = trkpt.child_text("time") {
                Some(
                    JulianDate::from_iso8601(&time_str)
                        .map_err(|_| DeveloperError::new(format!("Invalid GPX time: {}", time_str)))?,
                )
            } else {
                None
            };
            times.push(t);
        }
    }

    let mut entity = Entity::new();
    entity.name = node.child_text("name");
    if let Some(desc) = build_description(node) {
        entity.description = Some(ConstantProperty::from_value(desc));
    }

    let track_color = options.track_color.unwrap_or(Color::RED);
    entity.polyline = Some(make_polyline(
        positions.clone(),
        track_color,
        options.clamp_to_ground,
    ));

    let has_all_times = !times.is_empty() && times.iter().all(|t| t.is_some());
    if has_all_times {
        let mut sampled = SampledPositionProperty::new(None);
        let mut min_time: Option<JulianDate> = None;
        let mut max_time: Option<JulianDate> = None;
        for (pos, t_opt) in positions.into_iter().zip(times) {
            if let Some(t) = t_opt {
                if min_time.is_none_or(|m| t.compare(&m) == std::cmp::Ordering::Less) {
                    min_time = Some(t);
                }
                if max_time.is_none_or(|m| t.compare(&m) == std::cmp::Ordering::Greater) {
                    max_time = Some(t);
                }
                sampled.add_sample(t, pos);
            }
        }
        entity.position = Some(ConstantPositionProperty::from_dynamic(Rc::new(sampled)));
        if let (Some(start), Some(stop)) = (min_time, max_time) {
            let mut avail = TimeIntervalCollection::new();
            avail.add_interval(TimeInterval::new(start, stop, true, true));
            entity.availability = Some(avail);
        }
        if let Some(ref img) = options.track_image {
            entity.billboard = Some(make_billboard(options, Some(img.as_str())));
        }
    }

    Ok(vec![entity])
}
