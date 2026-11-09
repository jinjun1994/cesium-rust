//! CloudCollection and CumulusCloud matching CesiumJS `Scene/CloudCollection.js` and `Scene/CumulusCloud.js` (Apache-2.0).

use crate::commands::DrawCommand;
use crate::compute_command::ComputeCommand;
use cesium_math::{Cartesian2, Cartesian3, Color, DeveloperError};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_CLOUD_ID: AtomicU64 = AtomicU64::new(1);

/// Specifies the type of cloud that is added to a `CloudCollection`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CloudType {
    /// Cumulus cloud.
    #[default]
    Cumulus = 0,
}

impl CloudType {
    /// Validates that the provided cloud type is valid.
    pub fn validate(cloud_type: i32) -> bool {
        cloud_type == Self::Cumulus as i32
    }
}

/// Initialization options for a `CumulusCloud`.
#[derive(Debug, Clone, Default)]
pub struct CumulusCloudOptions {
    /// The type of cloud. Defaults to Cumulus.
    pub cloud_type: Option<CloudType>,
    /// Determines if this cumulus cloud will be shown. Defaults to true.
    pub show: Option<bool>,
    /// The Cartesian position of this cumulus cloud. Defaults to Cartesian3::ZERO.
    pub position: Option<Cartesian3>,
    /// The scale of the cumulus cloud billboard in meters. Defaults to Cartesian2(20.0, 12.0).
    pub scale: Option<Cartesian2>,
    /// The maximum size of the cumulus cloud rendered on the billboard.
    pub maximum_size: Option<Cartesian3>,
    /// The slice of the cloud that is rendered on the billboard. Defaults to -1.0.
    pub slice: Option<f64>,
    /// The brightness of the cloud. Defaults to 1.0.
    pub brightness: Option<f64>,
    /// Sets the color of the cloud. Defaults to Color::WHITE.
    pub color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq)]
struct CumulusCloudData {
    id: u64,
    show: bool,
    position: Cartesian3,
    scale: Cartesian2,
    maximum_size: Cartesian3,
    slice: f64,
    brightness: f64,
    color: Color,
    cloud_type: CloudType,
}

/// A cumulus cloud billboard positioned in the 3D scene, managed by `CloudCollection`.
#[derive(Debug, Clone)]
pub struct CumulusCloud {
    data: Rc<RefCell<CumulusCloudData>>,
}

impl CumulusCloud {
    /// Creates a new CumulusCloud from options.
    fn new(options: Option<CumulusCloudOptions>) -> Result<Self, DeveloperError> {
        let options = options.unwrap_or_default();

        let cloud_type = options.cloud_type.unwrap_or(CloudType::Cumulus);
        let show = options.show.unwrap_or(true);
        let position = options.position.unwrap_or(Cartesian3::ZERO);

        let (scale, maximum_size) = match (options.scale, options.maximum_size) {
            (None, Some(max_size)) => (Cartesian2::new(max_size.x, max_size.y), max_size),
            (Some(s), Some(max_size)) => (s, max_size),
            (Some(s), None) => {
                let max_size = Cartesian3::new(s.x, s.y, s.x.min(s.y) / 1.5);
                (s, max_size)
            }
            (None, None) => {
                let s = Cartesian2::new(20.0, 12.0);
                let max_size = Cartesian3::new(20.0, 12.0, 8.0);
                (s, max_size)
            }
        };

        let slice = options.slice.unwrap_or(-1.0);
        let brightness = options.brightness.unwrap_or(1.0);
        let color = options.color.unwrap_or(Color::WHITE);

        let id = NEXT_CLOUD_ID.fetch_add(1, Ordering::Relaxed);

        Ok(Self {
            data: Rc::new(RefCell::new(CumulusCloudData {
                id,
                show,
                position,
                scale,
                maximum_size,
                slice,
                brightness,
                color,
                cloud_type,
            })),
        })
    }

    /// Gets cloud unique id.
    pub fn id(&self) -> u64 {
        self.data.borrow().id
    }

    /// Determines if this cumulus cloud will be shown.
    pub fn show(&self) -> bool {
        self.data.borrow().show
    }

    /// Sets whether this cumulus cloud will be shown.
    pub fn set_show(&self, show: bool) {
        self.data.borrow_mut().show = show;
    }

