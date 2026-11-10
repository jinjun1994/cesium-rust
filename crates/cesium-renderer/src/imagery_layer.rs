use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use cesium_math::{Cartesian3, Color, GeoRectangle};

static NEXT_LAYER_ID: AtomicU64 = AtomicU64::new(1);

/// The direction of split screen imagery display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitDirection {
    #[default]
    None = 0,
    Left = -1,
    Right = 1,
}

/// Texture minification filter options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextureMinificationFilter {
    #[default]
    Linear,
    Nearest,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

/// Texture magnification filter options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextureMagnificationFilter {
    #[default]
    Linear,
    Nearest,
}

/// Feature metadata returned from picking an imagery tile.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImageryLayerFeatureInfo {
    pub name: Option<String>,
    pub description: Option<String>,
    pub position: Option<Cartesian3>,
    pub data: Option<serde_json::Value>,
}

/// Common trait for imagery providers that back an [`ImageryLayer`].
pub trait ImageryProvider: Send + Sync {
    fn rectangle(&self) -> GeoRectangle {
        GeoRectangle::MAXIMUM
    }
    fn tile_width(&self) -> u32 {
        256
    }
    fn tile_height(&self) -> u32 {
        256
    }
    fn minimum_level(&self) -> u32 {
        0
    }
    fn maximum_level(&self) -> Option<u32> {
        None
    }
    fn has_alpha_channel(&self) -> bool {
        true
    }
    fn is_ready(&self) -> bool {
        true
    }
    fn pick_features(
        &self,
        _x: u32,
        _y: u32,
        _level: u32,
        _longitude: f64,
        _latitude: f64,
    ) -> Option<Vec<ImageryLayerFeatureInfo>> {
        None
    }
}

/// Initialization options for an [`ImageryLayer`].
#[derive(Clone, Debug, Default)]
pub struct ImageryLayerOptions {
    pub rectangle: Option<GeoRectangle>,
    pub alpha: Option<f32>,
    pub night_alpha: Option<f32>,
    pub day_alpha: Option<f32>,
    pub brightness: Option<f32>,
    pub contrast: Option<f32>,
    pub hue: Option<f32>,
    pub saturation: Option<f32>,
    pub gamma: Option<f32>,
    pub split_direction: Option<SplitDirection>,
    pub minification_filter: Option<TextureMinificationFilter>,
    pub magnification_filter: Option<TextureMagnificationFilter>,
    pub show: Option<bool>,
    pub cutout_rectangle: Option<GeoRectangle>,
    pub color_to_alpha: Option<Color>,
    pub color_to_alpha_threshold: Option<f32>,
    pub minimum_terrain_level: Option<u32>,
    pub maximum_terrain_level: Option<u32>,
}

/// An imagery layer on the globe or 3D tileset.
#[derive(Clone)]
pub struct ImageryLayer {
    id: u64,
    provider: Option<Arc<dyn ImageryProvider>>,
    show: bool,
    previous_show: Option<bool>,
    alpha: f32,
    night_alpha: f32,
    day_alpha: f32,
    brightness: f32,
    contrast: f32,
    hue: f32,
    saturation: f32,
    gamma: f32,
    split_direction: SplitDirection,
    minification_filter: TextureMinificationFilter,
    magnification_filter: TextureMagnificationFilter,
    rectangle: GeoRectangle,
    cutout_rectangle: Option<GeoRectangle>,
    color_to_alpha: Option<Color>,
    color_to_alpha_threshold: f32,
    minimum_terrain_level: Option<u32>,
    maximum_terrain_level: Option<u32>,

    layer_index: usize,
    is_base_layer: bool,
    ready: bool,
    is_destroyed: bool,
}

impl std::fmt::Debug for ImageryLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageryLayer")
            .field("id", &self.id)
            .field("show", &self.show)
            .field("alpha", &self.alpha)
            .finish()
    }
}

