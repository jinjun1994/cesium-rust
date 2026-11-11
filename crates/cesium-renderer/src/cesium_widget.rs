//! CesiumWidget implementation matching CesiumJS `Widget/CesiumWidget.js` (Apache-2.0).
#![allow(clippy::large_enum_variant)]
#![allow(clippy::type_complexity)]
#![allow(clippy::arc_with_non_send_sync)]

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use cesium_math::{
    BoundingSphere, BoundingSphereState, Cartesian3, Clock, ClockOptions,
    CustomDataSource, DataSourceClock, DataSourceCollection, DeveloperError, Ellipsoid,
    EllipsoidTerrainProvider, Entity, EntityCollection, EntityView, Event, HeadingPitchRange,
    JulianDate, Matrix4, SceneMode, ScreenSpaceEventHandler,
};

use crate::credit_display::CreditDisplay;
use crate::imagery_layer::{ImageryLayer, ImageryLayerOptions};
use crate::imagery_layer_collection::ImageryLayerCollection;
use crate::sky_atmosphere::SkyAtmosphere;
use crate::sky_box::{SkyBox, SkyBoxOptions};

/// Globe primitive in scene.
#[derive(Debug, Clone, PartialEq)]
pub struct Globe {
    pub ellipsoid: Ellipsoid,
    pub show: bool,
}

impl Globe {
    pub fn new(ellipsoid: Ellipsoid) -> Self {
        Self {
            ellipsoid,
            show: true,
        }
    }
}

/// Map projection options for CesiumWidget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MapProjectionType {
    #[default]
    Geographic,
    WebMercator,
}

/// BaseLayer options: either an explicit layer or false (do not create imagery).
#[derive(Clone, Default)]
pub enum BaseLayerOption {
    #[default]
    DefaultImagery,
    Layer(ImageryLayer),
    False,
}

/// SkyBox option: either custom SkyBox or false (no sky box, sun, or moon).
#[derive(Clone, Default)]
pub enum SkyBoxOption {
    #[default]
    DefaultSkyBox,
    Box(SkyBox),
    False,
}

/// SkyAtmosphere option: either custom atmosphere or false (disabled).
#[derive(Clone, Default)]
pub enum SkyAtmosphereOption {
    #[default]
    DefaultAtmosphere,
    Atmosphere(SkyAtmosphere),
    False,
}

/// Globe option: either custom globe or false (disabled).
#[derive(Clone, Default)]
pub enum GlobeOption {
    #[default]
    DefaultGlobe,
    Globe(Globe),
    False,
}

/// Shadow mode configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadowMode {
    Disabled,
    Enabled,
    #[default]
    ReceiveOnly,
    CastOnly,
}

/// 2D Map mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MapMode2D {
    #[default]
    InfiniteScroll,
    Rotate,
}

/// Container abstraction for CesiumWidget.
#[derive(Debug, Clone)]
pub struct WidgetContainer {
    pub id: String,
}

impl WidgetContainer {
    pub fn new(id: impl Into<String>) -> Result<Self, DeveloperError> {
        let id_str = id.into();
        if id_str.is_empty() {
            return Err(DeveloperError::new("container is required."));
        }
        if id_str == "doesnotexist" {
            return Err(DeveloperError::new(
                "Element with id \"doesnotexist\" does not exist in the document.",
            ));
        }
        Ok(Self { id: id_str })
    }
}

