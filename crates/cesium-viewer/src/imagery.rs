//! Progressive globe coverage. Tile Y is always north-down internally; TMS flips only in URLs.
use cesium_formats::{Material, Primitive, Vertex};
use cesium_math::{Camera, Cartographic, Sphere, WGS84_A};
use cesium_renderer::{BaseColorTexture, GpuMesh, RenderError, Renderer};
use glam::{DMat4, DVec3};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    f64::consts::{PI, TAU},
    io::Cursor,
};
use url::Url;

pub const MAX_IMAGE_BYTES: usize = 4 * 1024 * 1024;
const GPU_BUDGET: usize = 64 * 1024 * 1024;
const MAX_LEAVES: usize = 384;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tile {
    pub z: u32,
    pub x: u32,
    pub y: u32,
}
impl Tile {
    pub fn parent(self) -> Option<Self> {
        (self.z > 0).then(|| Self {
            z: self.z - 1,
            x: self.x / 2,
            y: self.y / 2,
        })
    }
    fn children(self) -> [Self; 4] {
        std::array::from_fn(|i| Self {
            z: self.z + 1,
            x: self.x * 2 + (i as u32 % 2),
            y: self.y * 2 + (i as u32 / 2),
        })
    }
    pub fn key(self) -> String {
        format!("{}/{}/{}", self.z, self.x, self.y)
    }
    pub fn parse(s: &str) -> Option<Self> {
        let v: Vec<_> = s
            .split('/')
            .map(str::parse::<u32>)
            .collect::<Result<_, _>>()
            .ok()?;
        if v.len() != 3 || v[0] > 20 {
            return None;
        }
        Some(Self {
            z: v[0],
            x: v[1],
            y: v[2],
        })
    }
    pub fn quadkey(self) -> String {
        (0..self.z)
            .rev()
            .map(|i| char::from(b'0' + ((self.x >> i) & 1) as u8 + (((self.y >> i) & 1) * 2) as u8))
            .collect()
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Projection {
    Geographic,
    Mercator,
}
#[derive(Clone)]
pub struct Provider {
    pub projection: Projection,
    pub max_level: u32,
    pub credits: String,
    template: String,
    subdomains: Vec<String>,
    culture: String,
}
impl Provider {
    pub fn natural_earth() -> Self {
        Self::template("https://cesium.com/downloads/cesiumjs/releases/1.146/Build/Cesium/Assets/Textures/NaturalEarthII/{z}/{x}/{reverseY}.jpg", Projection::Geographic,2,"Natural Earth II · Cesium sample imagery").expect("static provider")
    }
    pub fn template(
        template: &str,
        projection: Projection,
        max_level: u32,
        credits: &str,
    ) -> Result<Self, String> {
        let check = template
            .replace("{z}", "0")
            .replace("{x}", "0")
            .replace("{y}", "0")
            .replace("{reverseY}", "0")
            .replace("{quadkey}", "0")
            .replace("{subdomain}", "t0")
            .replace("{culture}", "");
        let url = Url::parse(&check).map_err(|_| "invalid imagery URL")?;
        if !matches!(url.scheme(), "https" | "http" | "file") || check.contains('{') {
            return Err("invalid imagery template".into());
        }
        if max_level > 20 || (projection == Projection::Mercator && max_level < 1) {
            return Err("imagery level must be within 0..20 (Mercator starts at 1)".into());
        }
        Ok(Self {
            projection,
            max_level,
            credits: credits.into(),
            template: template.into(),
            subdomains: vec!["t0".into()],
            culture: String::new(),
        })
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
    pub fn url(&self, t: Tile) -> Result<Url, String> {
        if t.z > 20 {
            return Err("invalid imagery tile level".into());
        }
        let n = 1u32 << t.z;
        let width = n * if self.projection == Projection::Geographic {
            2
        } else {
            1
        };
        if t.z > self.max_level || t.x >= width || t.y >= n {
            return Err("invalid imagery tile".into());
        }
        let s = self
            .template
            .replace("{z}", &t.z.to_string())
            .replace("{x}", &t.x.to_string())
            .replace("{y}", &t.y.to_string())
            .replace("{reverseY}", &(n - 1 - t.y).to_string())
            .replace("{quadkey}", &t.quadkey())
            .replace(
                "{subdomain}",
                &self.subdomains[((t.x + t.y + t.z) as usize) % self.subdomains.len()],
            )
            .replace("{culture}", &self.culture);
        Url::parse(&s).map_err(|_| "invalid imagery URL".into())
    }
    /// ion default global aerial imagery is asset 2. Do not embed Cesium's evaluation token.
    pub fn ion_url(token: &str) -> Result<Url, String> {
        if token.trim().is_empty() {
            return Err("Cesium ion access token required".into());
        }
        let mut u = Url::parse("https://api.cesium.com/v1/assets/2/endpoint").unwrap();
        u.query_pairs_mut().append_pair("access_token", token);
        Ok(u)
    }
    pub fn bing_metadata_url(endpoint: &Value) -> Result<Url, String> {
        if endpoint["type"] != "IMAGERY" || endpoint["externalType"] != "BING" {
            return Err("unsupported ion imagery provider; expected BING".into());
        }
        let o = &endpoint["options"];
        let base = o["url"].as_str().ok_or("missing Bing URL")?;
        let mut u = Url::parse(&format!(
            "{}/REST/v1/Imagery/Metadata/{}",
            base.trim_end_matches('/'),
            o["mapStyle"].as_str().unwrap_or("Aerial")
        ))
        .map_err(|_| "invalid Bing URL")?;
        if u.scheme() != "https" {
            u.set_scheme("https").map_err(|_| "invalid Bing scheme")?;
        }
        u.query_pairs_mut()
            .append_pair("incl", "ImageryProviders")
            .append_pair("uriScheme", "https")
            .append_pair("key", o["key"].as_str().ok_or("missing Bing key")?)
            .append_pair("culture", o["culture"].as_str().unwrap_or("en-US"));
        Ok(u)
    }
    pub fn from_bing(endpoint: &Value, metadata: &Value) -> Result<Self, String> {
        Self::bing_metadata_url(endpoint)?;
        let r = &metadata["resourceSets"][0]["resources"][0];
        if metadata["statusCode"].as_u64().is_some_and(|c| c != 200) {
            return Err("Bing imagery metadata rejected".into());
        }
        let mut credits = vec!["Cesium ion".to_owned(), "Microsoft Bing".into()];
        if let Some(a) = endpoint["attributions"].as_array() {
            for c in a {
                if let Some(s) = c["html"].as_str() {
                    credits.push(s.into());
                }
            }
        }
        // Retain every returned attribution conservatively, including providers outside current view.
        if let Some(a) = r["imageryProviders"].as_array() {
            for c in a {
                if let Some(s) = c["attribution"].as_str() {
                    credits.push(s.into());
                }
            }
        }
        let mut p = Self::template(
            &format!(
                "{}{}n=z",
                r["imageUrl"].as_str().ok_or("missing Bing tile template")?,
                if r["imageUrl"].as_str().unwrap().contains('?') {
                    "&"
                } else {
                    "?"
                }
            ),
            Projection::Mercator,
            r["zoomMax"]
                .as_u64()
                .ok_or("missing Bing maximum level")?
                .min(20) as u32,
            &credits.join(" | "),
        )?;
        p.subdomains = r["imageUrlSubdomains"]
            .as_array()
            .ok_or("missing Bing subdomains")?
            .iter()
            .filter_map(|s| s.as_str().map(str::to_owned))
            .collect();
        if p.subdomains.is_empty() {
            return Err("empty Bing subdomains".into());
        }
        p.culture = endpoint["options"]["culture"]
            .as_str()
            .unwrap_or("en-US")
            .into();
        Ok(p)
    }
}
fn coord(t: Tile, p: Projection, u: f64, v: f64) -> (f64, f64) {
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
fn bounds(t: Tile, p: Projection) -> (Sphere, f64) {
    let (lon, lat) = coord(t, p, 0.5, 0.5);
    let (w, s) = coord(t, p, 0.0, 1.0);
    let (e, n) = coord(t, p, 1.0, 0.0);
    let angle = ((e - w).abs() + (n - s).abs()) * 0.5;
    (
        Sphere {
            center: Cartographic::radians(lon, lat, 0.0).unwrap().to_ecef(),
            radius: WGS84_A * angle.min(2.0),
        },
        angle,
    )
}
fn visible(t: Tile, p: Projection, c: Camera) -> bool {
    let (b, angle) = bounds(t, p);
    if !c.visible(b) {
        return false;
    }
    let horizon = (cesium_math::WGS84_B / c.eye.length())
        .clamp(0.0, 1.0)
        .acos();
    b.center
        .normalize()
        .dot(c.eye.normalize())
        .clamp(-1.0, 1.0)
        .acos()
        <= horizon + angle + 0.02
}
/// Disjoint adaptive coverage, limited independently of network availability.
fn select(p: &Provider, c: Camera) -> Vec<Tile> {
    let mut leaves = p.roots();
    let mut i = 0;
    while i < leaves.len() {
        let t = leaves[i];
        let (b, angle) = bounds(t, p.projection);
        if !visible(t, p.projection, c) {
            leaves.swap_remove(i);
            continue;
        }
        if t.z < p.max_level
            && leaves.len() + 3 <= MAX_LEAVES
            && c.sse(b, WGS84_A * angle / 128.0) > 1.8
        {
            leaves.swap_remove(i);
            leaves.extend(t.children());
        } else {
            i += 1;
        }
    }
    leaves.sort_unstable();
    leaves
}
/// Geometry samples the ellipsoid exactly; parent UVs select just the covered quadrant.
fn patch(t: Tile, p: Projection, ancestor: Option<Tile>) -> Primitive {
    let steps = (128u32 >> t.z.min(3)).max(16);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for y in 0..=steps {
        for x in 0..=steps {
            let u = x as f64 / steps as f64;
            let v = y as f64 / steps as f64;
            let (lon, lat) = coord(t, p, u, v);
            let point = Cartographic::radians(lon, lat, 0.0).unwrap().to_ecef();
            let normal = DVec3::new(lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin());
            let uv = if let Some(a) = ancestor {
                let scale = (1u32 << (t.z - a.z)) as f64;
                [
                    (t.x as f64 + u) / scale - a.x as f64,
                    (t.y as f64 + v) / scale - a.y as f64,
                ]
            } else {
                [(lon + PI) / TAU, (PI * 0.5 - lat) / PI]
            };
            vertices.push(Vertex {
                position: point.as_vec3().to_array(),
                normal: normal.as_vec3().to_array(),
                uv: uv.map(|x| x as f32),
                feature_id: 0,
                uv1: [0.0; 2],
                color: [1.0; 4],
                tangent: [0.0; 4],
            });
        }
    }
    for y in 0..steps {
        for x in 0..steps {
            let a = y * (steps + 1) + x;
            let b = a + steps + 1;
            indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    // Skirts close T-junction cracks between neighboring resolutions without overlaying parents.
    for edge in 0..4 {
        for j in 0..steps {
            let a = match edge {
                0 => j,
                1 => j * (steps + 1) + steps,
                2 => steps * (steps + 1) + steps - j,
                _ => (steps - j) * (steps + 1),
            };
            let b = match edge {
                0 => a + 1,
                1 => a + steps + 1,
                2 => a - 1,
                _ => a - steps - 1,
            };
            let start = vertices.len() as u32;
            for id in [a, b] {
                let mut v = vertices[id as usize];
                let point = DVec3::from_array(v.position.map(f64::from))
                    - DVec3::from_array(v.normal.map(f64::from)) * (5000.0 / (1u32 << t.z) as f64);
                v.position = point.as_vec3().to_array();
                vertices.push(v);
            }
            indices.extend_from_slice(&[a, start, b, b, start, start + 1]);
        }
    }
    Primitive {
        vertices,
        indices,
        transform: DMat4::IDENTITY,
        material: Material {
            metallic: 0.0,
            roughness: 0.85,
            double_sided: true,
            ..Default::default()
        },
    }
}
pub fn decode(bytes: &[u8]) -> Result<image::RgbaImage, String> {
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err("imagery download exceeds 4 MiB".into());
    }
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| "invalid imagery format")?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(1024);
    limits.max_image_height = Some(1024);
    limits.max_alloc = Some(8 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|_| "imagery decode failed or exceeds limits")?
        .to_rgba8();
    if image.width() < 2 || image.height() < 2 {
        return Err("empty imagery tile".into());
    }
    Ok(image)
}
struct Resident {
    mesh: GpuMesh,
    image: image::RgbaImage,
    last: u64,
}
#[derive(Default)]
struct RequestState {
    inflight: bool,
    attempts: u8,
    retry_at: u64,
}
pub struct Imagery {
    pub provider: Provider,
    resident: HashMap<Tile, Resident>,
    requests: HashMap<Tile, RequestState>,
    coverage: HashMap<(Tile, Option<Tile>), GpuMesh>,
    draws: Vec<(Tile, Option<Tile>)>,
    leaves: Vec<Tile>,
    frame: u64,
    pub errors: usize,
    revision: u64,
}
impl Imagery {
    pub fn new(provider: Provider) -> Self {
        Self {
            provider,
            resident: HashMap::new(),
            requests: HashMap::new(),
            coverage: HashMap::new(),
            draws: Vec::new(),
            leaves: Vec::new(),
            frame: 0,
            errors: 0,
            revision: 0,
        }
    }
    pub fn update(
        &mut self,
        renderer: &mut Renderer,
        base: &GpuMesh,
        camera: Camera,
    ) -> Result<(), RenderError> {
        self.frame = self.frame.wrapping_add(1);
        self.leaves = select(&self.provider, camera);
        self.draws.clear();
        for &tile in &self.leaves {
            let mut a = Some(tile);
            while let Some(t) = a {
                if self.resident.contains_key(&t) {
                    break;
                }
                a = t.parent();
            }
            if let Some(t) = a {
                self.resident.get_mut(&t).unwrap().last = self.frame;
            }
            let key = (tile, a);
            if a != Some(tile) && !self.coverage.contains_key(&key) {
                let source = a.map_or(base, |t| &self.resident[&t].mesh);
                let mesh = renderer
                    .upload_shared_globe(&patch(tile, self.provider.projection, a), source)?;
                self.coverage.insert(key, mesh);
            }
            self.draws.push(key);
        }
        let used: HashSet<_> = self.draws.iter().copied().collect();
        self.coverage.retain(|k, _| used.contains(k));
        // Pin ALL visible ancestors: eviction must never destroy progressive fallback.
        let mut pinned = HashSet::new();
        for &t in &self.leaves {
            let mut a = Some(t);
            while let Some(t) = a {
                pinned.insert(t);
                a = t.parent();
            }
        }
        while self.bytes() > GPU_BUDGET {
            let oldest = self
                .resident
                .iter()
                .filter(|(t, _)| !pinned.contains(t))
                .min_by_key(|(_, r)| r.last)
                .map(|(t, _)| *t);
            if let Some(t) = oldest {
                self.resident.remove(&t);
                self.revision = self.revision.wrapping_add(1);
            } else {
                break;
            }
        }
        // Bounded request-state history; only current coverage, residents and active work survive.
        self.requests
            .retain(|t, s| s.inflight || pinned.contains(t) || self.resident.contains_key(t));
        Ok(())
    }
    pub fn take_requests(&mut self, max: usize) -> Vec<(Tile, Url)> {
        let mut wanted = HashSet::new();
        for &t in &self.leaves {
            let mut a = Some(t);
            while let Some(t) = a {
                if t.z
                    >= if self.provider.projection == Projection::Mercator {
                        1
                    } else {
                        0
                    }
                {
                    wanted.insert(t);
                }
                a = t.parent();
            }
        }
        let mut order: Vec<_> = wanted.into_iter().collect();
        order.sort_unstable();
        let mut out = Vec::new();
        for t in order {
            if out.len() >= max.min(4) {
                break;
            }
            if self.resident.contains_key(&t) {
                continue;
            }
            let s = self.requests.entry(t).or_default();
            if s.inflight || s.attempts >= 3 || self.frame < s.retry_at {
                continue;
            }
            if let Ok(url) = self.provider.url(t) {
                s.inflight = true;
                s.attempts += 1;
                out.push((t, url));
            }
        }
        out
    }
    pub fn submit(
        &mut self,
        renderer: &mut Renderer,
        t: Tile,
        image: image::RgbaImage,
    ) -> Result<(), RenderError> {
        if !self.requests.get(&t).is_some_and(|s| s.inflight) {
            return Err(RenderError("stale imagery response".into()));
        }
        if image.width() > 1024 || image.height() > 1024 {
            return Err(RenderError("imagery dimensions exceed limits".into()));
        }
        let primitive = patch(t, self.provider.projection, Some(t));
        let bytes = primitive.vertices.len() * std::mem::size_of::<Vertex>()
            + primitive.indices.len() * 4
            + 128
            + image.as_raw().len() * 2;
        let mut pinned = HashSet::new();
        for &leaf in &self.leaves {
            let mut a = Some(leaf);
            while let Some(t) = a {
                pinned.insert(t);
                a = t.parent();
            }
        }
        let resident_budget = GPU_BUDGET - 16 * 1024 * 1024;
        while self
            .resident
            .values()
            .map(|r| r.mesh.bytes + r.image.as_raw().len())
            .sum::<usize>()
            + bytes
            > resident_budget
        {
            let oldest = self
                .resident
                .iter()
                .filter(|(t, _)| !pinned.contains(t))
                .min_by_key(|(_, r)| r.last)
                .map(|(t, _)| *t);
            if let Some(t) = oldest {
                self.resident.remove(&t);
                self.revision = self.revision.wrapping_add(1);
            } else {
                return Err(RenderError(
                    "imagery cache full; retaining ancestor coverage".into(),
                ));
            }
        }
        let mesh = renderer.upload_imagery(
            &primitive,
            &BaseColorTexture {
                width: image.width(),
                height: image.height(),
                rgba: image.as_raw(),
            },
        )?;
        self.resident.insert(
            t,
            Resident {
                mesh,
                image,
                last: self.frame,
            },
        );
        self.requests.remove(&t);
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }
    pub fn cancel_requests(&mut self) {
        for state in self.requests.values_mut() {
            if state.inflight {
                state.inflight = false;
                state.attempts = state.attempts.saturating_sub(1);
                state.retry_at = 0;
            }
        }
    }
    pub fn fail(&mut self, t: Tile) {
        if let Some(s) = self.requests.get_mut(&t) {
            s.inflight = false;
            s.retry_at = self.frame + 120 * (s.attempts as u64);
            self.errors += 1;
        }
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    /// Sample the finest resident image; terrain owns reprojected tile textures.
    pub fn sample_color(&self, lon: f64, lat: f64) -> Option<image::Rgba<u8>> {
        if !lon.is_finite() || !lat.is_finite() || lat.abs() > std::f64::consts::FRAC_PI_2 {
            return None;
        }
        let fx = ((lon + PI) / TAU).clamp(0.0, 1.0 - f64::EPSILON);
        let fy = match self.provider.projection {
            Projection::Geographic => (PI * 0.5 - lat) / PI,
            Projection::Mercator => {
                if lat.abs() > 85.0511287798066f64.to_radians() {
                    return None;
                }
                (1.0 - (lat.tan() + 1.0 / lat.cos()).ln() / PI) * 0.5
            }
        }
        .clamp(0.0, 1.0 - f64::EPSILON);
        for z in (0..=self.provider.max_level).rev() {
            let n = (1u32 << z) as f64;
            let x = fx
                * n
                * if self.provider.projection == Projection::Geographic {
                    2.0
                } else {
                    1.0
                };
            let y = fy * n;
            let t = Tile {
                z,
                x: x.floor() as u32,
                y: y.floor() as u32,
            };
            if let Some(r) = self.resident.get(&t) {
                let px = ((x - x.floor()) * (r.image.width() - 1) as f64).round() as u32;
                let py = ((y - y.floor()) * (r.image.height() - 1) as f64).round() as u32;
                return Some(*r.image.get_pixel(px, py));
            }
        }
        None
    }
    pub fn meshes(&self) -> Vec<&GpuMesh> {
        self.draws
            .iter()
            .filter_map(|(t, a)| {
                if *a == Some(*t) {
                    self.resident.get(t).map(|r| &r.mesh)
                } else {
                    self.coverage.get(&(*t, *a))
                }
            })
            .collect()
    }
    pub fn gpu_bytes(&self) -> usize {
        self.resident.values().map(|r| r.mesh.bytes).sum::<usize>()
            + self.coverage.values().map(|m| m.bytes).sum::<usize>()
    }
    pub fn bytes(&self) -> usize {
        self.resident
            .values()
            .map(|r| r.mesh.bytes + r.image.as_raw().len())
            .sum::<usize>()
            + self.coverage.values().map(|m| m.bytes).sum::<usize>()
    }
    pub fn stats(&self) -> String {
        format!(
            "imagery {} ready / {} regions | requests {} | LOD {} | failures {}",
            self.resident.len(),
            self.draws.len(),
            self.requests.values().filter(|s| s.inflight).count(),
            self.leaves.iter().map(|t| t.z).max().unwrap_or(0),
            self.errors
        )
    }
}
/// Mercator stops at ±85.051°; the bundled geographic image fills both polar caps.
pub fn polar_patch(north: bool) -> Primitive {
    let mut mesh = cesium_formats::globe_mesh(128, 8);
    for y in 0..=8 {
        for x in 0..=128 {
            let lon = x as f64 / 128.0 * TAU - PI;
            let fraction = y as f64 / 8.0;
            let limit = PI.sinh().atan();
            let lat = if north {
                PI * 0.5 - fraction * (PI * 0.5 - limit)
            } else {
                -limit - fraction * (PI * 0.5 - limit)
            };
            let v = &mut mesh.vertices[(y * 129 + x) as usize];
            v.position = Cartographic::radians(lon, lat, 0.0)
                .unwrap()
                .to_ecef()
                .as_vec3()
                .to_array();
            v.normal = DVec3::new(lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin())
                .as_vec3()
                .to_array();
            v.uv = [x as f32 / 128.0, ((PI * 0.5 - lat) / PI) as f32];
        }
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires a native GPU or Mesa Vulkan software adapter"]
    fn progressive_coverage_cache_and_cancellation() {
        pollster::block_on(async {
            let instance = cesium_renderer::instance();
            let (_, device, queue) = cesium_renderer::device(&instance, None).await.unwrap();
            let mut renderer =
                Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 800, 600)
                    .unwrap();
            let mut scene = crate::Scene::new(&mut renderer, None).unwrap();
            scene
                .set_imagery(
                    &mut renderer,
                    Provider::template(
                        "https://example.com/{z}/{x}/{y}.jpg",
                        Projection::Mercator,
                        8,
                        "test",
                    )
                    .unwrap(),
                )
                .unwrap();
            let layer = scene.imagery.as_mut().unwrap();
            let roots = layer.take_requests(4);
            assert_eq!(roots.len(), 4);
            assert!(roots.iter().all(|(t, _)| t.z == 1));
            for (t, _) in roots {
                layer
                    .submit(
                        &mut renderer,
                        t,
                        image::RgbaImage::from_pixel(16, 16, image::Rgba([80, 140, 60, 255])),
                    )
                    .unwrap();
            }
            scene.update_imagery(&mut renderer).unwrap();
            let layer = scene.imagery.as_mut().unwrap();
            assert!(layer.draws.iter().all(|(_, a)| a.is_some()));
            assert_eq!(layer.meshes().len(), layer.draws.len());
            let requests = layer.take_requests(4);
            let cancelled = requests[0].0;
            layer.cancel_requests();
            assert!(layer
                .submit(&mut renderer, cancelled, image::RgbaImage::new(16, 16))
                .is_err());
            let repeat = layer.take_requests(4);
            assert_eq!(
                requests.iter().map(|(t, _)| *t).collect::<Vec<_>>(),
                repeat.iter().map(|(t, _)| *t).collect::<Vec<_>>()
            );
            for (t, _) in repeat {
                layer.fail(t);
            }
            assert!(layer
                .take_requests(4)
                .iter()
                .all(|(t, _)| !requests.iter().any(|(old, _)| old == t)));
            layer.cancel_requests();
            // Large valid images cannot exhaust the cache even while visible ancestors are pinned.
            for _ in 0..30 {
                let work = layer.take_requests(4);
                if work.is_empty() {
                    break;
                }
                for (t, _) in work {
                    if layer
                        .submit(
                            &mut renderer,
                            t,
                            image::RgbaImage::from_pixel(
                                1024,
                                1024,
                                image::Rgba([60, 120, 170, 255]),
                            ),
                        )
                        .is_err()
                    {
                        layer.fail(t);
                    }
                }
                assert!(layer.bytes() <= GPU_BUDGET);
            }
            scene.update_imagery(&mut renderer).unwrap();
            let layer = scene.imagery.as_ref().unwrap();
            assert_eq!(layer.meshes().len(), layer.draws.len());
            assert!(layer.draws.iter().all(|(_, a)| a.is_some()));
            scene.orbit.zoom(100.0);
            scene.orbit.zoom(100.0);
            assert!(scene.orbit.height() < 20.0);
            scene.update_imagery(&mut renderer).unwrap();
            let layer = scene.imagery.as_ref().unwrap();
            assert!(!layer.meshes().is_empty());
            assert_eq!(layer.meshes().len(), layer.draws.len());
            let target = cesium_renderer::attachment(
                &renderer.device,
                800,
                600,
                renderer.format(),
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            );
            scene
                .render(
                    &mut renderer,
                    &target.create_view(&Default::default()),
                    None,
                )
                .unwrap();
            renderer
                .device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(std::time::Duration::from_secs(30)),
                })
                .unwrap();
            renderer.resize(800, 600).unwrap();
        });
    }
    #[test]
    fn quadkeys_tms_and_poles() {
        assert_eq!(Tile { z: 3, x: 3, y: 5 }.quadkey(), "213");
        let p = Provider::natural_earth();
        assert!(p
            .url(Tile { z: 2, x: 7, y: 0 })
            .unwrap()
            .as_str()
            .ends_with("/2/7/3.jpg"));
        assert!(p.url(Tile { z: 2, x: 8, y: 0 }).is_err());
        assert!(p.url(Tile { z: 32, x: 0, y: 0 }).is_err());
        assert!(
            (coord(Tile { z: 0, x: 0, y: 0 }, Projection::Geographic, 0.0, 0.0).1 - PI * 0.5).abs()
                < 1e-10
        );
        assert!(
            (coord(Tile { z: 1, x: 0, y: 0 }, Projection::Mercator, 0.0, 0.0)
                .1
                .to_degrees()
                - 85.05112878)
                .abs()
                < 1e-7
        );
    }
    #[test]
    fn refinement_and_parent_uv() {
        let p = Provider::template(
            "https://example.com/{z}/{x}/{y}.jpg",
            Projection::Mercator,
            18,
            "",
        )
        .unwrap();
        let far = crate::Orbit::earth().camera(1280, 800);
        let mut orbit = crate::Orbit::earth();
        orbit.zoom(100.0);
        let a = select(&p, far);
        let b = select(&p, orbit.camera(1280, 800));
        assert!(a.len() <= MAX_LEAVES && b.len() <= MAX_LEAVES);
        assert!(b.iter().map(|t| t.z).max() > a.iter().map(|t| t.z).max());
        let mesh = patch(
            Tile { z: 2, x: 3, y: 1 },
            Projection::Mercator,
            Some(Tile { z: 1, x: 1, y: 0 }),
        );
        assert_eq!(mesh.vertices[0].uv, [0.5, 0.5]);
        let n = ((128u32 >> 2) + 1).pow(2) as usize;
        assert_eq!(mesh.vertices[n - 1].uv, [1.0, 1.0]);
    }
    #[test]
    fn metadata_credentials_and_bounded_decode() {
        let e = serde_json::json!({"type":"IMAGERY","externalType":"BING","options":{"url":"https://dev.virtualearth.net","key":"test","mapStyle":"Aerial"},"attributions":[{"html":"provider credit"}]});
        let m = serde_json::json!({"statusCode":200,"resourceSets":[{"resources":[{"imageUrl":"https://{subdomain}.example.com/{quadkey}.jpeg?mkt={culture}","imageUrlSubdomains":["t0","t1"],"zoomMax":20,"imageryProviders":[{"attribution":"source credit"}]}]}]});
        let p = Provider::from_bing(&e, &m).unwrap();
        assert!(p.credits.contains("source credit"));
        assert!(p
            .url(Tile { z: 1, x: 1, y: 0 })
            .unwrap()
            .path()
            .ends_with("1.jpeg"));
        assert!(Provider::ion_url("").is_err());
        assert!(decode(&vec![0; MAX_IMAGE_BYTES + 1]).is_err());
        assert!(decode(b"broken jpeg").is_err());
    }
}
