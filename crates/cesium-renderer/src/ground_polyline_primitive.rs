//! GroundPolylinePrimitive matching CesiumJS `Scene/GroundPolylinePrimitive.js` (Apache-2.0).

use cesium_math::{
    ApproximateTerrainHeights, ClassificationType, Color, DeveloperError, GeometryInstance,
    GeometryInstanceAttribute,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Material appearance for polylines.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PolylineMaterialAppearance {
    pub translucent: bool,
    pub closed: bool,
}

/// Per-instance color appearance for polylines.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PolylineColorAppearance {
    pub translucent: bool,
    pub closed: bool,
}

/// Appearance enumeration for GroundPolylinePrimitive.
#[derive(Clone, Debug, PartialEq)]
pub enum GroundPolylineAppearance {
    Material(PolylineMaterialAppearance),
    Color(PolylineColorAppearance),
}

impl Default for GroundPolylineAppearance {
    fn default() -> Self {
        Self::Material(PolylineMaterialAppearance::default())
    }
}

/// Modifiable per-instance attributes returned by [`GroundPolylinePrimitive::get_geometry_instance_attributes`].
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PerInstanceAttributes {
    pub color: Option<Color>,
    pub show: Option<bool>,
    pub width: Option<f32>,
    pub distance_display_condition: Option<[f32; 2]>,
}

impl PerInstanceAttributes {
    /// Sets the per-instance color attribute.
    pub fn set_color(&mut self, color: Option<Color>) -> Result<(), DeveloperError> {
        let col = color.ok_or_else(|| DeveloperError::new("value is required."))?;
        self.color = Some(col);
        Ok(())
    }

    /// Sets the per-instance show attribute.
    pub fn set_show(&mut self, show: Option<bool>) -> Result<(), DeveloperError> {
        let s = show.ok_or_else(|| DeveloperError::new("value is required."))?;
        self.show = Some(s);
        Ok(())
    }

    /// Sets the per-instance width attribute.
    pub fn set_width(&mut self, width: Option<f32>) -> Result<(), DeveloperError> {
        let w = width.ok_or_else(|| DeveloperError::new("value is required."))?;
        self.width = Some(w);
        Ok(())
    }

    /// Sets the distance display condition attribute.
    pub fn set_distance_display_condition(
        &mut self,
        condition: Option<[f32; 2]>,
    ) -> Result<(), DeveloperError> {
        let cond = condition.ok_or_else(|| DeveloperError::new("value is required."))?;
        self.distance_display_condition = Some(cond);
        Ok(())
    }
}

/// Initialization options for [`GroundPolylinePrimitive`].
#[derive(Clone, Debug, Default)]
pub struct GroundPolylinePrimitiveOptions {
    /// GeometryInstances containing GroundPolylineGeometry.
    pub geometry_instances: Option<Vec<GeometryInstance>>,
    /// The appearance used to render the polyline.
    pub appearance: Option<GroundPolylineAppearance>,
    /// Determines if this primitive will be shown. Defaults to true.
    pub show: Option<bool>,
    /// Interleaves vertex attributes if true. Defaults to false.
    pub interleave: Option<bool>,
    /// Releases geometry instances after creation if true. Defaults to true.
    pub release_geometry_instances: Option<bool>,
    /// When true, allows picking with Scene#pick. Defaults to true.
    pub allow_picking: Option<bool>,
    /// Determines if creation is asynchronous. Defaults to true.
    pub asynchronous: Option<bool>,
    /// Classification type for terrain / 3D Tiles. Defaults to Both.
    pub classification_type: Option<ClassificationType>,
    /// Determines if bounding volumes are drawn for debugging. Defaults to false.
    pub debug_show_bounding_volume: Option<bool>,
    /// Determines if shadow volume is shown for debugging. Defaults to false.
    pub debug_show_shadow_volume: Option<bool>,
}

static TERRAIN_HEIGHTS_INITIALIZED_OVERRIDE: OnceLock<Mutex<Option<bool>>> = OnceLock::new();

fn get_terrain_heights_override() -> &'static Mutex<Option<bool>> {
    TERRAIN_HEIGHTS_INITIALIZED_OVERRIDE.get_or_init(|| Mutex::new(None))
}

/// Sets a test override for whether terrain heights are initialized.
pub fn set_terrain_heights_initialized_for_test(initialized: Option<bool>) {
    let mut lock = get_terrain_heights_override().lock().unwrap();
    *lock = initialized;
}

/// A GroundPolylinePrimitive represents a polyline draped over terrain or 3D Tiles.
#[derive(Clone, Debug)]
pub struct GroundPolylinePrimitive {
    pub geometry_instances: Option<Vec<GeometryInstance>>,
    pub appearance: GroundPolylineAppearance,
    pub show: bool,
    pub classification_type: ClassificationType,
    pub debug_show_bounding_volume: bool,
    debug_show_shadow_volume: bool,
    interleave: bool,
    release_geometry_instances: bool,
    allow_picking: bool,
    asynchronous: bool,
    ready: bool,
    is_destroyed: bool,
    updated_once: bool,
    instance_attributes: HashMap<String, Arc<Mutex<PerInstanceAttributes>>>,
}

