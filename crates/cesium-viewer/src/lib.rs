//! Small viewer SDK. Native streaming and browser static loading share the same renderer.
use cesium_formats::{
    globe_mesh, gltf_y_up_to_z_up, Content, DecodeLimits, Material, PointCloud, Primitive, Vertex,
};
use cesium_math::{linear_part, Camera};
use cesium_renderer::{BaseColorTexture, FeatureRef, GpuMesh, PointStyle, RenderError, Renderer};
use cesium_streaming::Residency;
use cesium_tiles::{ContentState, Selection, Tileset};
use glam::{DMat3, DMat4, DVec3};
use std::collections::{BTreeMap, HashMap};

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod web;

pub mod camera_control;
pub mod clock;
#[cfg(test)]
mod ellipsoid_tests;
pub mod entities;
pub mod entity_framing;
pub mod entity_material;
pub mod entity_resources;
pub mod geodesic;
mod geographic;
#[cfg(test)]
mod geographic_case_tests;
pub mod geojson;
pub mod imagery;
pub mod label;
pub mod label_appearance;
pub mod outline;
pub mod plane;
#[cfg(test)]
mod primitive_fill_tests;
pub mod primitive_instances;
pub mod screen;
pub mod screen_options;
pub mod terrain;
pub mod widgets;

pub fn limits() -> DecodeLimits {
    DecodeLimits {
        max_decoded_bytes: 64 * 1024 * 1024,
        max_nodes: 200_000,
        ..Default::default()
    }
}
pub struct Orbit {
    pub target: DVec3,
    pub distance: f64,
    pub yaw: f64,
    pub pitch: f64,
    basis: DMat3,
    minimum_distance: f64,
    globe_mode: bool,
    view: Option<camera_control::Pose>,
}
impl Orbit {
    pub fn new(target: DVec3, radius: f64) -> Self {
        let up = target.try_normalize().unwrap_or(DVec3::Z);
        let east = if up.z.abs() > 0.99 {
            DVec3::X
        } else {
            DVec3::Z.cross(up).normalize()
        };
        Self {
            target,
            distance: (radius * 4.0).max(100.0),
            yaw: 0.5,
            pitch: 0.6,
            basis: DMat3::from_cols(east, up.cross(east), up),
            minimum_distance: 1.0,
            globe_mode: false,
            view: None,
        }
    }
    pub fn earth() -> Self {
        let mut orbit = Self::new(DVec3::ZERO, cesium_math::WGS84_A);
        orbit.distance = cesium_math::WGS84_A * 2.65;
        orbit.globe_mode = true;
        orbit.minimum_distance = 10.0; // Minimum radial height above the local WGS84 surface.
        orbit.yaw = 110.0f64.to_radians();
        orbit.pitch = 25.0f64.to_radians();
        orbit
    }
    fn direction(&self) -> DVec3 {
        self.basis
            * DVec3::new(
                self.yaw.cos() * self.pitch.cos(),
                self.yaw.sin() * self.pitch.cos(),
                self.pitch.sin(),
            )
    }
    fn surface_radius(&self) -> f64 {
        cesium_math::ray_ellipsoid(DVec3::ZERO, self.direction()).expect("finite earth direction")
    }
    pub fn height(&self) -> f64 {
        if let Some(view) = self.view {
            return cesium_math::Cartographic::from_ecef(view.eye).map_or(10., |c| c.height);
        }
        if self.globe_mode {
            self.distance - self.surface_radius()
        } else {
            self.distance
        }
    }
    pub fn drag(&mut self, dx: f64, dy: f64) {
        if let Some(view) = &mut self.view {
            view.drag(dx, dy);
            return;
        }
        if !dx.is_finite() || !dy.is_finite() {
            return;
        }
        let height = self.height();
        let speed = if self.globe_mode {
            (height / cesium_math::WGS84_A).clamp(1e-6, 1.0)
        } else {
            1.0
        };
        self.yaw = (self.yaw - dx * 0.005 * speed).rem_euclid(std::f64::consts::TAU);
        let limit = if self.globe_mode {
            std::f64::consts::FRAC_PI_2 - 1e-6
        } else {
            1.4
        };
        self.pitch = (self.pitch + dy * 0.005 * speed).clamp(-limit, limit);
        if self.globe_mode {
            self.distance = self.surface_radius() + height.max(self.minimum_distance);
        }
    }
    /// Positive steps zoom in. Globe zoom scales height, rather than distance to Earth's centre.
    pub fn zoom(&mut self, delta: f64) {
        if let Some(view) = &mut self.view {
            view.zoom(delta);
            return;
        }
        if !delta.is_finite() {
            return;
        }
        let scale = (-delta.clamp(-100.0, 100.0) * 0.1).exp();
        let surface = if self.globe_mode {
            self.surface_radius()
        } else {
            0.0
        };
        self.distance = surface
            + ((self.distance - surface) * scale)
                .clamp(self.minimum_distance, 100_000_000.0 - surface);
    }
    pub fn camera(&self, w: u32, h: u32) -> Camera {
        if let Some(view) = self.view {
            return view.camera(w, h);
        }
        let dir = DVec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
        );
        let mut camera = Camera::look_at(
            self.target + self.basis * dir * self.distance,
            self.target,
            self.basis.z_axis,
            w,
            h,
        )
        .expect("bounded orbit camera");
        camera.near = if self.globe_mode {
            (self.height() * 0.01).clamp(0.1, 1000.0)
        } else {
            (self.distance * 1e-5).clamp(0.1, 1000.0)
        };
        camera
    }
}
struct ResidentContent {
    meshes: Vec<GpuMesh>,
    metadata: serde_json::Value,
    point_resolution: Option<f64>,
}