impl ImageryLayer {
    /// Creates a new `ImageryLayer` backed by an imagery provider.
    pub fn new(
        provider: Option<Arc<dyn ImageryProvider>>,
        options: Option<ImageryLayerOptions>,
    ) -> Self {
        let opts = options.unwrap_or_default();
        let rect = opts
            .rectangle
            .or_else(|| provider.as_ref().map(|p| p.rectangle()))
            .unwrap_or(GeoRectangle::MAXIMUM);
        let ready = provider.as_ref().map(|p| p.is_ready()).unwrap_or(true);

        Self {
            id: NEXT_LAYER_ID.fetch_add(1, Ordering::Relaxed),
            provider,
            show: opts.show.unwrap_or(true),
            previous_show: None,
            alpha: opts.alpha.unwrap_or(1.0),
            night_alpha: opts.night_alpha.unwrap_or(1.0),
            day_alpha: opts.day_alpha.unwrap_or(1.0),
            brightness: opts.brightness.unwrap_or(1.0),
            contrast: opts.contrast.unwrap_or(1.0),
            hue: opts.hue.unwrap_or(0.0),
            saturation: opts.saturation.unwrap_or(1.0),
            gamma: opts.gamma.unwrap_or(1.0),
            split_direction: opts.split_direction.unwrap_or_default(),
            minification_filter: opts.minification_filter.unwrap_or_default(),
            magnification_filter: opts.magnification_filter.unwrap_or_default(),
            rectangle: rect,
            cutout_rectangle: opts.cutout_rectangle,
            color_to_alpha: opts.color_to_alpha,
            color_to_alpha_threshold: opts.color_to_alpha_threshold.unwrap_or(0.004),
            minimum_terrain_level: opts.minimum_terrain_level,
            maximum_terrain_level: opts.maximum_terrain_level,

            layer_index: 0,
            is_base_layer: false,
            ready,
            is_destroyed: false,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn imagery_provider(&self) -> Option<&Arc<dyn ImageryProvider>> {
        self.provider.as_ref()
    }

    pub fn show(&self) -> bool {
        self.show
    }

    pub fn set_show(&mut self, show: bool) {
        self.show = show;
    }

    pub fn cutout_rectangle(&self) -> Option<GeoRectangle> {
        self.cutout_rectangle
    }

    pub fn set_cutout_rectangle(&mut self, rect: Option<GeoRectangle>) {
        self.cutout_rectangle = rect;
    }

    pub fn color_to_alpha(&self) -> Option<Color> {
        self.color_to_alpha
    }

    pub fn set_color_to_alpha(&mut self, color: Option<Color>) {
        self.color_to_alpha = color;
    }

    pub fn color_to_alpha_threshold(&self) -> f32 {
        self.color_to_alpha_threshold
    }

    pub fn set_color_to_alpha_threshold(&mut self, threshold: f32) {
        self.color_to_alpha_threshold = threshold;
    }

    pub fn previous_show(&self) -> Option<bool> {
        self.previous_show
    }

    pub fn set_previous_show(&mut self, show: Option<bool>) {
        self.previous_show = show;
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
    }

    pub fn night_alpha(&self) -> f32 {
        self.night_alpha
    }

    pub fn set_night_alpha(&mut self, alpha: f32) {
        self.night_alpha = alpha;
    }

    pub fn day_alpha(&self) -> f32 {
        self.day_alpha
    }

    pub fn set_day_alpha(&mut self, alpha: f32) {
        self.day_alpha = alpha;
    }

    pub fn brightness(&self) -> f32 {
        self.brightness
    }

    pub fn set_brightness(&mut self, val: f32) {
        self.brightness = val;
    }

    pub fn contrast(&self) -> f32 {
        self.contrast
    }

    pub fn set_contrast(&mut self, val: f32) {
        self.contrast = val;
    }

    pub fn hue(&self) -> f32 {
        self.hue
    }

    pub fn set_hue(&mut self, val: f32) {
        self.hue = val;
    }

    pub fn saturation(&self) -> f32 {
        self.saturation
    }

    pub fn set_saturation(&mut self, val: f32) {
        self.saturation = val;
    }

    pub fn gamma(&self) -> f32 {
        self.gamma
    }

    pub fn set_gamma(&mut self, val: f32) {
        self.gamma = val;
    }

    pub fn split_direction(&self) -> SplitDirection {
        self.split_direction
    }

    pub fn set_split_direction(&mut self, direction: SplitDirection) {
        self.split_direction = direction;
    }

    pub fn minification_filter(&self) -> TextureMinificationFilter {
        self.minification_filter
    }

    pub fn set_minification_filter(&mut self, filter: TextureMinificationFilter) {
        self.minification_filter = filter;
    }

    pub fn magnification_filter(&self) -> TextureMagnificationFilter {
        self.magnification_filter
    }

    pub fn set_magnification_filter(&mut self, filter: TextureMagnificationFilter) {
        self.magnification_filter = filter;
    }

    pub fn rectangle(&self) -> GeoRectangle {
        self.rectangle
    }

    pub fn is_base_layer(&self) -> bool {
        self.is_base_layer
    }

    pub fn set_is_base_layer(&mut self, is_base: bool) {
        self.is_base_layer = is_base;
    }

    pub fn layer_index(&self) -> usize {
        self.layer_index
    }

    pub fn set_layer_index(&mut self, idx: usize) {
        self.layer_index = idx;
    }

    pub fn minimum_terrain_level(&self) -> Option<u32> {
        self.minimum_terrain_level
    }

    pub fn set_minimum_terrain_level(&mut self, level: Option<u32>) {
        self.minimum_terrain_level = level;
    }

    pub fn maximum_terrain_level(&self) -> Option<u32> {
        self.maximum_terrain_level
    }

    pub fn set_maximum_terrain_level(&mut self, level: Option<u32>) {
        self.maximum_terrain_level = level;
    }

    pub fn is_tile_level_supported(&self, level: u32) -> bool {
        let min = self.minimum_terrain_level.unwrap_or(0);
        let max = self.maximum_terrain_level.unwrap_or(u32::MAX);
        level >= min && level <= max
    }

    pub fn get_imagery_rectangle(&self) -> Option<GeoRectangle> {
        if let Some(provider) = &self.provider {
            let p_rect = provider.rectangle();
            self.rectangle.intersection(p_rect)
        } else {
            Some(self.rectangle)
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn set_ready(&mut self, ready: bool) {
        self.ready = ready;
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.provider = None;
    }
}
