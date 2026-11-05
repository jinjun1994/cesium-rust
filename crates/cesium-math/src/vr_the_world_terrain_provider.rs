use std::f64::consts::PI;

use crate::{
    Credit, DeveloperError, GeographicTilingScheme, GeoRectangle, HeightmapBuffer,
    HeightmapTerrainData, HeightmapTerrainDataOptions, RuntimeError,
};

/// Data extent rectangle from VR-TheWorld metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct VRTheWorldDataRectangle {
    pub rectangle: GeoRectangle,
    pub max_level: u32,
}

/// Options for `VRTheWorldTerrainProvider`.
#[derive(Debug, Clone, Default)]
pub struct VRTheWorldTerrainProviderOptions {
    pub credit: Option<Credit>,
}

/// A TerrainProvider that produces terrain geometry by tessellating height maps
/// retrieved from a VT MÄK VR-TheWorld server.
#[derive(Debug, Clone)]
pub struct VRTheWorldTerrainProvider {
    url: String,
    tiling_scheme: GeographicTilingScheme,
    heightmap_width: u32,
    heightmap_height: u32,
    level_zero_maximum_geometric_error: f64,
    rectangles: Vec<VRTheWorldDataRectangle>,
    credit: Option<Credit>,
}

impl VRTheWorldTerrainProvider {
    pub fn from_url(
        url: &str,
        options: Option<VRTheWorldTerrainProviderOptions>,
    ) -> Result<Self, DeveloperError> {
        if url.trim().is_empty() {
            return Err(DeveloperError::new(
                "url is required, actual value was undefined",
            ));
        }

        // Default metadata configuration for VR-TheWorld
        let default_xml = r#"
            <TileMap version="1.0.0">
                <SRS>EPSG:4326</SRS>
                <TileFormat width="32" height="32" mime-type="image/tif" extension="tif"/>
                <DataExtents>
                    <DataExtent minx="-180.000000" miny="-90.000000" maxx="180.000000" maxy="90.000000" minlevel="0" maxlevel="9"/>
                </DataExtents>
            </TileMap>
        "#;
        Self::from_xml(url, default_xml, options)
            .map_err(|e| DeveloperError::new(e.message))
    }