/// Implemented subset of Cesium point cloud shading options.
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct PointCloudOptions {
    pub point_size: f32,
    pub attenuation: bool,
    pub geometric_error_scale: f64,
    pub maximum_attenuation: Option<f32>,
    pub base_resolution: Option<f64>,
    pub normal_shading: bool,
    pub back_face_culling: bool,
}
impl Default for PointCloudOptions {
    fn default() -> Self {
        Self {
            point_size: 1.0,
            attenuation: false,
            geometric_error_scale: 1.0,
            maximum_attenuation: None,
            base_resolution: None,
            normal_shading: true,
            back_face_culling: false,
        }
    }
}
impl PointCloudOptions {
    fn style(
        self,
        tile_error: Option<f64>,
        estimate: f64,
        default_max: f32,
    ) -> Result<PointStyle, RenderError> {
        let positive = |v: f64| v.is_finite() && v >= 0.0;
        if !positive(self.geometric_error_scale)
            || self.base_resolution.is_some_and(|v| !positive(v))
        {
            return Err(RenderError("invalid point cloud options".into()));
        }
        let error = tile_error
            .filter(|&v| v > 0.0)
            .or(self.base_resolution)
            .unwrap_or(estimate)
            * self.geometric_error_scale;
        if !positive(error) || error > f32::MAX as f64 {
            return Err(RenderError("point geometric error out of range".into()));
        }
        let style = PointStyle {
            size: self.point_size,
            geometric_error: self.attenuation.then_some(error as f32),
            maximum_attenuation: self.maximum_attenuation.unwrap_or(default_max),
            normal_shading: self.normal_shading,
            back_face_culling: self.back_face_culling,
        };
        // Renderer validates again when the style is applied to an actual mesh.
        if !style.size.is_finite()
            || !(1.0..=4096.0).contains(&style.size)
            || !style.maximum_attenuation.is_finite()
            || !(1.0..=4096.0).contains(&style.maximum_attenuation)
        {
            return Err(RenderError("invalid point cloud size".into()));
        }
        Ok(style)
    }
}

fn estimated_point_resolution(cloud: &PointCloud) -> f64 {
    if cloud.points.is_empty() {
        return 0.0;
    }
    let mut min = DVec3::splat(f64::INFINITY);
    let mut max = DVec3::splat(f64::NEG_INFINITY);
    for p in &cloud.points {
        let v = DVec3::from_array(p.position.map(f64::from));
        min = min.min(v);
        max = max.max(v);
    }
    let d = max - min;
    (d.x * d.y * d.z / cloud.points.len() as f64).cbrt()
}
struct GeoLayer {
    meshes: Vec<GpuMesh>,
    features: Vec<serde_json::Value>,
    center: DVec3,
    radius: f64,
}
struct ResidentEntity {
    spec: entities::Entity,
    mesh: GpuMesh,
    local_transform: DMat4,
    extra_meshes: Vec<(GpuMesh, DMat4)>,
    model_radius: f64,
    outline: Option<GpuMesh>,
    visible: bool,
}

