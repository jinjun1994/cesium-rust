//! TileCoordinatesImageryProvider matching CesiumJS `Scene/TileCoordinatesImageryProvider.js` (Apache-2.0).

use cesium_math::{Color, Ellipsoid, GeoRectangle, GeographicTilingScheme};

use crate::imagery_layer::ImageryProvider;
use crate::quadtree_tile::AnyTilingScheme;

/// Construction options for `TileCoordinatesImageryProvider`.
#[derive(Clone, Debug, Default)]
pub struct TileCoordinatesImageryProviderOptions {
    pub tiling_scheme: Option<AnyTilingScheme>,
    pub ellipsoid: Option<Ellipsoid>,
    pub color: Option<Color>,
    pub tile_width: Option<u32>,
    pub tile_height: Option<u32>,
}

/// An `ImageryProvider` that draws a box around every rendered tile in the tiling scheme
/// and renders coordinate information. Mostly useful for debugging terrain and imagery rendering.
#[derive(Clone, Debug)]
pub struct TileCoordinatesImageryProvider {
    tiling_scheme: AnyTilingScheme,
    color: Color,
    tile_width: u32,
    tile_height: u32,
    has_alpha_channel: bool,
}

impl Default for TileCoordinatesImageryProvider {
    fn default() -> Self {
        Self::new(None)
    }
}

impl TileCoordinatesImageryProvider {
    /// Creates a new `TileCoordinatesImageryProvider`.
    pub fn new(options: Option<TileCoordinatesImageryProviderOptions>) -> Self {
        let opts = options.unwrap_or_default();
        let ellipsoid = opts.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let tiling_scheme = opts.tiling_scheme.unwrap_or_else(|| {
            let geo = GeographicTilingScheme::new(ellipsoid, GeoRectangle::MAXIMUM, 2, 1)
                .unwrap_or_else(|_| GeographicTilingScheme::default());
            AnyTilingScheme::Geographic(geo)
        });
        let color = opts.color.unwrap_or(Color::YELLOW);
        let tile_width = opts.tile_width.unwrap_or(256);
        let tile_height = opts.tile_height.unwrap_or(256);

        Self {
            tiling_scheme,
            color,
            tile_width,
            tile_height,
            has_alpha_channel: true,
        }
    }

    pub fn tiling_scheme(&self) -> &AnyTilingScheme {
        &self.tiling_scheme
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn tile_width(&self) -> u32 {
        self.tile_width
    }

    pub fn tile_height(&self) -> u32 {
        self.tile_height
    }

    pub fn maximum_level(&self) -> Option<u32> {
        None
    }

    pub fn minimum_level(&self) -> Option<u32> {
        None
    }

    pub fn rectangle(&self) -> GeoRectangle {
        self.tiling_scheme.rectangle()
    }

    pub fn has_alpha_channel(&self) -> bool {
        self.has_alpha_channel
    }

    /// Requests an image for a given tile coordinate, drawing a border box and coordinate indicator.
    pub fn request_image(&self, x: u32, y: u32, level: u32) -> Option<Vec<u8>> {
        let width = self.tile_width as usize;
        let height = self.tile_height as usize;
        let mut pixels = vec![0_u8; width * height * 4];

        let r = (self.color.red * 255.0) as u8;
        let g = (self.color.green * 255.0) as u8;
        let b = (self.color.blue * 255.0) as u8;
        let a = (self.color.alpha * 255.0) as u8;

        // Draw 2-pixel outer border
        for py in 0..height {
            for px in 0..width {
                let is_border = px < 2 || px >= width - 2 || py < 2 || py >= height - 2;
                if is_border {
                    let idx = (py * width + px) * 4;
                    pixels[idx] = r;
                    pixels[idx + 1] = g;
                    pixels[idx + 2] = b;
                    pixels[idx + 3] = a;
                }
            }
        }

        // Draw simple hash tick for level/x/y in center area
        let hash = (level.wrapping_mul(73856093) ^ x.wrapping_mul(19349663) ^ y.wrapping_mul(83492791)) as usize;
        let cx = width / 2;
        let cy = height / 2;
        for dy in 0..8 {
            for dx in 0..8 {
                if (hash >> (dx + dy)) & 1 == 1 {
                    let py = cy - 4 + dy;
                    let px = cx - 4 + dx;
                    if px < width && py < height {
                        let idx = (py * width + px) * 4;
                        pixels[idx] = r;
                        pixels[idx + 1] = g;
                        pixels[idx + 2] = b;
                        pixels[idx + 3] = a;
                    }
                }
            }
        }

        Some(pixels)
    }
}

impl ImageryProvider for TileCoordinatesImageryProvider {
    fn rectangle(&self) -> GeoRectangle {
        self.tiling_scheme.rectangle()
    }

    fn tile_width(&self) -> u32 {
        self.tile_width
    }

    fn tile_height(&self) -> u32 {
        self.tile_height
    }

    fn minimum_level(&self) -> u32 {
        0
    }

    fn maximum_level(&self) -> Option<u32> {
        None
    }

    fn has_alpha_channel(&self) -> bool {
        self.has_alpha_channel
    }

    fn ellipsoid(&self) -> Ellipsoid {
        self.tiling_scheme.ellipsoid()
    }
}
