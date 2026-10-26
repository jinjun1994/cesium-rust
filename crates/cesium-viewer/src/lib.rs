//! Small viewer SDK. Native streaming and browser static loading share the same renderer.
use cesium_formats::{globe_mesh, gltf_y_up_to_z_up, Content, DecodeLimits};
use cesium_math::{linear_part, Camera};
use cesium_renderer::{BaseColorTexture, FeatureRef, GpuMesh, RenderError, Renderer};
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

pub fn limits() -> DecodeLimits {
    DecodeLimits {
        max_decoded_bytes: 64 * 1024 * 1024,
        max_nodes: 10_000,
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
    outline: Option<GpuMesh>,
    visible: bool,
}
pub struct Scene {
    pub clock: clock::Clock,
    entities: BTreeMap<String, ResidentEntity>,
    entity_groups: Vec<entity_groups::EntityGroup>,
    pub tileset: Option<Tileset>,
    pub orbit: Orbit,
    pub max_sse: f64,
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
            let primitive = spec.primitive().map_err(RenderError)?;
            let model = spec.model(self.clock.current_time);
            let mut mesh = if spec.shape.screen() {
                renderer.upload_screen(&primitive, model.unwrap_or(DMat4::IDENTITY), true)?
            } else if matches!(spec.shape, entities::Shape::WidePolyline { .. }) {
                renderer.upload_wide_line(&primitive, model.unwrap_or(DMat4::IDENTITY), true)?
            } else {
                renderer.upload(&primitive, model.unwrap_or(DMat4::IDENTITY), true)?
            };
            if spec.shape.screen() {
                Renderer::configure_screen_distance(
                    &mut mesh,
                    spec.screen.distance(),
                    crate::screen::pixel_offset(&spec),
                    spec.screen.rotation as f32,
                )?;
            }
            let outline = crate::outline::primitive(&spec)
                .map_err(RenderError)?
                .map(|p| renderer.upload_lines(&p, model.unwrap_or(DMat4::IDENTITY), true))
                .transpose()?;
            residents.insert(
                spec.id.clone(),
                ResidentEntity {
                    spec,
                    mesh,
                    outline,
                    visible: model.is_some(),
                },
            );
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
            let primitive = spec.primitive().map_err(RenderError)?;
            let model = spec.model(self.clock.current_time);
            let mut mesh = if spec.shape.screen() {
                renderer.upload_screen(&primitive, model.unwrap_or(DMat4::IDENTITY), true)?
            } else if matches!(spec.shape, entities::Shape::WidePolyline { .. }) {
                renderer.upload_wide_line(&primitive, model.unwrap_or(DMat4::IDENTITY), true)?
            } else {
                renderer.upload(&primitive, model.unwrap_or(DMat4::IDENTITY), true)?
            };
            if spec.shape.screen() {
                Renderer::configure_screen_distance(
                    &mut mesh,
                    spec.screen.distance(),
                    crate::screen::pixel_offset(&spec),
                    spec.screen.rotation as f32,
                )?;
            }
            let outline = crate::outline::primitive(&spec)
                .map_err(RenderError)?
                .map(|p| renderer.upload_lines(&p, model.unwrap_or(DMat4::IDENTITY), true))
                .transpose()?;
            residents.push(ResidentEntity {
                spec,
                mesh,
                outline,
                visible: model.is_some(),
            });
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
        for entity in self.entities.values_mut() {
            let model = entity.spec.model(self.clock.current_time);
            entity.visible = model.is_some()
                && self
                    .entity_groups
                    .iter()
                    .all(|g| g.show || !g.entity_ids.contains(&entity.spec.id));
            if let Some(model) = model {
                entity.mesh.model = model;
                if let Some(o) = &mut entity.outline {
                    o.model = model;
                }
            }
        }
        self.advance_camera(elapsed);
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
        for e in self.entities.values().filter(|e| e.visible) {
            if e.spec.fill {
                meshes.push(&e.mesh);
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
                + self.entities.values().map(|e|e.mesh.bytes+e.outline.as_ref().map_or(0,|o|o.bytes)).sum::<usize>()
                + globe_bytes
                + self.imagery.as_ref().map_or(0, |i| i.gpu_bytes())
                + self.terrain.as_ref().map_or(0,|t|t.gpu_bytes())
                + self.poles.iter().map(|m| m.bytes).sum::<usize>()) as f64
                / 1048576.0,
            draw_calls+geo_draws+self.entities.values().filter(|e|e.visible).map(|e|usize::from(e.spec.fill)+usize::from(e.outline.is_some())).sum::<usize>(),
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

pub mod entity_variants;