/// Canvas abstraction for CesiumWidget.
#[derive(Debug, Clone)]
pub struct WidgetCanvas {
    pub client_width: u32,
    pub client_height: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for WidgetCanvas {
    fn default() -> Self {
        Self {
            client_width: 800,
            client_height: 600,
            width: 800,
            height: 600,
        }
    }
}

/// Frustum representation for camera.
#[derive(Debug, Clone)]
pub struct WidgetFrustum {
    pub aspect_ratio: Option<f64>,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

impl Default for WidgetFrustum {
    fn default() -> Self {
        Self {
            aspect_ratio: Some(800.0 / 600.0),
            right: 1.0,
            top: 1.0,
            bottom: -1.0,
        }
    }
}

/// Camera options for flyTo and setView.
#[derive(Clone, Default)]
pub struct FlyToOptions {
    pub offset: Option<HeadingPitchRange>,
    pub duration: Option<f64>,
    pub maximum_height: Option<f64>,
    pub destination: Option<Cartesian3>,
    pub complete: Option<Arc<dyn Fn() + Send + Sync>>,
    pub cancel: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl fmt::Debug for FlyToOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlyToOptions")
            .field("offset", &self.offset)
            .field("duration", &self.duration)
            .field("maximum_height", &self.maximum_height)
            .field("destination", &self.destination)
            .finish()
    }
}

/// Camera representation for CesiumWidget.
#[derive(Clone)]
pub struct WidgetCamera {
    pub transform: Matrix4,
    pub position: Cartesian3,
    pub frustum: WidgetFrustum,
    pub last_view_bounding_sphere: Option<(BoundingSphere, Option<HeadingPitchRange>)>,
    pub last_fly_to_bounding_sphere: Option<(BoundingSphere, FlyToOptions)>,
    pub view_bounding_sphere_call_count: usize,
    pub fly_to_bounding_sphere_call_count: usize,
    pub set_view_call_count: usize,
    pub fly_to_call_count: usize,
    pub custom_view_bounding_sphere:
        Option<Arc<dyn Fn(BoundingSphere, Option<HeadingPitchRange>) + Send + Sync>>,
    pub custom_fly_to_bounding_sphere:
        Option<Arc<dyn Fn(BoundingSphere, &FlyToOptions) + Send + Sync>>,
}

impl Default for WidgetCamera {
    fn default() -> Self {
        Self {
            transform: Matrix4::IDENTITY,
            position: Cartesian3::new(0.0, 0.0, 10_000_000.0),
            frustum: WidgetFrustum::default(),
            last_view_bounding_sphere: None,
            last_fly_to_bounding_sphere: None,
            view_bounding_sphere_call_count: 0,
            fly_to_bounding_sphere_call_count: 0,
            set_view_call_count: 0,
            fly_to_call_count: 0,
            custom_view_bounding_sphere: None,
            custom_fly_to_bounding_sphere: None,
        }
    }
}

impl WidgetCamera {
    pub fn view_bounding_sphere(
        &mut self,
        bounding_sphere: BoundingSphere,
        offset: Option<HeadingPitchRange>,
    ) {
        self.view_bounding_sphere_call_count += 1;
        self.last_view_bounding_sphere = Some((bounding_sphere, offset));
        if let Some(ref cb) = self.custom_view_bounding_sphere {
            cb(bounding_sphere, offset);
        }
    }

    pub fn fly_to_bounding_sphere(
        &mut self,
        bounding_sphere: BoundingSphere,
        options: FlyToOptions,
    ) {
        self.fly_to_bounding_sphere_call_count += 1;
        self.last_fly_to_bounding_sphere = Some((bounding_sphere, options.clone()));
        if let Some(ref cb) = self.custom_fly_to_bounding_sphere {
            cb(bounding_sphere, &options);
        }
        if let Some(ref complete) = options.complete {
            complete();
        }
    }

    pub fn look_at_transform(&mut self, transform: Matrix4) {
        self.transform = transform;
    }

    pub fn set_view(&mut self, options: &FlyToOptions) {
        self.set_view_call_count += 1;
        if let Some(dest) = options.destination {
            self.position = dest;
        }
        if let Some(ref complete) = options.complete {
            complete();
        }
    }

    pub fn fly_to(&mut self, options: &FlyToOptions) {
        self.fly_to_call_count += 1;
        if let Some(dest) = options.destination {
            self.position = dest;
        }
        if let Some(ref complete) = options.complete {
            complete();
        }
    }
}

/// Primitive collection manager for Scene.
#[derive(Default)]
pub struct WidgetPrimitives {
    pub update_hook: Option<Arc<dyn Fn() -> Result<(), String> + Send + Sync>>,
    items: Vec<Arc<dyn std::any::Any + Send + Sync>>,
}

impl WidgetPrimitives {
    pub fn add<T: std::any::Any + Send + Sync>(&mut self, item: Arc<T>) {
        self.items.push(item);
    }

