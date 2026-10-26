use crate::{limits, Scene};
use cesium_formats::{decode_content, Content};
use cesium_renderer::{device, instance, RenderError, Renderer};
use cesium_streaming::{Budget, Epochs, TaskToken};
use cesium_tiles::{ContentState, Tileset};
use std::{
    io::Read,
    path::Path,
    sync::{mpsc, Arc},
    time::Duration,
};
use url::Url;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub fn source_url(value: &str) -> Result<Url> {
    if let Ok(url) = Url::parse(value) {
        if matches!(url.scheme(), "http" | "https" | "file") {
            return Ok(url);
        }
    }
    let path = std::fs::canonicalize(Path::new(value))?;
    Url::from_file_path(path).map_err(|_| "invalid file path".into())
}
fn client() -> Result<reqwest::blocking::Client> {
    Ok(reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()?)
}
/// Bounded reading, including chunked HTTP responses; no credential propagation or redirects.
pub fn fetch(url: &Url, client: &reqwest::blocking::Client, max: usize) -> Result<Vec<u8>> {
    let mut reader: Box<dyn Read> = match url.scheme() {
        "file" => Box::new(std::fs::File::open(
            url.to_file_path().map_err(|_| "file URL")?,
        )?),
        "http" | "https" => {
            let response = client.get(url.clone()).send()?.error_for_status()?;
            if response.status().is_redirection() {
                return Err("redirects need an explicit resource policy".into());
            }
            if response.content_length().is_some_and(|n| n > max as u64) {
                return Err("download size limit".into());
            }
            Box::new(response)
        }
        _ => return Err("unsupported URL scheme".into()),
    };
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take(max as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > max {
        return Err("download size limit".into());
    }
    Ok(bytes)
}
enum Downloaded {
    Model(Content),
    External(Vec<u8>),
}
struct Completed {
    id: usize,
    token: TaskToken,
    download: u64,
    cpu: u64,
    result: std::result::Result<Downloaded, String>,
}
struct Streamer {
    client: reqwest::blocking::Client,
    tx: mpsc::SyncSender<Completed>,
    rx: mpsc::Receiver<Completed>,
    download: Budget,
    cpu: Budget,
    epochs: Epochs,
    inflight: usize,
}
impl Streamer {
    fn new() -> Result<Self> {
        let (tx, rx) = mpsc::sync_channel(4);
        Ok(Self {
            client: client()?,
            tx,
            rx,
            download: Budget::new(256 * 1024 * 1024),
            cpu: Budget::new(256 * 1024 * 1024),
            epochs: Epochs::default(),
            inflight: 0,
        })
    }
    fn dispatch(&mut self, scene: &mut Scene, requests: Vec<usize>) {
        let Some(set) = &mut scene.tileset else {
            return;
        };
        for id in requests {
            if self.inflight >= 4 {
                break;
            }
            let t = &mut set.tiles[id];
            if t.state != ContentState::Unrequested {
                continue;
            }
            let Some(url) = t.uri.clone() else {
                continue;
            };
            let Ok(download) = self.download.reserve(limits().max_input_bytes) else {
                break;
            };
            let Ok(cpu) = self.cpu.reserve(limits().max_decoded_bytes) else {
                let _ = self.download.release(download);
                break;
            };
            t.state = ContentState::Downloading;
            self.inflight += 1;
            let tx = self.tx.clone();
            let client = self.client.clone();
            let token = self.epochs.token();
            std::thread::spawn(move || {
                let result = fetch(&url, &client, limits().max_input_bytes)
                    .and_then(|b| {
                        if b.iter().find(|v| !v.is_ascii_whitespace()) == Some(&b'{') {
                            Ok(Downloaded::External(b))
                        } else {
                            Ok(Downloaded::Model(decode_content(&b, &limits())?))
                        }
                    })
                    .map_err(|e| e.to_string());
                let _ = tx.send(Completed {
                    id,
                    token,
                    download,
                    cpu,
                    result,
                });
            });
        }
    }
    /// At most one bounded content upload per frame; large uploads are not yet segmented.
    fn apply(&mut self, scene: &mut Scene, renderer: &mut Renderer) {
        if let Ok(c) = self.rx.try_recv() {
            self.inflight -= 1;
            let _ = self.download.release(c.download);
            if self.epochs.accepts(c.token) {
                let result = c.result.and_then(|content| match content {
                    Downloaded::Model(content) => scene
                        .commit_tile(renderer, c.id, content)
                        .map_err(|e| e.to_string()),
                    Downloaded::External(data) => scene
                        .tileset
                        .as_mut()
                        .ok_or_else(|| "no tileset".to_string())
                        .and_then(|set| {
                            set.attach_external(c.id, &data, &limits())
                                .map_err(|e| e.to_string())
                        }),
                });
                if let Err(error) = result {
                    scene.errors += 1;
                    if let Some(set) = &mut scene.tileset {
                        set.tiles[c.id].state = ContentState::PermanentFailure;
                    }
                    // Error text contains parser messages only in this path; URLs are never logged.
                    eprintln!("content {} failed: {}", c.id, redact_error(&error));
                }
            }
            let _ = self.cpu.release(c.cpu);
        }
    }
}
fn redact_error(error: &str) -> String {
    if error.contains("http://") || error.contains("https://") {
        "resource fetch failed (URL redacted)".into()
    } else {
        error.into()
    }
}
impl Drop for Streamer {
    fn drop(&mut self) {
        self.epochs.unload();
    }
}

struct ImageryCompleted {
    tile: crate::imagery::Tile,
    result: std::result::Result<image::RgbaImage, String>,
}
struct ImageryStreamer {
    client: reqwest::blocking::Client,
    provider: mpsc::Receiver<std::result::Result<crate::imagery::Provider, String>>,
    tx: mpsc::SyncSender<ImageryCompleted>,
    rx: mpsc::Receiver<ImageryCompleted>,
    inflight: usize,
}
impl ImageryStreamer {
    fn new() -> Result<Self> {
        Self::configured(None)
    }
    fn configured(initial: Option<crate::imagery::Provider>) -> Result<Self> {
        let client = client()?;
        let (ptx, provider) = mpsc::sync_channel(1);
        let (tx, rx) = mpsc::sync_channel(4);
        let http = client.clone();
        std::thread::spawn(move || {
            use crate::imagery::{Projection, Provider};
            let result = (|| -> std::result::Result<Provider, String> {
                if let Some(provider) = initial {
                    return Ok(provider);
                }
                if let Ok(template) = std::env::var("CESIUM_IMAGERY_URL") {
                    let projection = if std::env::var("CESIUM_IMAGERY_PROJECTION").as_deref()
                        == Ok("geographic")
                    {
                        Projection::Geographic
                    } else {
                        Projection::Mercator
                    };
                    let max = std::env::var("CESIUM_IMAGERY_MAX_LEVEL")
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(if projection == Projection::Geographic {
                            2
                        } else {
                            18
                        });
                    return Provider::template(
                        &template,
                        projection,
                        max,
                        "Configured imagery source",
                    );
                }
                if let Ok(token) = std::env::var("CESIUM_ION_TOKEN") {
                    let url = Provider::ion_url(&token)?;
                    let data = fetch(&url, &http, 1024 * 1024)
                        .map_err(|_| "ion endpoint request failed (check token/network)")?;
                    let endpoint =
                        serde_json::from_slice(&data).map_err(|_| "invalid ion endpoint")?;
                    let url = Provider::bing_metadata_url(&endpoint)?;
                    let data = fetch(&url, &http, 1024 * 1024)
                        .map_err(|_| "Bing metadata request failed")?;
                    let metadata =
                        serde_json::from_slice(&data).map_err(|_| "invalid Bing metadata")?;
                    return Provider::from_bing(&endpoint, &metadata);
                }
                Ok(Provider::natural_earth())
            })();
            let _ = ptx.send(result);
        });
        Ok(Self {
            client,
            provider,
            tx,
            rx,
            inflight: 0,
        })
    }
    fn frame(
        &mut self,
        scene: &mut Scene,
        renderer: &mut Renderer,
    ) -> std::result::Result<(), RenderError> {
        if let Ok(result) = self.provider.try_recv() {
            match result {
                Ok(provider) => {
                    println!("Imagery credits: {}", provider.credits);
                    scene.set_imagery(renderer, provider)?;
                }
                Err(error) => {
                    scene.errors += 1;
                    eprintln!("Imagery initialization: {error}");
                }
            }
        }
        if let Ok(done) = self.rx.try_recv() {
            self.inflight -= 1;
            if let Some(layer) = &mut scene.imagery {
                match done.result {
                    Ok(image) => {
                        if layer.submit(renderer, done.tile, image).is_err() {
                            layer.fail(done.tile);
                        }
                    }
                    Err(_) => layer.fail(done.tile),
                }
            }
        }
        scene.update_imagery(renderer)?;
        if let Some(layer) = &mut scene.imagery {
            for (tile, url) in layer.take_requests(4 - self.inflight) {
                self.inflight += 1;
                let http = self.client.clone();
                let tx = self.tx.clone();
                std::thread::spawn(move || {
                    let result = fetch(&url, &http, crate::imagery::MAX_IMAGE_BYTES)
                        .map_err(|_| "imagery request failed".into())
                        .and_then(|b| crate::imagery::decode(&b));
                    let _ = tx.send(ImageryCompleted { tile, result });
                });
            }
        }
        Ok(())
    }
}

struct TerrainCompleted {
    tile: crate::imagery::Tile,
    result: std::result::Result<crate::terrain::Data, String>,
}
struct TerrainStreamer {
    client: reqwest::blocking::Client,
    provider: mpsc::Receiver<std::result::Result<crate::terrain::Provider, String>>,
    tx: mpsc::SyncSender<TerrainCompleted>,
    rx: mpsc::Receiver<TerrainCompleted>,
    inflight: usize,
}
/// Authentication stays on the approved provider origin; failures never log tokens or URLs.
pub fn fetch_terrain(
    url: &Url,
    client: &reqwest::blocking::Client,
    max: usize,
    accept: &str,
    authorization: Option<&str>,
) -> Result<Vec<u8>> {
    if url.scheme() == "file" {
        return fetch(url, client, max);
    }
    if !matches!(url.scheme(), "http" | "https") {
        return Err("terrain URL scheme".into());
    }
    let mut request = client.get(url.clone()).header("Accept", accept);
    if let Some(value) = authorization {
        request = request.header("Authorization", value);
    }
    let mut response = request.send()?.error_for_status()?;
    if response.status().is_redirection() {
        return Err("terrain redirects are not supported".into());
    }
    if response.content_length().is_some_and(|n| n > max as u64) {
        return Err("terrain response byte limit".into());
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut response)
        .take(max as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > max {
        return Err("terrain response byte limit".into());
    }
    Ok(bytes)
}
impl TerrainStreamer {
    fn new() -> Result<Self> {
        let client = client()?;
        let (ptx, provider) = mpsc::sync_channel(1);
        let (tx, rx) = mpsc::sync_channel(4);
        let http = client.clone();
        std::thread::spawn(move || {
            let result = (|| -> std::result::Result<crate::terrain::Provider, String> {
                use crate::terrain::Provider;
                if let Ok(input) = std::env::var("CESIUM_TERRAIN_URL") {
                    if input.contains("{z}") {
                        return Provider::terrarium(&input, 15);
                    }
                    let url = source_url(&input).map_err(|_| "terrain layer URL/path")?;
                    let bytes = fetch_terrain(&url, &http, 1024 * 1024, "application/json", None)
                        .map_err(|_| "terrain layer request failed")?;
                    return Provider::layer(&bytes, url);
                }
                if let Ok(token) = std::env::var("CESIUM_ION_TOKEN") {
                    let url = Provider::ion_endpoint(&token)?;
                    let bytes = fetch(&url, &http, 1024 * 1024)
                        .map_err(|_| "ion terrain endpoint request failed")?;
                    let endpoint: serde_json::Value =
                        serde_json::from_slice(&bytes).map_err(|_| "ion terrain endpoint JSON")?;
                    let (url, authorization) = Provider::ion_layer(&endpoint)?;
                    let bytes = fetch_terrain(
                        &url,
                        &http,
                        1024 * 1024,
                        "application/json",
                        Some(&authorization),
                    )
                    .map_err(|_| "ion terrain layer request failed")?;
                    return Provider::layer(&bytes, url)?.with_authorization(authorization);
                }
                Ok(Provider::public_terrain())
            })();
            let _ = ptx.send(result);
        });
        Ok(Self {
            client,
            provider,
            tx,
            rx,
            inflight: 0,
        })
    }
    fn frame(
        &mut self,
        scene: &mut Scene,
        renderer: &mut Renderer,
    ) -> std::result::Result<(), RenderError> {
        if let Ok(result) = self.provider.try_recv() {
            match result {
                Ok(provider) => {
                    println!("Terrain credits: {}", provider.credits);
                    scene.set_terrain(renderer, provider)?;
                }
                Err(error) => {
                    scene.errors += 1;
                    eprintln!("Terrain initialization: {error}");
                }
            }
        }
        if let Ok(done) = self.rx.try_recv() {
            self.inflight = self.inflight.saturating_sub(1);
            if let Some(layer) = &mut scene.terrain {
                match done.result {
                    Ok(data) => {
                        if let Err(error) =
                            layer.submit(renderer, done.tile, data, scene.imagery.as_ref())
                        {
                            eprintln!("Terrain decode/upload: {error}");
                            layer.fail(done.tile);
                        }
                    }
                    Err(error) => {
                        eprintln!("Terrain request/decode: {error}");
                        layer.fail(done.tile);
                    }
                }
            }
        }
        scene.update_terrain(renderer)?;
        if let Some(layer) = &mut scene.terrain {
            let format = layer.provider.format;
            let auth = layer.provider.authorization().map(str::to_owned);
            let accept = layer.provider.accept().to_owned();
            for (tile, url) in layer.take_requests(4 - self.inflight) {
                self.inflight += 1;
                let http = self.client.clone();
                let tx = self.tx.clone();
                let auth = auth.clone();
                let accept = accept.clone();
                std::thread::spawn(move || {
                    let result = fetch_terrain(
                        &url,
                        &http,
                        crate::terrain::MAX_TILE_BYTES,
                        &accept,
                        auth.as_deref(),
                    )
                    .map_err(|_| "terrain request failed".into())
                    .and_then(|bytes| crate::terrain::decode(&bytes, format));
                    let _ = tx.send(TerrainCompleted { tile, result });
                });
            }
        }
        Ok(())
    }
}

pub enum Startup {
    Globe,
    Demo,
    Tileset(Tileset),
    Static(Content),
    GeoJson(crate::geojson::Data),
    WithGeoJson(Box<Startup>, Vec<crate::geojson::Data>),
}
struct Running {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
    scene: Scene,
    streamer: Streamer,
    imagery_streamer: Option<ImageryStreamer>,
    terrain_streamer: Option<TerrainStreamer>,
    cursor: (f64, f64),
    pressed: bool,
    drag: f64,
    pick: Option<(u32, u32)>,
    last_frame: std::time::Instant,
    frames: u32,
    smoke_frames: Option<u32>,
}
struct App {
    startup: Option<Startup>,
    running: Option<Running>,
    smoke_frames: Option<u32>,
    failure: Option<String>,
}
impl Running {
    async fn new(window: Arc<Window>, startup: Startup, smoke_frames: Option<u32>) -> Result<Self> {
        let size = window.inner_size();
        let instance = instance();
        let surface = instance.create_surface(window.clone())?;
        let (adapter, device, queue) = device(&instance, Some(&surface)).await?;
        let caps = surface.get_capabilities(&adapter);
        let (base_format, format) = cesium_renderer::surface_formats(&caps)?;
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or("surface configuration unavailable")?;
        config.format = base_format;
        if base_format != format {
            config.view_formats = vec![format];
        }
        config.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &config);
        let mut renderer = Renderer::new(device, queue, format, size.width, size.height)?;
        let mut startup = startup;
        let mut overlays = Vec::new();
        let mut depth = 0;
        loop {
            match startup {
                Startup::WithGeoJson(base, layers) => {
                    depth += 1;
                    if depth > 32 || overlays.len() + layers.len() > 32 {
                        return Err("too many GeoJSON overlays".into());
                    }
                    overlays.extend(layers);
                    startup = *base;
                }
                other => {
                    startup = other;
                    break;
                }
            }
        }
        let (startup, geojson) = match startup {
            Startup::GeoJson(data) => (Startup::Globe, Some(data)),
            other => (other, None),
        };
        let (set, content, demo) = match startup {
            Startup::Globe => (None, None, false),
            Startup::Demo => {
                let set = Tileset::parse(
                    include_bytes!("../../../tests/fixtures/tileset.json"),
                    Url::parse("https://fixtures.invalid/tileset.json")?,
                    &limits(),
                )?;
                (Some(set), None, true)
            }
            Startup::Tileset(t) => (Some(t), None, false),
            Startup::Static(c) => (None, Some(c), false),
            Startup::GeoJson(_) | Startup::WithGeoJson(_, _) => unreachable!(),
        };
        let mut scene = Scene::new(&mut renderer, set)?;
        if let Some(c) = content {
            scene.add_static(&mut renderer, c)?;
        }
        if demo {
            scene.commit_tile(
                &mut renderer,
                0,
                decode_content(
                    include_bytes!("../../../tests/fixtures/triangle.b3dm"),
                    &limits(),
                )?,
            )?;
        }
        if let Some(data) = geojson {
            let id = scene.add_geojson(&mut renderer, data)?;
            scene.focus_geojson(id);
        }
        for data in overlays {
            scene.add_geojson(&mut renderer, data)?;
        }
        let mut offline_provider = None;
        let variants_path = std::env::var("CESIUM_VARIANTS").ok();
        let entity_path = std::env::var("CESIUM_ENTITIES").ok();
        let czml_path = std::env::var("CESIUM_CZML").ok();
        let primitive_path = std::env::var("CESIUM_PRIMITIVES").ok();
        let groups_path = std::env::var("CESIUM_GROUPS").ok();
        let scene_path = std::env::var("CESIUM_SCENE").ok();
        if usize::from(entity_path.is_some())
            + usize::from(czml_path.is_some())
            + usize::from(primitive_path.is_some())
            + usize::from(groups_path.is_some())
            + usize::from(scene_path.is_some())
            + usize::from(variants_path.is_some())
            > 1
        {
            return Err("choose CESIUM_ENTITIES, CESIUM_CZML or CESIUM_PRIMITIVES".into());
        }
        if let Some(path) = entity_path
            .as_ref()
            .or(czml_path.as_ref())
            .or(primitive_path.as_ref())
            .or(groups_path.as_ref())
            .or(scene_path.as_ref())
            .or(variants_path.as_ref())
        {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(8 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes)?;
            let base = source_url(path)?;
            let image_client = client()?;
            let bytes = if groups_path.is_some() || scene_path.is_some() || variants_path.is_some()
            {
                bytes
            } else {
                crate::entity_resources::hydrate(&bytes, &base, |url, max| {
                    fetch(url, &image_client, max).map_err(|e| e.to_string())
                })?
            };
            if scene_path.is_some() {
                if matches!(
                    crate::scene_recipes::decode(&bytes)?,
                    crate::scene_recipes::SceneRecipe::Offline { .. }
                ) {
                    offline_provider = Some(crate::scene_recipes::offline_provider()?);
                }
                scene.clear_entities();
            } else if variants_path.is_some() {
                let index = std::env::var("CESIUM_VARIANT_INDEX")
                    .ok()
                    .map(|v| v.parse::<usize>())
                    .transpose()?
                    .unwrap_or(0);
                scene.set_entity_variant(&mut renderer, &bytes, index)?;
            } else if groups_path.is_some() {
                scene.set_grouped_entities(&mut renderer, &bytes)?;
            } else if primitive_path.is_some() {
                scene.set_primitive_instances(&mut renderer, &bytes)?;
                scene.set_view(crate::camera_control::ViewOptions {
                    destination_degrees: [-100., 40., 17500000.],
                    heading: 0.,
                    pitch: -std::f64::consts::FRAC_PI_2,
                    roll: 0.,
                })?;
            } else if czml_path.is_some() {
                let (entities, mut clock) = crate::entities::czml(&bytes)?;
                scene.set_entities(&mut renderer, entities)?;
                clock.should_animate = true;
                scene.clock = clock;
                scene.tick(0.);
            } else {
                scene.set_entities(&mut renderer, crate::entities::decode(&bytes)?)?;
            }
            if let Ok(id) = std::env::var("CESIUM_ENTITY_FOCUS") {
                if !scene.focus_entity(&id, 100000.) {
                    return Err("entity focus unavailable at current time".into());
                }
            }
        }
        if let Ok(path) = std::env::var("CESIUM_CAMERA") {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(16385)
                .read_to_end(&mut bytes)?;
            if bytes.len() > 16384 {
                return Err("camera command byte limit".into());
            }
            let commands: Vec<crate::camera_control::Command> = serde_json::from_slice(&bytes)?;
            if commands.len() > 16 {
                return Err("camera command count limit".into());
            }
            for command in commands {
                scene.camera_command(command)?;
            }
        }
        let imagery_streamer = if let Some(provider) = offline_provider {
            Some(ImageryStreamer::configured(Some(provider))?)
        } else if scene.globe.is_some() && std::env::var("CESIUM_OFFLINE").as_deref() != Ok("1") {
            Some(ImageryStreamer::new()?)
        } else {
            None
        };
        let terrain_streamer = if scene.globe.is_some()
            && std::env::var("CESIUM_TERRAIN").as_deref() != Ok("0")
            && (std::env::var("CESIUM_OFFLINE").as_deref() != Ok("1")
                || std::env::var("CESIUM_TERRAIN_URL").is_ok())
        {
            Some(TerrainStreamer::new()?)
        } else {
            None
        };
        if let Ok(location) = std::env::var("CESIUM_VIEW_LOCATION") {
            let values: Vec<f64> = location
                .split(',')
                .map(str::parse)
                .collect::<std::result::Result<_, _>>()?;
            if values.len() != 3 {
                return Err("CESIUM_VIEW_LOCATION requires lon,lat,height".into());
            }
            scene.focus_location(values[0], values[1], values[2])?;
        }
        if std::env::var("CESIUM_ZOOM_ENTITIES").as_deref() == Ok("1") {
            scene.zoom_to_entities(&[], renderer.width, renderer.height)?;
        }
        Ok(Self {
            window,
            surface,
            config,
            renderer,
            scene,
            streamer: Streamer::new()?,
            imagery_streamer,
            terrain_streamer,
            cursor: (0.0, 0.0),
            pressed: false,
            drag: 0.0,
            pick: None,
            last_frame: std::time::Instant::now(),
            frames: 0,
            smoke_frames,
        })
    }
    fn resize(&mut self, w: u32, h: u32) -> std::result::Result<(), RenderError> {
        self.renderer.resize(w, h)?;
        self.pick = None;
        if w > 0 && h > 0 {
            self.config.width = w;
            self.config.height = h;
            self.surface.configure(&self.renderer.device, &self.config);
        }
        Ok(())
    }
    fn frame(&mut self) -> std::result::Result<(), RenderError> {
        let now = std::time::Instant::now();
        let before = self.scene.orbit.camera(1, 1);
        self.scene
            .tick((now - self.last_frame).as_secs_f64().min(1.));
        let after = self.scene.orbit.camera(1, 1);
        if before.eye != after.eye || before.camera_to_ecef != after.camera_to_ecef {
            self.renderer.invalidate_pick();
        }
        self.last_frame = now;
        if let Some(feature) = self.renderer.poll_pick() {
            self.renderer.selected = feature;
            if let Some(f) = feature {
                if let Some(properties) = self.scene.properties(f) {
                    println!("picked feature {}: {}", f.feature, properties);
                }
            }
        }
        if self.renderer.width == 0 || self.renderer.height == 0 {
            return Ok(());
        }
        self.scene.select(self.renderer.width, self.renderer.height); // Pin current coverage before eviction.
        self.streamer.apply(&mut self.scene, &mut self.renderer);
        let requests = self.scene.select(self.renderer.width, self.renderer.height);
        self.streamer.dispatch(&mut self.scene, requests);
        if let Some(streamer) = &mut self.imagery_streamer {
            streamer.frame(&mut self.scene, &mut self.renderer)?;
        }
        if let Some(streamer) = &mut self.terrain_streamer {
            streamer.frame(&mut self.scene, &mut self.renderer)?;
        }
        let texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.renderer.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(())
            }
            other => return Err(RenderError(format!("surface acquisition: {other:?}"))),
        };
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.renderer.format()),
            ..Default::default()
        });
        self.scene
            .render(&mut self.renderer, &view, self.pick.take())?;
        self.renderer.queue.present(texture);
        self.frames += 1;
        self.window
            .set_title(&format!("cesium-rs | {}", self.scene.stats()));
        Ok(())
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.running.is_some() {
            return;
        }
        let result = (|| -> Result<Running> {
            let window = Arc::new(
                event_loop.create_window(
                    Window::default_attributes()
                        .with_title("cesium-rs")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280, 800)),
                )?,
            );
            pollster::block_on(Running::new(
                window,
                self.startup.take().ok_or("startup already consumed")?,
                self.smoke_frames,
            ))
        })();
        match result {
            Ok(r) => {
                r.window.request_redraw();
                self.running = Some(r);
            }
            Err(e) => {
                self.failure = Some(e.to_string());
                event_loop.exit();
            }
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(r) = &mut self.running else {
            return;
        };
        if id != r.window.id() {
            return;
        }
        let mut error = None;
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => error = r.resize(size.width, size.height).err(),
            WindowEvent::CursorMoved { position, .. } => {
                if r.pressed {
                    let dx = position.x - r.cursor.0;
                    let dy = position.y - r.cursor.1;
                    r.drag += dx.abs() + dy.abs();
                    r.scene.camera_drag(dx, dy);
                    r.renderer.invalidate_pick();
                }
                r.cursor = (position.x, position.y);
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if state == ElementState::Pressed {
                    r.pressed = true;
                    r.drag = 0.0;
                } else {
                    r.pressed = false;
                    if r.drag < 4.0 {
                        r.pick = Some((r.cursor.0.max(0.0) as u32, r.cursor.1.max(0.0) as u32));
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y as f64,
                    MouseScrollDelta::PixelDelta(p) => p.y * 0.02,
                };
                r.scene.camera_zoom(d);
                r.renderer.invalidate_pick();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let winit::keyboard::Key::Character(key) = &event.logical_key {
                        match key.as_str() {
                            "+" | "=" => {
                                r.scene.camera_zoom(2.0);
                                r.renderer.invalidate_pick();
                            }
                            "-" | "_" => {
                                r.scene.camera_zoom(-2.0);
                                r.renderer.invalidate_pick();
                            }
                            "r" | "R" => r.scene.clock.multiplier = -r.scene.clock.multiplier,
                            "c" | "C" => {
                                r.scene.cancel_flight();
                            }
                            "u" | "U" => {
                                r.scene.stop_tracking();
                            }
                            _ => {}
                        }
                    }
                }
                if event.state == ElementState::Pressed
                    && event.logical_key
                        == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space)
                {
                    r.scene.clock.should_animate = !r.scene.clock.should_animate;
                }
                if event.state == ElementState::Pressed
                    && event.logical_key
                        == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape)
                {
                    event_loop.exit();
                } else if event.state == ElementState::Pressed
                    && event.logical_key
                        == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Home)
                {
                    r.scene.focus_globe();
                }
            }
            WindowEvent::RedrawRequested => {
                error = r.frame().err();
                if r.smoke_frames.is_some_and(|n| r.frames >= n) {
                    println!("native smoke: {} frames; {}", r.frames, r.scene.stats());
                    println!("camera state: {}", r.scene.camera_state());
                    event_loop.exit();
                }
            }
            _ => {}
        }
        if let Some(e) = error {
            self.failure = Some(e.to_string());
            event_loop.exit();
        }
    }
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(r) = &self.running {
            r.window.request_redraw();
        }
    }
}
pub fn run(startup: Startup, smoke_frames: Option<u32>) -> Result<()> {
    let mut app = App {
        startup: Some(startup),
        running: None,
        smoke_frames,
        failure: None,
    };
    EventLoop::new()?.run_app(&mut app)?;
    if let Some(error) = app.failure {
        return Err(error.into());
    }
    Ok(())
}
pub fn load_geojson(url: Url) -> Result<crate::geojson::Data> {
    let data = fetch(
        &url,
        &client()?,
        crate::geojson::Limits::default().max_input_bytes,
    )?;
    Ok(crate::geojson::decode(
        &data,
        &Default::default(),
        &Default::default(),
    )?)
}
pub fn load(url: Url) -> Result<Startup> {
    let data = fetch(&url, &client()?, limits().max_input_bytes)?;
    if data.starts_with(b"glTF") || data.starts_with(b"b3dm") {
        Ok(Startup::Static(decode_content(&data, &limits())?))
    } else {
        let json: serde_json::Value = serde_json::from_slice(&data)?;
        if json.get("type").is_some() {
            return Ok(Startup::GeoJson(crate::geojson::decode(
                &data,
                &Default::default(),
                &Default::default(),
            )?));
        }
        Ok(Startup::Tileset(Tileset::parse(&data, url, &limits())?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_file_fetch() {
        let url = source_url(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/triangle.glb"
        ))
        .unwrap();
        assert!(fetch(&url, &client().unwrap(), 10).is_err());
        let bytes = fetch(&url, &client().unwrap(), 65536).unwrap();
        assert!(bytes.starts_with(b"glTF"));
    }
    #[test]
    fn scene_normal_and_orbit() {
        let n =
            crate::normal_matrix(glam::DMat4::from_scale(glam::DVec3::new(2.0, 3.0, 4.0))).unwrap();
        assert_eq!(n.x_axis.x, 0.5);
        let mut o = crate::Orbit::new(glam::DVec3::new(6_378_137.0, 0.0, 0.0), 60.0);
        o.drag(1.0, 1.0);
        o.zoom(2.0);
        assert!(o.camera(800, 600).eye.is_finite());
    }
}
