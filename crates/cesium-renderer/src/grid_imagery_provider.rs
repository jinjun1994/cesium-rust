//! GridImageryProvider matching CesiumJS `Scene/GridImageryProvider.js` (Apache-2.0).

use cesium_math::{
    Color, Ellipsoid, GeoRectangle, GeographicTilingScheme, TilingScheme, WebMercatorTilingScheme,
};

/// Common tiling scheme wrapper for GridImageryProvider.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridTilingScheme {
    Geographic(GeographicTilingScheme),
    WebMercator(WebMercatorTilingScheme),
}

impl GridTilingScheme {
    /// Gets the ellipsoid of this tiling scheme.
    pub fn ellipsoid(&self) -> Ellipsoid {
        match self {
            Self::Geographic(g) => g.ellipsoid(),
            Self::WebMercator(w) => w.ellipsoid(),
        }
    }

    /// Gets the bounding rectangle of this tiling scheme.
    pub fn rectangle(&self) -> GeoRectangle {
        match self {
            Self::Geographic(g) => g.rectangle(),
            Self::WebMercator(w) => w.rectangle(),
        }
    }
}

impl From<GeographicTilingScheme> for GridTilingScheme {
    fn from(s: GeographicTilingScheme) -> Self {
        Self::Geographic(s)
    }
}

impl From<WebMercatorTilingScheme> for GridTilingScheme {
    fn from(s: WebMercatorTilingScheme) -> Self {
        Self::WebMercator(s)
    }
}

/// Initialization options for [`GridImageryProvider`].
#[derive(Clone, Debug, Default)]
pub struct GridImageryProviderOptions {
    /// The tiling scheme for which to draw tiles. Defaults to GeographicTilingScheme.
    pub tiling_scheme: Option<GridTilingScheme>,
    /// The ellipsoid. Ignored if `tiling_scheme` is specified.
    pub ellipsoid: Option<Ellipsoid>,
    /// The number of grid cells. Defaults to 8.
    pub cells: Option<u32>,
    /// The color to draw grid lines. Defaults to Color(1.0, 1.0, 1.0, 0.4).
    pub color: Option<Color>,
    /// The color to draw glow for grid lines. Defaults to Color(0.0, 1.0, 0.0, 0.05).
    pub glow_color: Option<Color>,
    /// The width of lines used for rendering the line glow effect. Defaults to 6.0.
    pub glow_width: Option<f64>,
    /// Background fill color. Defaults to Color(0.0, 0.5, 0.0, 0.2).
    pub background_color: Option<Color>,
    /// Tile width in pixels. Defaults to 256.
    pub tile_width: Option<u32>,
    /// Tile height in pixels. Defaults to 256.
    pub tile_height: Option<u32>,
    /// Size of the generated canvas in pixels. Defaults to 256.
    pub canvas_size: Option<u32>,
}

/// An ImageryProvider that draws a wireframe grid on every tile with controllable background and glow.
#[derive(Clone, Debug)]
pub struct GridImageryProvider {
    tiling_scheme: GridTilingScheme,
    cells: u32,
    color: Color,
    glow_color: Color,
    glow_width: f64,
    background_color: Color,
    tile_width: u32,
    tile_height: u32,
    canvas_size: u32,
    canvas_image_data: Vec<u8>,
}

const DEFAULT_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.4);
const DEFAULT_GLOW_COLOR: Color = Color::new(0.0, 1.0, 0.0, 0.05);
const DEFAULT_BACKGROUND_COLOR: Color = Color::new(0.0, 0.5, 0.0, 0.2);

impl GridImageryProvider {
    /// Creates a new `GridImageryProvider`.
    pub fn new(options: GridImageryProviderOptions) -> Self {
        let tiling_scheme = options.tiling_scheme.unwrap_or_else(|| {
            let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
            GridTilingScheme::Geographic(
                GeographicTilingScheme::new(ellipsoid, GeoRectangle::MAXIMUM, 2, 1)
                    .unwrap_or_default(),
            )
        });

        let cells = options.cells.unwrap_or(8);
        let color = options.color.unwrap_or(DEFAULT_COLOR);
        let glow_color = options.glow_color.unwrap_or(DEFAULT_GLOW_COLOR);
        let glow_width = options.glow_width.unwrap_or(6.0);
        let background_color = options.background_color.unwrap_or(DEFAULT_BACKGROUND_COLOR);
        let tile_width = options.tile_width.unwrap_or(256);
        let tile_height = options.tile_height.unwrap_or(256);
        let canvas_size = options.canvas_size.unwrap_or(256);

        let canvas_image_data = Self::create_grid_canvas(
            canvas_size,
            cells,
            background_color,
            glow_color,
            glow_width,
            color,
        );

        Self {
            tiling_scheme,
            cells,
            color,
            glow_color,
            glow_width,
            background_color,
            tile_width,
            tile_height,
            canvas_size,
            canvas_image_data,
        }
    }

