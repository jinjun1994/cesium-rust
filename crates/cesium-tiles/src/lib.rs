//! Explicit tree parsing and transactional whole-subtree replacement.
use cesium_formats::{check_input, DecodeError, DecodeLimits};
use cesium_math::{linear_part, scale_bound, Camera, Sphere, WGS84_A};
use glam::{DMat4, DVec3};
use serde::Deserialize;
use serde_json::Value;
use url::Url;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Refine {
    Replace,
    Add,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentState {
    Unrequested,
    Downloading,
    CpuReady,
    GpuReady,
    PermanentFailure,
}
#[derive(Debug)]
pub struct Tile {
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub bounds: Sphere,
    pub world_transform: DMat4,
    pub geometric_error: f64,
    pub refine: Refine,
    pub uri: Option<Url>,
    pub state: ContentState,
    pub topology_complete: bool,
    pub document_source: Url,
}
#[derive(Debug)]
pub struct Tileset {
    pub tiles: Vec<Tile>,
    pub root: usize,
    pub source: Url,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSet {
    asset: Asset,
    geometric_error: f64,
    root: RawTile,
    #[serde(default)]
    extensions_required: Vec<String>,
}
#[derive(Deserialize)]
struct Asset {
    version: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawTile {
    bounding_volume: Value,
    geometric_error: f64,
    refine: Option<Refine>,
    transform: Option<[f64; 16]>,
    content: Option<RawContent>,
    contents: Option<Vec<RawContent>>,
    #[serde(default)]
    children: Vec<RawTile>,
    implicit_tiling: Option<Value>,
    viewer_request_volume: Option<Value>,
}
#[derive(Deserialize)]
struct RawContent {
    uri: Option<String>,
    url: Option<String>,
}
fn bad(s: &str) -> DecodeError {
    DecodeError::Invalid(s.into())
}
impl Tileset {
    pub fn parse(data: &[u8], source: Url, limits: &DecodeLimits) -> Result<Self, DecodeError> {
        Self::parse_context(data, source, limits, DMat4::IDENTITY, Refine::Replace, 0)
    }
    fn parse_context(
        data: &[u8],
        source: Url,
        limits: &DecodeLimits,
        transform: DMat4,
        refine: Refine,
        depth: usize,
    ) -> Result<Self, DecodeError> {
        check_input(data, limits)?;
        let raw: RawSet = serde_json::from_slice(data)?;
        if !["1.0", "1.1"].contains(&raw.asset.version.as_str()) {
            return Err(DecodeError::Unsupported("3D Tiles asset.version".into()));
        }
        if !raw.extensions_required.is_empty() {
            return Err(DecodeError::Unsupported(format!(
                "required tileset extensions: {:?}",
                raw.extensions_required
            )));
        }
        if !raw.geometric_error.is_finite() || raw.geometric_error < 0.0 {
            return Err(bad("tileset geometricError"));
        }
        let mut set = Self {
            tiles: Vec::new(),
            root: 0,
            source,
        };
        let mut stack = vec![(raw.root, None, transform, refine, depth)];
        while let Some((r, parent, parent_transform, inherited, depth)) = stack.pop() {
            if depth > limits.max_depth || set.tiles.len() >= limits.max_nodes {
                return Err(DecodeError::Limit("tile depth/count"));
            }
            if r.implicit_tiling.is_some() || r.viewer_request_volume.is_some() {
                return Err(DecodeError::Unsupported(
                    "implicit tiling / viewerRequestVolume".into(),
                ));
            }
            if !r.geometric_error.is_finite() || r.geometric_error < 0.0 {
                return Err(bad("tile geometricError"));
            }
            let m = r
                .transform
                .map(|a| DMat4::from_cols_array(&a))
                .unwrap_or(DMat4::IDENTITY);
            if !m.is_finite()
                || m.x_axis.w != 0.0
                || m.y_axis.w != 0.0
                || m.z_axis.w != 0.0
                || m.w_axis.w != 1.0
                || m.determinant().abs() < 1e-20
            {
                return Err(bad("tile affine transform"));
            }
            let world = parent_transform * m;
            if !world.is_finite() {
                return Err(bad("composed transform overflow"));
            }
            let bounds = parse_bounds(&r.bounding_volume, world)?;
            let error = r.geometric_error * scale_bound(linear_part(world));
            if !bounds.center.is_finite() || !bounds.radius.is_finite() || !error.is_finite() {
                return Err(bad("transformed bounds/error overflow"));
            }
            let refine = r.refine.unwrap_or(inherited);
            if r.content.is_some() && r.contents.is_some() {
                return Err(bad("both content and contents"));
            }
            let content = match r.contents {
                Some(mut c) if c.len() == 1 => c.pop(),
                Some(c) if c.is_empty() => None,
                Some(_) => return Err(DecodeError::Unsupported("multiple tile contents".into())),
                None => r.content,
            };
            let uri = content
                .map(|c| {
                    c.uri
                        .or(c.url)
                        .ok_or_else(|| bad("content URI"))
                        .and_then(|u| set.source.join(&u).map_err(|_| bad("content URL")))
                })
                .transpose()?;
            let id = set.tiles.len();
            set.tiles.push(Tile {
                parent,
                children: Vec::new(),
                bounds,
                world_transform: world,
                geometric_error: error,
                refine,
                uri,
                state: ContentState::Unrequested,
                topology_complete: true,
                document_source: set.source.clone(),
            });
            if let Some(p) = parent {
                set.tiles[p].children.push(id);
            }
            for child in r.children.into_iter().rev() {
                stack.push((child, Some(id), world, refine, depth + 1));
            }
        }
        Ok(set)
    }
    /// Attach a fetched external root atomically. Its transform and omitted refine inherit the referencing tile.
    pub fn attach_external(
        &mut self,
        id: usize,
        data: &[u8],
        limits: &DecodeLimits,
    ) -> Result<(), DecodeError> {
        let anchor = self.tiles.get(id).ok_or_else(|| bad("external tile id"))?;
        if anchor.state != ContentState::Downloading {
            return Err(bad("stale external tileset response"));
        }
        let source = anchor.uri.clone().ok_or_else(|| bad("external tile URI"))?;
        if !anchor.children.is_empty() {
            return Err(bad(
                "external tileset reference cannot have explicit children",
            ));
        }
        let mut identity = source.clone();
        identity.set_fragment(None);
        let mut ancestor = Some(id);
        let mut depth = 0;
        while let Some(a) = ancestor {
            let mut url = self.tiles[a].document_source.clone();
            url.set_fragment(None);
            if url == identity {
                return Err(bad("external tileset URL cycle"));
            }
            ancestor = self.tiles[a].parent;
            depth += 1;
            if depth > limits.max_depth {
                return Err(DecodeError::Limit("external tile depth"));
            }
        }
        let mut nested = Self::parse_context(
            data,
            source,
            limits,
            anchor.world_transform,
            anchor.refine,
            depth,
        )?;
        let offset = self.tiles.len();
        if offset
            .checked_add(nested.tiles.len())
            .is_none_or(|n| n > limits.max_nodes)
        {
            return Err(DecodeError::Limit("total tiles across external documents"));
        }
        for tile in &mut nested.tiles {
            tile.parent = Some(tile.parent.map_or(id, |p| p + offset));
            for child in &mut tile.children {
                *child += offset;
            }
        }
        self.tiles.extend(nested.tiles);
        self.tiles[id].children.push(offset + nested.root);
        self.tiles[id].uri = None;
        self.tiles[id].state = ContentState::GpuReady;
        self.tiles[id].topology_complete = true;
        Ok(())
    }
    pub fn select(&self, camera: Camera, max_sse: f64) -> Selection {
        let mut s = self.select_node(self.root, camera, max_sse);
        s.requests.sort_unstable();
        s.requests.dedup();
        s.retain.sort_unstable();
        s.retain.dedup();
        s
    }
    fn select_node(&self, id: usize, camera: Camera, max_sse: f64) -> Selection {
        let t = &self.tiles[id];
        if !camera.visible(t.bounds) {
            return Selection::complete();
        }
        let own = t.uri.is_some();
        let ready = own && t.state == ContentState::GpuReady;
        let mut s = Selection::complete();
        if own && t.state == ContentState::Unrequested {
            s.requests.push(id);
        }
        if ready {
            s.retain.push(id);
        }
        let refine = !own || camera.sse(t.bounds, t.geometric_error) > max_sse || !ready;
        if !refine || t.children.is_empty() {
            if ready {
                s.draws.push(id);
                return s;
            }
            if t.children.is_empty() {
                s.coverage = if !own && t.topology_complete {
                    Coverage::Complete
                } else {
                    Coverage::Incomplete
                };
                return s;
            }
        }
        let mut child = Selection::complete();
        if !t.topology_complete {
            child.coverage = Coverage::Incomplete;
        }
        for &c in &t.children {
            child.merge(self.select_node(c, camera, max_sse));
        }
        s.requests.extend(child.requests);
        s.retain.extend(child.retain);
        if t.refine == Refine::Add {
            if ready {
                s.draws.push(id);
            }
            s.draws.extend(child.draws);
            if (own && !ready) || child.coverage == Coverage::Incomplete {
                s.coverage = Coverage::Incomplete;
            }
        } else if child.coverage == Coverage::Complete {
            s.draws = child.draws;
        } else if ready {
            s.draws = vec![id]; // Roll back all temporary descendant draws, keep requests and pins.
        } else {
            s.draws = child.draws;
            s.coverage = Coverage::Incomplete;
        }
        s
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    Complete,
    Incomplete,
}
#[derive(Debug)]
pub struct Selection {
    pub draws: Vec<usize>,
    pub requests: Vec<usize>,
    pub retain: Vec<usize>,
    pub coverage: Coverage,
}
impl Selection {
    fn complete() -> Self {
        Self {
            draws: vec![],
            requests: vec![],
            retain: vec![],
            coverage: Coverage::Complete,
        }
    }
    fn merge(&mut self, other: Self) {
        self.draws.extend(other.draws);
        self.requests.extend(other.requests);
        self.retain.extend(other.retain);
        if other.coverage == Coverage::Incomplete {
            self.coverage = Coverage::Incomplete;
        }
    }
}
fn numbers<const N: usize>(v: &Value) -> Result<[f64; N], DecodeError> {
    let a = v
        .as_array()
        .filter(|v| v.len() == N)
        .ok_or_else(|| bad("boundingVolume length"))?;
    let mut out = [0.0; N];
    for (x, v) in out.iter_mut().zip(a) {
        *x = v
            .as_f64()
            .filter(|v| v.is_finite())
            .ok_or_else(|| bad("boundingVolume finite"))?;
    }
    Ok(out)
}
fn parse_bounds(v: &Value, m: DMat4) -> Result<Sphere, DecodeError> {
    let n = ["sphere", "box", "region"]
        .iter()
        .filter(|k| v.get(**k).is_some())
        .count();
    if n != 1 {
        return Err(bad("exactly one boundingVolume"));
    }
    if let Some(v) = v.get("sphere") {
        let a = numbers::<4>(v)?;
        if a[3] < 0.0 {
            return Err(bad("negative radius"));
        }
        Ok(Sphere {
            center: DVec3::new(a[0], a[1], a[2]),
            radius: a[3],
        }
        .transformed(m))
    } else if let Some(v) = v.get("box") {
        let a = numbers::<12>(v)?;
        let center = m.transform_point3(DVec3::new(a[0], a[1], a[2]));
        let axes = [
            DVec3::new(a[3], a[4], a[5]),
            DVec3::new(a[6], a[7], a[8]),
            DVec3::new(a[9], a[10], a[11]),
        ]
        .map(|a| m.transform_vector3(a));
        let mut radius = 0f64;
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                for z in [-1.0, 1.0] {
                    radius = radius.max((axes[0] * x + axes[1] * y + axes[2] * z).length());
                }
            }
        }
        Ok(Sphere { center, radius })
    } else {
        let a = numbers::<6>(&v["region"])?;
        if a[0].abs() > std::f64::consts::PI
            || a[2].abs() > std::f64::consts::PI
            || a[1].abs() > std::f64::consts::FRAC_PI_2
            || a[3].abs() > std::f64::consts::FRAC_PI_2
            || a[1] > a[3]
            || a[4] > a[5]
        {
            return Err(bad("geographic region"));
        }
        Ok(region_sphere(a))
    }
}
// Interval bounds enclose every WGS84 position, including curved region interiors.
// Region coordinates are ECEF and intentionally ignore the tile transform.
fn region_sphere(a: [f64; 6]) -> Sphere {
    use std::f64::consts::{FRAC_PI_2, PI, TAU};
    fn sine(lo: f64, hi: f64) -> (f64, f64) {
        let mut min = lo.sin().min(hi.sin());
        let mut max = lo.sin().max(hi.sin());
        let first = ((lo - FRAC_PI_2) / PI).ceil() as i32;
        let last = ((hi - FRAC_PI_2) / PI).floor() as i32;
        for k in first..=last {
            if k.rem_euclid(2) == 0 {
                max = 1.0;
            } else {
                min = -1.0;
            }
        }
        (min, max)
    }
    fn product(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
        let v = [a.0 * b.0, a.0 * b.1, a.1 * b.0, a.1 * b.1];
        (
            v.iter().copied().fold(f64::INFINITY, f64::min),
            v.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        )
    }
    let east = if a[2] < a[0] { a[2] + TAU } else { a[2] };
    let sin_lat = sine(a[1], a[3]);
    let cos_lat = sine(a[1] + FRAC_PI_2, a[3] + FRAC_PI_2);
    let sq_min = if sin_lat.0 <= 0.0 && sin_lat.1 >= 0.0 {
        0.0
    } else {
        sin_lat.0.powi(2).min(sin_lat.1.powi(2))
    };
    let sq_max = sin_lat.0.powi(2).max(sin_lat.1.powi(2));
    let e2 = 1.0 - (cesium_math::WGS84_B / WGS84_A).powi(2);
    let n = (
        WGS84_A / (1.0 - e2 * sq_min).sqrt(),
        WGS84_A / (1.0 - e2 * sq_max).sqrt(),
    );
    let radial = product((n.0 + a[4], n.1 + a[5]), cos_lat);
    let x = product(radial, sine(a[0] + FRAC_PI_2, east + FRAC_PI_2));
    let y = product(radial, sine(a[0], east));
    let z = product((n.0 * (1.0 - e2) + a[4], n.1 * (1.0 - e2) + a[5]), sin_lat);
    let min = DVec3::new(x.0, y.0, z.0);
    let max = DVec3::new(x.1, y.1, z.1);
    Sphere {
        center: (min + max) * 0.5,
        radius: (max - min).length() * 0.5 + 1e-6,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn region_bounds_are_tight_and_conservative() {
        use cesium_math::Cartographic;
        for a in [
            [0.07, 0.90, 0.071, 0.901, -20.0, 150.0],
            [3.13, -0.2, -3.13, 0.3, 0.0, 1000.0],
            [
                -std::f64::consts::PI,
                1.50,
                std::f64::consts::PI,
                std::f64::consts::FRAC_PI_2,
                0.0,
                20.0,
            ],
            [
                -std::f64::consts::PI,
                -std::f64::consts::FRAC_PI_2,
                std::f64::consts::PI,
                std::f64::consts::FRAC_PI_2,
                -100.0,
                100.0,
            ],
        ] {
            let sphere = region_sphere(a);
            let east = if a[2] < a[0] {
                a[2] + std::f64::consts::TAU
            } else {
                a[2]
            };
            for i in 0..=20 {
                for j in 0..=20 {
                    for h in [a[4], a[5]] {
                        let p = Cartographic {
                            longitude: a[0] + (east - a[0]) * i as f64 / 20.0,
                            latitude: a[1] + (a[3] - a[1]) * j as f64 / 20.0,
                            height: h,
                        }
                        .to_ecef();
                        assert!(p.distance(sphere.center) <= sphere.radius + 1e-6);
                    }
                }
            }
        }
        assert!(region_sphere([0.07, 0.90, 0.071, 0.901, 0.0, 150.0]).radius < 10_000.0);
    }
    fn set(refine: Refine) -> Tileset {
        let sphere = Sphere {
            center: DVec3::ZERO,
            radius: 10.0,
        };
        let make = |parent, children, state| Tile {
            parent,
            children,
            bounds: sphere,
            world_transform: DMat4::IDENTITY,
            geometric_error: 100.0,
            refine,
            uri: Some(Url::parse("https://example.com/a.glb").unwrap()),
            state,
            topology_complete: true,
            document_source: Url::parse("https://example.com/tileset.json").unwrap(),
        };
        Tileset {
            tiles: vec![
                make(None, vec![1, 2], ContentState::GpuReady),
                make(Some(0), vec![], ContentState::GpuReady),
                make(Some(0), vec![], ContentState::Unrequested),
            ],
            root: 0,
            source: Url::parse("https://example.com/tileset.json").unwrap(),
        }
    }
    fn camera() -> Camera {
        Camera::look_at(DVec3::Z * 100.0, DVec3::ZERO, DVec3::Y, 800, 600).unwrap()
    }
    #[test]
    fn external_inheritance_cycles_and_atomic_limits() {
        let root = serde_json::json!({"asset":{"version":"1.1"},"geometricError":10,"root":{"boundingVolume":{"sphere":[0,0,0,50]},"geometricError":10,"refine":"ADD","transform":[1,0,0,0,0,1,0,0,0,0,1,0,100,0,0,1],"content":{"uri":"nested/child.json"}}});
        let child = serde_json::json!({"asset":{"version":"1.1"},"geometricError":0,"root":{"boundingVolume":{"sphere":[0,0,0,5]},"geometricError":0,"transform":[1,0,0,0,0,1,0,0,0,0,1,0,0,20,0,1],"content":{"uri":"mesh.glb"}}});
        let mut set = Tileset::parse(
            &serde_json::to_vec(&root).unwrap(),
            Url::parse("https://example.com/root.json").unwrap(),
            &Default::default(),
        )
        .unwrap();
        set.tiles[0].state = ContentState::Downloading;
        let limit = DecodeLimits {
            max_nodes: 1,
            ..Default::default()
        };
        assert!(set
            .attach_external(0, &serde_json::to_vec(&child).unwrap(), &limit)
            .is_err());
        assert_eq!(set.tiles.len(), 1);
        assert!(set.tiles[0].uri.is_some());
        set.attach_external(0, &serde_json::to_vec(&child).unwrap(), &Default::default())
            .unwrap();
        assert_eq!(set.tiles[1].bounds.center, DVec3::new(100.0, 20.0, 0.0));
        assert_eq!(set.tiles[1].refine, Refine::Add);
        assert_eq!(set.tiles[1].parent, Some(0));
        assert_eq!(
            set.tiles[1].uri.as_ref().unwrap().as_str(),
            "https://example.com/nested/mesh.glb"
        );
        set.tiles[1].uri = Some(Url::parse("https://example.com/root.json").unwrap());
        set.tiles[1].state = ContentState::Downloading;
        assert!(set
            .attach_external(1, &serde_json::to_vec(&child).unwrap(), &Default::default())
            .is_err());
        assert_eq!(set.tiles.len(), 2);
    }
    #[test]
    fn replace_transaction_and_failure() {
        let mut t = set(Refine::Replace);
        let s = t.select(camera(), 16.0);
        assert_eq!(s.draws, vec![0]);
        assert_eq!(s.requests, vec![2]);
        t.tiles[2].state = ContentState::PermanentFailure;
        assert_eq!(t.select(camera(), 16.0).draws, vec![0]);
        t.tiles[2].state = ContentState::GpuReady;
        assert_eq!(t.select(camera(), 16.0).draws, vec![1, 2]);
    }
    #[test]
    fn add_and_empty_parent() {
        let mut t = set(Refine::Add);
        assert_eq!(t.select(camera(), 16.0).draws, vec![0, 1]);
        assert_eq!(t.select(camera(), 16.0).coverage, Coverage::Incomplete);
        t.tiles[0].uri = None;
        t.tiles[2].state = ContentState::GpuReady;
        assert_eq!(t.select(camera(), 16.0).draws, vec![1, 2]);
    }
    #[test]
    fn inherited_refine_transform_and_relative_url() {
        let data = include_bytes!("../../../tests/fixtures/tileset.json");
        let t = Tileset::parse(
            data,
            Url::parse("https://example.com/path/tileset.json?token=secret").unwrap(),
            &DecodeLimits::default(),
        )
        .unwrap();
        assert_eq!(
            t.tiles[0].uri.as_ref().unwrap().as_str(),
            "https://example.com/path/triangle.b3dm"
        );
        assert!((t.tiles[0].bounds.center.x - 6_378_257.0).abs() < 1e-6);
    }
    #[test]
    fn randomized_replace_never_mixes_ancestor_and_descendant() {
        let mut seed = 7u64;
        for _ in 0..256 {
            let mut t = set(Refine::Replace);
            for node in &mut t.tiles {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                node.state = match seed % 3 {
                    0 => ContentState::GpuReady,
                    1 => ContentState::Unrequested,
                    _ => ContentState::PermanentFailure,
                };
            }
            let root_ready = t.tiles[0].state == ContentState::GpuReady;
            let s = t.select(camera(), 16.0);
            if s.draws.contains(&0) {
                assert!(!s.draws.contains(&1) && !s.draws.contains(&2));
            }
            if root_ready {
                assert_eq!(s.coverage, Coverage::Complete);
                assert!(!s.draws.is_empty());
            }
            for id in s.requests {
                assert_eq!(t.tiles[id].state, ContentState::Unrequested);
            }
        }
    }
    #[test]
    fn region_not_transformed() {
        let s = parse_bounds(
            &serde_json::json!({"region":[3.0,-0.1,-3.0,0.1,0.0,100.0]}),
            DMat4::from_translation(DVec3::splat(1e8)),
        )
        .unwrap();
        assert_eq!(
            s.center,
            parse_bounds(
                &serde_json::json!({"region":[3.0,-0.1,-3.0,0.1,0.0,100.0]}),
                DMat4::IDENTITY
            )
            .unwrap()
            .center
        );
    }
}