    /// Gets the Cartesian position of this cumulus cloud.
    pub fn position(&self) -> Cartesian3 {
        self.data.borrow().position
    }

    /// Sets the Cartesian position of this cumulus cloud.
    pub fn set_position(&self, position: Cartesian3) {
        self.data.borrow_mut().position = position;
    }

    /// Gets the scale of the cumulus cloud billboard in meters.
    pub fn scale(&self) -> Cartesian2 {
        self.data.borrow().scale
    }

    /// Sets the scale of the cumulus cloud billboard in meters.
    pub fn set_scale(&self, scale: Cartesian2) {
        self.data.borrow_mut().scale = scale;
    }

    /// Gets the maximum size of the cumulus cloud.
    pub fn maximum_size(&self) -> Cartesian3 {
        self.data.borrow().maximum_size
    }

    /// Sets the maximum size of the cumulus cloud.
    pub fn set_maximum_size(&self, maximum_size: Cartesian3) {
        self.data.borrow_mut().maximum_size = maximum_size;
    }

    /// Gets the slice of the cloud rendered on billboard.
    pub fn slice(&self) -> f64 {
        self.data.borrow().slice
    }

    /// Sets the slice of the cloud.
    pub fn set_slice(&self, slice: f64) {
        self.data.borrow_mut().slice = slice;
    }

    /// Gets the brightness of the cloud.
    pub fn brightness(&self) -> f64 {
        self.data.borrow().brightness
    }

    /// Sets the brightness of the cloud.
    pub fn set_brightness(&self, brightness: f64) {
        self.data.borrow_mut().brightness = brightness;
    }

    /// Gets the color of the cloud.
    pub fn color(&self) -> Color {
        self.data.borrow().color
    }

    /// Sets the color of the cloud.
    pub fn set_color(&self, color: Color) {
        self.data.borrow_mut().color = color;
    }

    /// Gets the cloud type.
    pub fn cloud_type(&self) -> CloudType {
        self.data.borrow().cloud_type
    }
}

impl PartialEq for CumulusCloud {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
    }
}

/// Initialization options for `CloudCollection`.
#[derive(Debug, Clone, Default)]
pub struct CloudCollectionOptions {
    /// Whether to display the clouds. Defaults to true.
    pub show: Option<bool>,
    /// Desired amount of detail in the noise texture. Defaults to 16.0.
    pub noise_detail: Option<f64>,
    /// Desired translation of data in noise texture. Defaults to Cartesian3::ZERO.
    pub noise_offset: Option<Cartesian3>,
    /// For debugging only. Determines if billboards are rendered with an opaque color. Defaults to false.
    pub debug_billboards: Option<bool>,
    /// For debugging only. Determines if clouds will be rendered as opaque ellipsoids. Defaults to false.
    pub debug_ellipsoids: Option<bool>,
    /// For debugging only. Determines if clouds will be rendered as debug clouds. Defaults to false.
    pub debug_clouds: Option<bool>,
}

/// A renderable collection of clouds in the 3D scene.
#[derive(Debug, Clone)]
pub struct CloudCollection {
    clouds: Vec<CumulusCloud>,
    /// Determines if billboards in this collection will be shown.
    pub show: bool,
    /// Controls the amount of detail in precomputed noise texture.
    pub noise_detail: f64,
    /// Applies a translation to noise texture coordinates.
    pub noise_offset: Cartesian3,
    /// Debug option to render billboards with opaque color.
    pub debug_billboards: bool,
    /// Debug option to draw clouds as opaque ellipsoids.
    pub debug_ellipsoids: bool,
    /// Debug option to draw debug clouds.
    pub debug_clouds: bool,

    // Internal noise texture layout
    texture_slice_width: u32,
    noise_texture_rows: u32,
    ready: bool,
    is_destroyed: bool,
}

impl Default for CloudCollection {
    fn default() -> Self {
        Self::new(CloudCollectionOptions::default())
    }
}

