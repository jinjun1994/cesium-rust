use crate::{limits, Scene};
use cesium_formats::decode_content;
use cesium_renderer::{device, instance, Renderer};
use cesium_tiles::{ContentState, Tileset};
use url::Url;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
fn js(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}
struct Running {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
    scene: Scene,
    // Browser WebGPU keeps its external instance alive for the device lifetime.
    _adapter: wgpu::Adapter,
    _instance: wgpu::Instance,
}
#[wasm_bindgen]
pub struct WebViewer {
    running: Option<Running>,
    last_pick: String,
}
#[wasm_bindgen]
pub async fn attach_canvas(canvas: HtmlCanvasElement) -> Result<WebViewer, JsValue> {
    std::panic::set_hook(Box::new(|info| {
        web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&info.to_string()));
    }));
    let w = canvas.width();
    let h = canvas.height();
    let instance = instance();
    let surface = instance
        .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
        .map_err(js)?;
    let (adapter, device, queue) = device(&instance, Some(&surface)).await.map_err(js)?;
    let (base_format, format) =
        cesium_renderer::surface_formats(&surface.get_capabilities(&adapter)).map_err(js)?;
    let mut config = surface
        .get_default_config(&adapter, w.max(1), h.max(1))
        .ok_or_else(|| js("no surface configuration"))?;
    config.format = base_format;
    if base_format != format {
        config.view_formats = vec![format];
    }
    surface.configure(&device, &config);
    let mut renderer = Renderer::new(device, queue, format, w, h).map_err(js)?;
    let scene = Scene::new(&mut renderer, None).map_err(js)?;
    Ok(WebViewer {
        running: Some(Running {
            surface,
            config,
            renderer,
            scene,
            _adapter: adapter,
            _instance: instance,
        }),
        last_pick: "null".into(),
    })
}
#[wasm_bindgen]
impl WebViewer {
    #[wasm_bindgen(js_name=cameraCommand)]
    pub fn camera_command(&mut self, json: &str) -> Result<(), JsValue> {
        if json.len() > 16384 {
            return Err(js("camera command byte limit"));
        }
        let command: crate::camera_control::Command = serde_json::from_str(json).map_err(js)?;
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene.camera_command(command).map_err(js)?;
        r.renderer.invalidate_pick();
        self.last_pick = "null".into();
        Ok(())
    }
    #[wasm_bindgen(getter,js_name=cameraState)]
    pub fn camera_state(&self) -> String {
        self.running
            .as_ref()
            .map_or_else(|| "null".into(), |r| r.scene.camera_state().to_string())
    }
    #[wasm_bindgen(js_name=setPrimitiveInstances)]
    pub fn set_primitive_instances(&mut self, bytes: &[u8]) -> Result<(), JsValue> {
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene
            .set_primitive_instances(&mut r.renderer, bytes)
            .map_err(js)?;
        self.last_pick = "null".into();
        Ok(())
    }
    #[wasm_bindgen(getter,js_name=maximumAliasedLineWidth)]
    pub fn maximum_aliased_line_width(&self) -> u32 {
        1
    }
    #[wasm_bindgen(js_name=setEntities)]
    pub fn set_entities(&mut self, bytes: &[u8]) -> Result<(), JsValue> {
        let specs = crate::entities::decode(bytes).map_err(js)?;
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene.set_entities(&mut r.renderer, specs).map_err(js)?;
        self.last_pick = "null".into();
        Ok(())
    }
    #[wasm_bindgen(js_name=setGroupedEntities)]
    pub fn set_grouped_entities(&mut self, bytes: &[u8]) -> Result<(), JsValue> {
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene
            .set_grouped_entities(&mut r.renderer, bytes)
            .map_err(js)
    }
    #[wasm_bindgen(getter,js_name=entityGroups)]
    pub fn entity_groups(&self) -> String {
        self.running
            .as_ref()
            .map(|r| serde_json::to_string(r.scene.entity_groups()).unwrap())
            .unwrap_or_else(|| "[]".into())
    }
    #[wasm_bindgen(js_name=showEntityGroup)]
    pub fn show_entity_group(&mut self, id: &str, show: bool) -> bool {
        self.running.as_mut().is_some_and(|r| {
            let ok = r.scene.show_entity_group(id, show);
            r.renderer.invalidate_pick();
            ok
        })
    }
    #[wasm_bindgen(js_name=projectCartesian)]
    pub fn project_cartesian(&self, value: &str) -> Result<String, JsValue> {
        if value.len() > 1024 {
            return Err(js("position limit"));
        }
        let p: [f64; 3] = serde_json::from_str(value).map_err(js)?;
        let r = self
            .running
            .as_ref()
            .ok_or_else(|| js("viewer destroyed"))?;
        serde_json::to_string(&r.scene.cartesian_to_canvas(
            glam::DVec3::from_array(p),
            r.renderer.width,
            r.renderer.height,
        ))
        .map_err(js)
    }
    #[wasm_bindgen(js_name=loadCzml)]
    pub fn load_czml(&mut self, bytes: &[u8]) -> Result<(), JsValue> {
        let (specs, clock) = crate::entities::czml(bytes).map_err(js)?;
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene.set_entities(&mut r.renderer, specs).map_err(js)?;
        r.scene.clock = clock;
        r.scene.tick(0.);
        self.last_pick = "null".into();
        Ok(())
    }
    #[wasm_bindgen(js_name=setClock)]
    pub fn set_clock(&mut self, json: &str) -> Result<(), JsValue> {
        let c: crate::clock::Clock = serde_json::from_str(json).map_err(js)?;
        c.validate().map_err(js)?;
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene.clock = c;
        r.scene.tick(0.);
        r.renderer.invalidate_pick();
        self.last_pick = "null".into();
        Ok(())
    }
    #[wasm_bindgen(getter,js_name=clockState)]
    pub fn clock_state(&self) -> String {
        self.running
            .as_ref()
            .map(|r| serde_json::to_string(&r.scene.clock).unwrap())
            .unwrap_or("null".into())
    }
    #[wasm_bindgen(js_name=advanceTime)]
    pub fn advance_time(&mut self, elapsed: f64) {
        if let Some(r) = &mut self.running {
            let before = r.scene.orbit.camera(1, 1);
            r.scene.tick(elapsed);
            let after = r.scene.orbit.camera(1, 1);
            if before.eye != after.eye || before.camera_to_ecef != after.camera_to_ecef {
                r.renderer.invalidate_pick();
                self.last_pick = "null".into();
            }
        }
    }
    #[wasm_bindgen(js_name=upsertEntities)]
    pub fn upsert_entities(&mut self, bytes: &[u8]) -> Result<(), JsValue> {
        let specs = crate::entities::decode(bytes).map_err(js)?;
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene
            .upsert_entities(&mut r.renderer, specs)
            .map_err(js)?;
        self.last_pick = "null".into();
        Ok(())
    }
    #[wasm_bindgen(getter,js_name=entitySpecs)]
    pub fn entity_specs(&self) -> String {
        self.running
            .as_ref()
            .map(|r| serde_json::to_string(&r.scene.entity_specs()).unwrap())
            .unwrap_or("[]".into())
    }
    #[wasm_bindgen(js_name=showEntity)]
    pub fn show_entity(&mut self, id: &str, show: bool) -> bool {
        if let Some(r) = &mut self.running {
            let found = r.scene.show_entity(id, show);
            if found {
                r.renderer.invalidate_pick();
                self.last_pick = "null".into();
            }
            return found;
        }
        false
    }
    #[wasm_bindgen(js_name=removeEntity)]
    pub fn remove_entity(&mut self, id: &str) -> bool {
        if let Some(r) = &mut self.running {
            let removed = r.scene.remove_entity(id);
            if removed {
                r.renderer.invalidate_pick();
                self.last_pick = "null".into();
            }
            return removed;
        }
        false
    }
    #[wasm_bindgen(js_name=clearEntities)]
    pub fn clear_entities(&mut self) {
        if let Some(r) = &mut self.running {
            r.scene.clear_entities();
            r.renderer.invalidate_pick();
            self.last_pick = "null".into();
        }
    }
    #[wasm_bindgen(js_name=zoomToEntities)]
    pub fn zoom_to_entities(&mut self, json: &str) -> Result<(), JsValue> {
        if json.len() > 65536 {
            return Err(js("entity framing id byte limit"));
        }
        let ids: Vec<String> = serde_json::from_str(json).map_err(js)?;
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene
            .zoom_to_entities(&ids, r.renderer.width, r.renderer.height)
            .map_err(js)?;
        r.renderer.invalidate_pick();
        self.last_pick = "null".into();
        Ok(())
    }
    #[wasm_bindgen(js_name=focusEntity)]
    pub fn focus_entity(&mut self, id: &str, range: f64) -> bool {
        if let Some(r) = &mut self.running {
            let found = r.scene.focus_entity(id, range);
            if found {
                r.renderer.invalidate_pick();
                self.last_pick = "null".into();
            }
            found
        } else {
            false
        }
    }
    #[wasm_bindgen(js_name=renderFrame)]
    pub fn render_frame(&mut self, pick_x: i32, pick_y: i32) -> Result<(), JsValue> {
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        if let Some(feature) = r.renderer.poll_pick() {
            r.renderer.selected = feature;
            self.last_pick=match feature {Some(f)=>serde_json::json!({"mesh":f.mesh.to_string(),"feature":f.feature,"properties":r.scene.properties(f)}).to_string(),None=>"null".into()};
        }
        if r.renderer.width == 0 || r.renderer.height == 0 {
            return Ok(());
        }
        r.scene.select(r.renderer.width, r.renderer.height);
        r.scene.update_imagery(&mut r.renderer).map_err(js)?;
        r.scene.update_terrain(&mut r.renderer).map_err(js)?;
        let texture = match r.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                r.surface.configure(&r.renderer.device, &r.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(())
            }
            other => return Err(js(format!("surface: {other:?}"))),
        };
        let pick = if pick_x >= 0 && pick_y >= 0 {
            Some((pick_x as u32, pick_y as u32))
        } else {
            None
        };
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(r.renderer.format()),
            ..Default::default()
        });
        r.scene.render(&mut r.renderer, &view, pick).map_err(js)?;

        r.renderer.queue.present(texture);
        Ok(())
    }
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsValue> {
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.renderer.resize(width, height).map_err(js)?;
        if width > 0 && height > 0 {
            r.config.width = width;
            r.config.height = height;
            r.surface.configure(&r.renderer.device, &r.config);
        }
        Ok(())
    }
    pub fn orbit(&mut self, dx: f64, dy: f64) {
        if dx.is_finite() && dy.is_finite() {
            if let Some(r) = &mut self.running {
                r.scene.camera_drag(dx, dy);
                r.renderer.invalidate_pick();
                self.last_pick = "null".into();
            }
        }
    }
    pub fn zoom(&mut self, delta: f64) {
        if delta.is_finite() {
            if let Some(r) = &mut self.running {
                r.scene.camera_zoom(delta);
                r.renderer.invalidate_pick();
                self.last_pick = "null".into();
            }
        }
    }
    #[wasm_bindgen(js_name=focusGlobe)]
    pub fn focus_globe(&mut self) {
        if let Some(r) = &mut self.running {
            r.scene.focus_globe();
            r.renderer.invalidate_pick();
            self.last_pick = "null".into();
        }
    }
    #[wasm_bindgen(js_name=showGlobe)]
    pub fn show_globe(&mut self) -> Result<(), JsValue> {
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        let scene = Scene::new(&mut r.renderer, None).map_err(js)?;
        r.renderer
            .resize(r.renderer.width, r.renderer.height)
            .map_err(js)?;
        r.renderer.selected = None;
        r.scene = scene;
        self.last_pick = "null".into();
        Ok(())
    }
    #[wasm_bindgen(js_name=addGeoJson)]
    pub fn add_geojson(&mut self, bytes: &[u8], style: &str) -> Result<u32, JsValue> {
        let options: crate::geojson::Style = serde_json::from_str(style).map_err(js)?;
        let data = crate::geojson::decode(bytes, &options, &Default::default()).map_err(js)?;
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene.add_geojson(&mut r.renderer, data).map_err(js)
    }
    #[wasm_bindgen(js_name=removeGeoJson)]
    pub fn remove_geojson(&mut self, id: u32) -> bool {
        if let Some(r) = &mut self.running {
            let removed = r.scene.remove_geojson(id);
            if removed {
                r.renderer.invalidate_pick();
                self.last_pick = "null".into();
            }
            return removed;
        }
        false
    }
    #[wasm_bindgen(js_name=clearGeoJson)]
    pub fn clear_geojson(&mut self) {
        if let Some(r) = &mut self.running {
            r.scene.clear_geojson();
            r.renderer.invalidate_pick();
            self.last_pick = "null".into();
        }
    }
    #[wasm_bindgen(js_name=focusGeoJson)]
    pub fn focus_geojson(&mut self, id: u32) -> bool {
        if let Some(r) = &mut self.running {
            let found = r.scene.focus_geojson(id);
            if found {
                r.renderer.invalidate_pick();
                self.last_pick = "null".into();
            }
            found
        } else {
            false
        }
    }
    #[wasm_bindgen(js_name=loadBytes)]
    pub fn load_bytes(&mut self, bytes: &[u8]) -> Result<(), JsValue> {
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.renderer
            .resize(r.renderer.width, r.renderer.height)
            .map_err(js)?;
        r.renderer.selected = None;
        if bytes.starts_with(b"pnts") {
            let cloud = cesium_formats::decode_point_cloud(bytes, &limits()).map_err(js)?;
            r.scene
                .add_static_points(&mut r.renderer, cloud)
                .map_err(js)?;
        } else {
            let c = decode_content(bytes, &limits()).map_err(js)?;
            r.scene.add_static(&mut r.renderer, c).map_err(js)?;
        }
        self.last_pick = "null".into();
        Ok(())
    }
    #[wasm_bindgen(getter, js_name=pointCloudOptions)]
    pub fn point_cloud_options(&self) -> String {
        self.running.as_ref().map_or_else(
            || "null".into(),
            |r| {
                serde_json::to_string(&r.scene.point_cloud_options())
                    .unwrap_or_else(|_| "null".into())
            },
        )
    }

    /// Set the implemented point shading options using camelCase JSON.
    #[wasm_bindgen(js_name=setPointCloudOptions)]
    pub fn set_point_cloud_options(&mut self, json: &str) -> Result<(), JsValue> {
        let options: crate::PointCloudOptions = serde_json::from_str(json).map_err(js)?;
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene.set_point_cloud_options(options).map_err(js)?;
        r.renderer.invalidate_pick();
        self.last_pick = "null".into();
        Ok(())
    }
    #[wasm_bindgen(js_name=loadTileset)]
    pub fn load_tileset(&mut self, bytes: &[u8], base: &str) -> Result<(), JsValue> {
        let t = Tileset::parse(bytes, Url::parse(base).map_err(js)?, &limits()).map_err(js)?;
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        let scene = Scene::new(&mut r.renderer, Some(t)).map_err(js)?;
        r.renderer
            .resize(r.renderer.width, r.renderer.height)
            .map_err(js)?;
        r.renderer.selected = None;
        r.scene = scene;
        self.last_pick = "null".into();
        Ok(())
    }
    #[wasm_bindgen(js_name=setPublicTerrain)]
    pub fn set_public_terrain(&mut self) -> Result<(), JsValue> {
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene
            .set_terrain(&mut r.renderer, crate::terrain::Provider::public_terrain())
            .map_err(js)
    }
    #[wasm_bindgen(js_name=setTerrariumTerrain)]
    pub fn set_terrarium_terrain(&mut self, template: &str, max_level: u32) -> Result<(), JsValue> {
        let provider = crate::terrain::Provider::terrarium(template, max_level).map_err(js)?;
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene.set_terrain(&mut r.renderer, provider).map_err(js)
    }
    #[wasm_bindgen(js_name=setTerrainLayer)]
    pub fn set_terrain_layer(
        &mut self,
        bytes: &[u8],
        source: &str,
        authorization: &str,
    ) -> Result<(), JsValue> {
        let mut provider =
            crate::terrain::Provider::layer(bytes, Url::parse(source).map_err(js)?).map_err(js)?;
        if !authorization.is_empty() {
            provider = provider
                .with_authorization(authorization.into())
                .map_err(js)?;
        }
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene.set_terrain(&mut r.renderer, provider).map_err(js)
    }
    #[wasm_bindgen(js_name=cancelTerrainRequests)]
    pub fn cancel_terrain_requests(&mut self) {
        if let Some(layer) = self.running.as_mut().and_then(|r| r.scene.terrain.as_mut()) {
            layer.cancel_requests();
        }
    }
    #[wasm_bindgen(js_name=clearTerrain)]
    pub fn clear_terrain(&mut self) {
        if let Some(r) = &mut self.running {
            r.scene.clear_terrain();
        }
    }
    #[wasm_bindgen(js_name=setTerrainOptions)]
    pub fn set_terrain_options(&mut self, exaggeration: f64, sse: f64) -> Result<(), JsValue> {
        self.running
            .as_mut()
            .and_then(|r| r.scene.terrain.as_mut())
            .ok_or_else(|| js("terrain not enabled"))?
            .options(exaggeration, sse)
            .map_err(js)
    }
    #[wasm_bindgen(js_name=sampleHeight)]
    pub fn sample_height(&self, longitude: f64, latitude: f64) -> Option<f64> {
        self.running
            .as_ref()?
            .scene
            .terrain
            .as_ref()?
            .height_at(longitude.to_radians(), latitude.to_radians())
    }
    #[wasm_bindgen(js_name=focusLocation)]
    pub fn focus_location(
        &mut self,
        longitude: f64,
        latitude: f64,
        height: f64,
    ) -> Result<(), JsValue> {
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene
            .focus_location(longitude, latitude, height)
            .map_err(js)?;
        r.renderer.invalidate_pick();
        self.last_pick = "null".into();
        Ok(())
    }
    #[wasm_bindgen(js_name=lookAtTerrain)]
    pub fn look_at_terrain(&mut self, lon: f64, lat: f64, range: f64) -> Result<(), JsValue> {
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene.look_at_terrain(lon, lat, range).map_err(js)?;
        r.renderer.invalidate_pick();
        self.last_pick = "null".into();
        Ok(())
    }
    #[wasm_bindgen(js_name=takeTerrainRequests)]
    pub fn take_terrain_requests(&mut self, max: u32) -> Result<String, JsValue> {
        let Some(layer) = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?
            .scene
            .terrain
            .as_mut()
        else {
            return Ok("[]".into());
        };
        let authorization = layer.provider.authorization().map(str::to_owned);
        let accept = layer.provider.accept().to_owned();
        let requests:Vec<_>=layer.take_requests(max.min(4) as usize).into_iter().map(|(tile,url)|serde_json::json!({"id":tile.key(),"url":url.as_str(),"authorization":authorization,"accept":accept})).collect();
        serde_json::to_string(&requests).map_err(js)
    }
    #[wasm_bindgen(js_name=submitTerrain)]
    pub fn submit_terrain(&mut self, id: &str, bytes: &[u8]) -> Result<(), JsValue> {
        let tile = crate::imagery::Tile::parse(id).ok_or_else(|| js("terrain tile key"))?;
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        let layer = r
            .scene
            .terrain
            .as_mut()
            .ok_or_else(|| js("terrain disabled"))?;
        let data = crate::terrain::decode(bytes, layer.provider.format).map_err(js)?;
        layer
            .submit(&mut r.renderer, tile, data, r.scene.imagery.as_ref())
            .map_err(js)
    }
    #[wasm_bindgen(js_name=failTerrain)]
    pub fn fail_terrain(&mut self, id: &str) {
        if let Some(layer) = self.running.as_mut().and_then(|r| r.scene.terrain.as_mut()) {
            if let Some(tile) = crate::imagery::Tile::parse(id) {
                layer.fail(tile);
            }
        }
    }
    #[wasm_bindgen(getter,js_name=terrainCredits)]
    pub fn terrain_credits(&self) -> String {
        self.running
            .as_ref()
            .and_then(|r| r.scene.terrain.as_ref())
            .map_or(String::new(), |t| t.provider.credits.clone())
    }
    #[wasm_bindgen(js_name=setImageryTemplate)]
    pub fn set_imagery_template(
        &mut self,
        template: &str,
        geographic: bool,
        max: u32,
        credits: &str,
    ) -> Result<(), JsValue> {
        let p = crate::imagery::Provider::template(
            template,
            if geographic {
                crate::imagery::Projection::Geographic
            } else {
                crate::imagery::Projection::Mercator
            },
            max,
            credits,
        )
        .map_err(js)?;
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene.set_imagery(&mut r.renderer, p).map_err(js)
    }
    #[wasm_bindgen(js_name=setWorldImagery)]
    pub fn set_world_imagery(&mut self, endpoint: &str, metadata: &str) -> Result<(), JsValue> {
        let e = serde_json::from_str(endpoint).map_err(js)?;
        let m = serde_json::from_str(metadata).map_err(js)?;
        let p = crate::imagery::Provider::from_bing(&e, &m).map_err(js)?;
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene.set_imagery(&mut r.renderer, p).map_err(js)
    }
    #[wasm_bindgen(js_name=cancelRequests)]
    pub fn cancel_requests(&mut self) {
        if let Some(r) = &mut self.running {
            if let Some(layer) = &mut r.scene.imagery {
                layer.cancel_requests();
            }
            if let Some(layer) = &mut r.scene.terrain {
                layer.cancel_requests();
            }
            if let Some(set) = &mut r.scene.tileset {
                for tile in &mut set.tiles {
                    if tile.state == ContentState::Downloading {
                        tile.state = ContentState::Unrequested;
                    }
                }
            }
        }
    }
    #[wasm_bindgen(js_name=takeImageryRequests)]
    pub fn take_imagery_requests(&mut self, max: u32) -> Result<String, JsValue> {
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        let Some(layer) = &mut r.scene.imagery else {
            return Ok("[]".into());
        };
        let a: Vec<_> = layer
            .take_requests(max as usize)
            .into_iter()
            .map(|(t, u)| serde_json::json!({"id":t.key(),"url":u.as_str()}))
            .collect();
        Ok(serde_json::to_string(&a).map_err(js)?)
    }
    #[wasm_bindgen(js_name=submitImagery)]
    pub fn submit_imagery(&mut self, key: &str, bytes: &[u8]) -> Result<(), JsValue> {
        let tile = crate::imagery::Tile::parse(key).ok_or_else(|| js("invalid tile key"))?;
        let image = crate::imagery::decode(bytes).map_err(js)?;
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        let layer = r
            .scene
            .imagery
            .as_mut()
            .ok_or_else(|| js("no imagery layer"))?;
        layer.submit(&mut r.renderer, tile, image).map_err(js)
    }
    #[wasm_bindgen(js_name=failImagery)]
    pub fn fail_imagery(&mut self, key: &str) {
        if let (Some(r), Some(t)) = (&mut self.running, crate::imagery::Tile::parse(key)) {
            if let Some(i) = &mut r.scene.imagery {
                i.fail(t);
            }
        }
    }
    #[wasm_bindgen(getter,js_name=imageryCredits)]
    pub fn imagery_credits(&self) -> String {
        self.running
            .as_ref()
            .map_or(String::new(), |r| r.scene.imagery_credits().into())
    }
    #[wasm_bindgen(js_name=takeRequests)]
    pub fn take_requests(&mut self, max: u32) -> Result<String, JsValue> {
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        if r.renderer.width == 0 || r.renderer.height == 0 {
            return Ok("[]".into());
        }
        let requested = r.scene.select(r.renderer.width, r.renderer.height);
        let mut result = Vec::new();
        if let Some(t) = &mut r.scene.tileset {
            for id in requested.into_iter().take(max.min(4) as usize) {
                if let Some(url) = t.tiles[id].uri.as_ref() {
                    result.push(serde_json::json!({"id":id,"url":url.as_str()}));
                    t.tiles[id].state = ContentState::Downloading;
                }
            }
        }
        Ok(serde_json::Value::Array(result).to_string())
    }
    #[wasm_bindgen(js_name=submitTile)]
    pub fn submit_tile(&mut self, id: u32, bytes: &[u8]) -> Result<(), JsValue> {
        let r = self
            .running
            .as_mut()
            .ok_or_else(|| js("viewer destroyed"))?;
        r.scene.select(r.renderer.width, r.renderer.height);
        if bytes.iter().find(|v| !v.is_ascii_whitespace()) == Some(&b'{') {
            return r
                .scene
                .tileset
                .as_mut()
                .ok_or_else(|| js("no tileset"))?
                .attach_external(id as usize, bytes, &limits())
                .map_err(js);
        }
        if bytes.starts_with(b"pnts") {
            let cloud = cesium_formats::decode_point_cloud(bytes, &limits()).map_err(js)?;
            r.scene
                .commit_point_tile(&mut r.renderer, id as usize, cloud)
                .map_err(js)
        } else {
            let c = decode_content(bytes, &limits()).map_err(js)?;
            r.scene
                .commit_tile(&mut r.renderer, id as usize, c)
                .map_err(js)
        }
    }
    #[wasm_bindgen(js_name=failTile)]
    pub fn fail_tile(&mut self, id: u32) {
        if let Some(r) = &mut self.running {
            if let Some(t) = &mut r.scene.tileset {
                if let Some(tile) = t.tiles.get_mut(id as usize) {
                    tile.state = ContentState::PermanentFailure;
                    r.scene.errors += 1;
                }
            }
        }
    }
    #[wasm_bindgen(getter,js_name=lastPick)]
    pub fn last_pick(&self) -> String {
        self.last_pick.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn stats(&self) -> String {
        self.running
            .as_ref()
            .map(|r| r.scene.stats())
            .unwrap_or_else(|| "destroyed".into())
    }
    pub fn destroy(&mut self) {
        if let Some(r) = self.running.take() {
            r.renderer.device.destroy();
        }
        self.last_pick = "null".into();
    }
}

#[wasm_bindgen(js_name=bingMetadataUrl)]
pub fn bing_metadata_url(endpoint: &str) -> Result<String, JsValue> {
    let e = serde_json::from_str(endpoint).map_err(js)?;
    crate::imagery::Provider::bing_metadata_url(&e)
        .map(|u| u.to_string())
        .map_err(js)
}