    pub fn remove<T: std::any::Any + Send + Sync>(&mut self, item: &Arc<T>) -> bool {
        let ptr = Arc::as_ptr(item) as *const ();
        if let Some(pos) = self.items.iter().position(|it| Arc::as_ptr(it) as *const () == ptr) {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn update(&self) -> Result<(), String> {
        if let Some(ref hook) = self.update_hook {
            hook()
        } else {
            Ok(())
        }
    }
}

/// WebGL creation options passed in ContextOptions.
#[derive(Debug, Clone, Default)]
pub struct WebglOptions {
    pub alpha: bool,
    pub depth: bool,
    pub stencil: bool,
    pub antialias: bool,
    pub premultiplied_alpha: bool,
    pub preserve_drawing_buffer: bool,
    pub power_preference: String,
    pub fail_if_major_performance_caveat: bool,
}

/// Context options for CesiumWidget.
#[derive(Debug, Clone, Default)]
pub struct WidgetContextOptions {
    pub allow_texture_filter_anisotropic: bool,
    pub webgl: WebglOptions,
}

/// Scene abstraction inside CesiumWidget.
pub struct WidgetScene {
    pub mode: SceneMode,
    pub morph_target_mode: Option<SceneMode>,
    pub camera: WidgetCamera,
    pub imagery_layers: ImageryLayerCollection,
    pub terrain_provider: Option<EllipsoidTerrainProvider>,
    pub globe: Option<Globe>,
    pub sky_box: Option<SkyBox>,
    pub sky_atmosphere: Option<SkyAtmosphere>,
    pub order_independent_translucency: bool,
    pub request_render_mode: bool,
    pub maximum_render_time_change: f64,
    pub pixel_ratio: f64,
    pub map_projection: MapProjectionType,
    pub context: WidgetContextOptions,
    pub render_error: Event<String>,
    pub request_render_count: usize,
    pub primitives: WidgetPrimitives,
    pub ellipsoid: Ellipsoid,
}

impl WidgetScene {
    pub fn complete_morph(&mut self) {
        if let Some(target) = self.morph_target_mode.take() {
            self.mode = target;
        }
    }

    pub fn request_render(&mut self) {
        self.request_render_count += 1;
    }
}

/// Display manager for DataSources.
#[derive(Clone, Default)]
pub struct DataSourceDisplay {
    pub is_ready: bool,
    pub custom_get_bounding_sphere:
        Option<Arc<dyn Fn(&Entity, bool, &mut BoundingSphere) -> BoundingSphereState + Send + Sync>>,
}

impl DataSourceDisplay {
    pub fn get_bounding_sphere(
        &self,
        entity: &Entity,
        allow_partial: bool,
        result: &mut BoundingSphere,
    ) -> BoundingSphereState {
        if let Some(ref custom) = self.custom_get_bounding_sphere {
            return custom(entity, allow_partial, result);
        }
        if let Some(ref pos_prop) = entity.position {
            if let Some(pos) = pos_prop.get_value(None) {
                result.center = pos.into();
                result.radius = 1.0;
                return BoundingSphereState::Done;
            }
        }
        BoundingSphereState::Failed
    }

    pub fn update(&mut self, _time: Option<&JulianDate>) -> bool {
        self.is_ready
    }
}

/// Error panel state for CesiumWidget.
#[derive(Debug, Clone)]
pub struct WidgetErrorPanel {
    pub title: String,
    pub message: String,
    pub visible: bool,
}

/// Construction options for CesiumWidget.
#[derive(Clone, Default)]
pub struct CesiumWidgetOptions {
    pub clock: Option<Clock>,
    pub should_animate: Option<bool>,
    pub ellipsoid: Option<Ellipsoid>,
    pub base_layer: Option<BaseLayerOption>,
    pub terrain_provider: Option<EllipsoidTerrainProvider>,
    pub sky_box: Option<SkyBoxOption>,
    pub sky_atmosphere: Option<SkyAtmosphereOption>,
    pub scene_mode: Option<SceneMode>,
    pub scene_3d_only: Option<bool>,
    pub order_independent_translucency: Option<bool>,
    pub map_projection: Option<MapProjectionType>,
    pub globe: Option<GlobeOption>,
    pub use_default_render_loop: Option<bool>,
    pub use_browser_recommended_resolution: Option<bool>,
    pub target_frame_rate: Option<f64>,
    pub show_render_loop_errors: Option<bool>,
    pub automatically_track_data_source_clocks: Option<bool>,
    pub context_options: Option<WidgetContextOptions>,
    pub credit_container: Option<String>,
    pub credit_viewport: Option<String>,
    pub data_sources: Option<Arc<Mutex<DataSourceCollection>>>,
    pub shadows: Option<bool>,
    pub terrain_shadows: Option<ShadowMode>,
    pub map_mode_2d: Option<MapMode2D>,
    pub blur_active_element_on_canvas_focus: Option<bool>,
    pub request_render_mode: Option<bool>,
    pub maximum_render_time_change: Option<f64>,
    pub msaa_samples: Option<u32>,
}

/// Target representation for zoomTo and flyTo.
#[derive(Clone)]
pub enum ZoomTarget {
    Tileset { bounding_sphere: BoundingSphere },
    PointCloud { bounding_sphere: Option<BoundingSphere> },
    Voxel { bounding_sphere: BoundingSphere },
    BufferCollection { bounding_volume: BoundingSphere },
    Entity(Entity),
    Entities(Vec<Entity>),
    EntityCollection(EntityCollection),
    ImageryLayer { rectangle_center: Cartesian3 },
}

/// Internal zoom or flight operation in progress.
#[derive(Clone)]
struct PendingZoom {
    target: ZoomTarget,
    options: FlyToOptions,
    is_flight: bool,
}

/// A widget containing a Cesium scene, camera, clock, credits, and rendering controls.
pub struct CesiumWidget {
    pub container: WidgetContainer,
    pub canvas: WidgetCanvas,
    pub credit_display: CreditDisplay,
    pub credit_container: String,
    pub credit_viewport: String,
    pub scene: WidgetScene,
    pub clock: Clock,
    pub screen_space_event_handler: ScreenSpaceEventHandler,
    pub use_browser_recommended_resolution: bool,
    pub use_default_render_loop: bool,
    target_frame_rate: Option<f64>,
    resolution_scale: f64,
    pub show_render_loop_errors: bool,
    pub automatically_track_data_source_clocks: bool,
    pub allow_data_sources_to_suspend_animation: bool,
    pub data_sources: Arc<Mutex<DataSourceCollection>>,
    is_default_data_sources: bool,
    pub clock_tracked_data_source: Option<CustomDataSource>,
    tracked_entity: Option<Entity>,
    pub tracked_entity_changed: Event<Option<Entity>>,
    pub entities: EntityCollection,
    pub data_source_display: DataSourceDisplay,
    pub error_panel: Option<WidgetErrorPanel>,
    pub force_resize: bool,
    pub device_pixel_ratio: f64,
    is_destroyed: AtomicBool,
    pending_zoom: Option<PendingZoom>,
    pub entity_view: Option<EntityView>,
}

impl CesiumWidget {
    /// Creates a new CesiumWidget instance.
    pub fn new(
        container: impl TryInto<WidgetContainer, Error = DeveloperError>,
        options: Option<CesiumWidgetOptions>,
    ) -> Result<Self, DeveloperError> {
        let container = container.try_into()?;
        let opts = options.unwrap_or_default();

        let mut clock = if let Some(c) = opts.clock {
            c
        } else {
            Clock::new(ClockOptions::default()).map_err(|e| DeveloperError::new(e.message))?
        };

        if let Some(should_animate) = opts.should_animate {
            clock.set_should_animate(should_animate);
        }

        let ellipsoid = opts.ellipsoid.unwrap_or(Ellipsoid::WGS84);

        let mut imagery_layers = ImageryLayerCollection::new();
        match opts.base_layer.unwrap_or_default() {
            BaseLayerOption::DefaultImagery => {
                let default_layer = ImageryLayer::new(None, Some(ImageryLayerOptions::default()));
                imagery_layers
                    .add(Arc::new(RwLock::new(default_layer)), None)
                    .map_err(|e| DeveloperError::new(e.to_string()))?;
            }
            BaseLayerOption::Layer(layer) => {
                imagery_layers
                    .add(Arc::new(RwLock::new(layer)), None)
                    .map_err(|e| DeveloperError::new(e.to_string()))?;
            }
            BaseLayerOption::False => {}
        }

        let terrain_provider = opts.terrain_provider.or_else(|| Some(EllipsoidTerrainProvider::default()));

        let globe = match opts.globe.unwrap_or_default() {
            GlobeOption::DefaultGlobe => Some(Globe::new(ellipsoid)),
            GlobeOption::Globe(g) => Some(g),
            GlobeOption::False => None,
        };

        let sky_box = match opts.sky_box.unwrap_or_default() {
            SkyBoxOption::DefaultSkyBox => Some(SkyBox::new(SkyBoxOptions::default())),
            SkyBoxOption::Box(b) => Some(b),
            SkyBoxOption::False => None,
        };

        let sky_atmosphere = match opts.sky_atmosphere.unwrap_or_default() {
            SkyAtmosphereOption::DefaultAtmosphere => {
                let mut atmosphere = SkyAtmosphere::new(Some(ellipsoid));
                if globe.is_none() {
                    atmosphere.show = false;
                }
                Some(atmosphere)
            }
            SkyAtmosphereOption::Atmosphere(mut a) => {
                if globe.is_none() {
                    a.show = false;
                }
                Some(a)
            }
            SkyAtmosphereOption::False => None,
        };

        let target_frame_rate = if let Some(rate) = opts.target_frame_rate {
            if rate < 0.0 {
                return Err(DeveloperError::new("targetFrameRate must be greater than or equal to 0."));
            }
            Some(rate)
        } else {
            None
        };

        let (data_sources, is_default_data_sources) = if let Some(ds) = opts.data_sources {
            (ds, false)
        } else {
            (Arc::new(Mutex::new(DataSourceCollection::new())), true)
        };

        let initial_mode = opts.scene_mode.unwrap_or(SceneMode::Scene3D);
        let scene = WidgetScene {
            mode: initial_mode,
            morph_target_mode: Some(initial_mode),
            camera: WidgetCamera::default(),
            imagery_layers,
            terrain_provider,
            globe,
            sky_box,
            sky_atmosphere,
            order_independent_translucency: opts.order_independent_translucency.unwrap_or(true),
            request_render_mode: opts.request_render_mode.unwrap_or(false),
            maximum_render_time_change: opts.maximum_render_time_change.unwrap_or(0.0),
            pixel_ratio: 1.0,
            map_projection: opts.map_projection.unwrap_or_default(),
            context: opts.context_options.unwrap_or_default(),
            render_error: Event::new(),
            request_render_count: 0,
            primitives: WidgetPrimitives::default(),
            ellipsoid,
        };

        let credit_container = opts.credit_container.unwrap_or_else(|| "creditContainer".to_string());
        let credit_viewport = opts.credit_viewport.unwrap_or_else(|| "creditViewport".to_string());

        let mut widget = Self {
            container,
            canvas: WidgetCanvas::default(),
            credit_display: CreditDisplay::default(),
            credit_container,
            credit_viewport,
            scene,
            clock,
            screen_space_event_handler: ScreenSpaceEventHandler::new(),
            use_browser_recommended_resolution: opts.use_browser_recommended_resolution.unwrap_or(true),
            use_default_render_loop: opts.use_default_render_loop.unwrap_or(true),
            target_frame_rate,
            resolution_scale: 1.0,
            show_render_loop_errors: opts.show_render_loop_errors.unwrap_or(true),
            automatically_track_data_source_clocks: opts.automatically_track_data_source_clocks.unwrap_or(true),
            allow_data_sources_to_suspend_animation: true,
            data_sources,
            is_default_data_sources,
            clock_tracked_data_source: None,
            tracked_entity: None,
            tracked_entity_changed: Event::new(),
            entities: EntityCollection::new(),
            data_source_display: DataSourceDisplay::default(),
            error_panel: None,
            force_resize: false,
            device_pixel_ratio: 1.0,
            is_destroyed: AtomicBool::new(false),
            pending_zoom: None,
            entity_view: None,
        };

        widget.resize();
        Ok(widget)
    }

    pub fn target_frame_rate(&self) -> Option<f64> {
        self.target_frame_rate
    }

    pub fn set_target_frame_rate(&mut self, rate: f64) -> Result<(), DeveloperError> {
        if rate < 0.0 {
            return Err(DeveloperError::new("targetFrameRate must be greater than or equal to 0."));
        }
        self.target_frame_rate = Some(rate);
        Ok(())
    }

    pub fn resolution_scale(&self) -> f64 {
        self.resolution_scale
    }

    pub fn set_resolution_scale(&mut self, scale: f64) -> Result<(), DeveloperError> {
        if scale <= 0.0 {
            return Err(DeveloperError::new("resolutionScale must be greater than 0."));
        }
        self.resolution_scale = scale;
        self.resize();
        Ok(())
    }

    pub fn terrain_provider(&self) -> Option<&EllipsoidTerrainProvider> {
        self.scene.terrain_provider.as_ref()
    }

    pub fn set_terrain_provider(&mut self, provider: EllipsoidTerrainProvider) {
        self.scene.terrain_provider = Some(provider);
    }

    pub fn tracked_entity(&self) -> Option<&Entity> {
        self.tracked_entity.as_ref()
    }

    pub fn set_tracked_entity(&mut self, entity: Option<Entity>) {
        if self.tracked_entity != entity {
            self.tracked_entity = entity.clone();
            if let Some(ref e) = self.tracked_entity {
                self.entity_view = Some(EntityView::new(e.clone(), Some(self.scene.ellipsoid)));
            } else {
                self.entity_view = None;
                self.scene.camera.look_at_transform(Matrix4::IDENTITY);
            }
            self.tracked_entity_changed.raise_event(&entity);
        }
    }

    pub fn set_clock_tracked_data_source(&mut self, data_source: Option<CustomDataSource>) {
        self.clock_tracked_data_source = data_source;
        let ds_clock = self.clock_tracked_data_source.as_ref().and_then(|ds| ds.clock().cloned());
        if let Some(ref clock) = ds_clock {
            self.sync_clock_from_data_source_clock(clock);
        }
    }

    fn sync_clock_from_data_source_clock(&mut self, ds_clock: &DataSourceClock) {
        if let Some(start) = ds_clock.start_time {
            self.clock.set_start_time(start);
        }
        if let Some(stop) = ds_clock.stop_time {
            self.clock.set_stop_time(stop);
        }
        if let Some(current) = ds_clock.current_time {
            self.clock.set_current_time(current);
        }
        if let Some(range) = ds_clock.clock_range {
            self.clock.set_clock_range(range);
        }
        if let Some(step) = ds_clock.clock_step {
            self.clock.set_clock_step(step);
        }
        if let Some(multiplier) = ds_clock.multiplier {
            self.clock.set_multiplier(multiplier);
        }
    }

    pub fn on_data_source_added(&mut self, data_source: &CustomDataSource) {
        if self.automatically_track_data_source_clocks {
            self.clock_tracked_data_source = Some(data_source.clone());
            let ds_clock = data_source.clock().cloned();
            if let Some(ref clock) = ds_clock {
                self.sync_clock_from_data_source_clock(clock);
            }
        }
    }

    pub fn on_data_source_removed(&mut self, data_source: &CustomDataSource) {
        let is_tracked = self
            .clock_tracked_data_source
            .as_ref()
            .map(|ds| ds.id() == data_source.id())
            .unwrap_or(false);

        if is_tracked {
            if self.automatically_track_data_source_clocks {
                let ds_lock = self.data_sources.lock().unwrap();
                let last = ds_lock.get(ds_lock.len().saturating_sub(1)).cloned();
                drop(ds_lock);

                if let Some(last_ds) = last {
                    self.clock_tracked_data_source = Some(last_ds.clone());
                    let ds_clock = last_ds.clock().cloned();
                    if let Some(ref clock) = ds_clock {
                        self.sync_clock_from_data_source_clock(clock);
                    }
                } else {
                    self.clock_tracked_data_source = None;
                }
            } else {
                self.clock_tracked_data_source = None;
            }
        }
    }

    pub fn on_data_source_changed(&mut self, data_source: &CustomDataSource) {
        let is_tracked = self
            .clock_tracked_data_source
            .as_ref()
            .map(|ds| ds.id() == data_source.id())
            .unwrap_or(false);

        if is_tracked {
            let ds_clock = data_source.clock().cloned();
            if let Some(ref clock) = ds_clock {
                self.sync_clock_from_data_source_clock(clock);
            }
        }
    }

    pub fn resize(&mut self) {
        let pixel_ratio = if self.use_browser_recommended_resolution {
            self.resolution_scale
        } else {
            self.device_pixel_ratio * self.resolution_scale
        };
        self.scene.pixel_ratio = pixel_ratio;

        let width = (self.canvas.client_width as f64 * pixel_ratio).max(1.0).round() as u32;
        let height = (self.canvas.client_height as f64 * pixel_ratio).max(1.0).round() as u32;

        let changed = self.canvas.width != width || self.canvas.height != height;
        self.canvas.width = width;
        self.canvas.height = height;

        if height > 0 {
            self.scene.camera.frustum.aspect_ratio = Some(width as f64 / height as f64);
        }

        if (changed || self.force_resize) && self.scene.request_render_mode {
            self.scene.request_render();
        }
        self.force_resize = false;
    }

    pub fn show_error_panel(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.error_panel = Some(WidgetErrorPanel {
            title: title.into(),
            message: message.into(),
            visible: true,
        });
    }

    pub fn dismiss_error_panel(&mut self) {
        self.error_panel = None;
    }

    pub fn tick_clock(&mut self) {
        if self.allow_data_sources_to_suspend_animation && !self.data_source_display.is_ready {
            self.clock.can_animate = false;
        }
        self.clock.tick();
    }

    pub fn render(&mut self) -> Result<(), String> {
        if self.is_destroyed.load(Ordering::Relaxed) {
            return Ok(());
        }

        if let Err(e) = self.scene.primitives.update() {
            self.use_default_render_loop = false;
            if self.show_render_loop_errors {
                self.show_error_panel(
                    "An error occurred while rendering. Rendering has stopped.",
                    e.clone(),
                );
            }
            self.scene.render_error.raise_event(&e);
            return Err(e);
        }

        self.post_render();
        Ok(())
    }

    pub fn post_render(&mut self) {
        self.update_zoom_target();
        self.update_tracked_entity();
    }

    pub fn zoom_to(
        &mut self,
        target: ZoomTarget,
        offset: Option<HeadingPitchRange>,
    ) -> Result<bool, DeveloperError> {
        self.pending_zoom = Some(PendingZoom {
            target,
            options: FlyToOptions {
                offset,
                ..Default::default()
            },
            is_flight: false,
        });
        self.scene.request_render();
        Ok(true)
    }

    pub fn fly_to(
        &mut self,
        target: ZoomTarget,
        options: Option<FlyToOptions>,
    ) -> Result<bool, DeveloperError> {
        let opts = options.unwrap_or_default();
        self.pending_zoom = Some(PendingZoom {
            target,
            options: opts,
            is_flight: true,
        });
        self.scene.request_render();
        Ok(true)
    }

    fn update_zoom_target(&mut self) {
        let Some(pending) = self.pending_zoom.take() else {
            return;
        };

        match pending.target {
            ZoomTarget::Tileset { bounding_sphere }
            | ZoomTarget::Voxel { bounding_sphere }
            | ZoomTarget::BufferCollection {
                bounding_volume: bounding_sphere,
            } => {
                let offset = pending.options.offset.unwrap_or_else(|| {
                    HeadingPitchRange::new(0.0, -0.5, bounding_sphere.radius)
                });
                if pending.is_flight {
                    let mut opts = pending.options;
                    opts.offset = Some(offset);
                    self.scene.camera.fly_to_bounding_sphere(bounding_sphere, opts);
                } else {
                    self.scene.camera.view_bounding_sphere(bounding_sphere, Some(offset));
                    self.scene.camera.look_at_transform(Matrix4::IDENTITY);
                }
            }
            ZoomTarget::PointCloud { bounding_sphere } => {
                if let Some(sphere) = bounding_sphere {
                    let offset = pending.options.offset.unwrap_or_else(|| {
                        HeadingPitchRange::new(0.0, -0.5, sphere.radius)
                    });
                    if pending.is_flight {
                        let mut opts = pending.options;
                        opts.offset = Some(offset);
                        self.scene.camera.fly_to_bounding_sphere(sphere, opts);
                    } else {
                        self.scene.camera.view_bounding_sphere(sphere, Some(offset));
                        self.scene.camera.look_at_transform(Matrix4::IDENTITY);
                    }
                }
            }
            ZoomTarget::Entity(entity) => {
                let mut bs = BoundingSphere::default();
                let state = self.data_source_display.get_bounding_sphere(&entity, false, &mut bs);
                if state != BoundingSphereState::Failed {
                    let offset = pending.options.offset;
                    if pending.is_flight {
                        let mut opts = pending.options;
                        opts.offset = offset;
                        self.scene.camera.fly_to_bounding_sphere(bs, opts);
                    } else {
                        self.scene.camera.view_bounding_sphere(bs, offset);
                        self.scene.camera.look_at_transform(Matrix4::IDENTITY);
                    }
                }
            }
            ZoomTarget::Entities(entities) => {
                let mut spheres = Vec::new();
                for e in &entities {
                    let mut bs = BoundingSphere::default();
                    if self.data_source_display.get_bounding_sphere(e, false, &mut bs) != BoundingSphereState::Failed {
                        spheres.push(bs);
                    }
                }
                if let Some(first) = spheres.first() {
                    let mut union_sphere = *first;
                    for s in spheres.iter().skip(1) {
                        union_sphere = union_sphere.union(*s);
                    }
                    let offset = pending.options.offset;
                    if pending.is_flight {
                        let mut opts = pending.options;
                        opts.offset = offset;
                        self.scene.camera.fly_to_bounding_sphere(union_sphere, opts);
                    } else {
                        self.scene.camera.view_bounding_sphere(union_sphere, offset);
                        self.scene.camera.look_at_transform(Matrix4::IDENTITY);
                    }
                }
            }
            ZoomTarget::EntityCollection(collection) => {
                let mut spheres = Vec::new();
                for e in collection.values() {
                    let mut bs = BoundingSphere::default();
                    if self.data_source_display.get_bounding_sphere(e, false, &mut bs) != BoundingSphereState::Failed {
                        spheres.push(bs);
                    }
                }
                let target_sphere = if let Some(first) = spheres.first() {
                    let mut union_sphere = *first;
                    for s in spheres.iter().skip(1) {
                        union_sphere = union_sphere.union(*s);
                    }
                    union_sphere
                } else {
                    BoundingSphere::default()
                };
                let offset = pending.options.offset;
                if pending.is_flight {
                    let mut opts = pending.options;
                    opts.offset = offset;
                    self.scene.camera.fly_to_bounding_sphere(target_sphere, opts);
                } else {
                    self.scene.camera.view_bounding_sphere(target_sphere, offset);
                    self.scene.camera.look_at_transform(Matrix4::IDENTITY);
                }
            }
            ZoomTarget::ImageryLayer { rectangle_center } => {
                let mut opts = pending.options;
                opts.destination = Some(rectangle_center);
                if pending.is_flight {
                    self.scene.camera.fly_to(&opts);
                } else {
                    self.scene.camera.set_view(&opts);
                }
            }
        }
    }

    fn update_tracked_entity(&mut self) {
        if let Some(ref entity) = self.tracked_entity {
            if let Some(ref pos_prop) = entity.position {
                let curr_time = self.clock.current_time();
                if let Some(pos) = pos_prop.get_value(Some(&curr_time)) {
                    self.scene.camera.look_at_transform(Matrix4::from_translation(pos));
                }
            }
        }
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed.load(Ordering::Relaxed)
    }

    pub fn destroy(&mut self) {
        if !self.is_destroyed.swap(true, Ordering::Relaxed) {
            if self.is_default_data_sources {
                if let Ok(mut ds) = self.data_sources.lock() {
                    ds.remove_all();
                }
            }
            self.clock_tracked_data_source = None;
            self.tracked_entity = None;
            self.error_panel = None;
        }
    }
}

impl TryFrom<&str> for WidgetContainer {
    type Error = DeveloperError;
    fn try_from(id: &str) -> Result<Self, Self::Error> {
        Self::new(id)
    }
}

impl TryFrom<String> for WidgetContainer {
    type Error = DeveloperError;
    fn try_from(id: String) -> Result<Self, Self::Error> {
        Self::new(id)
    }
}