fn upload_entity(
    renderer: &mut Renderer,
    spec: entities::Entity,
    current_time: f64,
) -> Result<ResidentEntity, RenderError> {
    let mut prims = spec.primitives().map_err(RenderError)?;
    let mut r_sq = 0.0f64;
    for p in &prims {
        for v in &p.vertices {
            let pt = p.transform.transform_point3(DVec3::from_array(v.position.map(f64::from)));
            r_sq = r_sq.max(pt.length_squared());
        }
    }
    let model_radius = r_sq.sqrt();
    let model = spec.model(current_time);
    let base_model = model.unwrap_or(DMat4::IDENTITY);
    let first = prims.remove(0);
    let local_transform = first.transform;
    let mut mesh = if spec.shape.screen() {
        renderer.upload_screen(&first, base_model * local_transform, true)?
    } else if matches!(spec.shape, entities::Shape::WidePolyline { .. }) {
        renderer.upload_wide_line(&first, base_model * local_transform, true)?
    } else {
        renderer.upload(&first, base_model * local_transform, true)?
    };
    mesh.z_index = spec.z_index;
    if spec.shape.screen() {
        Renderer::configure_screen_distance(
            &mut mesh,
            spec.screen.distance(),
            crate::screen::pixel_offset(&spec),
            spec.screen.rotation as f32,
        )?;
    }
    let mut extra_meshes = Vec::with_capacity(prims.len());
    for p in prims {
        let local = p.transform;
        let mut m = renderer.upload(&p, base_model * local, true)?;
        m.z_index = spec.z_index;
        extra_meshes.push((m, local));
    }
    let mut outline = crate::outline::primitive(&spec)
        .map_err(RenderError)?
        .map(|p| renderer.upload_lines(&p, base_model, true))
        .transpose()?;
    if let Some(o) = &mut outline {
        o.z_index = spec.z_index;
    }
    Ok(ResidentEntity {
        spec,
        mesh,
        local_transform,
        extra_meshes,
        model_radius,
        outline,
        visible: model.is_some(),
    })
}
pub struct Scene {
    pub clock: clock::Clock,
    entities: BTreeMap<String, ResidentEntity>,
    entity_groups: Vec<entity_groups::EntityGroup>,
    pub tileset: Option<Tileset>,
    pub orbit: Orbit,
    pub max_sse: f64,
    point_cloud_options: PointCloudOptions,
    contents: HashMap<usize, ResidentContent>,
    cache: Residency<usize>,
    globe: Option<GpuMesh>,
    pub imagery: Option<imagery::Imagery>,
    pub terrain: Option<terrain::Terrain>,
    poles: Vec<GpuMesh>,
    draws: Vec<usize>,
    pub last_selection: Option<Selection>,
    pub errors: usize,
    geo_layers: BTreeMap<u32, GeoLayer>,
    next_geo_id: u32,
    terrain_focus: Option<(f64, f64)>,
    camera_control: camera_control::Control,
    request_render_mode: std::sync::atomic::AtomicBool,
    dirty: std::sync::atomic::AtomicBool,
}
impl Scene {
    pub fn new(renderer: &mut Renderer, tileset: Option<Tileset>) -> Result<Self, RenderError> {
        let (target, radius) = tileset
            .as_ref()
            .map(|t| (t.tiles[t.root].bounds.center, t.tiles[t.root].bounds.radius))
            .unwrap_or((DVec3::ZERO, cesium_math::WGS84_A));
        let map = image::load_from_memory(include_bytes!("../../../assets/earth-base-color.png"))
            .map_err(|e| RenderError(format!("Earth texture: {e}")))?
            .to_rgba8();
        let globe = Some(renderer.upload_globe(
            &globe_mesh(256, 128),
            &BaseColorTexture {
                width: map.width(),
                height: map.height(),
                rgba: map.as_raw(),
            },
        )?);
        let orbit = if tileset.is_some() {
            Orbit::new(target, radius)
        } else {
            Orbit::earth()
        };
        Ok(Self {
            clock: Default::default(),
            entities: BTreeMap::new(),
            entity_groups: vec![],
            tileset,
            orbit,
            max_sse: 16.0,
            point_cloud_options: PointCloudOptions::default(),
            contents: HashMap::new(),
            cache: Residency::new(256 * 1024 * 1024),
            globe,
            imagery: None,
            terrain: None,
            poles: Vec::new(),
            draws: Vec::new(),
            last_selection: None,
            errors: 0,
            geo_layers: BTreeMap::new(),
            next_geo_id: 0,
            terrain_focus: None,
            camera_control: Default::default(),
            request_render_mode: std::sync::atomic::AtomicBool::new(false),
            dirty: std::sync::atomic::AtomicBool::new(true),
        })
    }
    /// Replace entities atomically after decoding and uploading the entire bounded collection.
    pub fn set_entities(
        &mut self,
        renderer: &mut Renderer,
        specs: Vec<entities::Entity>,
    ) -> Result<(), RenderError> {
        // Validate collection budgets even for callers using the Rust API directly.
        let encoded = serde_json::to_vec(&specs).map_err(|e| RenderError(e.to_string()))?;
        let specs = entities::decode(&encoded).map_err(RenderError)?;
        let mut residents = BTreeMap::new();
        for spec in specs {
            let resident = upload_entity(renderer, spec, self.clock.current_time)?;
            residents.insert(resident.spec.id.clone(), resident);
        }
        self.stop_tracking();
        self.entities = residents;
        self.entity_groups.clear();
        renderer.invalidate_pick();
        Ok(())
    }
    pub fn entity_specs(&self) -> Vec<entities::Entity> {
        self.entities.values().map(|e| e.spec.clone()).collect()
    }
    /// Upsert complete entity specs atomically, preserving unrelated resident meshes and IDs.
    pub fn upsert_entities(
        &mut self,
        renderer: &mut Renderer,
        updates: Vec<entities::Entity>,
    ) -> Result<(), RenderError> {
        let bytes = serde_json::to_vec(&updates).map_err(|e| RenderError(e.to_string()))?;
        let updates = entities::decode(&bytes).map_err(RenderError)?;
        let mut merged: BTreeMap<_, _> = self
            .entity_specs()
            .into_iter()
            .map(|e| (e.id.clone(), e))
            .collect();
        for e in &updates {
            merged.insert(e.id.clone(), e.clone());
        }
        let bytes = serde_json::to_vec(&merged.into_values().collect::<Vec<_>>())
            .map_err(|e| RenderError(e.to_string()))?;
        entities::decode(&bytes).map_err(RenderError)?;
        let mut residents = Vec::new();
        for spec in updates {
            residents.push(upload_entity(renderer, spec, self.clock.current_time)?);
        }
        for resident in residents {
            self.entities.insert(resident.spec.id.clone(), resident);
        }
        renderer.invalidate_pick();
        Ok(())
    }
    pub fn show_entity(&mut self, id: &str, show: bool) -> bool {
        let Some(e) = self.entities.get_mut(id) else {
            return false;
        };
        if !show
            && self
                .camera_control
                .tracked
                .as_ref()
                .is_some_and(|(tracked, _)| tracked == id)
        {
            self.camera_control.tracked = None;
            self.camera_control.mode = "orbit";
        }
        e.spec.show = show;
        let model = e.spec.model(self.clock.current_time);
        e.visible = model.is_some()
            && self
                .entity_groups
                .iter()
                .all(|g| g.show || !g.entity_ids.contains(&e.spec.id));
        if let Some(m) = model {
            e.mesh.model = m;
            if let Some(o) = &mut e.outline {
                o.model = m;
            }
        }
        true
    }
    pub fn remove_entity(&mut self, id: &str) -> bool {
        if self
            .camera_control
            .tracked
            .as_ref()
            .is_some_and(|(tracked, _)| tracked == id)
        {
            self.stop_tracking();
        }
        self.entities.remove(id).is_some()
    }
    pub fn clear_entities(&mut self) {
        self.stop_tracking();
        self.entities.clear();
        self.entity_groups.clear();
    }
    pub fn tick(&mut self, elapsed: f64) {
        self.clock.tick(elapsed);
        self.advance_camera(elapsed);
        let cam = self.orbit.camera(960, 540);
        for entity in self.entities.values_mut() {
            let model = entity.spec.model(self.clock.current_time);
            entity.visible = model.is_some()
                && self
                    .entity_groups
                    .iter()
                    .all(|g| g.show || !g.entity_ids.contains(&entity.spec.id));
            if let Some(model) = model {
                let mut scale_mat = DMat4::IDENTITY;
                if entity.model_radius > 0.0 {
                    if let entities::Shape::Model {
                        minimum_pixel_size,
                        maximum_scale,
                        scale,
                        ..
                    } = entity.spec.shape
                    {
                        if minimum_pixel_size > 0.0 {
                            let dist = cam.eye.distance(model.w_axis.truncate());
                            let world_per_px =
                                2.0 * dist * (cam.fovy * 0.5).tan() / (cam.height.max(1) as f64);
                            let mut extra = ((minimum_pixel_size * world_per_px)
                                / (2.0 * entity.model_radius))
                                .max(1.0);
                            if let Some(max_s) = maximum_scale {
                                extra = extra.min((max_s / scale.max(1e-6)).max(1.0));
                            }
                            scale_mat = DMat4::from_scale(DVec3::splat(extra));
                        }
                    }
                }
                let scaled = model * scale_mat;
                entity.mesh.model = scaled * entity.local_transform;
                for (m, local) in &mut entity.extra_meshes {
                    m.model = scaled * *local;
                }
                if let Some(o) = &mut entity.outline {
                    o.model = scaled;
                }
            }
        }
    }
    pub fn focus_entity(&mut self, id: &str, range: f64) -> bool {
        if !range.is_finite() || !(10.0..=30_000_000.0).contains(&range) {
            return false;
        }
        let Some(position) = self
            .entities
            .get(id)
            .and_then(|e| e.spec.position(self.clock.current_time))
        else {
            return false;
        };
        self.release_camera();
        self.orbit = Orbit::new(position, range / 4.0);
        true
    }
    /// Transactional independent GeoJSON layer. Existing Earth, imagery and tile content remain visible.
    pub fn add_geojson(
        &mut self,
        renderer: &mut Renderer,
        data: geojson::Data,
    ) -> Result<u32, RenderError> {
        let bytes = data.primitives.iter().map(|p| p.gpu_bytes()).sum::<usize>();
        let used = self
            .geo_layers
            .values()
            .flat_map(|l| l.meshes.iter())
            .map(|m| m.bytes)
            .sum::<usize>();
        if self.geo_layers.len() >= 32
            || used.checked_add(bytes).is_none_or(|v| v > 64 * 1024 * 1024)
        {
            return Err(RenderError("GeoJSON layer/GPU budget".into()));
        }
        let next = self
            .next_geo_id
            .checked_add(1)
            .ok_or_else(|| RenderError("GeoJSON layer id overflow".into()))?;
        let mut meshes = Vec::new();
        for primitive in &data.primitives {
            meshes.push(renderer.upload(primitive, primitive.transform, true)?);
        }
        let id = self.next_geo_id;
        self.next_geo_id = next;
        self.geo_layers.insert(
            id,
            GeoLayer {
                meshes,
                features: data.features,
                center: data.center,
                radius: data.radius,
            },
        );
        Ok(id)
    }
    pub fn remove_geojson(&mut self, id: u32) -> bool {
        self.geo_layers.remove(&id).is_some()
    }
    pub fn clear_geojson(&mut self) {
        self.geo_layers.clear();
    }
    pub fn focus_geojson(&mut self, id: u32) -> bool {
        let Some(layer) = self.geo_layers.get(&id) else {
            return false;
        };
        let Ok(position) = cesium_math::Cartographic::from_ecef(layer.center) else {
            return false;
        };
        let mut orbit = Orbit::earth();
        orbit.yaw = position.longitude;
        let limit = std::f64::consts::FRAC_PI_2 - 1e-6;
        let surface = cesium_math::Cartographic::radians(position.longitude, position.latitude, 0.)
            .unwrap()
            .to_ecef();
        orbit.pitch = surface
            .z
            .atan2(surface.truncate().length())
            .clamp(-limit, limit);
        orbit.distance = orbit.surface_radius() + (layer.radius * 4.0).clamp(500.0, 30000000.0);
        self.release_camera();
        self.orbit = orbit;
        true
    }
    pub fn set_imagery(
        &mut self,
        renderer: &mut Renderer,
        provider: imagery::Provider,
    ) -> Result<(), RenderError> {
        let base = self
            .globe
            .as_ref()
            .ok_or_else(|| RenderError("imagery requires a globe scene".into()))?;
        self.poles.clear();
        if provider.projection == imagery::Projection::Mercator {
            for north in [true, false] {
                self.poles
                    .push(renderer.upload_shared_globe(&imagery::polar_patch(north), base)?);
            }
        }
        self.imagery = Some(imagery::Imagery::new(provider));
        if let Some(t) = &mut self.terrain {
            t.invalidate_drape();
        }
        self.update_imagery(renderer)
    }
    pub fn update_imagery(&mut self, renderer: &mut Renderer) -> Result<(), RenderError> {
        if renderer.width > 0 && renderer.height > 0 {
            if let (Some(layer), Some(base)) = (&mut self.imagery, &self.globe) {
                layer.update(
                    renderer,
                    base,
                    self.orbit.camera(renderer.width, renderer.height),
                )?;
            }
        }
        Ok(())
    }
    pub fn set_tileset(&mut self, t: Tileset) {
        let (target, radius) = (t.tiles[t.root].bounds.center, t.tiles[t.root].bounds.radius);
        self.clear_tileset();
        self.tileset = Some(t);
        self.release_camera();
        self.orbit = Orbit::new(target, radius);
    }
    pub fn clear_tileset(&mut self) {
        self.tileset = None;
        self.contents.clear();
        self.draws.clear();
        self.cache = Residency::new(256 * 1024 * 1024);
    }
    pub fn imagery_credits(&self) -> &str {
        self.imagery
            .as_ref()
            .map_or("NASA Earth Observatory · Blue Marble", |i| {
                i.provider.credits.as_str()
            })
    }
    pub fn add_static(
        &mut self,
        renderer: &mut Renderer,
        content: Content,
    ) -> Result<(), RenderError> {
        self.clear_geojson();
        self.clear_entities();
        self.globe = None;
        self.imagery = None;
        self.terrain = None;
        self.poles.clear();
        self.tileset = None;
        self.contents.clear();
        self.cache = Residency::new(256 * 1024 * 1024);
        let base = if content.tile_frame {
            DMat4::from_translation(content.rtc_center) * gltf_y_up_to_z_up()
        } else {
            DMat4::IDENTITY
        };
        let mut center = DVec3::ZERO;
        let mut n = 0usize;
        for p in &content.primitives {
            for v in &p.vertices {
                center += (base * p.transform)
                    .transform_point3(DVec3::from_array(v.position.map(|x| x as f64)));
                n += 1;
            }
        }
        if n == 0 {
            return Err(RenderError("empty content".into()));
        }
        center /= n as f64;
        let mut radius = 1f64;
        for p in &content.primitives {
            for v in &p.vertices {
                radius = radius.max(
                    center.distance(
                        (base * p.transform)
                            .transform_point3(DVec3::from_array(v.position.map(|x| x as f64))),
                    ),
                );
            }
        }
        self.orbit = Orbit::new(center, radius);
        self.commit(renderer, 0, content, base)?;
        self.draws = vec![0];
        Ok(())
    }
    pub fn commit_tile(
        &mut self,
        renderer: &mut Renderer,
        id: usize,
        content: Content,
    ) -> Result<(), RenderError> {
        let t = self
            .tileset
            .as_ref()
            .ok_or_else(|| RenderError("no tileset".into()))?;
        let world = t
            .tiles
            .get(id)
            .ok_or_else(|| RenderError("stale tile key".into()))?
            .world_transform;
        let model = world * DMat4::from_translation(content.rtc_center) * gltf_y_up_to_z_up();
        self.commit(renderer, id, content, model)?;
        if let Some(t) = &mut self.tileset {
            t.tiles[id].state = ContentState::GpuReady;
        }
        Ok(())
    }
    /// Current point cloud shading options, including settings for future tiles.
    pub fn point_cloud_options(&self) -> PointCloudOptions {
        self.point_cloud_options
    }

