//! Bounded RFC 7946 geometry subset, tessellated onto WGS84. No terrain clamping.
use cesium_formats::{Material, Primitive, Vertex};
use cesium_math::{Cartographic, WGS84_A};
use glam::{DMat4, DVec3};
use serde::Deserialize;
use serde_json::{json, Value};
use std::f64::consts::TAU;

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Style {
    pub point_color: [f32; 4],
    pub line_color: [f32; 4],
    pub fill_color: [f32; 4],
    pub point_radius: f64,
    pub line_width: f64,
    pub surface_offset: f64,
    pub segment_degrees: f64,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            point_color: [1.0, 0.15, 0.02, 1.0],
            line_color: [1.0, 0.7, 0.03, 1.0],
            fill_color: [0.02, 0.65, 0.25, 1.0],
            point_radius: 5000.0,
            line_width: 2000.0,
            surface_offset: 100.0,
            segment_degrees: 0.5,
        }
    }
}
impl Style {
    fn validate(&self) -> Result<(), String> {
        for color in [self.point_color, self.line_color, self.fill_color] {
            if color
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
                || color[3] != 1.0
            {
                return Err("GeoJSON colors must be finite opaque RGBA in 0..1".into());
            }
        }
        if !self.point_radius.is_finite()
            || !(1.0..=200000.0).contains(&self.point_radius)
            || !self.line_width.is_finite()
            || !(1.0..=200000.0).contains(&self.line_width)
            || !self.surface_offset.is_finite()
            || !(0.0..=100000.0).contains(&self.surface_offset)
            || !self.segment_degrees.is_finite()
            || !(0.05..=0.5).contains(&self.segment_degrees)
        {
            return Err("invalid GeoJSON style dimensions".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct Limits {
    pub max_input_bytes: usize,
    pub max_decoded_bytes: usize,
    pub max_features: usize,
    pub max_positions: usize,
    pub max_vertices: usize,
    pub max_indices: usize,
    pub max_depth: usize,
    pub max_intersection_checks: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_input_bytes: 8 * 1024 * 1024,
            max_decoded_bytes: 32 * 1024 * 1024,
            max_features: 10000,
            max_positions: 100000,
            max_vertices: 300000,
            max_indices: 1800000,
            max_depth: 32,
            max_intersection_checks: 4000000,
        }
    }
}
pub struct Data {
    pub primitives: Vec<Primitive>,
    pub features: Vec<Value>,
    pub center: DVec3,
    pub radius: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct Coord {
    lon: f64,
    lat: f64,
    height: f64,
}
impl Coord {
    fn ecef(self, offset: f64) -> DVec3 {
        Cartographic::degrees(self.lon, self.lat, self.height + offset)
            .unwrap()
            .to_ecef()
    }
    fn interpolate(self, other: Self, t: f64) -> Self {
        Self {
            lon: self.lon + (other.lon - self.lon) * t,
            lat: self.lat + (other.lat - self.lat) * t,
            height: self.height + (other.height - self.height) * t,
        }
    }
}
fn array(v: &Value) -> Result<&[Value], String> {
    v.as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| "GeoJSON coordinate/geometry array expected".into())
}
fn unwrap_lon(lon: f64, reference: f64) -> f64 {
    reference + (lon - reference + 180.0).rem_euclid(360.0) - 180.0
}
fn span(a: Coord, b: Coord) -> f64 {
    (a.lon - b.lon).abs().max((a.lat - b.lat).abs())
}
fn orient(a: Coord, b: Coord, c: Coord) -> f64 {
    (b.lon - a.lon) * (c.lat - a.lat) - (b.lat - a.lat) * (c.lon - a.lon)
}
fn inside(p: Coord, ring: &[Coord]) -> bool {
    let mut result = false;
    let mut j = ring.len() - 1;
    for i in 0..ring.len() {
        let (a, b) = (ring[i], ring[j]);
        if (a.lat > p.lat) != (b.lat > p.lat)
            && p.lon < (b.lon - a.lon) * (p.lat - a.lat) / (b.lat - a.lat) + a.lon
        {
            result = !result;
        }
        j = i;
    }
    result
}
fn intersect(a: Coord, b: Coord, c: Coord, d: Coord) -> bool {
    let o = [
        orient(a, b, c),
        orient(a, b, d),
        orient(c, d, a),
        orient(c, d, b),
    ];
    if o[0] * o[1] < 0.0 && o[2] * o[3] < 0.0 {
        return true;
    }
    let on = |p: Coord, x: Coord, y: Coord| {
        p.lon >= x.lon.min(y.lon) - 1e-12
            && p.lon <= x.lon.max(y.lon) + 1e-12
            && p.lat >= x.lat.min(y.lat) - 1e-12
            && p.lat <= x.lat.max(y.lat) + 1e-12
    };
    (o[0].abs() < 1e-12 && on(c, a, b))
        || (o[1].abs() < 1e-12 && on(d, a, b))
        || (o[2].abs() < 1e-12 && on(a, c, d))
        || (o[3].abs() < 1e-12 && on(b, c, d))
}
struct Builder<'a> {
    style: &'a Style,
    limits: &'a Limits,
    batches: [Primitive; 3],
    features: Vec<Value>,
    positions: usize,
    checks: usize,
    origin: Option<DVec3>,
}
impl<'a> Builder<'a> {
    fn budget(&self, vertices: usize, indices: usize) -> Result<(), String> {
        let v = self
            .batches
            .iter()
            .map(|p| p.vertices.len())
            .sum::<usize>()
            .checked_add(vertices)
            .ok_or("GeoJSON size overflow")?;
        let i = self
            .batches
            .iter()
            .map(|p| p.indices.len())
            .sum::<usize>()
            .checked_add(indices)
            .ok_or("GeoJSON size overflow")?;
        if v > self.limits.max_vertices
            || i > self.limits.max_indices
            || v * std::mem::size_of::<Vertex>() + i * 4 > self.limits.max_decoded_bytes
        {
            return Err("GeoJSON tessellation exceeds geometry budget".into());
        }
        Ok(())
    }
    fn coord(&mut self, value: &Value) -> Result<Coord, String> {
        self.positions += 1;
        if self.positions > self.limits.max_positions {
            return Err("GeoJSON position limit".into());
        }
        let v = array(value)?;
        if !(2..=3).contains(&v.len()) {
            return Err(
                "GeoJSON positions must have longitude, latitude and optional height".into(),
            );
        }
        let number = |i: usize| {
            v[i].as_f64()
                .filter(|n| n.is_finite())
                .ok_or("finite GeoJSON coordinate required")
        };
        let c = Coord {
            lon: number(0)?,
            lat: number(1)?,
            height: if v.len() == 3 { number(2)? } else { 0.0 },
        };
        if !(-180.0..=180.0).contains(&c.lon)
            || !(-90.0..=90.0).contains(&c.lat)
            || c.height.abs() > 1000000.0
        {
            return Err("GeoJSON coordinate outside WGS84/height range".into());
        }
        Ok(c)
    }
    fn sequence(&mut self, value: &Value) -> Result<Vec<Coord>, String> {
        let mut out: Vec<Coord> = Vec::new();
        for value in array(value)? {
            let mut c = self.coord(value)?;
            if let Some(previous) = out.last() {
                c.lon = unwrap_lon(c.lon, previous.lon);
            }
            out.push(c);
        }
        Ok(out)
    }
    fn vertex(&mut self, slot: usize, c: Coord, id: u32) {
        let (lat, lon) = (c.lat.to_radians(), c.lon.to_radians());
        let world = c.ecef(self.style.surface_offset);
        let origin = *self.origin.get_or_insert(world);
        self.batches[slot].vertices.push(Vertex {
            position: (world - origin).as_vec3().to_array(),
            normal: [
                (lat.cos() * lon.cos()) as f32,
                (lat.cos() * lon.sin()) as f32,
                lat.sin() as f32,
            ],
            uv: [0.0; 2],
            uv1: [0.0; 2],
            color: [1.0; 4],
            tangent: [0.0; 4],
            feature_id: id,
        });
    }
    fn triangle(
        &mut self,
        slot: usize,
        a: Coord,
        b: Coord,
        c: Coord,
        id: u32,
    ) -> Result<(), String> {
        let n = (span(a, b).max(span(a, c)).max(span(b, c)) / self.style.segment_degrees)
            .ceil()
            .max(1.0) as usize;
        let vertices = (n + 1)
            .checked_mul(n + 2)
            .and_then(|v| v.checked_div(2))
            .ok_or("GeoJSON triangle size")?;
        let indices = n
            .checked_mul(n)
            .and_then(|v| v.checked_mul(3))
            .ok_or("GeoJSON triangle size")?;
        self.budget(vertices, indices)?;
        let base = self.batches[slot].vertices.len() as u32;
        let mut rows = Vec::new();
        for row in 0..=n {
            rows.push(self.batches[slot].vertices.len() as u32);
            for col in 0..=n - row {
                let u = row as f64 / n as f64;
                let v = col as f64 / n as f64;
                self.vertex(
                    slot,
                    Coord {
                        lon: a.lon + (b.lon - a.lon) * u + (c.lon - a.lon) * v,
                        lat: a.lat + (b.lat - a.lat) * u + (c.lat - a.lat) * v,
                        height: a.height + (b.height - a.height) * u + (c.height - a.height) * v,
                    },
                    id,
                );
            }
        }
        for row in 0..n {
            for col in 0..n - row {
                let a = rows[row] + col as u32;
                let b = rows[row + 1] + col as u32;
                self.batches[slot].indices.extend_from_slice(&[a, b, a + 1]);
                if col + 1 < n - row {
                    self.batches[slot]
                        .indices
                        .extend_from_slice(&[a + 1, b, b + 1]);
                }
            }
        }
        debug_assert_eq!(
            self.batches[slot].vertices.len() as u32 - base,
            vertices as u32
        );
        Ok(())
    }
    fn point(&mut self, c: Coord, id: u32) -> Result<(), String> {
        // A circle on the local tangent plane, lifted enough to stay outside the ellipsoid's sagitta.
        self.budget(33, 96)?;
        let center =
            c.ecef(self.style.surface_offset + self.style.point_radius.powi(2) / (2.0 * WGS84_A));
        let frame = Cartographic::degrees(c.lon, c.lat, 0.0)
            .unwrap()
            .enu_to_ecef();
        let base = self.batches[0].vertices.len() as u32;
        let mut raised = c;
        raised.height += self.style.point_radius.powi(2) / (2.0 * WGS84_A);
        self.vertex(0, raised, id);
        for i in 0..32 {
            let angle = i as f64 / 32.0 * TAU;
            let p = center
                + frame.transform_vector3(DVec3::new(angle.cos(), angle.sin(), 0.0))
                    * self.style.point_radius;
            let position = Cartographic::from_ecef(p).map_err(|_| "invalid GeoJSON marker")?;
            self.vertex(
                0,
                Coord {
                    lon: position.longitude.to_degrees(),
                    lat: position.latitude.to_degrees(),
                    height: position.height - self.style.surface_offset,
                },
                id,
            );
        }
        for i in 0..32 {
            self.batches[0].indices.extend_from_slice(&[
                base,
                base + 1 + i,
                base + 1 + (i + 1) % 32,
            ]);
        }
        Ok(())
    }
    fn line(&mut self, coords: &Value, id: u32) -> Result<(), String> {
        let sequence = self.sequence(coords)?;
        if sequence.len() < 2 {
            return Err("LineString requires two positions".into());
        }
        for pair in sequence.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let n = (span(a, b) / self.style.segment_degrees).ceil().max(1.0) as usize;
            // Subdivide across wide ribbons too: a single wide quad cuts through the ellipsoid.
            let across = (self.style.line_width
                / (WGS84_A * self.style.segment_degrees.to_radians()))
            .ceil()
            .max(1.) as usize;
            self.budget(n * 2 * (across + 1), n * across * 6)?;
            for i in 0..n {
                let (a, b) = (
                    a.interpolate(b, i as f64 / n as f64),
                    a.interpolate(b, (i + 1) as f64 / n as f64),
                );
                let (pa, pb) = (a.ecef(0.0), b.ecef(0.0));
                let direction = pb - pa;
                let Some(side) = direction.cross((pa + pb).normalize()).try_normalize() else {
                    continue;
                };
                let base = self.batches[1].vertices.len() as u32;
                for (coord, p) in [(a, pa), (b, pb)] {
                    for j in 0..=across {
                        let sign = 2. * j as f64 / across as f64 - 1.;
                        let point = Cartographic::from_ecef(
                            p + side * (sign * self.style.line_width * 0.5),
                        )
                        .map_err(|_| "invalid GeoJSON ribbon")?;
                        self.vertex(
                            1,
                            Coord {
                                lon: point.longitude.to_degrees(),
                                lat: point.latitude.to_degrees(),
                                height: coord.height,
                            },
                            id,
                        );
                    }
                }
                for j in 0..across as u32 {
                    let a = base + j;
                    let b = a + (across + 1) as u32;
                    self.batches[1]
                        .indices
                        .extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
                }
            }
        }
        Ok(())
    }
    fn check_segments(&mut self, a: Coord, b: Coord, c: Coord, d: Coord) -> Result<(), String> {
        self.checks += 1;
        if self.checks > self.limits.max_intersection_checks {
            return Err("GeoJSON polygon validation work limit".into());
        }
        if intersect(a, b, c, d) {
            return Err("self-intersecting or overlapping GeoJSON rings".into());
        }
        Ok(())
    }
    fn polygon(&mut self, value: &Value, id: u32) -> Result<(), String> {
        let mut rings: Vec<Vec<Coord>> = Vec::new();
        for r in array(value)? {
            // Check closure in the original coordinates. Longitude unwrapping can introduce
            // roundoff even when the caller supplied identical first/last positions.
            let input = array(r)?;
            let mut ring = self.sequence(r)?;
            let closed = input.first().zip(input.last()).is_some_and(|(a, b)| {
                (0..3).all(|i| {
                    a.get(i).and_then(Value::as_f64).unwrap_or(0.)
                        == b.get(i).and_then(Value::as_f64).unwrap_or(0.)
                })
            });
            if ring.len() < 4 || !closed {
                return Err("Polygon ring must be closed with at least four positions".into());
            }
            ring.pop();
            if ring.len() > 1024 || ring.iter().any(|c| c.lat.abs() > 85.0) {
                return Err(
                    "GeoJSON polygon ring too large or outside supported ±85° latitude".into(),
                );
            }
            if let Some(outer) = rings.first() {
                let shift = unwrap_lon(ring[0].lon, outer[0].lon) - ring[0].lon;
                for c in &mut ring {
                    c.lon += shift;
                }
            }
            let min = ring.iter().map(|c| c.lon).fold(f64::INFINITY, f64::min);
            let max = ring.iter().map(|c| c.lon).fold(f64::NEG_INFINITY, f64::max);
            if max - min > 180.0 {
                return Err("GeoJSON polygon spans more than a hemisphere; split it first".into());
            }
            for i in 0..ring.len() {
                if ring[i] == ring[(i + 1) % ring.len()] {
                    return Err("duplicate polygon ring position".into());
                }
                for j in i + 1..ring.len() {
                    if j == i + 1 || (i == 0 && j + 1 == ring.len()) {
                        continue;
                    }
                    self.check_segments(
                        ring[i],
                        ring[(i + 1) % ring.len()],
                        ring[j],
                        ring[(j + 1) % ring.len()],
                    )?;
                }
            }
            rings.push(ring);
        }
        if rings.is_empty() {
            return Ok(());
        }
        for i in 1..rings.len() {
            if !inside(rings[i][0], &rings[0]) {
                return Err("GeoJSON polygon hole outside exterior".into());
            }
            for j in 0..i {
                if j > 0 && (inside(rings[i][0], &rings[j]) || inside(rings[j][0], &rings[i])) {
                    return Err("nested/overlapping GeoJSON holes".into());
                }
                for a in 0..rings[i].len() {
                    for b in 0..rings[j].len() {
                        self.check_segments(
                            rings[i][a],
                            rings[i][(a + 1) % rings[i].len()],
                            rings[j][b],
                            rings[j][(b + 1) % rings[j].len()],
                        )?;
                    }
                }
            }
        }
        let mut points = Vec::new();
        let mut holes = Vec::new();
        for (i, ring) in rings.into_iter().enumerate() {
            if i > 0 {
                holes.push(points.len());
            }
            points.extend(ring);
        }
        let flat: Vec<_> = points.iter().flat_map(|c| [c.lon, c.lat]).collect();
        let triangles =
            earcutr::earcut(&flat, &holes, 2).map_err(|_| "GeoJSON triangulation failed")?;
        if triangles.is_empty() {
            return Err("degenerate GeoJSON polygon".into());
        }
        for t in triangles.as_chunks::<3>().0 {
            self.triangle(2, points[t[0]], points[t[1]], points[t[2]], id)?;
        }
        Ok(())
    }
    fn geometry(&mut self, g: &Value, id: u32, depth: usize) -> Result<(), String> {
        if depth > self.limits.max_depth {
            return Err("GeoJSON geometry depth limit".into());
        }
        if g.is_null() {
            return Ok(());
        }
        if g.get("crs").is_some() {
            return Err("GeoJSON CRS overrides are unsupported; use WGS84".into());
        }
        let c = &g["coordinates"];
        match g["type"].as_str().ok_or("GeoJSON geometry type required")? {
            "Point" => {
                let point = self.coord(c)?;
                self.point(point, id)
            }
            "MultiPoint" => {
                for v in array(c)? {
                    let point = self.coord(v)?;
                    self.point(point, id)?;
                }
                Ok(())
            }
            "LineString" => self.line(c, id),
            "MultiLineString" => {
                for v in array(c)? {
                    self.line(v, id)?;
                }
                Ok(())
            }
            "Polygon" => self.polygon(c, id),
            "MultiPolygon" => {
                for v in array(c)? {
                    self.polygon(v, id)?;
                }
                Ok(())
            }
            "GeometryCollection" => {
                for v in array(&g["geometries"])? {
                    self.geometry(v, id, depth + 1)?;
                }
                Ok(())
            }
            _ => Err("unsupported GeoJSON geometry type".into()),
        }
    }
    fn feature(&mut self, f: &Value) -> Result<(), String> {
        if f.get("crs").is_some() {
            return Err("GeoJSON CRS overrides are unsupported; use WGS84".into());
        }
        if self.features.len() >= self.limits.max_features {
            return Err("GeoJSON feature limit".into());
        }
        if f["type"] != "Feature" {
            return Err("FeatureCollection element must be a Feature".into());
        }
        let properties = f
            .get("properties")
            .ok_or("GeoJSON Feature properties required")?;
        if !properties.is_object() && !properties.is_null() {
            return Err("GeoJSON properties must be an object or null".into());
        }
        let id = f.get("id").cloned().unwrap_or(Value::Null);
        if !id.is_null() && !id.is_number() && !id.is_string() {
            return Err("GeoJSON id must be a number or string".into());
        }
        let index = self.features.len() as u32;
        self.features.push(json!({"id":id,"properties":properties}));
        self.geometry(
            f.get("geometry")
                .ok_or("GeoJSON Feature geometry required")?,
            index,
            0,
        )
    }
}
pub fn decode(bytes: &[u8], style: &Style, limits: &Limits) -> Result<Data, String> {
    style.validate()?;
    if bytes.len() > limits.max_input_bytes {
        return Err("GeoJSON input byte limit".into());
    }
    let root: Value = serde_json::from_slice(bytes).map_err(|_| "invalid GeoJSON JSON")?;
    if root.get("crs").is_some() {
        return Err("GeoJSON CRS overrides are unsupported; use WGS84".into());
    }
    let batches = std::array::from_fn(|i| {
        let color = [style.point_color, style.line_color, style.fill_color][i];
        Primitive {
            vertices: Vec::new(),
            indices: Vec::new(),
            transform: DMat4::IDENTITY,
            material: Material {
                base_color: color,
                unlit: true,
                metallic: 0.0,
                roughness: 1.0,
                double_sided: true,
                ..Default::default()
            },
        }
    });
    let mut b = Builder {
        style,
        limits,
        batches,
        features: Vec::new(),
        positions: 0,
        checks: 0,
        origin: None,
    };
    match root["type"].as_str().ok_or("GeoJSON type required")? {
        "FeatureCollection" => {
            for f in array(&root["features"])? {
                b.feature(f)?;
            }
        }
        "Feature" => b.feature(&root)?,
        _ => b.feature(&json!({"type":"Feature","properties":{},"geometry":root}))?,
    }
    let origin = b.origin.unwrap_or(DVec3::ZERO);
    let mut primitives: Vec<_> = b
        .batches
        .into_iter()
        .filter(|p| !p.indices.is_empty())
        .collect();
    let mut center = DVec3::ZERO;
    let mut n = 0;
    for p in &primitives {
        for v in &p.vertices {
            center += DVec3::from_array(v.position.map(f64::from));
            n += 1;
        }
    }
    if n > 0 {
        center /= n as f64;
    }
    let radius = primitives
        .iter()
        .flat_map(|p| p.vertices.iter())
        .map(|v| center.distance(DVec3::from_array(v.position.map(f64::from))))
        .fold(1.0, f64::max);
    for primitive in &mut primitives {
        primitive.transform = DMat4::from_translation(origin);
    }
    Ok(Data {
        primitives,
        features: b.features,
        center: center + origin,
        radius,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn decode_value(value: Value) -> Result<Data, String> {
        decode(
            &serde_json::to_vec(&value).unwrap(),
            &Style::default(),
            &Limits::default(),
        )
    }
    #[test]
    fn geometry_types_properties_and_limits() {
        let geometry = json!({"type":"GeometryCollection","geometries":[{"type":"MultiPoint","coordinates":[[110,25],[111,26,50]]},{"type":"MultiLineString","coordinates":[[[110,25],[111,26]],[[120,20],[121,21]]]},{"type":"MultiPolygon","coordinates":[[[[110,25],[111,25],[111,26],[110,26],[110,25]]]]}]});
        let data=decode_value(json!({"type":"Feature","id":"abc","properties":{"name":"test","nested":{"value":2}},"geometry":geometry})).unwrap();
        assert_eq!(data.primitives.len(), 3);
        assert_eq!(data.features[0]["id"], "abc");
        assert_eq!(data.features[0]["properties"]["nested"]["value"], 2);
        assert!(data
            .primitives
            .iter()
            .flat_map(|p| p.vertices.iter())
            .all(|v| v.feature_id == 0 && v.position.iter().all(|n| n.is_finite())));
        assert!(decode_value(json!({"type":"Point","coordinates":[181,0]})).is_err());
        assert!(decode_value(json!({"type":"Point","coordinates":[0,91]})).is_err());
        let bytes = serde_json::to_vec(&json!({"type":"Point","coordinates":[110,25]})).unwrap();
        let small = Limits {
            max_vertices: 2,
            ..Default::default()
        };
        assert!(decode(&bytes, &Style::default(), &small).is_err());
        let empty =
            decode_value(json!({"type":"Feature","properties":null,"geometry":null})).unwrap();
        assert!(empty.primitives.is_empty());
    }
    #[test]
    fn holes_dateline_and_invalid_polygons() {
        let polygon = json!({"type":"Polygon","coordinates":[[[105,20],[115,20],[115,30],[105,30],[105,20]],[[108,23],[108,27],[112,27],[112,23],[108,23]]]});
        let data = decode_value(polygon).unwrap();
        let p = &data.primitives[0];
        let mut area = 0.0;
        for t in p.indices.as_chunks::<3>().0 {
            let c: Vec<_> = t
                .iter()
                .map(|i| {
                    Cartographic::from_ecef(p.transform.transform_point3(DVec3::from_array(
                        p.vertices[*i as usize].position.map(f64::from),
                    )))
                    .unwrap()
                })
                .collect();
            let x: Vec<_> = c
                .iter()
                .map(|c| Coord {
                    lon: c.longitude.to_degrees(),
                    lat: c.latitude.to_degrees(),
                    height: 0.0,
                })
                .collect();
            area += orient(x[0], x[1], x[2]).abs() * 0.5;
            let (lon, lat) = (
                (x[0].lon + x[1].lon + x[2].lon) / 3.0,
                (x[0].lat + x[1].lat + x[2].lat) / 3.0,
            );
            assert!(!(lon > 108.01 && lon < 111.99 && lat > 23.01 && lat < 26.99));
        }
        assert!((area - 84.0).abs() < 0.01, "filled area {area}");
        let dateline = decode_value(
            json!({"type":"Polygon","coordinates":[[[179,0],[-179,0],[-179,1],[179,1],[179,0]]]}),
        )
        .unwrap();
        assert!(dateline.primitives[0]
            .vertices
            .iter()
            .all(|v| dateline.primitives[0]
                .transform
                .transform_point3(DVec3::from_array(v.position.map(f64::from)))
                .x
                < -6000000.0));
        assert!(decode_value(
            json!({"type":"Polygon","coordinates":[[[0,0],[1,1],[0,1],[1,0],[0,0]]]})
        )
        .is_err());
        assert!(
            decode_value(json!({"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1]]]}))
                .is_err()
        );
    }
    #[test]
    fn decimal_ring_closure_and_wide_ribbon_curvature() {
        let data=decode_value(json!({"type":"Polygon","coordinates":[[[109.2,25],[109.6,25],[109.6,25.4],[109.2,25.4],[109.2,25]]]})).unwrap();
        assert!(!data.primitives[0].indices.is_empty());
        let style = Style {
            line_width: 100000.,
            surface_offset: 10.,
            segment_degrees: 0.1,
            ..Default::default()
        };
        let data = decode(
            &serde_json::to_vec(&json!({"type":"LineString","coordinates":[[0,0],[1,0]]})).unwrap(),
            &style,
            &Default::default(),
        )
        .unwrap();
        for p in data.primitives {
            for t in p.indices.as_chunks::<3>().0 {
                let center = t
                    .iter()
                    .map(|i| {
                        p.transform.transform_point3(DVec3::from_array(
                            p.vertices[*i as usize].position.map(f64::from),
                        ))
                    })
                    .sum::<DVec3>()
                    / 3.;
                assert!(
                    Cartographic::from_ecef(center).unwrap().height > 0.,
                    "ribbon triangle interiors must stay above ellipsoid"
                );
            }
        }
    }
    #[test]
    #[ignore = "requires a native GPU or Mesa Vulkan software adapter"]
    fn globe_overlay_hole_pick_and_layer_lifecycle() {
        pollster::block_on(async {
            use cesium_renderer::{attachment, device, instance, Renderer};
            let inst = instance();
            let (_, device, queue) = device(&inst, None).await.unwrap();
            let mut renderer =
                Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 800, 600)
                    .unwrap();
            let mut scene = crate::Scene::new(&mut renderer, None).unwrap();
            let style = Style {
                point_radius: 100000.0,
                surface_offset: 10000.0,
                ..Default::default()
            };
            let polygon = json!({"type":"Feature","id":"area","properties":{"name":"Area"},"geometry":{"type":"Polygon","coordinates":[[[105,20],[115,20],[115,30],[105,30],[105,20]],[[108,23],[108,27],[112,27],[112,23],[108,23]]]}});
            let data = decode(
                &serde_json::to_vec(&polygon).unwrap(),
                &style,
                &Limits::default(),
            )
            .unwrap();
            let area = scene.add_geojson(&mut renderer, data).unwrap();
            let target = attachment(
                &renderer.device,
                800,
                600,
                renderer.format(),
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            );
            let render = |scene: &crate::Scene, r: &mut Renderer| {
                scene
                    .render(
                        r,
                        &target.create_view(&Default::default()),
                        Some((400, 300)),
                    )
                    .unwrap();
                r.device
                    .poll(wgpu::PollType::Wait {
                        submission_index: None,
                        timeout: Some(std::time::Duration::from_secs(30)),
                    })
                    .unwrap();
                r.poll_pick().unwrap()
            };
            assert_eq!(render(&scene, &mut renderer), None); // Hole passes through to non-pickable Earth.
            scene.orbit.yaw = 106.0f64.to_radians();
            let picked = render(&scene, &mut renderer).unwrap();
            assert_eq!(
                scene.properties(picked).unwrap()["properties"]["name"],
                "Area"
            );
            scene.orbit.yaw = 110.0f64.to_radians();
            let point = json!({"type":"Feature","id":"pin","properties":{"name":"Marker"},"geometry":{"type":"Point","coordinates":[110,25]}});
            let point = scene
                .add_geojson(
                    &mut renderer,
                    decode(
                        &serde_json::to_vec(&point).unwrap(),
                        &style,
                        &Limits::default(),
                    )
                    .unwrap(),
                )
                .unwrap();
            let picked = render(&scene, &mut renderer).unwrap();
            assert_eq!(scene.properties(picked).unwrap()["id"], "pin");
            assert!(scene.remove_geojson(point));
            assert_eq!(scene.properties(picked), None);
            assert_eq!(render(&scene, &mut renderer), None);
            assert!(scene.focus_geojson(area));
            assert!(scene.orbit.height() > 0.0);
            scene.clear_geojson();
            assert!(!scene.remove_geojson(area));
            let pole = scene
                .add_geojson(
                    &mut renderer,
                    decode(
                        &serde_json::to_vec(&json!({"type":"Point","coordinates":[0,90]})).unwrap(),
                        &style,
                        &Limits::default(),
                    )
                    .unwrap(),
                )
                .unwrap();
            assert!(scene.focus_geojson(pole));
            assert!(scene.orbit.camera(800, 600).eye.is_finite());
            assert!(render(&scene, &mut renderer).is_some());
        });
    }
}
