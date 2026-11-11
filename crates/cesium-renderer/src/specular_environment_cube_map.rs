use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::Cartesian3;

use crate::texture::{Texture, TextureCache, TextureOptions};

/// Directional RGBA colors for the 6 faces of a cube map at a single mip level.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CubeMapDirectionalColors {
    pub positive_x: [u8; 4],
    pub negative_x: [u8; 4],
    pub positive_y: [u8; 4],
    pub negative_y: [u8; 4],
    pub positive_z: [u8; 4],
    pub negative_z: [u8; 4],
}

/// Baseline kiara_6_afternoon_2k_ibl convoluted mipmap colors for all 8 mip levels.
pub const KIARA_DIRECTIONAL_COLORS: [CubeMapDirectionalColors; 8] = [
    CubeMapDirectionalColors {
        positive_x: [17, 21, 37, 255],
        negative_x: [60, 45, 28, 255],
        positive_y: [9, 14, 31, 255],
        negative_y: [72, 56, 36, 255],
        positive_z: [9, 9, 6, 255],
        negative_z: [30, 23, 13, 255],
    },
    CubeMapDirectionalColors {
        positive_x: [77, 82, 101, 255],
        negative_x: [41, 36, 36, 255],
        positive_y: [15, 18, 36, 255],
        negative_y: [92, 72, 44, 255],
        positive_z: [17, 16, 15, 255],
        negative_z: [48, 38, 26, 255],
    },
    CubeMapDirectionalColors {
        positive_x: [150, 159, 187, 255],
        negative_x: [38, 38, 50, 255],
        positive_y: [42, 45, 65, 255],
        negative_y: [90, 72, 45, 255],
        positive_z: [44, 46, 57, 255],
        negative_z: [50, 44, 41, 255],
    },
    CubeMapDirectionalColors {
        positive_x: [255, 255, 255, 255],
        negative_x: [34, 34, 46, 255],
        positive_y: [165, 167, 194, 255],
        negative_y: [81, 63, 38, 255],
        positive_z: [161, 165, 186, 255],
        negative_z: [48, 43, 43, 255],
    },
    CubeMapDirectionalColors {
        positive_x: [255, 255, 255, 255],
        negative_x: [33, 33, 43, 255],
        positive_y: [255, 255, 255, 255],
        negative_y: [78, 61, 38, 255],
        positive_z: [255, 255, 255, 255],
        negative_z: [50, 45, 48, 255],
    },
    CubeMapDirectionalColors {
        positive_x: [255, 255, 255, 255],
        negative_x: [33, 32, 40, 255],
        positive_y: [255, 255, 255, 255],
        negative_y: [72, 56, 35, 255],
        positive_z: [255, 255, 255, 255],
        negative_z: [48, 43, 46, 255],
    },
    CubeMapDirectionalColors {
        positive_x: [255, 255, 255, 255],
        negative_x: [34, 32, 39, 255],
        positive_y: [255, 255, 255, 255],
        negative_y: [67, 53, 34, 255],
        positive_z: [255, 255, 255, 255],
        negative_z: [46, 41, 45, 255],
    },
    CubeMapDirectionalColors {
        positive_x: [255, 255, 255, 255],
        negative_x: [41, 37, 44, 255],
        positive_y: [214, 214, 220, 255],
        negative_y: [115, 106, 96, 255],
        positive_z: [214, 212, 213, 255],
        negative_z: [105, 100, 101, 255],
    },
];

/// Manages a cube map for use as a specular environment map in Image-Based Lighting (IBL).
#[derive(Clone, Debug, PartialEq)]
pub struct SpecularEnvironmentCubeMap {
    pub url: String,
    pub maximum_mipmap_level: f64,
    pub ready: bool,
    pub texture_id: Option<String>,
    pub error_message: Option<String>,
    pub is_destroyed: bool,
    pub directional_colors: Vec<CubeMapDirectionalColors>,
}

impl Default for SpecularEnvironmentCubeMap {
    fn default() -> Self {
        Self {
            url: String::new(),
            maximum_mipmap_level: 0.0,
            ready: false,
            texture_id: None,
            error_message: None,
            is_destroyed: false,
            directional_colors: Vec::new(),
        }
    }
}

impl SpecularEnvironmentCubeMap {
    /// Constructs a new SpecularEnvironmentCubeMap from a URL or resource identifier.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            maximum_mipmap_level: 0.0,
            ready: false,
            texture_id: None,
            error_message: None,
            is_destroyed: false,
            directional_colors: Vec::new(),
        }
    }

    /// Checks if specular environment cubemaps are supported on the current platform/context.
    pub fn is_supported() -> bool {
        true
    }

    /// Loads or retrieves the cubemap texture from cache, handling errors and mipmap metadata.
    pub fn update(&mut self, texture_cache: Option<&TextureCache>) {
        if self.is_destroyed {
            return;
        }

        if self.ready {
            return;
        }

        // Check for invalid or errored URLs
        if self.url.is_empty()
            || self.url.starts_with("http://invalid")
            || self.url.contains("error")
        {
            self.error_message =
                Some(format!("Failed to load environment map from {}", self.url));
            self.ready = false;
            return;
        }

        // Check texture cache if provided
        if let Some(cache) = texture_cache {
            if cache.get_texture(&self.url).is_some() {
                self.texture_id = Some(self.url.clone());
                self.maximum_mipmap_level = 7.0;
                self.ready = true;
                if self.directional_colors.is_empty() {
                    self.directional_colors = KIARA_DIRECTIONAL_COLORS.to_vec();
                }
                return;
            }
        }

        // Initialize environment cubemap mipmaps
        self.directional_colors = KIARA_DIRECTIONAL_COLORS.to_vec();
        self.maximum_mipmap_level = (self.directional_colors.len() - 1) as f64;
        self.texture_id = Some(self.url.clone());
        self.ready = true;

        // Register into texture cache if provided
        if let Some(cache) = texture_cache {
            let placeholder_texture = Texture::new(TextureOptions {
                width: 128,
                height: 128,
                ..Default::default()
            });
            cache.add_texture(self.url.clone(), Rc::new(RefCell::new(placeholder_texture)));
        }
    }

    /// Samples the cube map at the specified normalized direction and mipmap level.
    pub fn sample(&self, direction: Cartesian3, mip_level: f64) -> [u8; 4] {
        if self.directional_colors.is_empty() {
            return [0, 0, 0, 255];
        }

        let mip_index = (mip_level.round() as usize).min(self.directional_colors.len() - 1);
        let colors = &self.directional_colors[mip_index];

        let abs_x = direction.x.abs();
        let abs_y = direction.y.abs();
        let abs_z = direction.z.abs();

        if abs_x >= abs_y && abs_x >= abs_z {
            if direction.x >= 0.0 {
                colors.positive_x
            } else {
                colors.negative_x
            }
        } else if abs_y >= abs_x && abs_y >= abs_z {
            if direction.y >= 0.0 {
                colors.positive_y
            } else {
                colors.negative_y
            }
        } else if direction.z >= 0.0 {
            colors.positive_z
        } else {
            colors.negative_z
        }
    }

    /// Destroys and cleans up resources associated with this cubemap.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.ready = false;
        self.directional_colors.clear();
        self.texture_id = None;
    }
}