    /// Apply sizing changes to resident point clouds and future uploads.
    pub fn set_point_cloud_options(
        &mut self,
        options: PointCloudOptions,
    ) -> Result<(), RenderError> {
        options.style(None, 0.0, 1.0)?;
        let is_tileset = self.tileset.is_some();
        let default_max = if is_tileset { self.max_sse as f32 } else { 1.0 };
        let styles: Vec<_> = self
            .contents
            .iter()
            .filter_map(|(&id, content)| {
                content.point_resolution.map(|resolution| {
                    let error = self
                        .tileset
                        .as_ref()
                        .and_then(|t| t.tiles.get(id))
                        .map(|t| t.geometric_error);
                    options
                        .style(error, resolution, default_max)
                        .map(|style| (id, style))
                })
            })
            .collect::<Result<_, _>>()?;
        for (id, style) in styles {
            for mesh in &mut self.contents.get_mut(&id).unwrap().meshes {
                Renderer::configure_points(mesh, style)?;
            }
        }
        self.point_cloud_options = options;
        self.request_render();
        Ok(())
    }

    /// Display a standalone PNTS file in its own coordinate frame.
    pub fn add_static_points(
        &mut self,
        renderer: &mut Renderer,
        cloud: PointCloud,
    ) -> Result<(), RenderError> {
        if cloud.points.is_empty() {
            return Err(RenderError("empty point cloud".into()));
        }
        let mut min = DVec3::splat(f64::INFINITY);
        let mut max = DVec3::splat(f64::NEG_INFINITY);
        for point in &cloud.points {
            let position = DVec3::from_array(point.position.map(f64::from));
            min = min.min(position);
            max = max.max(position);
        }
        let local_center = (min + max) * 0.5;
        let radius = cloud
            .points
            .iter()
            .map(|point| DVec3::from_array(point.position.map(f64::from)).distance(local_center))
            .fold(1.0_f64, f64::max);
        let resolution = estimated_point_resolution(&cloud);
        let style = self.point_cloud_options.style(None, resolution, 1.0)?;
        let primitive = point_primitive(&cloud)?;
        let mut cache = Residency::new(256 * 1024 * 1024);
        cache
            .make_room(primitive.gpu_bytes())
            .map_err(|e| RenderError(e.to_string()))?;
        let mut mesh =
            renderer.upload_points(&primitive, DMat4::from_translation(cloud.origin), true)?;
        Renderer::configure_points(&mut mesh, style)?;
        cache
            .insert(0, mesh.bytes)
            .map_err(|e| RenderError(e.to_string()))?;
        self.clear_geojson();
        self.clear_entities();
        self.globe = None;
        self.imagery = None;
        self.terrain = None;
        self.poles.clear();
        self.tileset = None;
        self.contents.clear();
        self.cache = cache;
        self.orbit = Orbit::new(cloud.origin + local_center, radius);
        self.contents.insert(
            0,
            ResidentContent {
                meshes: vec![mesh],
                metadata: cloud.metadata,
                point_resolution: Some(resolution),
            },
        );
        self.draws = vec![0];
        Ok(())
    }