impl CloudCollection {
    /// Creates a new `CloudCollection`.
    pub fn new(options: CloudCollectionOptions) -> Self {
        Self {
            clouds: Vec::new(),
            show: options.show.unwrap_or(true),
            noise_detail: options.noise_detail.unwrap_or(16.0),
            noise_offset: options.noise_offset.unwrap_or(Cartesian3::ZERO),
            debug_billboards: options.debug_billboards.unwrap_or(false),
            debug_ellipsoids: options.debug_ellipsoids.unwrap_or(false),
            debug_clouds: options.debug_clouds.unwrap_or(false),
            texture_slice_width: 128,
            noise_texture_rows: 4,
            ready: false,
            is_destroyed: false,
        }
    }

    /// Adds a cloud to the collection.
    pub fn add(
        &mut self,
        options: Option<CumulusCloudOptions>,
    ) -> Result<CumulusCloud, DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        let cloud = CumulusCloud::new(options)?;
        self.clouds.push(cloud.clone());
        Ok(cloud)
    }

    /// Removes a cloud from the collection.
    pub fn remove(&mut self, cloud: Option<&CumulusCloud>) -> bool {
        if self.is_destroyed {
            return false;
        }

        let Some(cloud) = cloud else {
            return false;
        };

        if let Some(pos) = self.clouds.iter().position(|c| c == cloud) {
            self.clouds.remove(pos);
            true
        } else {
            false
        }
    }

    /// Removes all clouds from the collection.
    pub fn remove_all(&mut self) {
        self.clouds.clear();
    }

    /// Checks if this collection contains the specified cloud.
    pub fn contains(&self, cloud: Option<&CumulusCloud>) -> bool {
        let Some(cloud) = cloud else {
            return false;
        };
        self.clouds.iter().any(|c| c == cloud)
    }

    /// Returns the cloud at the specified index.
    pub fn get(&self, index: Option<usize>) -> Result<CumulusCloud, DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        let index = index.ok_or_else(|| DeveloperError::new("index is required."))?;
        self.clouds
            .get(index)
            .cloned()
            .ok_or_else(|| DeveloperError::new("index out of range."))
    }

    /// Returns the number of clouds in the collection.
    pub fn len(&self) -> usize {
        self.clouds.len()
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.clouds.is_empty()
    }

    /// Gets internal texture slice width.
    pub fn texture_slice_width(&self) -> u32 {
        self.texture_slice_width
    }

    /// Sets internal texture slice width.
    pub fn set_texture_slice_width(&mut self, width: u32) {
        self.texture_slice_width = width;
    }

    /// Gets internal noise texture rows.
    pub fn noise_texture_rows(&self) -> u32 {
        self.noise_texture_rows
    }

    /// Sets internal noise texture rows.
    pub fn set_noise_texture_rows(&mut self, rows: u32) {
        self.noise_texture_rows = rows;
    }

    /// Gets whether the cloud collection texture is ready.
    pub fn ready(&self) -> bool {
        self.ready
    }

    /// Sets whether the cloud collection texture is ready.
    pub fn set_ready(&mut self, ready: bool) {
        self.ready = ready;
    }

    /// Updates collection and produces compute commands and draw commands.
    pub fn update(
        &mut self,
    ) -> Result<(Option<ComputeCommand>, Option<DrawCommand>), DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        if self.noise_texture_rows == 0
            || self.texture_slice_width / self.noise_texture_rows < 1
            || !self.texture_slice_width.is_multiple_of(self.noise_texture_rows)
        {
            return Err(DeveloperError::new(
                "noiseTextureRows must evenly divide textureSliceWidth",
            ));
        }

        let compute_command = if !self.debug_billboards && !self.ready {
            Some(ComputeCommand::default())
        } else {
            None
        };

        let can_draw = self.show
            && (self.debug_billboards || self.ready)
            && self.clouds.iter().any(|c| {
                let s = c.scale();
                let max_s = c.maximum_size();
                c.show()
                    && s.x > 0.0
                    && s.y > 0.0
                    && (!self.debug_ellipsoids || (max_s.x > 0.0 && max_s.y > 0.0 && max_s.z > 0.0))
                    && (c.slice() < 0.0 || (c.slice() >= 0.0 && c.slice() <= 1.0))
                    && c.brightness() > 0.0
            });

        let draw_command = if can_draw {
            Some(DrawCommand::default())
        } else {
            None
        };

        Ok((compute_command, draw_command))
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys resources.
    pub fn destroy(&mut self) {
        self.clouds.clear();
        self.is_destroyed = true;
    }
}