    /// Gets whether this provider has an alpha channel.
    pub fn has_alpha_channel(&self) -> bool {
        true
    }

    /// Gets the tiling scheme used by this provider.
    pub fn tiling_scheme(&self) -> &GridTilingScheme {
        &self.tiling_scheme
    }

    /// Gets the width of each tile, in pixels.
    pub fn tile_width(&self) -> u32 {
        self.tile_width
    }

    /// Gets the height of each tile, in pixels.
    pub fn tile_height(&self) -> u32 {
        self.tile_height
    }

    /// Gets the number of grid cells.
    pub fn cells(&self) -> u32 {
        self.cells
    }

    /// Gets the grid line color.
    pub fn color(&self) -> Color {
        self.color
    }

    /// Gets the glow color.
    pub fn glow_color(&self) -> Color {
        self.glow_color
    }

    /// Gets the glow width.
    pub fn glow_width(&self) -> f64 {
        self.glow_width
    }

    /// Gets the background fill color.
    pub fn background_color(&self) -> Color {
        self.background_color
    }

    /// Gets the canvas size in pixels.
    pub fn canvas_size(&self) -> u32 {
        self.canvas_size
    }

    /// Gets the maximum level-of-detail that can be requested (None for GridImageryProvider).
    pub fn maximum_level(&self) -> Option<u32> {
        None
    }

    /// Gets the minimum level-of-detail that can be requested (None for GridImageryProvider).
    pub fn minimum_level(&self) -> Option<u32> {
        None
    }

    /// Gets the tile discard policy (None for GridImageryProvider).
    pub fn tile_discard_policy(&self) -> Option<()> {
        None
    }

    /// Gets the rectangle, in radians, of the imagery provided by this instance.
    pub fn rectangle(&self) -> GeoRectangle {
        self.tiling_scheme.rectangle()
    }

    /// Requests the image data (RGBA bytes) for a given tile.
    pub async fn request_image(&self, _x: u32, _y: u32, _level: u32) -> Vec<u8> {
        self.canvas_image_data.clone()
    }

    /// Gets a slice of the pre-rendered canvas RGBA pixel buffer.
    pub fn canvas_data(&self) -> &[u8] {
        &self.canvas_image_data
    }

    /// Generates the RGBA pixel array representing the grid tile.
    fn create_grid_canvas(
        canvas_size: u32,
        cells: u32,
        background_color: Color,
        glow_color: Color,
        glow_width: f64,
        color: Color,
    ) -> Vec<u8> {
        let size = canvas_size as usize;
        let mut buffer = vec![0u8; size * size * 4];

        // Background color
        let bg_bytes = background_color.to_bytes();

        // Precompute line coordinates
        let cells_f = cells.max(1) as f64;
        let mut grid_coords = Vec::with_capacity((cells + 1) as usize);
        for i in 0..=cells {
            let nx = i as f64 / cells_f;
            let val = 1.0 + nx * (canvas_size as f64 - 1.0);
            grid_coords.push(val);
        }

        let blend = |dst: &mut [u8], src: Color, factor: f32| {
            let sa = (src.alpha * factor).clamp(0.0, 1.0);
            if sa <= 0.0 {
                return;
            }
            let da = dst[3] as f32 / 255.0;
            let out_a = sa + da * (1.0 - sa);
            if out_a > 0.0 {
                let dr = dst[0] as f32 / 255.0;
                let dg = dst[1] as f32 / 255.0;
                let db = dst[2] as f32 / 255.0;

                let out_r = (src.red * sa + dr * da * (1.0 - sa)) / out_a;
                let out_g = (src.green * sa + dg * da * (1.0 - sa)) / out_a;
                let out_b = (src.blue * sa + db * da * (1.0 - sa)) / out_a;

                dst[0] = (out_r * 255.0).round() as u8;
                dst[1] = (out_g * 255.0).round() as u8;
                dst[2] = (out_b * 255.0).round() as u8;
                dst[3] = (out_a * 255.0).round() as u8;
            }
        };

        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) * 4;
                buffer[idx..idx + 4].copy_from_slice(&bg_bytes);

                let px = x as f64 + 0.5;
                let py = y as f64 + 0.5;

                // Find minimal distance to any grid line or border
                let mut min_dist = px.min(py).min(canvas_size as f64 - px).min(canvas_size as f64 - py);
                for &coord in &grid_coords {
                    let dx = (px - coord).abs();
                    let dy = (py - coord).abs();
                    min_dist = min_dist.min(dx).min(dy);
                }

                // Apply glow
                if min_dist < glow_width {
                    let factor = (1.0 - (min_dist / glow_width)).powi(2) as f32;
                    blend(&mut buffer[idx..idx + 4], glow_color, factor);
                }

                // Apply grid line
                if min_dist < 1.0 {
                    let factor = (1.0 - min_dist) as f32;
                    blend(&mut buffer[idx..idx + 4], color, factor);
                }
            }
        }

        buffer
    }
}
