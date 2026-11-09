use cesium_math::DeveloperError;

/// The pass in which a 3D Tileset is updated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Cesium3DTilePass {
    Render = 0,
    Pick = 1,
    Shadow = 2,
    Preload = 3,
    PreloadFlight = 4,
    RequestRenderModeDeferCheck = 5,
    MostDetailedPreload = 6,
    MostDetailedPick = 7,
}

impl Cesium3DTilePass {
    pub const NUMBER_OF_PASSES: usize = 8;

    /// Whether this pass renders to screen.
    pub const fn is_render(&self) -> bool {
        matches!(self, Self::Render)
    }

    /// Whether this pass requests tiles.
    pub const fn request_tiles(&self) -> bool {
        !matches!(self, Self::Pick | Self::MostDetailedPick)
    }

    /// Whether this pass ignores rendering commands.
    pub const fn ignore_commands(&self) -> bool {
        matches!(
            self,
            Self::Preload
                | Self::PreloadFlight
                | Self::RequestRenderModeDeferCheck
                | Self::MostDetailedPreload
        )
    }
}

/// Options for constructing a [`Cesium3DTilePassState`].
#[derive(Debug, Clone, Default)]
pub struct Cesium3DTilePassStateOptions<C = (), Cam = (), Cul = ()> {
    pub pass: Option<Cesium3DTilePass>,
    pub command_list: Option<C>,
    pub camera: Option<Cam>,
    pub culling_volume: Option<Cul>,
}

/// The state for a 3D Tiles update pass.
#[derive(Debug, Clone, PartialEq)]
pub struct Cesium3DTilePassState<C = (), Cam = (), Cul = ()> {
    /// The pass type.
    pub pass: Cesium3DTilePass,
    /// An array of rendering commands to use for this pass.
    pub command_list: Option<C>,
    /// Camera to use for this pass.
    pub camera: Option<Cam>,
    /// Culling volume to use for this pass.
    pub culling_volume: Option<Cul>,
    /// Whether all tiles needed by the pass are ready.
    pub ready: bool,
}

impl<C, Cam, Cul> Cesium3DTilePassState<C, Cam, Cul> {
    /// Creates a new `Cesium3DTilePassState`.
    /// Throws DeveloperError if `options` or `options.pass` is undefined.
    pub fn new(options: Option<Cesium3DTilePassStateOptions<C, Cam, Cul>>) -> Result<Self, DeveloperError> {
        let opts = options.ok_or_else(|| DeveloperError::new("options is required."))?;
        let pass = opts.pass.ok_or_else(|| DeveloperError::new("options.pass is required."))?;

        Ok(Self {
            pass,
            command_list: opts.command_list,
            camera: opts.camera,
            culling_volume: opts.culling_volume,
            ready: false,
        })
    }
}
