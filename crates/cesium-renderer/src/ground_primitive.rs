//! GroundPrimitive matching CesiumJS `Scene/GroundPrimitive.js` (Apache-2.0).

use crate::ground_polyline_primitive::PerInstanceAttributes;
use cesium_math::{
    ApproximateTerrainHeights, BoundingSphere, ClassificationType, Color, DeveloperError,
    Ellipsoid, GeoRectangle, GeometryInstance, GeometryInstanceAttribute,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Material appearance for surface geometry.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct EllipsoidSurfaceAppearance {
    pub translucent: bool,
    pub flat: bool,
}

/// Per-instance color appearance for surface geometry.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PerInstanceColorAppearance {
    pub translucent: bool,
    pub flat: bool,
}

/// Appearance enumeration for GroundPrimitive.
#[derive(Clone, Debug, PartialEq)]
pub enum GroundPrimitiveAppearance {
    PerInstanceColor(PerInstanceColorAppearance),
    EllipsoidSurface(EllipsoidSurfaceAppearance),
}

impl Default for GroundPrimitiveAppearance {
    fn default() -> Self {
        Self::PerInstanceColor(PerInstanceColorAppearance {
            translucent: false,
            flat: true,
        })
    }
}

/// Initialization options for [`GroundPrimitive`].
#[derive(Clone, Debug, Default)]
pub struct GroundPrimitiveOptions {
    /// The geometry instances to render.
    pub geometry_instances: Option<Vec<GeometryInstance>>,
    /// The appearance used to render the primitive.
    pub appearance: Option<GroundPrimitiveAppearance>,
    /// Determines if this primitive will be shown. Defaults to true.
    pub show: Option<bool>,
    /// Optimizes vertices for pre and post vertex shader caches. Defaults to false.
    pub vertex_cache_optimize: Option<bool>,
    /// Interleaves vertex attributes if true. Defaults to false.
    pub interleave: Option<bool>,
    /// Releases geometry instances after creation if true. Defaults to true.
    pub release_geometry_instances: Option<bool>,
    /// When true, allows picking with Scene#pick. Defaults to true.
    pub allow_picking: Option<bool>,
    /// Determines if creation is asynchronous. Defaults to true.
    pub asynchronous: Option<bool>,
    /// Compresses geometry vertices if true. Defaults to true.
    pub compress_vertices: Option<bool>,
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
pub fn set_ground_primitive_terrain_heights_initialized_for_test(initialized: Option<bool>) {
    let mut lock = get_terrain_heights_override().lock().unwrap();
    *lock = initialized;
}

/// A ground primitive represents geometry draped over terrain or 3D Tiles.
#[derive(Clone, Debug)]
pub struct GroundPrimitive {
    pub geometry_instances: Option<Vec<GeometryInstance>>,
    pub appearance: Option<GroundPrimitiveAppearance>,
    pub show: bool,
    pub classification_type: ClassificationType,
    pub debug_show_bounding_volume: bool,
    pub debug_show_shadow_volume: bool,
    vertex_cache_optimize: bool,
    interleave: bool,
    release_geometry_instances: bool,
    allow_picking: bool,
    asynchronous: bool,
    compress_vertices: bool,
    ready: bool,
    is_destroyed: bool,
    updated_once: bool,
    bounding_spheres: HashMap<String, BoundingSphere>,
    instance_attributes: HashMap<String, Arc<Mutex<PerInstanceAttributes>>>,
}

impl GroundPrimitive {
    /// Creates a new `GroundPrimitive`.
    pub fn new(options: GroundPrimitiveOptions) -> Result<Self, DeveloperError> {
        let asynchronous = options.asynchronous.unwrap_or(true);
        if !asynchronous && !Self::is_terrain_heights_initialized() {
            return Err(DeveloperError::new(
                "ApproximateTerrainHeights was not initialized. Call initializeTerrainHeights before creating synchronous GroundPrimitives.",
            ));
        }

        let mut appearance = options.appearance;
        let geometry_instances = options.geometry_instances;

        if appearance.is_none() {
            if let Some(instances) = &geometry_instances {
                for inst in instances {
                    if inst.attributes.contains_key("color") {
                        appearance = Some(GroundPrimitiveAppearance::PerInstanceColor(
                            PerInstanceColorAppearance {
                                translucent: false,
                                flat: true,
                            },
                        ));
                        break;
                    }
                }
            }
        }

        let show = options.show.unwrap_or(true);
        let vertex_cache_optimize = options.vertex_cache_optimize.unwrap_or(false);
        let interleave = options.interleave.unwrap_or(false);
        let release_geometry_instances = options.release_geometry_instances.unwrap_or(true);
        let allow_picking = options.allow_picking.unwrap_or(true);
        let compress_vertices = options.compress_vertices.unwrap_or(true);
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
            vertex_cache_optimize,
            interleave,
            release_geometry_instances,
            allow_picking,
            asynchronous,
            compress_vertices,
            ready: false,
            is_destroyed: false,
            updated_once: false,
            bounding_spheres: HashMap::new(),
            instance_attributes: HashMap::new(),
        })
    }

    /// Initializes terrain heights dataset.
    pub fn initialize_terrain_heights() {
        set_ground_primitive_terrain_heights_initialized_for_test(Some(true));
    }

    fn is_terrain_heights_initialized() -> bool {
        if let Some(val) = *get_terrain_heights_override().lock().unwrap() {
            return val;
        }
        ApproximateTerrainHeights::is_initialized()
    }

    /// Gets whether vertex cache optimization is enabled.
    pub fn vertex_cache_optimize(&self) -> bool {
        self.vertex_cache_optimize
    }

    /// Gets whether vertex attributes are interleaved.
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

    /// Gets whether vertices are compressed.
    pub fn compress_vertices(&self) -> bool {
        self.compress_vertices
    }

    /// Gets whether this primitive is ready to render.
    pub fn ready(&self) -> bool {
        self.ready
    }

    /// Checks if GroundPrimitive is supported in the current scene context.
    pub fn is_supported(depth_texture_supported: bool) -> bool {
        depth_texture_supported
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

        // Validate batched instances colors
        if instances.len() > 1 {
            let mut first_color: Option<Vec<f64>> = None;
            for inst in instances {
                let col = inst.attributes.get("color");
                if col.is_none() {
                    return Err(DeveloperError::new(
                        "All batched instances must have a color attribute.",
                    ));
                }
                let val = col.unwrap().value.clone();
                if let Some(ref first) = first_color {
                    if first != &val {
                        return Err(DeveloperError::new(
                            "Batched instance colors must be identical when depth textures are not supported.",
                        ));
                    }
                } else {
                    first_color = Some(val);
                }
            }
        }

        // Populate per-instance attributes and bounding spheres
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

                // Compute bounding sphere from approximate terrain heights
                let sphere = ApproximateTerrainHeights::get_bounding_sphere(
                    &GeoRectangle::MAXIMUM,
                    Some(Ellipsoid::WGS84),
                );
                self.bounding_spheres.insert(id.clone(), sphere);
            }
        }

        self.ready = true;

        if self.release_geometry_instances {
            self.geometry_instances = None;
        }

        Ok(())
    }

    /// Gets the bounding sphere of the geometry instance with given ID.
    pub fn get_bounding_sphere(&self, id: &str) -> Option<BoundingSphere> {
        self.bounding_spheres.get(id).copied()
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
        self.bounding_spheres.clear();
    }
}