    pub fn from_xml(
        url: &str,
        xml_str: &str,
        options: Option<VRTheWorldTerrainProviderOptions>,
    ) -> Result<Self, RuntimeError> {
        if url.trim().is_empty() {
            return Err(RuntimeError::new("url is required."));
        }

        let srs = extract_tag_value(xml_str, "SRS").unwrap_or_default();
        if srs != "EPSG:4326" {
            return Err(RuntimeError::new(format!(
                "An error occurred while accessing {url}: SRS {srs} is not supported"
            )));
        }

        let width = extract_attr_u32(xml_str, "TileFormat", "width").unwrap_or(32);
        let height = extract_attr_u32(xml_str, "TileFormat", "height").unwrap_or(32);

        let tiling_scheme = GeographicTilingScheme::default();
        // Estimated level zero geometric error:
        // 2 * PI * 6378137.0 / (2 * width)
        let max_dimension = width.min(height) as f64;
        let x_tiles_at_level_zero = tiling_scheme.number_of_x_tiles_at_level(0) as f64;
        let level_zero_max_error =
            (6378137.0 * 2.0 * PI) / (max_dimension * x_tiles_at_level_zero);

        let rectangles = parse_data_extents(xml_str);

        let options = options.unwrap_or_default();

        let mut url_clean = url.to_string();
        if !url_clean.ends_with('/') {
            url_clean.push('/');
        }

        Ok(Self {
            url: url_clean,
            tiling_scheme,
            heightmap_width: width,
            heightmap_height: height,
            level_zero_maximum_geometric_error: level_zero_max_error,
            rectangles,
            credit: options.credit,
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn tiling_scheme(&self) -> &GeographicTilingScheme {
        &self.tiling_scheme
    }

    pub fn heightmap_width(&self) -> u32 {
        self.heightmap_width
    }

    pub fn heightmap_height(&self) -> u32 {
        self.heightmap_height
    }

    pub fn credit(&self) -> Option<&Credit> {
        self.credit.as_ref()
    }

    pub fn has_water_mask(&self) -> bool {
        false
    }

    pub fn has_vertex_normals(&self) -> bool {
        false
    }

    pub fn get_level_maximum_geometric_error(&self, level: u32) -> f64 {
        self.level_zero_maximum_geometric_error / (1u64 << level) as f64
    }

    pub fn get_child_mask(&self, x: u32, y: u32, level: u32) -> u8 {
        let parent_rect = match self.tiling_scheme.tile_rectangle(x, y, level) {
            Some(r) => r,
            None => return 0,
        };

        let mut child_mask = 0u8;
        for rect in &self.rectangles {
            if rect.max_level <= level {
                continue;
            }
            if let Some(_inter) = rect.rectangle.intersection(parent_rect) {
                child_mask = 15;
                break;
            }
        }
        child_mask
    }

    pub fn build_tile_url(&self, x: u32, y: u32, level: u32) -> String {
        let y_tiles = self.tiling_scheme.number_of_y_tiles_at_level(level);
        let flipped_y = y_tiles.saturating_sub(y).saturating_sub(1);
        format!("{}{}/{}/{}.tif?cesium=true", self.url, level, x, flipped_y)
    }

    pub fn request_tile_geometry(
        &self,
        x: u32,
        y: u32,
        level: u32,
    ) -> Result<HeightmapTerrainData, RuntimeError> {
        let child_mask = self.get_child_mask(x, y, level);
        let buffer_size = (self.heightmap_width * self.heightmap_height) as usize;
        let buffer = HeightmapBuffer::Uint8(vec![0u8; buffer_size]);

        let data = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
            buffer: Some(buffer),
            width: Some(self.heightmap_width as usize),
            height: Some(self.heightmap_height as usize),
            child_tile_mask: Some(child_mask),
            water_mask: None,
            structure: None,
            encoding: None,
            created_by_upsampling: Some(false),
        })
        .map_err(|e| RuntimeError::new(e.message))?;

        Ok(data)
    }
}

fn extract_tag_value(xml: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{tag}>");
    let close_tag = format!("</{tag}>");
    let start = xml.find(&open_tag)? + open_tag.len();
    let end = xml[start..].find(&close_tag)? + start;
    Some(xml[start..end].trim().to_string())
}

fn extract_attr_u32(xml: &str, tag: &str, attr: &str) -> Option<u32> {
    let tag_open = format!("<{tag}");
    let tag_start = xml.find(&tag_open)?;
    let tag_end = xml[tag_start..].find('>')? + tag_start;
    let tag_content = &xml[tag_start..tag_end];

    let attr_pattern = format!("{attr}=\"");
    let attr_start = tag_content.find(&attr_pattern)? + attr_pattern.len();
    let attr_end = tag_content[attr_start..].find('"')? + attr_start;
    tag_content[attr_start..attr_end].parse::<u32>().ok()
}

fn parse_data_extents(xml: &str) -> Vec<VRTheWorldDataRectangle> {
    let mut results = Vec::new();
    let mut remaining = xml;

    while let Some(start) = remaining.find("<DataExtent") {
        let tag_end = match remaining[start..].find("/>") {
            Some(end) => start + end + 2,
            None => break,
        };
        let tag_content = &remaining[start..tag_end];

        let minx = extract_attr_f64_from_str(tag_content, "minx").unwrap_or(-180.0);
        let miny = extract_attr_f64_from_str(tag_content, "miny").unwrap_or(-90.0);
        let maxx = extract_attr_f64_from_str(tag_content, "maxx").unwrap_or(180.0);
        let maxy = extract_attr_f64_from_str(tag_content, "maxy").unwrap_or(90.0);
        let max_level = extract_attr_u32_from_str(tag_content, "maxlevel").unwrap_or(9);

        if let Ok(rect) = GeoRectangle::degrees(minx, miny, maxx, maxy) {
            results.push(VRTheWorldDataRectangle {
                rectangle: rect,
                max_level,
            });
        }

        remaining = &remaining[tag_end..];
    }

    results
}

fn extract_attr_f64_from_str(tag_content: &str, attr: &str) -> Option<f64> {
    let attr_pattern = format!("{attr}=\"");
    let attr_start = tag_content.find(&attr_pattern)? + attr_pattern.len();
    let attr_end = tag_content[attr_start..].find('"')? + attr_start;
    tag_content[attr_start..attr_end].parse::<f64>().ok()
}

fn extract_attr_u32_from_str(tag_content: &str, attr: &str) -> Option<u32> {
    let attr_pattern = format!("{attr}=\"");
    let attr_start = tag_content.find(&attr_pattern)? + attr_pattern.len();
    let attr_end = tag_content[attr_start..].find('"')? + attr_start;
    tag_content[attr_start..attr_end].parse::<u32>().ok()
}
