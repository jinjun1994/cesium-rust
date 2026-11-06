//! RenderState matching CesiumJS `Renderer/RenderState.js` (Apache-2.0).

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use cesium_math::{BoundingRectangle, Color, WindingOrder};

/// Front face culling modes matching WebGLConstants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CullFace {
    Front = 0x0404,
    #[default]
    Back = 0x0405,
    FrontAndBack = 0x0408,
}

/// Depth comparison functions matching WebGLConstants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DepthFunction {
    Never = 0x0200,
    #[default]
    Less = 0x0201,
    Equal = 0x0202,
    LEqual = 0x0203,
    Greater = 0x0204,
    NotEqual = 0x0205,
    GEqual = 0x0206,
    Always = 0x0207,
}

/// Blending equations matching WebGLConstants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BlendEquation {
    #[default]
    Add = 0x8006,
    Subtract = 0x800A,
    ReverseSubtract = 0x800B,
    Min = 0x8007,
    Max = 0x8008,
}

/// Blend factors matching WebGLConstants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BlendFunction {
    Zero = 0,
    #[default]
    One = 1,
    SrcColor = 0x0300,
    OneMinusSrcColor = 0x0301,
    SrcAlpha = 0x0302,
    OneMinusSrcAlpha = 0x0303,
    DstAlpha = 0x0304,
    OneMinusDstAlpha = 0x0305,
    DstColor = 0x0306,
    OneMinusDstColor = 0x0307,
    SrcAlphaSaturate = 0x0308,
    ConstantColor = 0x8001,
    OneMinusConstantColor = 0x8002,
    ConstantAlpha = 0x8003,
    OneMinusConstantAlpha = 0x8004,
}

/// Stencil test functions matching WebGLConstants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StencilFunction {
    Never = 0x0200,
    Less = 0x0201,
    Equal = 0x0202,
    LEqual = 0x0203,
    Greater = 0x0204,
    NotEqual = 0x0205,
    GEqual = 0x0206,
    #[default]
    Always = 0x0207,
}

/// Stencil actions matching WebGLConstants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StencilOperation {
    Zero = 0,
    #[default]
    Keep = 0x1E00,
    Replace = 0x1E01,
    Incr = 0x1E02,
    Decr = 0x1E03,
    Invert = 0x150A,
    IncrWrap = 0x8507,
    DecrWrap = 0x8508,
}

/// Cull configuration state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CullState {
    pub enabled: bool,
    pub face: CullFace,
}

/// Polygon offset configuration state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PolygonOffsetState {
    pub enabled: bool,
    pub factor: f64,
    pub units: f64,
}

impl Eq for PolygonOffsetState {}
impl std::hash::Hash for PolygonOffsetState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.enabled.hash(state);
        self.factor.to_bits().hash(state);
        self.units.to_bits().hash(state);
    }
}

/// Scissor test state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScissorTestState {
    pub enabled: bool,
    pub rectangle: BoundingRectangle,
}

impl Eq for ScissorTestState {}
impl std::hash::Hash for ScissorTestState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.enabled.hash(state);
        self.rectangle.x.to_bits().hash(state);
        self.rectangle.y.to_bits().hash(state);
        self.rectangle.width.to_bits().hash(state);
        self.rectangle.height.to_bits().hash(state);
    }
}

/// Depth range state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DepthRangeState {
    pub near: f64,
    pub far: f64,
}

impl Default for DepthRangeState {
    fn default() -> Self {
        Self {
            near: 0.0,
            far: 1.0,
        }
    }
}

impl Eq for DepthRangeState {}
impl std::hash::Hash for DepthRangeState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.near.to_bits().hash(state);
        self.far.to_bits().hash(state);
    }
}

/// Depth test state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DepthTestState {
    pub enabled: bool,
    pub func: DepthFunction,
}

/// Color write mask state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ColorMaskState {
    pub red: bool,
    pub green: bool,
    pub blue: bool,
    pub alpha: bool,
}

impl Default for ColorMaskState {
    fn default() -> Self {
        Self {
            red: true,
            green: true,
            blue: true,
            alpha: true,
        }
    }
}

