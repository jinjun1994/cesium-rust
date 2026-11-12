//! exportKml matching CesiumJS `DataSources/exportKml.js` (Apache-2.0).

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::{
    Cartesian3, Color, ColorMaterialProperty, DeveloperError, Ellipsoid, Entity,
    EntityCollection, HeightReference, HorizontalOrigin, JulianDate, TimeInterval,
    VerticalOrigin,
};

pub struct ExportKmlOptions<'a> {
    pub entities: &'a EntityCollection,
    pub ellipsoid: Option<Ellipsoid>,
    pub default_availability: Option<TimeInterval>,
    pub sample_duration: f64,
}

impl<'a> ExportKmlOptions<'a> {
    pub fn new(entities: &'a EntityCollection) -> Self {
        Self {
            entities,
            ellipsoid: None,
            default_availability: None,
            sample_duration: 60.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExportKmlResult {
    pub kml: String,
    pub external_files: HashMap<String, Vec<u8>>,
}

pub fn export_kml(options: &ExportKmlOptions<'_>) -> Result<ExportKmlResult, DeveloperError> {
    let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
    let now = JulianDate::now();
    let entities = options.entities.values();

    let mut styles_xml = String::new();
    let mut style_counter = 0usize;
    let mut entity_style_ids: HashMap<String, String> = HashMap::new();

    // Build children map by parent id
    let mut children_map: HashMap<Option<String>, Vec<&Entity>> = HashMap::new();
    for entity in &entities {
        let parent_id = entity.parent.as_ref().map(|p| p.borrow().id.clone());
        children_map.entry(parent_id).or_default().push(*entity);
    }

    for entity in &entities {
        if has_geometry(entity) {
            style_counter += 1;
            let style_id = format!("style-{}", style_counter);
            let style_xml = build_entity_style_xml(entity, &style_id, &now);
            styles_xml.push_str(&style_xml);
            entity_style_ids.insert(entity.id.clone(), style_id);
        }
    }

    let mut body_xml = String::new();
    if let Some(roots) = children_map.get(&None) {
        for root in roots {
            write_entity_node(
                root,
                &children_map,
                &entity_style_ids,
                &ellipsoid,
                &now,
                options,
                &mut body_xml,
            );
        }
    }

    let kml = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\
        <kml xmlns=\"http://www.opengis.net/kml/2.2\" xmlns:gx=\"http://www.google.com/kml/ext/2.2\">\
        <Document>{}{}</Document></kml>",
        styles_xml, body_xml
    );

    Ok(ExportKmlResult {
        kml,
        external_files: HashMap::new(),
    })
}

fn has_geometry(entity: &Entity) -> bool {
    entity.position.is_some()
        || entity.point.is_some()
        || entity.billboard.is_some()
        || entity.label.is_some()
        || entity.polyline.is_some()
        || entity.polygon.is_some()
        || entity.rectangle.is_some()
        || entity.path.is_some()
}

fn format_kml_color(c: Color) -> String {
    let a = (c.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (c.blue.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (c.green.clamp(0.0, 1.0) * 255.0).round() as u8;
    let r = (c.red.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("{:02x}{:02x}{:02x}{:02x}", a, b, g, r)
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn format_coord(pos: Cartesian3, ellipsoid: &Ellipsoid, height_override: Option<f64>) -> String {
    if let Some(carto) = ellipsoid.cartesian_to_cartographic(pos.into()) {
        let lon = carto.longitude.to_degrees();
        let lat = carto.latitude.to_degrees();
        let h = height_override.unwrap_or(carto.height);
        format!("{},{},{}", lon, lat, h)
    } else {
        "0,0,0".to_string()
    }
}

fn altitude_mode_from_hr(hr: Option<HeightReference>) -> &'static str {
    match hr {
        Some(HeightReference::ClampToGround) => "clampToGround",
        Some(HeightReference::RelativeToGround) => "relativeToGround",
        _ => "absolute",
    }
}

fn build_entity_style_xml(entity: &Entity, style_id: &str, time: &JulianDate) -> String {
    let mut inner = String::new();

    if let Some(ref pt) = entity.point {
        let color = pt
            .color
            .as_ref()
            .and_then(|p| p.get_value(Some(time)))
            .unwrap_or(Color::WHITE);
        let pixel_size = pt
            .pixel_size
            .as_ref()
            .and_then(|p| p.get_value(Some(time)))
            .unwrap_or(1.0);
        let scale = pixel_size / 32.0;
        let _ = write!(
            inner,
            "<IconStyle><color>{}</color><colorMode>normal</colorMode><scale>{}</scale></IconStyle>",
            format_kml_color(color),
            scale
        );
    } else if let Some(ref bb) = entity.billboard {
        inner.push_str("<IconStyle>");
        if let Some(color) = bb.color.as_ref().and_then(|p| p.get_value(Some(time))) {
            let _ = write!(
                inner,
                "<color>{}</color><colorMode>normal</colorMode>",
                format_kml_color(color)
            );
        }
        if let Some(scale) = bb.scale.as_ref().and_then(|p| p.get_value(Some(time))) {
            let _ = write!(inner, "<scale>{}</scale>", scale);
        }
        if let Some(rot) = bb.rotation.as_ref().and_then(|p| p.get_value(Some(time))) {
            let mut deg = -rot.to_degrees();
            if deg.abs() < 1e-12 {
                deg = 360.0;
            } else if deg < 0.0 {
                deg += 360.0;
            }
            let _ = write!(inner, "<heading>{}</heading>", deg);
        }
        if let Some(img) = bb.image.as_ref().and_then(|p| p.get_value(Some(time))) {
            let _ = write!(inner, "<Icon><href>{}</href></Icon>", escape_xml(&img));
        }
        let h_origin = bb
            .horizontal_origin
            .as_ref()
            .and_then(|p| p.get_value(Some(time)));
        let v_origin = bb
            .vertical_origin
            .as_ref()
            .and_then(|p| p.get_value(Some(time)));
        if h_origin.is_some() || v_origin.is_some() {
            let x = match h_origin {
                Some(HorizontalOrigin::Left) => 0.0,
                Some(HorizontalOrigin::Right) => 1.0,
                _ => 0.5,
            };
            let y = match v_origin {
                Some(VerticalOrigin::Bottom) => 0.0,
                Some(VerticalOrigin::Top) => 1.0,
                _ => 0.5,
            };
            let _ = write!(
                inner,
                "<hotSpot x=\"{}\" y=\"{}\" xunits=\"fraction\" yunits=\"fraction\"/>",
                x, y
            );
        }
        inner.push_str("</IconStyle>");
    }

    if let Some(ref lbl) = entity.label {
        inner.push_str("<LabelStyle>");
        if let Some(c) = lbl.fill_color.as_ref().and_then(|p| p.get_value(Some(time))) {
            let _ = write!(
                inner,
                "<color>{}</color><colorMode>normal</colorMode>",
                format_kml_color(c)
            );
        }
        if let Some(s) = lbl.scale.as_ref().and_then(|p| p.get_value(Some(time))) {
            let _ = write!(inner, "<scale>{}</scale>", s);
        }
        inner.push_str("</LabelStyle>");
    }

    if let Some(ref pl) = entity.polyline {
        inner.push_str("<LineStyle>");
        if let Some(mat) = pl.material.as_ref() {
            if let Some(cm) = mat.as_any().downcast_ref::<ColorMaterialProperty>() {
                if let Some(c) = cm.color.as_ref().and_then(|p| p.get_value(Some(time))) {
                    let _ = write!(
                        inner,
                        "<color>{}</color><colorMode>normal</colorMode>",
                        format_kml_color(c)
                    );
                }
            }
        }
        if let Some(w) = pl.width.as_ref().and_then(|p| p.get_value(Some(time))) {
            let _ = write!(inner, "<width>{}</width>", w);
        }
        inner.push_str("</LineStyle>");
    } else if let Some(ref pth) = entity.path {
        inner.push_str("<LineStyle>");
        if let Some(mat) = pth.material.as_ref() {
            if let Some(cm) = mat.as_any().downcast_ref::<ColorMaterialProperty>() {
                if let Some(c) = cm.color.as_ref().and_then(|p| p.get_value(Some(time))) {
                    let _ = write!(
                        inner,
                        "<color>{}</color><colorMode>normal</colorMode>",
                        format_kml_color(c)
                    );
                }
            }
        }
        if let Some(w) = pth.width.as_ref().and_then(|p| p.get_value(Some(time))) {
            let _ = write!(inner, "<width>{}</width>", w);
        }
        inner.push_str("</LineStyle>");
    }

    if let Some(ref pg) = entity.polygon {
        let outline = pg
            .outline
            .as_ref()
            .and_then(|p| p.get_value(Some(time)))
            .unwrap_or(false);
        if outline {
            let oc = pg
                .outline_color
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(Color::BLACK);
            let ow = pg
                .outline_width
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(1.0);
            let _ = write!(
                inner,
                "<LineStyle><color>{}</color><colorMode>normal</colorMode><width>{}</width></LineStyle>",
                format_kml_color(oc),
                ow
            );
        }
        inner.push_str("<PolyStyle>");
        if let Some(mat) = pg.material.as_ref() {
            if let Some(cm) = mat.as_any().downcast_ref::<ColorMaterialProperty>() {
                if let Some(c) = cm.color.as_ref().and_then(|p| p.get_value(Some(time))) {
                    let _ = write!(
                        inner,
                        "<color>{}</color><colorMode>normal</colorMode>",
                        format_kml_color(c)
                    );
                }
            }
        }
        let fill = pg
            .fill
            .as_ref()
            .and_then(|p| p.get_value(Some(time)))
            .unwrap_or(true);
        let _ = write!(
            inner,
            "<fill>{}</fill><outline>{}</outline></PolyStyle>",
            if fill { 1 } else { 0 },
            if outline { 1 } else { 0 }
        );
    } else if let Some(ref rg) = entity.rectangle {
        inner.push_str("<PolyStyle>");
        if let Some(mat) = rg.material.as_ref() {
            if let Some(cm) = mat.as_any().downcast_ref::<ColorMaterialProperty>() {
                if let Some(c) = cm.color.as_ref().and_then(|p| p.get_value(Some(time))) {
                    let _ = write!(
                        inner,
                        "<color>{}</color><colorMode>normal</colorMode>",
                        format_kml_color(c)
                    );
                }
            }
        }
        inner.push_str("<fill>1</fill><outline>0</outline></PolyStyle>");
    }

    format!("<Style id=\"{}\">{}</Style>", escape_xml(style_id), inner)
}

fn write_entity_node(
    entity: &Entity,
    children_map: &HashMap<Option<String>, Vec<&Entity>>,
    entity_style_ids: &HashMap<String, String>,
    ellipsoid: &Ellipsoid,
    time: &JulianDate,
    options: &ExportKmlOptions<'_>,
    out: &mut String,
) {
    let children = children_map.get(&Some(entity.id.clone()));
    if !has_geometry(entity) {
        let _ = write!(out, "<Folder id=\"{}\">", escape_xml(&entity.id));
        if let Some(ref name) = entity.name {
            let _ = write!(out, "<name>{}</name>", escape_xml(name));
        }
        let _ = write!(
            out,
            "<visibility>{}</visibility>",
            if entity.show { 1 } else { 0 }
        );
        if let Some(desc) = entity.description.as_ref().and_then(|p| p.get_value(Some(time))) {
            let _ = write!(out, "<description>{}</description>", escape_xml(&desc));
        }
        if let Some(kids) = children {
            for kid in kids {
                write_entity_node(
                    kid,
                    children_map,
                    entity_style_ids,
                    ellipsoid,
                    time,
                    options,
                    out,
                );
            }
        }
        out.push_str("</Folder>");
        return;
    }

    let _ = write!(out, "<Placemark id=\"{}\">", escape_xml(&entity.id));
    if let Some(ref name) = entity.name {
        let _ = write!(out, "<name>{}</name>", escape_xml(name));
    }
    let _ = write!(
        out,
        "<visibility>{}</visibility>",
        if entity.show { 1 } else { 0 }
    );
    if let Some(desc) = entity.description.as_ref().and_then(|p| p.get_value(Some(time))) {
        let _ = write!(out, "<description>{}</description>", escape_xml(&desc));
    }
    if let Some(style_id) = entity_style_ids.get(&entity.id) {
        let _ = write!(out, "<styleUrl>#{}</styleUrl>", escape_xml(style_id));
    }

    // Geometry: Point or gx:Track
    if let Some(ref pos_prop) = entity.position {
        if !pos_prop.is_constant() {
            if let Some(ref avail) = entity.availability {
                if let (Some(start), Some(stop)) = (avail.start(), avail.stop()) {
                    let _ = write_gx_track(
                        pos_prop,
                        start,
                        stop,
                        options.sample_duration.max(1.0),
                        ellipsoid,
                        out,
                    );
                }
            }
        } else if let Some(pos) = pos_prop.get_value(Some(time)) {
            let hr = entity
                .point
                .as_ref()
                .and_then(|p| p.height_reference.as_ref())
                .or_else(|| {
                    entity
                        .billboard
                        .as_ref()
                        .and_then(|b| b.height_reference.as_ref())
                })
                .and_then(|p| p.get_value(Some(time)));
            let mode = altitude_mode_from_hr(hr);
            let coord = format_coord(pos, ellipsoid, None);
            let _ = write!(
                out,
                "<Point><altitudeMode>{}</altitudeMode><coordinates>{}</coordinates></Point>",
                mode, coord
            );
        }
    }

    // Polyline
    if let Some(ref pl) = entity.polyline {
        if let Some(positions) = pl.positions.as_ref().and_then(|p| p.get_value(Some(time))) {
            let clamp = pl
                .clamp_to_ground
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(false);
            let mode = if clamp { "clampToGround" } else { "absolute" };
            let coords: Vec<String> = positions
                .into_iter()
                .map(|p| format_coord(p, ellipsoid, None))
                .collect();
            let _ = write!(
                out,
                "<LineString><tessellate>{}</tessellate><altitudeMode>{}</altitudeMode><coordinates>{}</coordinates></LineString>",
                if clamp { 1 } else { 0 },
                mode,
                coords.join(" ")
            );
        }
    }

    // Polygon
    if let Some(ref pg) = entity.polygon {
        if let Some(hier) = pg.hierarchy.as_ref().and_then(|p| p.get_value(Some(time))) {
            let per_pos_h = pg
                .per_position_height
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .unwrap_or(false);
            let height = pg.height.as_ref().and_then(|p| p.get_value(Some(time)));
            let extrude = pg
                .extruded_height
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .is_some();
            let mode = if !per_pos_h && height.is_none() && !extrude {
                "clampToGround"
            } else {
                "absolute"
            };
            let h_override = if per_pos_h { None } else { height };
            let outer_coords: Vec<String> = hier
                .positions
                .iter()
                .map(|&p| format_coord(p, ellipsoid, h_override))
                .collect();
            let mut holes_xml = String::new();
            for hole in &hier.holes {
                let h_coords: Vec<String> = hole
                    .positions
                    .iter()
                    .map(|&p| format_coord(p, ellipsoid, h_override))
                    .collect();
                let _ = write!(
                    holes_xml,
                    "<innerBoundaryIs><LinearRing><coordinates>{}</coordinates></LinearRing></innerBoundaryIs>",
                    h_coords.join(" ")
                );
            }
            let _ = write!(
                out,
                "<Polygon><extrude>{}</extrude><altitudeMode>{}</altitudeMode><outerBoundaryIs><LinearRing><coordinates>{}</coordinates></LinearRing></outerBoundaryIs>{}</Polygon>",
                if extrude { 1 } else { 0 },
                mode,
                outer_coords.join(" "),
                holes_xml
            );
        }
    }

    // Rectangle -> Polygon
    if let Some(ref rg) = entity.rectangle {
        if let Some(rect) = rg.coordinates.as_ref().and_then(|p| p.get_value(Some(time))) {
            let h = rg.height.as_ref().and_then(|p| p.get_value(Some(time)));
            let extrude = rg
                .extruded_height
                .as_ref()
                .and_then(|p| p.get_value(Some(time)))
                .is_some();
            let mode = if h.is_none() && !extrude {
                "clampToGround"
            } else {
                "absolute"
            };
            let alt = h.unwrap_or(0.0);
            let w = rect.west.to_degrees();
            let s = rect.south.to_degrees();
            let e = rect.east.to_degrees();
            let n = rect.north.to_degrees();
            let coords = format!(
                "{},{},{} {},{},{} {},{},{} {},{},{}",
                w, s, alt, e, s, alt, e, n, alt, w, n, alt
            );
            let _ = write!(
                out,
                "<Polygon><extrude>{}</extrude><altitudeMode>{}</altitudeMode><outerBoundaryIs><LinearRing><coordinates>{}</coordinates></LinearRing></outerBoundaryIs></Polygon>",
                if extrude { 1 } else { 0 },
                mode,
                coords
            );
        }
    }

    out.push_str("</Placemark>");
}

fn write_gx_track(
    pos_prop: &crate::ConstantPositionProperty,
    start: JulianDate,
    stop: JulianDate,
    step_seconds: f64,
    ellipsoid: &Ellipsoid,
    out: &mut String,
) -> std::fmt::Result {
    let total = JulianDate::seconds_difference(&stop, &start).max(0.0);
    let mut t = start;
    let mut whens = String::new();
    let mut coords = String::new();
    let mut elapsed = 0.0;
    loop {
        if let Some(pos) = pos_prop.get_value(Some(&t)) {
            if let Some(carto) = ellipsoid.cartesian_to_cartographic(pos.into()) {
                let iso = t.to_iso8601();
                write!(whens, "<when>{}</when>", iso)?;
                write!(
                    coords,
                    "<gx:coord>{} {} {}</gx:coord>",
                    carto.longitude.to_degrees(),
                    carto.latitude.to_degrees(),
                    carto.height
                )?;
            }
        }
        if elapsed >= total {
            break;
        }
        elapsed = (elapsed + step_seconds).min(total);
        t = start.add_seconds(elapsed);
    }
    write!(
        out,
        "<gx:Track><altitudeMode>absolute</altitudeMode>{}{}</gx:Track>",
        whens, coords
    )
}