    /// Upload PNTS content in its native tile coordinate frame, including its RTC origin.
    pub fn commit_point_tile(
        &mut self,
        renderer: &mut Renderer,
        id: usize,
        cloud: PointCloud,
    ) -> Result<(), RenderError> {
        let world = self
            .tileset
            .as_ref()
            .ok_or_else(|| RenderError("no tileset".into()))?
            .tiles
            .get(id)
            .ok_or_else(|| RenderError("stale tile key".into()))?
            .world_transform;
        let model = world * DMat4::from_translation(cloud.origin);
        let resolution =
            estimated_point_resolution(&cloud) * cesium_math::scale_bound(linear_part(world));
        let tile_error = self.tileset.as_ref().unwrap().tiles[id].geometric_error;
        let style =
            self.point_cloud_options
                .style(Some(tile_error), resolution, self.max_sse as f32)?;
        let primitive = point_primitive(&cloud)?;
        let evicted = self
            .cache
            .make_room(primitive.gpu_bytes())
            .map_err(|e| RenderError(e.to_string()))?;
        for key in evicted {
            self.contents.remove(&key);
            if let Some(t) = &mut self.tileset {
                t.tiles[key].state = ContentState::Unrequested;
            }
        }
        let mut mesh = renderer.upload_points(&primitive, model, true)?;
        Renderer::configure_points(&mut mesh, style)?;
        self.cache
            .insert(id, mesh.bytes)
            .map_err(|e| RenderError(e.to_string()))?;
        self.contents.insert(
            id,
            ResidentContent {
                meshes: vec![mesh],
                metadata: cloud.metadata,
                point_resolution: Some(resolution),
            },
        );
        if let Some(t) = &mut self.tileset {
            t.tiles[id].state = ContentState::GpuReady;
        }
        Ok(())
    }
    fn commit(
        &mut self,
        renderer: &mut Renderer,
        id: usize,
        content: Content,
        model: DMat4,
    ) -> Result<(), RenderError> {
        let estimate = content
            .primitives
            .iter()
            .try_fold(0usize, |n, p| n.checked_add(p.gpu_bytes()))
            .ok_or_else(|| RenderError("GPU size overflow".into()))?;
        let evicted = self
            .cache
            .make_room(estimate)
            .map_err(|e| RenderError(e.to_string()))?;
        for key in evicted {
            self.contents.remove(&key);
            if let Some(t) = &mut self.tileset {
                t.tiles[key].state = ContentState::Unrequested;
            }
        }
        let mut meshes = Vec::new();
        for p in &content.primitives {
            meshes.push(renderer.upload(p, model * p.transform, true)?);
        }
        let bytes = meshes.iter().map(|m| m.bytes).sum();
        self.cache
            .insert(id, bytes)
            .map_err(|e| RenderError(e.to_string()))?;
        self.contents.insert(
            id,
            ResidentContent {
                meshes,
                metadata: content.metadata,
                point_resolution: None,
            },
        );
        Ok(())
    }
    pub fn select(&mut self, w: u32, h: u32) -> Vec<usize> {
        self.cache.begin_frame();
        if let Some(t) = &self.tileset {
            let selection = t.select(self.orbit.camera(w, h), self.max_sse);
            self.draws = selection.draws.clone();
            for &id in &selection.retain {
                self.cache.pin(id);
            }
            let requests = selection.requests.clone();
            self.last_selection = Some(selection);
            requests
        } else {
            for id in &self.draws {
                self.cache.pin(*id);
            }
            vec![]
        }
    }
    pub fn render(
        &self,
        renderer: &mut Renderer,
        target: &wgpu::TextureView,
        pick: Option<(u32, u32)>,
    ) -> Result<(), RenderError> {
        let mut meshes = Vec::new();
        if let Some(terrain) = &self.terrain {
            meshes.extend(terrain.meshes());
        } else if let Some(layer) = &self.imagery {
            meshes.extend(layer.meshes());
            meshes.extend(self.poles.iter());
        } else if let Some(g) = &self.globe {
            meshes.push(g);
        }
        let mut sorted_entities: Vec<&ResidentEntity> =
            self.entities.values().filter(|e| e.visible).collect();
        sorted_entities.sort_by_key(|e| e.spec.z_index);
        for e in sorted_entities {
            if e.spec.fill {
                meshes.push(&e.mesh);
                for (m, _) in &e.extra_meshes {
                    meshes.push(m);
                }
            }
            if let Some(o) = &e.outline {
                meshes.push(o);
            }
        }
        for layer in self.geo_layers.values() {
            meshes.extend(layer.meshes.iter());
        }
        for id in &self.draws {
            if let Some(c) = self.contents.get(id) {
                meshes.extend(c.meshes.iter());
            }
        }
        self.dirty
            .store(false, std::sync::atomic::Ordering::Relaxed);
        renderer.render(
            target,
            self.orbit.camera(renderer.width, renderer.height),
            &meshes,
            pick,
        )
    }
    pub fn request_render(&self) {
        self.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn mark_clean(&self) {
        self.dirty
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn set_request_render_mode(&self, enabled: bool) {
        self.request_render_mode
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn request_render_mode(&self) -> bool {
        self.request_render_mode
            .load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn properties(&self, feature: FeatureRef) -> Option<serde_json::Value> {
        if let Some(e) = self.entities.values().find(|e| {
            (e.mesh.id == feature.mesh && e.spec.fill
                || e.outline.as_ref().is_some_and(|o| o.id == feature.mesh))
                && e.visible
        }) {
            return Some(e.spec.metadata());
        }
        if let Some(layer) = self
            .geo_layers
            .values()
            .find(|l| l.meshes.iter().any(|m| m.id == feature.mesh))
        {
            return layer.features.get(feature.feature as usize).cloned();
        }
        let c = self
            .contents
            .values()
            .find(|c| c.meshes.iter().any(|m| m.id == feature.mesh))?;
        let mut out = serde_json::Map::new();
        for (k, v) in c.metadata.as_object()? {
            if let Some(v) = v.as_array().and_then(|a| a.get(feature.feature as usize)) {
                out.insert(k.clone(), v.clone());
            }
        }
        Some(serde_json::Value::Object(out))
    }
    pub fn set_terrain(
        &mut self,
        renderer: &mut Renderer,
        provider: terrain::Provider,
    ) -> Result<(), RenderError> {
        if self.globe.is_none() {
            return Err(RenderError("terrain requires an Earth scene".into()));
        }
        let layer = terrain::Terrain::new(renderer, provider)?;
        self.terrain = Some(layer);
        Ok(())
    }
    pub fn clear_terrain(&mut self) {
        self.terrain = None;
    }
    pub fn focus_location(&mut self, lon: f64, lat: f64, height: f64) -> Result<(), RenderError> {
        if !lon.is_finite()
            || !lat.is_finite()
            || lat.abs() > 90.0
            || !height.is_finite()
            || !(10.0..=30_000_000.0).contains(&height)
        {
            return Err(RenderError("location longitude/latitude/height".into()));
        }
        if self.globe.is_none() {
            return Err(RenderError("location requires Earth scene".into()));
        }
        self.release_camera();
        self.orbit = Orbit::earth();
        self.orbit.yaw = lon.to_radians().rem_euclid(std::f64::consts::TAU);
        // Orbit angles are geocentric. Convert the requested geodetic latitude
        // to the radial direction through the WGS84 surface point.
        let surface = cesium_math::Cartographic::degrees(lon, lat, 0.)
            .map_err(|e| RenderError(e.to_string()))?
            .to_ecef();
        self.orbit.pitch = surface.z.atan2(surface.truncate().length()).clamp(
            -std::f64::consts::FRAC_PI_2 + 1e-6,
            std::f64::consts::FRAC_PI_2 - 1e-6,
        );
        self.orbit.distance = self.orbit.surface_radius() + height;
        Ok(())
    }
    pub fn look_at_terrain(&mut self, lon: f64, lat: f64, range: f64) -> Result<(), RenderError> {
        self.focus_location(lon, lat, range)?;
        let lon = lon.to_radians();
        let lat = lat.to_radians();
        let height = self
            .terrain
            .as_ref()
            .and_then(|t| t.height_at(lon, lat).map(|h| h * t.exaggeration))
            .unwrap_or(0.0);
        self.orbit = Orbit::new(
            cesium_math::Cartographic::radians(lon, lat, height)
                .map_err(|e| RenderError(e.to_string()))?
                .to_ecef(),
            range / 4.0,
        );
        self.terrain_focus = Some((lon, lat));
        Ok(())
    }
    pub fn update_terrain(&mut self, renderer: &mut Renderer) -> Result<(), RenderError> {
        let Some(terrain) = &mut self.terrain else {
            return Ok(());
        };
        if let Some((lon, lat)) = self.terrain_focus {
            if !self.orbit.globe_mode {
                if let Some(h) = terrain.height_at(lon, lat) {
                    self.orbit.target =
                        cesium_math::Cartographic::radians(lon, lat, h * terrain.exaggeration)
                            .unwrap()
                            .to_ecef();
                }
            }
        }
        let camera = self.orbit.camera(renderer.width, renderer.height);
        if let Ok(position) = cesium_math::Cartographic::from_ecef(camera.eye) {
            let height = terrain
                .height_at(position.longitude, position.latitude)
                .unwrap_or(0.0)
                * terrain.exaggeration;
            if position.height < height + 10.0 {
                let outward = DVec3::new(
                    position.latitude.cos() * position.longitude.cos(),
                    position.latitude.cos() * position.longitude.sin(),
                    position.latitude.sin(),
                );
                let direction = (camera.eye - self.orbit.target).normalize();
                if let Some(view) = &mut self.orbit.view {
                    let mut c = position;
                    c.height = height + 10.;
                    view.eye = c.to_ecef();
                } else {
                    self.orbit.distance +=
                        (height + 10.0 - position.height) / direction.dot(outward).max(0.1);
                }
            }
        }
        terrain.update(
            renderer,
            self.orbit.camera(renderer.width, renderer.height),
            self.imagery.as_ref(),
        )
    }
    pub fn focus_globe(&mut self) {
        if self.globe.is_some() {
            self.release_camera();
            self.orbit = Orbit::earth();
            self.terrain_focus = None;
        }
    }
    pub fn stats(&self) -> String {
        let geo_bytes = self
            .geo_layers
            .values()
            .flat_map(|l| l.meshes.iter())
            .map(|m| m.bytes)
            .sum::<usize>();
        let geo_draws = self
            .geo_layers
            .values()
            .map(|l| l.meshes.len())
            .sum::<usize>();
        let globe_bytes = self.globe.as_ref().map_or(0, |g| g.bytes);
        let draw_calls = self
            .draws
            .iter()
            .filter_map(|id| self.contents.get(id))
            .map(|c| c.meshes.len())
            .sum::<usize>()
            + self.terrain.as_ref().map_or_else(
                || {
                    self.imagery
                        .as_ref()
                        .map_or(usize::from(self.globe.is_some()), |i| {
                            i.meshes().len() + self.poles.len()
                        })
                },
                |t| t.meshes().len(),
            );
        let terrain_status = self
            .terrain
            .as_ref()
            .map_or("terrain off".into(), |t| t.stats());
        format!(
            "Earth {} | height {:.1} m | resident {} | GPU {:.1} MiB | draws {} | errors {} | GeoJSON {} layers | Entities {} | {} | {}",
            if self.globe.is_some() { "yes" } else { "no" },
            self.orbit.height(),
            self.contents.len(),
            (self.cache.budget.used()
                + geo_bytes
                + self.entities.values().map(|e|e.mesh.bytes+e.extra_meshes.iter().map(|(m,_)|m.bytes).sum::<usize>()+e.outline.as_ref().map_or(0,|o|o.bytes)).sum::<usize>()
                + globe_bytes
                + self.imagery.as_ref().map_or(0, |i| i.gpu_bytes())
                + self.terrain.as_ref().map_or(0,|t|t.gpu_bytes())
                + self.poles.iter().map(|m| m.bytes).sum::<usize>()) as f64
                / 1048576.0,
            draw_calls+geo_draws+self.entities.values().filter(|e|e.visible).map(|e|if e.spec.fill {1+e.extra_meshes.len()} else {0}+usize::from(e.outline.is_some())).sum::<usize>(),
            self.errors,
            self.geo_layers.len(),
            self.entities.len(),
            self.imagery
                .as_ref()
                .map_or("offline base".into(), |i| i.stats()),
            terrain_status
        )
    }
}
/// Checked feature-preserving transformed normal, useful to API consumers.
pub fn normal_matrix(model: DMat4) -> Option<DMat3> {
    let a = linear_part(model);
    if !a.is_finite() || a.determinant().abs() < 1e-20 {
        None
    } else {
        Some(a.inverse().transpose())
    }
}

#[cfg(test)]
mod earth_tests {
    use super::*;
    #[test]
    fn global_camera_stays_outside_and_sees_earth() {
        let mut orbit = Orbit::earth();
        let sphere = cesium_math::Sphere {
            center: DVec3::ZERO,
            radius: cesium_math::WGS84_A,
        };
        let c = orbit.camera(1280, 800);
        assert!(c.visible(sphere));
        assert!(c.eye.length() > sphere.radius);
        assert_eq!(orbit.target, DVec3::ZERO);
        orbit.zoom(10000.0);
        let position = cesium_math::Cartographic::from_ecef(orbit.camera(1280, 800).eye).unwrap();
        assert!(position.height > 9.9);
        orbit.zoom(-10000.0);
        assert!(orbit.distance <= 100_000_000.0);
    }
    #[test]
    fn globe_zoom_reaches_close_view_and_roundtrips() {
        let mut orbit = Orbit::earth();
        let initial = orbit.height();
        orbit.zoom(10.0);
        assert!(orbit.height() < initial * 0.4);
        orbit.zoom(-10.0);
        assert!((orbit.height() - initial).abs() < 1e-6);
        for _ in 0..100 {
            orbit.zoom(2.0);
        }
        assert!(orbit.height() < 11.0);
        assert!(orbit.camera(800, 600).near < 1.0);
        let position = cesium_math::Cartographic::from_ecef(orbit.camera(800, 600).eye).unwrap();
        assert!(position.height > 9.9);
        orbit.drag(20000.0, 20000.0);
        assert!((orbit.height() - 10.0).abs() < 1e-6);
        let before = orbit.distance;
        orbit.zoom(f64::NAN);
        orbit.zoom(f64::INFINITY);
        assert_eq!(orbit.distance, before);
        orbit.zoom(-2.0);
        assert!(orbit.height() > 12.0);
        // Surface distance varies with latitude. Rotation must preserve clearance, including poles.
        for latitude in [-1.4, 0.0, 1.4] {
            orbit.pitch = latitude;
            orbit.distance = orbit.surface_radius() + 10.0;
            orbit.drag(1000.0, 1000.0);
            assert!(
                cesium_math::Cartographic::from_ecef(orbit.camera(800, 600).eye)
                    .unwrap()
                    .height
                    > 9.9
            );
        }
    }
    #[test]
    fn bundled_map_has_global_two_to_one_extent() {
        let map = image::load_from_memory(include_bytes!("../../../assets/earth-base-color.png"))
            .unwrap();
        assert_eq!(map.width(), 1024);
        assert_eq!(map.width(), map.height() * 2);
    }
    #[test]
    fn request_render_dirty_lifecycle() {
        let scene = Scene {
            clock: Default::default(),
            entities: BTreeMap::new(),
            entity_groups: vec![],
            tileset: None,
            orbit: Orbit::earth(),
            max_sse: 16.0,
            point_cloud_options: PointCloudOptions::default(),
            contents: HashMap::new(),
            cache: Residency::new(1024),
            globe: None,
            imagery: None,
            terrain: None,
            poles: Vec::new(),
            draws: Vec::new(),
            last_selection: None,
            errors: 0,
            geo_layers: BTreeMap::new(),
            next_geo_id: 0,
            terrain_focus: None,
            camera_control: Default::default(),
            request_render_mode: std::sync::atomic::AtomicBool::new(false),
            dirty: std::sync::atomic::AtomicBool::new(true),
        };
        assert!(scene.is_dirty());
        assert!(!scene.request_render_mode());
        scene.mark_clean();
        assert!(!scene.is_dirty());
        scene.set_request_render_mode(true);
        assert!(scene.request_render_mode());
        scene.request_render();
        assert!(scene.is_dirty());
    }
}

mod extended_instances;
mod line_set;
pub mod wide_polyline;

mod corridor;

pub mod entity_groups;
mod scene_projection;

pub mod scene_recipes;

pub mod polygon_surface;

pub mod polyline_volume;

pub mod entity_variants;

// Reuse the mesh vertex layout while keeping PNTS coordinates in the tile frame.
fn point_primitive(cloud: &PointCloud) -> Result<Primitive, RenderError> {
    let count = u32::try_from(cloud.points.len())
        .map_err(|_| RenderError("point count overflow".into()))?;
    let vertices = cloud
        .points
        .iter()
        .map(|p| Vertex {
            position: p.position,
            normal: p.normal,
            uv: [0.; 2],
            feature_id: p.feature_id,
            uv1: [0.; 2],
            color: p.color,
            tangent: [0.; 4],
        })
        .collect();
    let material = Material {
        unlit: true,
        lambert: cloud.has_normals,
        double_sided: true,
        alpha_blend: cloud.points.iter().any(|p| p.color[3] < 1.),
        ..Default::default()
    };
    Ok(Primitive {
        vertices,
        indices: (0..count).collect(),
        transform: DMat4::IDENTITY,
        material,
    })
}

#[cfg(test)]
mod point_cloud_render_tests {
    use super::*;

    #[test]
    fn sizing_options_follow_geometric_error_and_defaults() {
        let defaults = PointCloudOptions::default();
        let style = defaults.style(Some(3.0), 2.0, 16.0).unwrap();
        assert_eq!(style.size, 1.0);
        assert_eq!(style.geometric_error, None);
        assert!(defaults.normal_shading);
        assert!(!defaults.back_face_culling);
        let options: PointCloudOptions = serde_json::from_str(r#"{"attenuation":true,"pointSize":3,"geometricErrorScale":2,"baseResolution":0.5,"maximumAttenuation":8,"normalShading":false,"backFaceCulling":true}"#).unwrap();
        assert!(!options.normal_shading);
        assert!(options.back_face_culling);
        assert_eq!(
            options.style(Some(3.0), 4.0, 16.0).unwrap().geometric_error,
            Some(6.0)
        );
        assert_eq!(
            options.style(Some(0.0), 4.0, 16.0).unwrap().geometric_error,
            Some(1.0)
        );
        assert_eq!(
            PointCloudOptions {
                base_resolution: None,
                ..options
            }
            .style(None, 4.0, 16.0)
            .unwrap()
            .geometric_error,
            Some(8.0)
        );
        assert!(PointCloudOptions {
            point_size: 0.0,
            ..options
        }
        .style(None, 0.0, 1.0)
        .is_err());
        assert!(serde_json::from_str::<PointCloudOptions>(r#"{"eyeDomeLighting":true}"#).is_err());
    }

    #[test]
    #[ignore = "requires a native GPU or Mesa Vulkan software adapter"]
    fn standalone_pnts_uploads_renders_and_retains_metadata() {
        pollster::block_on(async {
            let inst = cesium_renderer::instance();
            let (_, device, queue) = cesium_renderer::device(&inst, None).await.unwrap();
            let mut renderer =
                Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 64, 64).unwrap();
            let mut scene = Scene::new(&mut renderer, None).unwrap();
            let cloud = cesium_formats::decode_point_cloud(
                include_bytes!("../../../tests/fixtures/cesium-pointcloud-batched.pnts"),
                &DecodeLimits::default(),
            )
            .unwrap();
            let id = cloud.points[0].feature_id;
            scene.add_static_points(&mut renderer, cloud).unwrap();
            let options = PointCloudOptions {
                point_size: 5.0,
                attenuation: true,
                base_resolution: Some(2.0),
                maximum_attenuation: Some(12.0),
                normal_shading: false,
                back_face_culling: true,
                ..Default::default()
            };
            scene.set_point_cloud_options(options).unwrap();
            assert_eq!(scene.point_cloud_options(), options);
            let style = scene.contents[&0].meshes[0].point_style().unwrap();
            assert_eq!(style.size, 5.0);
            assert_eq!(style.geometric_error, Some(2.0));
            assert_eq!(style.maximum_attenuation, 12.0);
            assert!(!style.normal_shading);
            assert!(style.back_face_culling);
            assert!(scene.tileset.is_none());
            assert!(scene.globe.is_none());
            assert_eq!(scene.draws, [0]);
            let resident = &scene.contents[&0];
            assert_eq!(resident.metadata["name"][0], "section0");
            let feature = FeatureRef {
                mesh: resident.meshes[0].id,
                feature: id,
            };
            assert_eq!(
                scene.properties(feature).unwrap()["name"],
                resident.metadata["name"][id as usize]
            );
            let target = cesium_renderer::attachment(
                &renderer.device,
                64,
                64,
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
        });
    }

    #[test]
    fn scene_renders_draco_point_cloud() {
        pollster::block_on(async {
            let inst = cesium_renderer::instance();
            let (_, device, queue) = cesium_renderer::device(&inst, None).await.unwrap();
            let mut renderer =
                Renderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb, 64, 64).unwrap();
            let mut scene = Scene::new(&mut renderer, None).unwrap();
            let cloud = cesium_formats::decode_point_cloud(
                include_bytes!("../../../tests/fixtures/cesium-pointcloud-draco-batched.pnts"),
                &DecodeLimits::default(),
            )
            .unwrap();
            let id = cloud.points[0].feature_id;
            scene.add_static_points(&mut renderer, cloud).unwrap();
            let target = cesium_renderer::attachment(
                &renderer.device,
                64,
                64,
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
            let resident = &scene.contents[&0];
            let feature = FeatureRef {
                mesh: resident.meshes[0].id,
                feature: id,
            };
            assert_eq!(
                scene.properties(feature).unwrap()["name"],
                resident.metadata["name"][id as usize]
            );
        });
    }

    #[test]
    fn upstream_batched_points_preserve_geometry_color_and_feature_ids() {
        let cloud = cesium_formats::decode_point_cloud(
            include_bytes!("../../../tests/fixtures/cesium-pointcloud-batched.pnts"),
            &DecodeLimits::default(),
        )
        .unwrap();
        let p = point_primitive(&cloud).unwrap();
        assert_eq!(p.vertices.len(), cloud.points.len());
        assert_eq!(p.indices.len(), cloud.points.len());
        for (i, (vertex, point)) in p.vertices.iter().zip(&cloud.points).enumerate() {
            assert_eq!(p.indices[i] as usize, i);
            assert_eq!(vertex.position, point.position);
            assert_eq!(vertex.color, point.color);
            assert_eq!(vertex.feature_id, point.feature_id);
        }
        assert!(p.material.unlit);
        assert_eq!(p.transform, DMat4::IDENTITY);
    }
}