/// Blending configuration state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlendingState {
    pub enabled: bool,
    pub color: Color,
    pub equation_rgb: BlendEquation,
    pub equation_alpha: BlendEquation,
    pub function_source_rgb: BlendFunction,
    pub function_source_alpha: BlendFunction,
    pub function_destination_rgb: BlendFunction,
    pub function_destination_alpha: BlendFunction,
}

impl Default for BlendingState {
    fn default() -> Self {
        Self {
            enabled: false,
            color: Color::new(0.0, 0.0, 0.0, 0.0),
            equation_rgb: BlendEquation::Add,
            equation_alpha: BlendEquation::Add,
            function_source_rgb: BlendFunction::One,
            function_source_alpha: BlendFunction::One,
            function_destination_rgb: BlendFunction::Zero,
            function_destination_alpha: BlendFunction::Zero,
        }
    }
}

impl Eq for BlendingState {}
impl std::hash::Hash for BlendingState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.enabled.hash(state);
        self.color.red.to_bits().hash(state);
        self.color.green.to_bits().hash(state);
        self.color.blue.to_bits().hash(state);
        self.color.alpha.to_bits().hash(state);
        self.equation_rgb.hash(state);
        self.equation_alpha.hash(state);
        self.function_source_rgb.hash(state);
        self.function_source_alpha.hash(state);
        self.function_destination_rgb.hash(state);
        self.function_destination_alpha.hash(state);
    }
}

/// Stencil operation per face.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct StencilOperationState {
    pub fail: StencilOperation,
    pub z_fail: StencilOperation,
    pub z_pass: StencilOperation,
}

/// Stencil test configuration state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StencilTestState {
    pub enabled: bool,
    pub front_function: StencilFunction,
    pub back_function: StencilFunction,
    pub reference: i32,
    pub mask: u32,
    pub front_operation: StencilOperationState,
    pub back_operation: StencilOperationState,
}

impl Default for StencilTestState {
    fn default() -> Self {
        Self {
            enabled: false,
            front_function: StencilFunction::Always,
            back_function: StencilFunction::Always,
            reference: 0,
            mask: !0,
            front_operation: StencilOperationState::default(),
            back_operation: StencilOperationState::default(),
        }
    }
}

/// Sample coverage state for multisampling.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampleCoverageState {
    pub enabled: bool,
    pub value: f64,
    pub invert: bool,
}

impl Default for SampleCoverageState {
    fn default() -> Self {
        Self {
            enabled: false,
            value: 1.0,
            invert: false,
        }
    }
}

impl Eq for SampleCoverageState {}
impl std::hash::Hash for SampleCoverageState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.enabled.hash(state);
        self.value.to_bits().hash(state);
        self.invert.hash(state);
    }
}

/// Immutable rendering state matching CesiumJS `Renderer/RenderState.js`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RenderState {
    pub front_face: WindingOrder,
    pub cull: CullState,
    pub line_width: u32,
    pub polygon_offset: PolygonOffsetState,
    pub scissor_test: ScissorTestState,
    pub depth_range: DepthRangeState,
    pub depth_test: DepthTestState,
    pub color_mask: ColorMaskState,
    pub depth_mask: bool,
    pub stencil_mask: u32,
    pub blending: BlendingState,
    pub stencil_test: StencilTestState,
    pub sample_coverage: SampleCoverageState,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            front_face: WindingOrder::CounterClockwise,
            cull: CullState::default(),
            line_width: 1,
            polygon_offset: PolygonOffsetState::default(),
            scissor_test: ScissorTestState::default(),
            depth_range: DepthRangeState::default(),
            depth_test: DepthTestState::default(),
            color_mask: ColorMaskState::default(),
            depth_mask: true,
            stencil_mask: !0,
            blending: BlendingState::default(),
            stencil_test: StencilTestState::default(),
            sample_coverage: SampleCoverageState::default(),
        }
    }
}

static RENDER_STATE_CACHE: OnceLock<Mutex<HashMap<RenderState, Arc<RenderState>>>> =
    OnceLock::new();

impl RenderState {
    /// Retrieves a cached, deduplicated RenderState instance.
    pub fn from_cache(state: Option<RenderState>) -> Arc<Self> {
        let rs = state.unwrap_or_default();
        let cache_mutex = RENDER_STATE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        let mut cache = cache_mutex.lock().unwrap();
        cache.entry(rs.clone()).or_insert_with(|| Arc::new(rs)).clone()
    }
}