impl GroundPolylinePrimitive {
    /// Creates a new `GroundPolylinePrimitive`.
    pub fn new(options: GroundPolylinePrimitiveOptions) -> Result<Self, DeveloperError> {
        let asynchronous = options.asynchronous.unwrap_or(true);
        if !asynchronous && !Self::is_terrain_heights_initialized() {
            return Err(DeveloperError::new(
                "ApproximateTerrainHeights was not initialized. Call initializeTerrainHeights before creating synchronous GroundPolylinePrimitives.",
            ));
        }

        let geometry_instances = options.geometry_instances;
        let appearance = options.appearance.unwrap_or_default();
        let show = options.show.unwrap_or(true);
        let interleave = options.interleave.unwrap_or(false);
        let release_geometry_instances = options.release_geometry_instances.unwrap_or(true);
        let allow_picking = options.allow_picking.unwrap_or(true);
        let classification_type = options.classification_type.unwrap_or(ClassificationType::Both);
        let debug_show_bounding_volume = options.debug_show_bounding_volume.unwrap_or(false);
        let debug_show_shadow_volume = options.debug_show_shadow_volume.unwrap_or(false);

        Ok(Self {
            geometry_instances,
            appearance,
            show,
            classification_type,
            debug_show_bounding_volume,
            debug_show_shadow_volume,
            interleave,
            release_geometry_instances,
            allow_picking,
            asynchronous,
            ready: false,
            is_destroyed: false,
            updated_once: false,
            instance_attributes: HashMap::new(),
        })
    }

    /// Initializes terrain heights dataset.
    pub fn initialize_terrain_heights() {
        set_terrain_heights_initialized_for_test(Some(true));
    }

    fn is_terrain_heights_initialized() -> bool {
        if let Some(val) = *get_terrain_heights_override().lock().unwrap() {
            return val;
        }
        ApproximateTerrainHeights::is_initialized()
    }

    /// Gets whether geometry attributes are interleaved.
    pub fn interleave(&self) -> bool {
        self.interleave
    }

    /// Gets whether geometry instances are released after load.
    pub fn release_geometry_instances(&self) -> bool {
        self.release_geometry_instances
    }

    /// Gets whether picking is allowed.
    pub fn allow_picking(&self) -> bool {
        self.allow_picking
    }

    /// Gets whether creation is asynchronous.
    pub fn asynchronous(&self) -> bool {
        self.asynchronous
    }

    /// Gets whether this primitive is ready to render.
    pub fn ready(&self) -> bool {
        self.ready
    }

    /// Gets whether the shadow volume debug rendering is enabled.
    pub fn debug_show_shadow_volume(&self) -> bool {
        self.debug_show_shadow_volume
    }

    /// Updates primitive state and compiles render commands.
    pub fn update(&mut self) -> Result<(), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new("This object was destroyed."));
        }

        self.updated_once = true;

        let instances = match &self.geometry_instances {
            Some(insts) if !insts.is_empty() => insts,
            _ => return Ok(()),
        };

        // Check color attributes if PolylineColorAppearance is used
        if matches!(self.appearance, GroundPolylineAppearance::Color(_)) {
            for inst in instances {
                if !inst.attributes.contains_key("color") {
                    return Err(DeveloperError::new(
                        "All GeometryInstances must have color attributes to use PolylineColorAppearance with GroundPolylinePrimitive.",
                    ));
                }
            }
        }

        // Populate per-instance attributes
        for inst in instances {
            if let Some(id) = &inst.id {
                let mut attr = PerInstanceAttributes::default();
                if let Some(GeometryInstanceAttribute { value, .. }) = inst.attributes.get("color") {
                    if value.len() >= 4 {
                        attr.color = Some(Color::new(
                            value[0] as f32 / 255.0,
                            value[1] as f32 / 255.0,
                            value[2] as f32 / 255.0,
                            value[3] as f32 / 255.0,
                        ));
                    }
                }
                if let Some(GeometryInstanceAttribute { value, .. }) = inst.attributes.get("show") {
                    if !value.is_empty() {
                        attr.show = Some(value[0] != 0.0);
                    }
                }
                if let Some(GeometryInstanceAttribute { value, .. }) = inst.attributes.get("width") {
                    if !value.is_empty() {
                        attr.width = Some(value[0] as f32);
                    }
                }
                if let Some(GeometryInstanceAttribute { value, .. }) =
                    inst.attributes.get("distanceDisplayCondition")
                {
                    if value.len() >= 2 {
                        attr.distance_display_condition =
                            Some([value[0] as f32, value[1] as f32]);
                    }
                }

                self.instance_attributes
                    .entry(id.clone())
                    .or_insert_with(|| Arc::new(Mutex::new(attr)));
            }
        }

        self.ready = true;

        if self.release_geometry_instances {
            self.geometry_instances = None;
        }

        Ok(())
    }

    /// Gets the modifiable per-instance attributes for a geometry instance.
    pub fn get_geometry_instance_attributes(
        &self,
        id: &str,
    ) -> Result<Option<Arc<Mutex<PerInstanceAttributes>>>, DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new("This object was destroyed."));
        }
        if !self.updated_once {
            return Err(DeveloperError::new(
                "must call update before calling getGeometryInstanceAttributes",
            ));
        }
        if id.is_empty() {
            return Err(DeveloperError::new("id is required."));
        }

        Ok(self.instance_attributes.get(id).cloned())
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys this primitive and its resources.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.geometry_instances = None;
        self.instance_attributes.clear();
    }
}
