//! Bounded streaming terrain: global geographic quantized-mesh and Mercator Terrarium.
use crate::imagery::{self, Projection, Tile};
use cesium_formats::{Material, Primitive, Vertex};
use cesium_math::{Camera, Cartographic, Sphere, WGS84_A};
use cesium_renderer::{BaseColorTexture, GpuMesh, RenderError, Renderer};
use glam::{DMat4, DVec3};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    f64::consts::{PI, TAU},
    io::Read,
};
use url::Url;
pub const MAX_TILE_BYTES: usize = 8 * 1024 * 1024;
const MAX_VERTICES: usize = 262_144;
const MAX_INDICES: usize = 1_500_000;
const CACHE_BUDGET: usize = 128 * 1024 * 1024;
const MAX_DRAWS: usize = 128;
const ACCEPT: &str = "application/vnd.quantized-mesh,application/octet-stream;q=0.9";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    QuantizedMesh,
    Terrarium,
}
#[derive(Clone, Debug)]
struct Rect {
    start_x: u32,
    start_y: u32,
    end_x: u32,
    end_y: u32,
}
#[derive(Clone)]
pub struct Provider {
    pub format: Format,
    pub projection: Projection,
    pub max_level: u32,
    pub credits: String,
    source: Url,
    templates: Vec<String>,
    version: String,
    available: HashMap<u32, Vec<Rect>>,
    metadata: bool,
    authorization: Option<String>,
    extensions: bool,
}
impl Provider {
    pub fn public_terrain() -> Self {
        Self::terrarium(
            "https://s3.amazonaws.com/elevation-tiles-prod/terrarium/{z}/{x}/{y}.png",
            15,
        )
        .expect("static terrain URL")
    }
    pub fn terrarium(template: &str, max_level: u32) -> Result<Self, String> {
        let source = Url::parse(
            &template
                .replace("{z}", "0")
                .replace("{x}", "0")
                .replace("{y}", "0"),
        )
        .map_err(|_| "terrain URL")?;
        if !matches!(source.scheme(), "http" | "https" | "file")
            || !(1..=15).contains(&max_level)
            || !["{z}", "{x}", "{y}"].iter().all(|s| template.contains(s))
        {
            return Err("Terrarium requires an XYZ template and level 1..15".into());
        }
        Ok(Self{format:Format::Terrarium,projection:Projection::Mercator,max_level,credits:"Mapzen / Tilezen Terrain Tiles · sources: https://github.com/tilezen/joerd/blob/master/docs/attribution.md · polar ellipsoid fallback".into(),source,templates:vec![template.into()],version:String::new(),available:HashMap::new(),metadata:false,authorization:None,extensions:false})
    }
    pub fn layer(data: &[u8], source: Url) -> Result<Self, String> {
        if data.len() > 1024 * 1024 {
            return Err("terrain layer JSON byte limit".into());
        }
        if !matches!(source.scheme(), "http" | "https" | "file") {
            return Err("terrain layer URL scheme".into());
        }
        let v: Value = serde_json::from_slice(data).map_err(|_| "invalid terrain layer JSON")?;
        if v.get("format")
            .and_then(Value::as_str)
            .unwrap_or("quantized-mesh-1.0")
            != "quantized-mesh-1.0"
        {
            return Err("unsupported terrain format (requires quantized-mesh-1.0)".into());
        }
        if v.get("projection")
            .and_then(Value::as_str)
            .is_some_and(|p| p != "EPSG:4326")
            || v.get("scheme")
                .and_then(Value::as_str)
                .is_some_and(|s| s != "tms")
            || v.get("parentUrl").is_some()
        {
            return Err("terrain requires geographic TMS; parent layers are unsupported".into());
        }
        let templates: Vec<String> = v["tiles"]
            .as_array()
            .ok_or("terrain tiles templates required")?
            .iter()
            .map(|s| s.as_str().map(str::to_owned).ok_or("terrain tile template"))
            .collect::<Result<_, _>>()?;
        if templates.is_empty()
            || templates.len() > 16
            || templates
                .iter()
                .any(|s| s.len() > 4096 || !["{z}", "{x}", "{y}"].iter().all(|p| s.contains(p)))
        {
            return Err("terrain tile templates/count".into());
        }
        let max_level = v
            .get("maxzoom")
            .map(|n| n.as_u64().ok_or("terrain maxzoom"))
            .transpose()?
            .unwrap_or(18);
        if max_level > 20 {
            return Err("terrain maximum level exceeds 20".into());
        }
        let metadata = v
            .get("metadataAvailability")
            .map(|n| {
                n.as_u64()
                    .filter(|n| *n > 0 && *n <= 20)
                    .ok_or("metadataAvailability 1..20")
            })
            .transpose()?
            .is_some();
        let mut p = Self {
            format: Format::QuantizedMesh,
            projection: Projection::Geographic,
            max_level: max_level as u32,
            credits: v["attribution"]
                .as_str()
                .unwrap_or("Quantized-mesh terrain")
                .into(),
            source,
            templates,
            version: v["version"].as_str().unwrap_or("1.0.0").into(),
            available: HashMap::new(),
            metadata,
            authorization: None,
            extensions: v["extensions"]
                .as_array()
                .is_some_and(|a| a.iter().any(|s| s == "metadata")),
        };
        if !metadata {
            if let Some(a) = v.get("available") {
                p.add_available(a, 0)?;
            }
        }
        for t in p.roots() {
            p.url(t)?;
        }
        Ok(p)
    }
    pub fn ion_endpoint(token: &str) -> Result<Url, String> {
        if token.trim().is_empty() {
            return Err("Cesium ion token required".into());
        }
        let mut u = Url::parse("https://api.cesium.com/v1/assets/1/endpoint").unwrap();
        u.query_pairs_mut().append_pair("access_token", token);
        Ok(u)
    }
    pub fn ion_layer(endpoint: &Value) -> Result<(Url, String), String> {
        if endpoint["type"] != "TERRAIN" {
            return Err("ion asset is not terrain".into());
        }
        let base = Url::parse(endpoint["url"].as_str().ok_or("ion terrain URL")?)
            .map_err(|_| "ion terrain URL")?;
        if !matches!(base.scheme(), "https" | "http")
            || base.username() != ""
            || base.password().is_some()
        {
            return Err("ion terrain URL scheme/credentials".into());
        }
        let token = endpoint["accessToken"]
            .as_str()
            .ok_or("ion terrain access token")?;
        if token.is_empty() || token.len() > 8192 || token.contains(['\r', '\n']) {
            return Err("ion terrain access token".into());
        }
        Ok((
            base.join("layer.json").map_err(|_| "ion layer URL")?,
            format!("Bearer {token}"),
        ))
    }
    pub fn with_authorization(mut self, authorization: String) -> Result<Self, String> {
        if authorization.len() > 8192
            || !authorization.starts_with("Bearer ")
            || authorization.contains(['\r', '\n'])
        {
            return Err("invalid terrain authorization".into());
        }
        self.authorization = Some(authorization);
        for t in self.roots() {
            self.url(t)?;
        }
        Ok(self)
    }
    fn add_available(&mut self, v: &Value, start: u32) -> Result<(), String> {
        let levels = v.as_array().ok_or("terrain availability array")?;
        if levels.len() > 21 || start + levels.len() as u32 > 21 {
            return Err("terrain availability levels".into());
        }
        for (i, level) in levels.iter().enumerate() {
            let z = start + i as u32;
            let n = 1u32 << z;
            let a = level.as_array().ok_or("terrain availability rectangles")?;
            if a.len() > 4096 {
                return Err("terrain availability rectangle limit".into());
            }
            let mut added = Vec::new();
            for r in a {
                let read = |k: &str| {
                    r[k].as_u64()
                        .and_then(|v| u32::try_from(v).ok())
                        .ok_or("terrain availability coordinate")
                };
                let r = Rect {
                    start_x: read("startX")?,
                    start_y: read("startY")?,
                    end_x: read("endX")?,
                    end_y: read("endY")?,
                };
                if r.start_x > r.end_x || r.start_y > r.end_y || r.end_x >= 2 * n || r.end_y >= n {
                    return Err("terrain availability bounds".into());
                }
                added.push(r);
            }
            let old = self.available.entry(z).or_default();
            for r in added {
                if !old.iter().any(|o| {
                    o.start_x == r.start_x
                        && o.start_y == r.start_y
                        && o.end_x == r.end_x
                        && o.end_y == r.end_y
                }) {
                    old.push(r);
                }
            }
        }
        if self.available.values().map(Vec::len).sum::<usize>() > 8192 {
            return Err("terrain availability budget".into());
        }
        Ok(())
    }
    pub fn roots(&self) -> Vec<Tile> {
        match self.projection {
            Projection::Geographic => (0..2).map(|x| Tile { z: 0, x, y: 0 }).collect(),
            Projection::Mercator => (0..4)
                .map(|i| Tile {
                    z: 1,
                    x: i % 2,
                    y: i / 2,
                })
                .collect(),
        }
    }
    fn available(&self, t: Tile) -> bool {
        if t.z > self.max_level {
            return false;
        }
        if t.z == 0 {
            return true;
        }
        if self.available.is_empty() && !self.metadata {
            return true;
        }
        let y = (1u32 << t.z) - 1 - t.y;
        self.available.get(&t.z).is_some_and(|a| {
            a.iter()
                .any(|r| t.x >= r.start_x && t.x <= r.end_x && y >= r.start_y && y <= r.end_y)
        })
    }
    pub fn url(&self, t: Tile) -> Result<Url, String> {
        let n = 1u32.checked_shl(t.z).ok_or("terrain tile level")?;
        let width = n * if self.projection == Projection::Geographic {
            2
        } else {
            1
        };
        if t.z > self.max_level || t.x >= width || t.y >= n {
            return Err("terrain tile coordinates".into());
        }
        let y = if self.format == Format::QuantizedMesh {
            n - 1 - t.y
        } else {
            t.y
        };
        let template = &self.templates[(t.x as usize + t.y as usize) % self.templates.len()];
        let template = template
            .replace("{z}", &t.z.to_string())
            .replace("{x}", &t.x.to_string())
            .replace("{y}", &y.to_string())
            .replace("{version}", &self.version);
        if template.contains('{') {
            return Err("unknown terrain URL placeholder".into());
        }
        let mut url = self
            .source
            .join(&template)
            .map_err(|_| "terrain tile URL")?;
        if !matches!(url.scheme(), "http" | "https" | "file") {
            return Err("terrain tile URL scheme".into());
        }
        if self.authorization.is_some()
            && (url.origin() != self.source.origin()
                || !url.path().starts_with(
                    self.source
                        .path()
                        .rsplit_once('/')
                        .map_or("/".to_owned(), |v| format!("{}/", v.0))
                        .as_str(),
                ))
        {
            return Err("terrain authorization cannot leave its service".into());
        }
        if url.origin() == self.source.origin() {
            for (k, v) in self.source.query_pairs() {
                if !url.query_pairs().any(|(q, _)| q == k) {
                    url.query_pairs_mut().append_pair(&k, &v);
                }
            }
        }
        if self.extensions {
            url.query_pairs_mut().append_pair("extensions", "metadata");
        }
        Ok(url)
    }
    pub fn authorization(&self) -> Option<&str> {
        self.authorization.as_deref()
    }
    pub fn accept(&self) -> &str {
        if self.format == Format::QuantizedMesh {
            ACCEPT
        } else {
            "image/png"
        }
    }
}
pub fn coord(t: Tile, p: Projection, u: f64, v: f64) -> (f64, f64) {
    let n = (1u32 << t.z) as f64;
    let x = (t.x as f64 + u)
        / (n * if p == Projection::Geographic {
            2.0
        } else {
            1.0
        });
    let y = (t.y as f64 + v) / n;
    (
        x * TAU - PI,
        if p == Projection::Geographic {
            PI * 0.5 - y * PI
        } else {
            (PI * (1.0 - 2.0 * y)).sinh().atan()
        },
    )
}
fn tile_uv(t: Tile, p: Projection, lon: f64, lat: f64) -> [f64; 2] {
    let n = (1u32 << t.z) as f64;
    let x = (lon + PI) / TAU
        * n
        * if p == Projection::Geographic {
            2.0
        } else {
            1.0
        };
    let y = if p == Projection::Geographic {
        (PI * 0.5 - lat) / PI * n
    } else {
        (1.0 - (lat.tan() + 1.0 / lat.cos()).ln() / PI) * 0.5 * n
    };
    [x - t.x as f64, y - t.y as f64]
}
#[derive(Clone)]
pub struct Data {
    uv: Vec<[f64; 2]>,
    heights: Vec<f32>,
    indices: Vec<u32>,
    edges: [Vec<u32>; 4],
    bins: Vec<Vec<u32>>,
    pub metadata: Option<Value>,
    pub min_height: f32,
    pub max_height: f32,
}
impl Data {
    fn finish(mut self) -> Result<Self, String> {
        if self.uv.len() > MAX_VERTICES || self.indices.len() > MAX_INDICES {
            return Err("terrain decoded geometry budget".into());
        }
        self.bins = vec![Vec::new(); 32 * 32];
        let mut refs = 0usize;
        for (i, tri) in self.indices.as_chunks::<3>().0.iter().enumerate() {
            let a = self.uv[tri[0] as usize];
            let b = self.uv[tri[1] as usize];
            let c = self.uv[tri[2] as usize];
            let xmin = (a[0].min(b[0]).min(c[0]) * 32.0).floor().clamp(0.0, 31.0) as usize;
            let xmax = (a[0].max(b[0]).max(c[0]) * 32.0).floor().clamp(0.0, 31.0) as usize;
            let ymin = (a[1].min(b[1]).min(c[1]) * 32.0).floor().clamp(0.0, 31.0) as usize;
            let ymax = (a[1].max(b[1]).max(c[1]) * 32.0).floor().clamp(0.0, 31.0) as usize;
            refs += (xmax - xmin + 1) * (ymax - ymin + 1);
            if refs > 1_000_000 {
                return Err("terrain height-query acceleration budget".into());
            }
            for y in ymin..=ymax {
                for x in xmin..=xmax {
                    self.bins[y * 32 + x].push(i as u32);
                }
            }
        }
        Ok(self)
    }
    pub fn sample(&self, uv: [f64; 2]) -> Option<f64> {
        if uv
            .iter()
            .any(|v| !v.is_finite() || *v < -1e-10 || *v > 1.0 + 1e-10)
        {
            return None;
        }
        let x = (uv[0] * 32.0).floor().clamp(0.0, 31.0) as usize;
        let y = (uv[1] * 32.0).floor().clamp(0.0, 31.0) as usize;
        for &id in &self.bins[y * 32 + x] {
            let ids = &self.indices[id as usize * 3..id as usize * 3 + 3];
            let a = self.uv[ids[0] as usize];
            let b = self.uv[ids[1] as usize];
            let c = self.uv[ids[2] as usize];
            let den = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
            if den.abs() < 1e-16 {
                continue;
            }
            let w0 = ((b[1] - c[1]) * (uv[0] - c[0]) + (c[0] - b[0]) * (uv[1] - c[1])) / den;
            let w1 = ((c[1] - a[1]) * (uv[0] - c[0]) + (a[0] - c[0]) * (uv[1] - c[1])) / den;
            let w2 = 1.0 - w0 - w1;
            if [w0, w1, w2].iter().all(|v| *v >= -1e-8) {
                return Some(
                    w0 * self.heights[ids[0] as usize] as f64
                        + w1 * self.heights[ids[1] as usize] as f64
                        + w2 * self.heights[ids[2] as usize] as f64,
                );
            }
        }
        None
    }
    fn bytes(&self) -> usize {
        self.uv.len() * 16
            + self.heights.len() * 4
            + self.indices.len() * 4
            + self.bins.iter().map(|b| b.len() * 4 + 24).sum::<usize>()
            + self.edges.iter().map(|e| e.len() * 4).sum::<usize>()
    }
    fn primitive(&self, t: Tile, p: Projection, scale: f64) -> Primitive {
        let (lon, lat) = coord(t, p, 0.5, 0.5);
        let center = Cartographic::radians(lon, lat, 0.0).unwrap().to_ecef();
        let mut positions = Vec::new();
        let mut normals = vec![DVec3::ZERO; self.uv.len()];
        for (&uv, &h) in self.uv.iter().zip(&self.heights) {
            let (lon, lat) = coord(t, p, uv[0], uv[1]);
            positions.push(
                Cartographic::radians(lon, lat, h as f64 * scale)
                    .unwrap()
                    .to_ecef()
                    - center,
            );
        }
        for tri in self.indices.as_chunks::<3>().0 {
            let ids = tri.map(|i| i as usize);
            let normal = (positions[ids[1]] - positions[ids[0]])
                .cross(positions[ids[2]] - positions[ids[0]]);
            for i in ids {
                normals[i] += normal;
            }
        }
        let mut vertices: Vec<Vertex> = positions
            .iter()
            .enumerate()
            .map(|(i, pos)| {
                let (lon, lat) = coord(t, p, self.uv[i][0], self.uv[i][1]);
                let outward = DVec3::new(lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin());
                let mut n = normals[i].try_normalize().unwrap_or(outward);
                if n.dot(outward) < 0.0 {
                    n = -n;
                }
                Vertex {
                    position: pos.as_vec3().to_array(),
                    normal: n.as_vec3().to_array(),
                    uv: self.uv[i].map(|v| v as f32),
                    uv1: [0.0; 2],
                    color: [1.0; 4],
                    tangent: [0.0; 4],
                    feature_id: 0,
                }
            })
            .collect();
        let mut indices = self.indices.clone();
        let depth = (50_000.0 / (1u32 << t.z) as f64)
            .max(30.0)
            .max((self.max_height - self.min_height) as f64 / 8.0)
            * scale;
        for edge in &self.edges {
            for pair in edge.windows(2) {
                let start = vertices.len() as u32;
                for &id in pair {
                    let mut v = vertices[id as usize];
                    let (lon, lat) = coord(t, p, self.uv[id as usize][0], self.uv[id as usize][1]);
                    let point = Cartographic::radians(
                        lon,
                        lat,
                        self.heights[id as usize] as f64 * scale - depth,
                    )
                    .unwrap()
                    .to_ecef()
                        - center;
                    v.position = point.as_vec3().to_array();
                    vertices.push(v);
                }
                indices.extend_from_slice(&[pair[0], start, pair[1], pair[1], start, start + 1]);
            }
        }
        Primitive {
            vertices,
            indices,
            transform: DMat4::from_translation(center),
            material: Material {
                metallic: 0.0,
                roughness: 0.85,
                double_sided: true,
                ..Default::default()
            },
        }
    }
}
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.pos.checked_add(n).ok_or("terrain range overflow")?;
        let v = self.bytes.get(self.pos..end).ok_or("truncated terrain")?;
        self.pos = end;
        Ok(v)
    }
    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, String> {
        let v = f32::from_le_bytes(self.take(4)?.try_into().unwrap());
        if !v.is_finite() {
            return Err("non-finite terrain header".into());
        }
        Ok(v)
    }
    fn f64(&mut self) -> Result<f64, String> {
        let v = f64::from_le_bytes(self.take(8)?.try_into().unwrap());
        if !v.is_finite() {
            return Err("non-finite terrain header".into());
        }
        Ok(v)
    }
    fn index(&mut self, wide: bool) -> Result<u32, String> {
        if wide {
            self.u32()
        } else {
            self.u16().map(u32::from)
        }
    }
}
pub fn decode(bytes: &[u8], format: Format) -> Result<Data, String> {
    if bytes.len() > MAX_TILE_BYTES {
        return Err("terrain encoded byte limit".into());
    }
    let uncompressed = if bytes.starts_with(&[0x1f, 0x8b]) {
        let mut out = Vec::new();
        flate2::read::GzDecoder::new(bytes)
            .take(MAX_TILE_BYTES as u64 + 1)
            .read_to_end(&mut out)
            .map_err(|_| "invalid gzip terrain")?;
        if out.len() > MAX_TILE_BYTES {
            return Err("terrain gzip output byte limit".into());
        }
        out
    } else {
        bytes.to_vec()
    };
    match format {
        Format::QuantizedMesh => decode_quantized(&uncompressed),
        Format::Terrarium => decode_terrarium(&uncompressed),
    }
}
fn decode_quantized(bytes: &[u8]) -> Result<Data, String> {
    let mut r = Reader { bytes, pos: 0 };
    for _ in 0..3 {
        if r.f64()?.abs() > 20_000_000.0 {
            return Err("terrain center range".into());
        }
    }
    let min = r.f32()?;
    let max = r.f32()?;
    if min > max || min < -100_000.0 || max > 100_000.0 {
        return Err("terrain height range".into());
    }
    for _ in 0..3 {
        r.f64()?;
    }
    if r.f64()? < 0.0 {
        return Err("terrain bounding radius".into());
    }
    for _ in 0..3 {
        r.f64()?;
    }
    let n = r.u32()? as usize;
    if !(3..=MAX_VERTICES).contains(&n) {
        return Err("terrain vertex count".into());
    }
    r.bytes
        .get(r.pos..r.pos + n * 6)
        .ok_or("truncated terrain vertices")?;
    let mut arrays = Vec::new();
    for _ in 0..3 {
        let mut previous = 0i64;
        let mut a = Vec::with_capacity(n);
        for _ in 0..n {
            let code = r.u16()? as i64;
            previous += (code >> 1) ^ -(code & 1);
            if !(0..=32767).contains(&previous) {
                return Err("terrain delta coordinate out of range".into());
            }
            a.push(previous as f64 / 32767.0);
        }
        arrays.push(a);
    }
    let wide = n > 65536;
    let align = if wide { 4 } else { 2 };
    r.take((align - r.pos % align) % align)?;
    let triangles = r.u32()? as usize;
    let count = triangles.checked_mul(3).ok_or("terrain index overflow")?;
    if count == 0 || count > MAX_INDICES {
        return Err("terrain index count".into());
    }
    r.bytes
        .get(r.pos..r.pos + count * if wide { 4 } else { 2 })
        .ok_or("truncated terrain indices")?;
    let mut indices = Vec::with_capacity(count);
    let mut highest = 0u32;
    for _ in 0..count {
        let code = r.index(wide)?;
        let id = highest.checked_sub(code).ok_or("terrain high water mark")?;
        if id as usize >= n {
            return Err("terrain triangle index out of range".into());
        }
        indices.push(id);
        if code == 0 {
            highest += 1;
        }
    }
    let uv: Vec<_> = (0..n).map(|i| [arrays[0][i], 1.0 - arrays[1][i]]).collect();
    let heights = (0..n)
        .map(|i| (min as f64 + (max - min) as f64 * arrays[2][i]) as f32)
        .collect();
    let mut edges = std::array::from_fn(|_| Vec::new());
    let mut edge_total = 0usize;
    for (edge, list) in edges.iter_mut().enumerate() {
        let count = r.u32()? as usize;
        edge_total += count;
        if count > n || edge_total > MAX_VERTICES {
            return Err("terrain edge count".into());
        }
        let mut seen = HashSet::new();
        for _ in 0..count {
            let id = r.index(wide)?;
            if id as usize >= n || !seen.insert(id) {
                return Err("terrain edge index/duplicate".into());
            }
            let v = uv[id as usize];
            let coordinate = match edge {
                0 => v[0],
                1 => 1.0 - v[1],
                2 => 1.0 - v[0],
                _ => v[1],
            };
            if coordinate.abs() > 1e-6 {
                return Err("terrain edge vertex is not on its boundary".into());
            }
            list.push(id);
        }
        let axis = if edge % 2 == 0 { 1 } else { 0 };
        list.sort_by(|a, b| uv[*a as usize][axis].total_cmp(&uv[*b as usize][axis]));
    }
    let mut metadata = None;
    let mut seen = HashSet::new();
    while r.pos < bytes.len() {
        let id = r.take(1)?[0];
        let size = r.u32()? as usize;
        let extension = r.take(size)?;
        if !seen.insert(id) {
            return Err("duplicate terrain extension".into());
        }
        match id {
            1 => {
                if size != n * 2 {
                    return Err("terrain oct-normal extension size".into());
                }
            }
            2 => {
                if size != 1 && size != 65536 {
                    return Err("terrain watermask size".into());
                }
            }
            4 => {
                if !(4..=1024 * 1024).contains(&size) {
                    return Err("terrain metadata size".into());
                }
                let length = u32::from_le_bytes(extension[..4].try_into().unwrap()) as usize;
                if length != size - 4 {
                    return Err("terrain metadata JSON length".into());
                }
                metadata = Some(
                    serde_json::from_slice(&extension[4..]).map_err(|_| "terrain metadata JSON")?,
                );
            }
            _ => {}
        }
    }
    Data {
        uv,
        heights,
        indices,
        edges,
        bins: Vec::new(),
        metadata,
        min_height: min,
        max_height: max,
    }
    .finish()
}
fn grid_data(steps: u32, mut height: impl FnMut(f64, f64) -> f32) -> Result<Data, String> {
    let mut uv = Vec::new();
    let mut heights = Vec::new();
    for y in 0..=steps {
        for x in 0..=steps {
            let u = x as f64 / steps as f64;
            let v = y as f64 / steps as f64;
            uv.push([u, v]);
            let h = height(u, v);
            if !h.is_finite() || !(-100_000.0..=100_000.0).contains(&h) {
                return Err("terrain raster height range".into());
            }
            heights.push(h);
        }
    }
    let mut indices = Vec::new();
    for y in 0..steps {
        for x in 0..steps {
            let a = y * (steps + 1) + x;
            let b = a + steps + 1;
            indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    let edges = [
        (0..=steps).map(|y| y * (steps + 1)).collect(),
        (0..=steps).map(|x| steps * (steps + 1) + x).collect(),
        (0..=steps).map(|y| y * (steps + 1) + steps).collect(),
        (0..=steps).collect(),
    ];
    let min = heights.iter().copied().fold(f32::INFINITY, f32::min);
    let max = heights.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    Data {
        uv,
        heights,
        indices,
        edges,
        bins: Vec::new(),
        metadata: None,
        min_height: min,
        max_height: max,
    }
    .finish()
}
fn decode_terrarium(bytes: &[u8]) -> Result<Data, String> {
    let reader =
        image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Png);
    let mut decoder = reader.into_decoder().map_err(|_| "invalid Terrarium PNG")?;
    use image::ImageDecoder;
    let (w, h) = decoder.dimensions();
    if w != 256 || h != 256 {
        return Err("Terrarium requires 256x256 PNG".into());
    }
    decoder
        .set_limits(image::Limits::default())
        .map_err(|_| "Terrarium image limits")?;
    let image = image::DynamicImage::from_decoder(decoder)
        .map_err(|_| "Terrarium image decode")?
        .to_rgba8();
    grid_data(64, |u, v| {
        let pixel = image.get_pixel((u * 255.0).round() as u32, (v * 255.0).round() as u32);
        if pixel[3] == 0 {
            return 0.0;
        }
        pixel[0] as f32 * 256.0 + pixel[1] as f32 + pixel[2] as f32 / 256.0 - 32768.0
    })
}
struct Resident {
    data: Data,
    mesh: GpuMesh,
    last: u64,
    revision: (u64, u64),
}
#[derive(Default)]
struct Request {
    inflight: bool,
    attempts: u8,
    retry: u64,
}
pub struct Terrain {
    pub provider: Provider,
    resident: HashMap<Tile, Resident>,
    fallback: HashMap<Tile, GpuMesh>,
    polar: Vec<GpuMesh>,
    base: image::RgbaImage,
    draws: Vec<Tile>,
    wanted: HashSet<Tile>,
    requests: HashMap<Tile, Request>,
    frame: u64,
    revision: u64,
    pub exaggeration: f64,
    pub max_sse: f64,
    pub errors: usize,
}
impl Terrain {
    pub fn new(renderer: &mut Renderer, provider: Provider) -> Result<Self, RenderError> {
        let base = image::load_from_memory(include_bytes!("../../../assets/earth-base-color.png"))
            .map_err(|e| RenderError(e.to_string()))?
            .to_rgba8();
        let mut out = Self {
            provider,
            resident: HashMap::new(),
            fallback: HashMap::new(),
            polar: Vec::new(),
            base,
            draws: Vec::new(),
            wanted: HashSet::new(),
            requests: HashMap::new(),
            frame: 0,
            revision: 0,
            exaggeration: 1.0,
            max_sse: 8.0,
            errors: 0,
        };
        let flat = grid_data(64, |_, _| 0.0).map_err(RenderError)?;
        for tile in out.provider.roots() {
            let primitive = flat.primitive(tile, out.provider.projection, 1.0);
            let texture = out.atlas(tile, None);
            let mesh = renderer.upload_terrain(
                &primitive,
                &BaseColorTexture {
                    width: 256,
                    height: 256,
                    rgba: texture.as_raw(),
                },
            )?;
            out.fallback.insert(tile, mesh);
        }
        if out.provider.projection == Projection::Mercator {
            for north in [true, false] {
                let p = imagery::polar_patch(north);
                out.polar.push(renderer.upload_globe(
                    &p,
                    &BaseColorTexture {
                        width: out.base.width(),
                        height: out.base.height(),
                        rgba: out.base.as_raw(),
                    },
                )?);
            }
        }
        Ok(out)
    }
    fn atlas(&self, t: Tile, imagery: Option<&imagery::Imagery>) -> image::RgbaImage {
        let mut out = image::RgbaImage::new(256, 256);
        for (x, y, pixel) in out.enumerate_pixels_mut() {
            let (lon, lat) = coord(
                t,
                self.provider.projection,
                x as f64 / 255.0,
                y as f64 / 255.0,
            );
            *pixel = imagery
                .and_then(|i| i.sample_color(lon, lat))
                .unwrap_or_else(|| {
                    let x = ((lon + PI) / TAU * (self.base.width() - 1) as f64)
                        .round()
                        .clamp(0.0, (self.base.width() - 1) as f64)
                        as u32;
                    let y = ((PI * 0.5 - lat) / PI * (self.base.height() - 1) as f64)
                        .round()
                        .clamp(0.0, (self.base.height() - 1) as f64)
                        as u32;
                    *self.base.get_pixel(x, y)
                });
        }
        out
    }
    fn bounds(&self, t: Tile) -> (Sphere, f64) {
        let (lon, lat) = coord(t, self.provider.projection, 0.5, 0.5);
        let (w, s) = coord(t, self.provider.projection, 0.0, 1.0);
        let (e, n) = coord(t, self.provider.projection, 1.0, 0.0);
        let angle = ((e - w).abs() + (n - s).abs()) * 0.5;
        let margin = self.resident.get(&t).map_or(12_000.0, |r| {
            r.data.min_height.abs().max(r.data.max_height.abs()) as f64
        }) * self.exaggeration.max(1.0)
            + 1000.0;
        let radius = WGS84_A * angle.min(2.0) + margin;
        (
            Sphere {
                center: Cartographic::radians(lon, lat, 0.0).unwrap().to_ecef(),
                radius,
            },
            angle,
        )
    }
    fn visible(&self, t: Tile, c: Camera) -> bool {
        let (b, angle) = self.bounds(t);
        let horizon = (cesium_math::WGS84_B / c.eye.length())
            .clamp(0.0, 1.0)
            .acos();
        c.visible(b)
            && b.center
                .normalize()
                .dot(c.eye.normalize())
                .clamp(-1.0, 1.0)
                .acos()
                <= horizon + angle + 0.04
    }
    fn select(&mut self, t: Tile, c: Camera) {
        if !self.visible(t, c) {
            return;
        }
        self.wanted.insert(t);
        let (b, angle) = self.bounds(t);
        let ready = self.resident.contains_key(&t);
        if ready
            && t.z < self.provider.max_level
            && self.draws.len() + 4 < MAX_DRAWS
            && self.wanted.len() < 512
            && c.sse(b, WGS84_A * angle / 128.0) > self.max_sse
        {
            let children: Vec<_> = (0..4)
                .map(|i| Tile {
                    z: t.z + 1,
                    x: t.x * 2 + i % 2,
                    y: t.y * 2 + i / 2,
                })
                .filter(|child| self.visible(*child, c))
                .collect();
            let mut all = true;
            for &child in &children {
                if !self.provider.available(child) || !self.resident.contains_key(&child) {
                    all = false;
                }
                if self.provider.available(child) {
                    self.wanted.insert(child);
                }
            }
            if all && !children.is_empty() {
                for child in children {
                    self.select(child, c);
                }
                return;
            }
        }
        self.draws.push(t);
    }
    pub fn update(
        &mut self,
        renderer: &mut Renderer,
        c: Camera,
        imagery: Option<&imagery::Imagery>,
    ) -> Result<(), RenderError> {
        self.frame = self.frame.wrapping_add(1);
        self.wanted.clear();
        self.draws.clear();
        for root in self.provider.roots() {
            self.select(root, c);
        }
        let revision = (self.revision, imagery.map_or(0, |i| i.revision()));
        for &t in &self.wanted {
            if let Some(r) = self.resident.get_mut(&t) {
                r.last = self.frame;
            }
        }
        // Only one drape/geometry refresh per frame; cached meshes remain visible meanwhile.
        if let Some(tile) = self
            .draws
            .iter()
            .copied()
            .find(|t| self.resident.get(t).is_some_and(|r| r.revision != revision))
        {
            let p = self.resident[&tile].data.primitive(
                tile,
                self.provider.projection,
                self.exaggeration,
            );
            let image = self.atlas(tile, imagery);
            let mesh = renderer.upload_terrain(
                &p,
                &BaseColorTexture {
                    width: 256,
                    height: 256,
                    rgba: image.as_raw(),
                },
            )?;
            let r = self.resident.get_mut(&tile).unwrap();
            r.mesh = mesh;
            r.revision = revision;
        }
        self.requests
            .retain(|t, r| r.inflight || self.wanted.contains(t) || self.resident.contains_key(t));
        Ok(())
    }
    pub fn take_requests(&mut self, max: usize) -> Vec<(Tile, Url)> {
        let mut order: Vec<_> = self.wanted.iter().copied().collect();
        order.sort_unstable();
        let mut out = Vec::new();
        for t in order {
            if out.len() >= max.min(4) {
                break;
            }
            if self.resident.contains_key(&t) || !self.provider.available(t) {
                continue;
            }
            let r = self.requests.entry(t).or_default();
            if r.inflight || r.attempts >= 3 || self.frame < r.retry {
                continue;
            }
            if let Ok(u) = self.provider.url(t) {
                r.inflight = true;
                r.attempts += 1;
                out.push((t, u));
            }
        }
        out
    }
    pub fn submit(
        &mut self,
        renderer: &mut Renderer,
        t: Tile,
        data: Data,
        imagery: Option<&imagery::Imagery>,
    ) -> Result<(), RenderError> {
        if !self.requests.get(&t).is_some_and(|r| r.inflight) {
            return Err(RenderError("stale terrain response".into()));
        }
        let p = data.primitive(t, self.provider.projection, self.exaggeration);
        let estimate = data.bytes() + p.gpu_bytes() + 256 * 256 * 4;
        while self.bytes() + estimate > CACHE_BUDGET {
            let oldest = self
                .resident
                .iter()
                .filter(|(t, _)| !self.wanted.contains(t))
                .min_by_key(|(_, r)| r.last)
                .map(|(t, _)| *t);
            if let Some(t) = oldest {
                self.resident.remove(&t);
            } else {
                return Err(RenderError("terrain cache budget; retaining parent".into()));
            }
        }
        let mut provider = self.provider.clone();
        if let Some(metadata) = &data.metadata {
            if let Some(available) = metadata.get("available") {
                provider
                    .add_available(available, t.z + 1)
                    .map_err(RenderError)?;
            }
        }
        let image = self.atlas(t, imagery);
        let mesh = renderer.upload_terrain(
            &p,
            &BaseColorTexture {
                width: 256,
                height: 256,
                rgba: image.as_raw(),
            },
        )?;
        self.provider = provider;
        self.resident.insert(
            t,
            Resident {
                data,
                mesh,
                last: self.frame,
                revision: (self.revision, imagery.map_or(0, |i| i.revision())),
            },
        );
        self.requests.remove(&t);
        Ok(())
    }
    pub fn fail(&mut self, t: Tile) {
        if let Some(r) = self.requests.get_mut(&t) {
            r.inflight = false;
            r.retry = self.frame + 120 * r.attempts as u64;
            self.errors += 1;
        }
    }
    pub fn cancel_requests(&mut self) {
        for r in self.requests.values_mut() {
            if r.inflight {
                r.inflight = false;
                r.attempts = r.attempts.saturating_sub(1);
                r.retry = 0;
            }
        }
    }
    pub fn invalidate_drape(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
    pub fn options(&mut self, exaggeration: f64, sse: f64) -> Result<(), String> {
        if !exaggeration.is_finite()
            || !(0.0..=10.0).contains(&exaggeration)
            || !sse.is_finite()
            || !(2.0..=64.0).contains(&sse)
        {
            return Err("terrain exaggeration 0..10; SSE 2..64".into());
        }
        if exaggeration != self.exaggeration {
            self.exaggeration = exaggeration;
            self.revision = self.revision.wrapping_add(1);
        }
        self.max_sse = sse;
        Ok(())
    }
    pub fn height_at(&self, lon: f64, lat: f64) -> Option<f64> {
        if !lon.is_finite() || !lat.is_finite() || lat.abs() > PI * 0.5 {
            return None;
        }
        let lon = (lon + PI).rem_euclid(TAU) - PI;
        let mut order: Vec<_> = self.resident.keys().copied().collect();
        order.sort_by_key(|t| std::cmp::Reverse(t.z));
        for t in order {
            if let Some(h) =
                self.resident[&t]
                    .data
                    .sample(tile_uv(t, self.provider.projection, lon, lat))
            {
                return Some(h);
            }
        }
        None
    }
    pub fn meshes(&self) -> Vec<&GpuMesh> {
        self.draws
            .iter()
            .filter_map(|t| {
                self.resident
                    .get(t)
                    .map(|r| &r.mesh)
                    .or_else(|| self.fallback.get(t))
            })
            .chain(self.polar.iter())
            .collect()
    }
    pub fn gpu_bytes(&self) -> usize {
        self.resident.values().map(|r| r.mesh.bytes).sum::<usize>()
            + self
                .fallback
                .values()
                .chain(self.polar.iter())
                .map(|m| m.bytes)
                .sum::<usize>()
    }
    pub fn bytes(&self) -> usize {
        self.resident
            .values()
            .map(|r| r.data.bytes() + r.mesh.bytes)
            .sum::<usize>()
            + self
                .fallback
                .values()
                .chain(self.polar.iter())
                .map(|m| m.bytes)
                .sum::<usize>()
    }
    pub fn stats(&self) -> String {
        format!(
            "terrain {} ready / {} draws | LOD {} | height x{:.1} | errors {}",
            self.resident.len(),
            self.draws.len(),
            self.draws.iter().map(|t| t.z).max().unwrap_or(0),
            self.exaggeration,
            self.errors
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn encoded() -> Vec<u8> {
        let mut b = Vec::new();
        for _ in 0..3 {
            b.extend_from_slice(&0f64.to_le_bytes());
        }
        for h in [100f32, 300f32] {
            b.extend_from_slice(&h.to_le_bytes());
        }
        for h in [0f64, 0.0, 0.0, WGS84_A, 0.0, 0.0, 0.0] {
            b.extend_from_slice(&h.to_le_bytes());
        }
        b.extend_from_slice(&4u32.to_le_bytes());
        for a in [
            [0i32, 32767, 0, 32767],
            [0, 0, 32767, 32767],
            [0, 0, 32767, 32767],
        ] {
            let mut previous = 0;
            for v in a {
                let d = v - previous;
                previous = v;
                let code = ((d << 1) ^ (d >> 31)) as u16;
                b.extend_from_slice(&code.to_le_bytes());
            }
        }
        b.extend_from_slice(&2u32.to_le_bytes());
        for c in [0u16, 0, 0, 2, 0, 2] {
            b.extend_from_slice(&c.to_le_bytes());
        }
        for edge in [[0u16, 2], [0, 1], [1, 3], [2, 3]] {
            b.extend_from_slice(&2u32.to_le_bytes());
            for i in edge {
                b.extend_from_slice(&i.to_le_bytes());
            }
        }
        b
    }
    #[test]
    fn quantized_decode_gzip_ranges_and_rtc() {
        let bytes = encoded();
        let d = decode(&bytes, Format::QuantizedMesh).unwrap();
        assert!((d.sample([0.25, 0.25]).unwrap() - 250.0).abs() < 0.01);
        assert_eq!(d.sample([2.0, 0.5]), None);
        for n in 0..bytes.len() {
            assert!(
                decode(&bytes[..n], Format::QuantizedMesh).is_err(),
                "truncation {n}"
            );
        }
        use std::io::Write;
        let mut zip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        zip.write_all(&bytes).unwrap();
        assert_eq!(
            decode(&zip.finish().unwrap(), Format::QuantizedMesh)
                .unwrap()
                .uv
                .len(),
            4
        );
        let p = d.primitive(
            Tile {
                z: 12,
                x: 6074,
                y: 1342,
            },
            Projection::Geographic,
            2.0,
        );
        let world = p
            .transform
            .transform_point3(DVec3::from_array(p.vertices[0].position.map(f64::from)));
        let c = Cartographic::from_ecef(world).unwrap();
        assert!((c.height - 200.0).abs() < 0.01);
        assert!(p.indices.len() > d.indices.len());
        let mut corrupt = bytes.clone();
        corrupt[116..118].copy_from_slice(&65535u16.to_le_bytes());
        assert!(decode(&corrupt, Format::QuantizedMesh).is_err());
        assert!(decode(&vec![0; MAX_TILE_BYTES + 1], Format::QuantizedMesh).is_err());
    }
    #[test]
    fn provider_tms_auth_metadata_and_validation() {
        let json=br#"{"format":"quantized-mesh-1.0","tiles":["{z}/{x}/{y}.terrain?v={version}"],"metadataAvailability":10,"extensions":["metadata"]}"#;
        let mut p = Provider::layer(
            json,
            Url::parse("https://example.test/terrain/layer.json").unwrap(),
        )
        .unwrap()
        .with_authorization("Bearer secret".into())
        .unwrap();
        assert!(!p.available(Tile { z: 1, x: 0, y: 0 }));
        p.add_available(
            &serde_json::json!([[{"startX":0,"startY":0,"endX":3,"endY":1}]]),
            1,
        )
        .unwrap();
        assert!(p.available(Tile { z: 1, x: 2, y: 0 }));
        assert!(p
            .url(Tile { z: 1, x: 2, y: 0 })
            .unwrap()
            .as_str()
            .contains("/1/2/1.terrain"));
        assert!(!p
            .url(Tile { z: 1, x: 2, y: 0 })
            .unwrap()
            .as_str()
            .contains("secret"));
        let remote = br#"{"tiles":["https://attacker.test/{z}/{x}/{y}.terrain"]}"#;
        assert!(Provider::layer(remote, p.source.clone())
            .unwrap()
            .with_authorization("Bearer secret".into())
            .is_err());
        assert!(Provider::terrarium("https://example.test/a/{z}/{x}/{y}.png", 16).is_err());
        assert!(Provider::layer(
            br#"{"format":"heightmap-1.0","tiles":["{z}/{x}/{y}"]}"#,
            p.source.clone()
        )
        .is_err());
    }
    #[test]
    fn terrarium_real_heights_and_official_mesh() {
        let d = decode(
            include_bytes!("../../../tests/fixtures/terrain/terrarium-root.png"),
            Format::Terrarium,
        )
        .unwrap();
        assert!(d.max_height > 1000.0);
        assert!(d.min_height < 0.0);
        assert!(d.sample([0.5, 0.5]).unwrap().is_finite());
        let d = decode(
            include_bytes!("../../../tests/fixtures/terrain/everest.png"),
            Format::Terrarium,
        )
        .unwrap();
        assert!(d.max_height > 8000.0, "{}", d.max_height);
        let tile = Tile {
            z: 12,
            x: 3037,
            y: 1716,
        };
        let uv = tile_uv(
            tile,
            Projection::Mercator,
            86.92528f64.to_radians(),
            27.9881f64.to_radians(),
        );
        assert!(d.sample(uv).unwrap() > 7000.0);
        for bytes in [
            include_bytes!("../../../tests/fixtures/terrain/quantized-mesh.terrain").as_slice(),
            include_bytes!("../../../tests/fixtures/terrain/quantized-wide.terrain").as_slice(),
            include_bytes!("../../../tests/fixtures/terrain/quantized-metadata.terrain").as_slice(),
        ] {
            let data = decode(bytes, Format::QuantizedMesh).unwrap();
            assert!(data.indices.len() >= 3);
            if let Some(available) = data.metadata.as_ref().and_then(|m| m.get("available")) {
                let mut provider=Provider::layer(br#"{"tiles":["{z}/{x}/{y}.terrain"],"metadataAvailability":10,"extensions":["metadata"]}"#,Url::parse("https://example.test/layer.json").unwrap()).unwrap();
                provider.add_available(available, 1).unwrap();
                assert!(!provider.available.is_empty());
            }
        }
    }
    #[test]
    #[ignore = "requires a native GPU or Mesa Vulkan software adapter"]
    fn terrain_replacement_drape_collision_and_lifecycle() {
        pollster::block_on(async {
            let instance = cesium_renderer::instance();
            let (_, device, queue) = cesium_renderer::device(&instance, None).await.unwrap();
            let mut renderer =
                Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 512, 384)
                    .unwrap();
            let mut scene = crate::Scene::new(&mut renderer, None).unwrap();
            let provider = Provider::layer(
                br#"{"tiles":["{z}/{x}/{y}.terrain"],"maxzoom":1}"#,
                Url::parse("https://example.test/layer.json").unwrap(),
            )
            .unwrap();
            scene.set_terrain(&mut renderer, provider).unwrap();
            scene.update_terrain(&mut renderer).unwrap();
            let c = scene.orbit.camera(512, 384);
            let layer = scene.terrain.as_mut().unwrap();
            let roots = layer.take_requests(4);
            assert_eq!(roots.len(), 2);
            for (t, _) in roots {
                layer
                    .submit(
                        &mut renderer,
                        t,
                        decode(&encoded(), Format::QuantizedMesh).unwrap(),
                        None,
                    )
                    .unwrap();
            }
            layer.max_sse = 2.0;
            layer.update(&mut renderer, c, None).unwrap();
            let requests = layer.take_requests(4);
            assert!(!requests.is_empty());
            assert!(layer.draws.iter().all(|t| t.z == 0));
            for (t, _) in requests {
                layer
                    .submit(
                        &mut renderer,
                        t,
                        decode(&encoded(), Format::QuantizedMesh).unwrap(),
                        None,
                    )
                    .unwrap();
            }
            layer.update(&mut renderer, c, None).unwrap();
            assert!(layer.draws.iter().any(|t| t.z == 1));
            for t in &layer.draws {
                assert!(!layer
                    .draws
                    .iter()
                    .any(|a| a.z < t.z && t.parent() == Some(*a)));
            }
            assert!(layer.bytes() <= CACHE_BUDGET);
            layer.cancel_requests();
            assert!(layer.requests.values().all(|s| !s.inflight));
            let fake = Tile { z: 1, x: 0, y: 0 };
            assert!(layer
                .submit(
                    &mut renderer,
                    fake,
                    decode(&encoded(), Format::QuantizedMesh).unwrap(),
                    None
                )
                .is_err());
            scene.focus_location(90.0, 0.0, 10.0).unwrap();
            scene.update_terrain(&mut renderer).unwrap();
            let p = Cartographic::from_ecef(scene.orbit.camera(512, 384).eye).unwrap();
            let h = scene
                .terrain
                .as_ref()
                .unwrap()
                .height_at(p.longitude, p.latitude)
                .unwrap();
            assert!(p.height >= h + 9.9);
            let provider = imagery::Provider::template(
                "https://example.test/{z}/{x}/{y}.png",
                Projection::Geographic,
                0,
                "test",
            )
            .unwrap();
            scene.set_imagery(&mut renderer, provider).unwrap();
            scene.update_imagery(&mut renderer).unwrap();
            let images = scene.imagery.as_mut().unwrap();
            for (t, _) in images.take_requests(4) {
                images
                    .submit(
                        &mut renderer,
                        t,
                        image::RgbaImage::from_pixel(8, 8, image::Rgba([220, 30, 10, 255])),
                    )
                    .unwrap();
            }
            assert_eq!(
                images.sample_color(90f64.to_radians(), 0.0).unwrap()[0],
                220
            );
            scene.update_terrain(&mut renderer).unwrap();
            scene.terrain.as_mut().unwrap().options(2.0, 8.0).unwrap();
            scene.update_terrain(&mut renderer).unwrap();
            scene.clear_terrain();
            scene.update_imagery(&mut renderer).unwrap();
            assert!(scene.terrain.is_none());
        });
    }
}
