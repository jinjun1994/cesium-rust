use glam::DVec3;

use crate::cartesian::Cartesian3;
use crate::errors::DeveloperError;
use crate::rectangle::GeoRectangle;
use crate::terrain_encoding::TerrainEncoding;
use crate::terrain_mesh::TerrainMesh;
use crate::tiling::TilingScheme;
use crate::{Cartographic, Sphere};

/// Maximum number of asynchronous terrain tasks running concurrently.
pub const MAXIMUM_ASYNCHRONOUS_TASKS: usize = 5;

/// The encoding that is used on the buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeightmapEncoding {
    #[default]
    None = 0,
    Lerc = 1,
}

/// Buffer element datatype for heightmaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeightmapBufferType {
    Uint8,
    Uint16,
    Int16,
    #[default]
    Float32,
    Float64,
}

/// Describes the internal layout and numeric representation of height data.
#[derive(Debug, Clone, PartialEq)]
pub struct HeightmapStructure {
    pub height_scale: f64,
    pub height_offset: f64,
    pub elements_per_height: usize,
    pub stride: usize,
    pub element_multiplier: f64,
    pub is_big_endian: bool,
    pub lowest_encoded_height: Option<f64>,
    pub highest_encoded_height: Option<f64>,
}

impl Default for HeightmapStructure {
    fn default() -> Self {
        Self {
            height_scale: 1.0,
            height_offset: 0.0,
            elements_per_height: 1,
            stride: 1,
            element_multiplier: 256.0,
            is_big_endian: false,
            lowest_encoded_height: None,
            highest_encoded_height: None,
        }
    }
}

/// Dynamic buffer for heightmap samples supporting u8, u16, f32, f64.
#[derive(Debug, Clone, PartialEq)]
pub enum HeightmapBuffer {
    Float32(Vec<f32>),
    Float64(Vec<f64>),
    Uint16(Vec<u16>),
    Uint8(Vec<u8>),
}

impl HeightmapBuffer {
    pub fn new_zeroed(buffer_type: HeightmapBufferType, len: usize) -> Self {
        match buffer_type {
            HeightmapBufferType::Float32 => Self::Float32(vec![0.0; len]),
            HeightmapBufferType::Float64 => Self::Float64(vec![0.0; len]),
            HeightmapBufferType::Uint16 => Self::Uint16(vec![0; len]),
            HeightmapBufferType::Uint8 | HeightmapBufferType::Int16 => Self::Uint8(vec![0; len]),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Float32(v) => v.len(),
            Self::Float64(v) => v.len(),
            Self::Uint16(v) => v.len(),
            Self::Uint8(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> f64 {
        match self {
            Self::Float32(v) => v.get(index).copied().unwrap_or(0.0) as f64,
            Self::Float64(v) => v.get(index).copied().unwrap_or(0.0),
            Self::Uint16(v) => v.get(index).copied().unwrap_or(0) as f64,
            Self::Uint8(v) => v.get(index).copied().unwrap_or(0) as f64,
        }
    }

    pub fn set(&mut self, index: usize, value: f64) {
        match self {
            Self::Float32(v) => {
                if let Some(elem) = v.get_mut(index) {
                    *elem = value as f32;
                }
            }
            Self::Float64(v) => {
                if let Some(elem) = v.get_mut(index) {
                    *elem = value;
                }
            }
            Self::Uint16(v) => {
                if let Some(elem) = v.get_mut(index) {
                    *elem = value.clamp(0.0, 65535.0) as u16;
                }
            }
            Self::Uint8(v) => {
                if let Some(elem) = v.get_mut(index) {
                    *elem = value.clamp(0.0, 255.0) as u8;
                }
            }
        }
    }

    pub fn buffer_type(&self) -> HeightmapBufferType {
        match self {
            Self::Float32(_) => HeightmapBufferType::Float32,
            Self::Float64(_) => HeightmapBufferType::Float64,
            Self::Uint16(_) => HeightmapBufferType::Uint16,
            Self::Uint8(_) => HeightmapBufferType::Uint8,
        }
    }

    pub fn to_vec_f64(&self) -> Vec<f64> {
        let n = self.len();
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            out.push(self.get(i));
        }
        out
    }
}

/// Options to construct a `HeightmapTerrainData`.
#[derive(Debug, Clone, Default)]
pub struct HeightmapTerrainDataOptions {
    pub buffer: Option<HeightmapBuffer>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub child_tile_mask: Option<u8>,
    pub encoding: Option<HeightmapEncoding>,
    pub structure: Option<HeightmapStructure>,
    pub created_by_upsampling: Option<bool>,
    pub water_mask: Option<Vec<u8>>,
}

/// Options to create a terrain mesh from a heightmap.
#[derive(Debug, Clone)]
pub struct CreateMeshOptions<'a, T: TilingScheme> {
    pub tiling_scheme: &'a T,
    pub x: u32,
    pub y: u32,
    pub level: u32,
    pub exaggeration: f64,
    pub exaggeration_relative_height: f64,
    pub throttle: bool,
}

/// Terrain data for a single tile represented as a regular grid of heights.
#[derive(Debug, Clone)]
pub struct HeightmapTerrainData {
    pub buffer: Option<HeightmapBuffer>,
    pub width: usize,
    pub height: usize,
    pub child_tile_mask: u8,
    pub encoding: HeightmapEncoding,
    pub structure: HeightmapStructure,
    pub created_by_upsampling: bool,
    pub water_mask: Option<Vec<u8>>,
    pub buffer_type: HeightmapBufferType,
    pub skirt_height: Option<f64>,
    pub mesh: Option<TerrainMesh>,
}

impl HeightmapTerrainData {
    pub fn new(options: HeightmapTerrainDataOptions) -> Result<Self, DeveloperError> {
        let buffer = options.buffer.ok_or_else(|| DeveloperError::new("options.buffer is required"))?;
        let width = options.width.ok_or_else(|| DeveloperError::new("options.width is required"))?;
        let height = options.height.ok_or_else(|| DeveloperError::new("options.height is required"))?;

        let encoding = options.encoding.unwrap_or(HeightmapEncoding::None);
        let buffer_type = if encoding == HeightmapEncoding::Lerc {
            HeightmapBufferType::Float32
        } else {
            buffer.buffer_type()
        };

        Ok(Self {
            buffer: Some(buffer),
            width,
            height,
            child_tile_mask: options.child_tile_mask.unwrap_or(15),
            encoding,
            structure: options.structure.unwrap_or_default(),
            created_by_upsampling: options.created_by_upsampling.unwrap_or(false),
            water_mask: options.water_mask,
            buffer_type,
            skirt_height: None,
            mesh: None,
        })
    }

    pub fn was_created_by_upsampling(&self) -> bool {
        self.created_by_upsampling
    }

    pub fn buffer(&self) -> Option<&HeightmapBuffer> {
        self.buffer.as_ref()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn child_tile_mask(&self) -> u8 {
        self.child_tile_mask
    }

    pub fn buffer_type(&self) -> HeightmapBufferType {
        self.buffer_type
    }

    pub fn encoding(&self) -> HeightmapEncoding {
        self.encoding
    }

    pub fn is_child_available(
        &self,
        this_x: Option<u32>,
        this_y: Option<u32>,
        child_x: Option<u32>,
        child_y: Option<u32>,
    ) -> Result<bool, DeveloperError> {
        let this_x = this_x.ok_or_else(|| DeveloperError::new("thisX is required."))?;
        let this_y = this_y.ok_or_else(|| DeveloperError::new("thisY is required."))?;
        let child_x = child_x.ok_or_else(|| DeveloperError::new("childX is required."))?;
        let child_y = child_y.ok_or_else(|| DeveloperError::new("childY is required."))?;

        let mut bit_number = 2; // northwest
        if child_x != this_x * 2 {
            bit_number += 1; // east
        }
        if child_y != this_y * 2 {
            bit_number -= 2; // south
        }

        Ok((self.child_tile_mask & (1 << bit_number)) != 0)
    }

    pub fn interpolate_height(
        &self,
        rectangle: &GeoRectangle,
        longitude: f64,
        latitude: f64,
    ) -> Option<f64> {
        if self.mesh.is_none() && self.encoding == HeightmapEncoding::Lerc {
            return None;
        }

        let width = self.width;
        let height = self.height;
        let structure = &self.structure;

        if let Some(ref mesh) = self.mesh {
            let buffer = &mesh.vertices;
            let encoding = &mesh.encoding;
            let h = interpolate_mesh_height(
                buffer,
                encoding,
                structure.height_offset,
                structure.height_scale,
                rectangle,
                width,
                height,
                longitude,
                latitude,
            );
            Some(h * structure.height_scale + structure.height_offset)
        } else if let Some(ref buf) = self.buffer {
            let h = interpolate_height(
                buf,
                structure.elements_per_height,
                structure.element_multiplier,
                structure.stride,
                structure.is_big_endian,
                rectangle,
                width,
                height,
                longitude,
                latitude,
            );
            Some(h * structure.height_scale + structure.height_offset)
        } else {
            None
        }
    }

    pub fn create_mesh<T: TilingScheme>(
        &mut self,
        options: CreateMeshOptions<'_, T>,
    ) -> Result<Option<&TerrainMesh>, DeveloperError> {
        let buf = self.buffer.as_ref().ok_or_else(|| {
            DeveloperError::new("Heightmap buffer is required to create a mesh")
        })?;

        let width = self.width;
        let height = self.height;
        let tiling_scheme = options.tiling_scheme;
        let ellipsoid = tiling_scheme.ellipsoid();
        let rectangle = tiling_scheme
            .tile_rectangle(options.x, options.y, options.level)
            .ok_or_else(|| DeveloperError::new("Failed to compute tile rectangle"))?;

        let center_carto = Cartographic::from_radians(
            (rectangle.west + rectangle.east) * 0.5,
            (rectangle.south + rectangle.north) * 0.5,
        );
        let center = Cartesian3::from(
            ellipsoid
                .cartographic_to_cartesian(center_carto)
                .unwrap_or(DVec3::ZERO),
        );

        let mut vertices = Vec::with_capacity(width * height * 6);
        let mut min_height = f64::MAX;
        let mut max_height = f64::MIN;

        for j in 0..height {
            let lat_ratio = if height > 1 { (j as f64) / ((height - 1) as f64) } else { 0.0 };
            let lat = rectangle.north + lat_ratio * (rectangle.south - rectangle.north);

            for i in 0..width {
                let lon_ratio = if width > 1 { (i as f64) / ((width - 1) as f64) } else { 0.0 };
                let lon = rectangle.west + lon_ratio * (rectangle.east - rectangle.west);

                let raw_h = get_height(
                    buf,
                    self.structure.elements_per_height,
                    self.structure.element_multiplier,
                    self.structure.stride,
                    self.structure.is_big_endian,
                    j * width + i,
                );
                let h = raw_h * self.structure.height_scale + self.structure.height_offset;
                if h < min_height { min_height = h; }
                if h > max_height { max_height = h; }

                let pos_carto = Cartographic::from_radians(lon, lat);
                let pos = Cartesian3::from(
                    ellipsoid
                        .cartographic_to_cartesian(pos_carto)
                        .unwrap_or(DVec3::ZERO),
                );
                let rel = pos - center;

                vertices.push(rel.x as f32);
                vertices.push(rel.y as f32);
                vertices.push(rel.z as f32);
                vertices.push(h as f32);
                vertices.push(lon_ratio as f32);
                vertices.push((1.0 - lat_ratio) as f32);
            }
        }

        let grid_indices = get_regular_grid_indices(width, height);
        let edge_indices = get_edge_indices(width, height);

        let encoding = TerrainEncoding::new(
            center,
            None,
            Some(min_height),
            Some(max_height),
            None,
            false,
            Some(false),
            Some(false),
            Some(options.exaggeration),
            Some(options.exaggeration_relative_height),
        );

        let mesh = TerrainMesh::new(
            center,
            vertices,
            grid_indices,
            (width - 1) * (height - 1) * 6,
            width * height,
            min_height,
            max_height,
            rectangle,
            Sphere::new(center, 50000.0),
            Cartesian3::ZERO,
            Some(6),
            None,
            encoding,
            edge_indices.west_indices_south_to_north,
            edge_indices.south_indices_east_to_west,
            edge_indices.east_indices_north_to_south,
            edge_indices.north_indices_west_to_east,
        );

        self.mesh = Some(mesh);
        Ok(self.mesh.as_ref())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsample<T: TilingScheme>(
        &self,
        tiling_scheme: Option<&T>,
        this_x: Option<u32>,
        this_y: Option<u32>,
        this_level: Option<u32>,
        descendant_x: Option<u32>,
        descendant_y: Option<u32>,
        descendant_level: Option<u32>,
    ) -> Result<Self, DeveloperError> {
        let tiling_scheme = tiling_scheme.ok_or_else(|| DeveloperError::new("tilingScheme is required."))?;
        let this_x = this_x.ok_or_else(|| DeveloperError::new("thisX is required."))?;
        let this_y = this_y.ok_or_else(|| DeveloperError::new("thisY is required."))?;
        let this_level = this_level.ok_or_else(|| DeveloperError::new("thisLevel is required."))?;
        let descendant_x = descendant_x.ok_or_else(|| DeveloperError::new("descendantX is required."))?;
        let descendant_y = descendant_y.ok_or_else(|| DeveloperError::new("descendantY is required."))?;
        let descendant_level = descendant_level.ok_or_else(|| DeveloperError::new("descendantLevel is required."))?;

        if descendant_level < this_level || descendant_level - this_level > 1 {
            return Err(DeveloperError::new(
                "Upsampling through more than one level at a time is not currently supported.",
            ));
        }

        let mesh = self.mesh.as_ref().ok_or_else(|| {
            DeveloperError::new("createMesh must be called prior to upsampling.")
        })?;

        let width = self.width;
        let height = self.height;
        let structure = &self.structure;
        let stride = structure.stride;

        let mut heights = HeightmapBuffer::new_zeroed(self.buffer_type, width * height * stride);

        let buffer = &mesh.vertices;
        let encoding = &mesh.encoding;

        let source_rect = tiling_scheme
            .tile_rectangle(this_x, this_y, this_level)
            .ok_or_else(|| DeveloperError::new("Failed to compute source tile rectangle"))?;
        let dest_rect = tiling_scheme
            .tile_rectangle(descendant_x, descendant_y, descendant_level)
            .ok_or_else(|| DeveloperError::new("Failed to compute descendant tile rectangle"))?;

        let height_offset = structure.height_offset;
        let height_scale = structure.height_scale;
        let elements_per_height = structure.elements_per_height;
        let element_multiplier = structure.element_multiplier;
        let is_big_endian = structure.is_big_endian;

        let divisor = element_multiplier.powi((elements_per_height as i32) - 1);

        for j in 0..height {
            let lat_ratio = if height > 1 { (j as f64) / ((height - 1) as f64) } else { 0.0 };
            let latitude = dest_rect.north + lat_ratio * (dest_rect.south - dest_rect.north);

            for i in 0..width {
                let lon_ratio = if width > 1 { (i as f64) / ((width - 1) as f64) } else { 0.0 };
                let longitude = dest_rect.west + lon_ratio * (dest_rect.east - dest_rect.west);

                let mut sample = interpolate_mesh_height(
                    buffer,
                    encoding,
                    height_offset,
                    height_scale,
                    &source_rect,
                    width,
                    height,
                    longitude,
                    latitude,
                );

                if let Some(lowest) = structure.lowest_encoded_height {
                    if sample < lowest {
                        sample = lowest;
                    }
                }
                if let Some(highest) = structure.highest_encoded_height {
                    if sample > highest {
                        sample = highest;
                    }
                }

                set_height(
                    &mut heights,
                    elements_per_height,
                    element_multiplier,
                    divisor,
                    stride,
                    is_big_endian,
                    j * width + i,
                    sample,
                );
            }
        }

        Self::new(HeightmapTerrainDataOptions {
            buffer: Some(heights),
            width: Some(width),
            height: Some(height),
            child_tile_mask: Some(0),
            structure: Some(self.structure.clone()),
            created_by_upsampling: Some(true),
            ..Default::default()
        })
    }
}

pub struct EdgeIndices {
    pub west_indices_south_to_north: Vec<u32>,
    pub south_indices_east_to_west: Vec<u32>,
    pub east_indices_north_to_south: Vec<u32>,
    pub north_indices_west_to_east: Vec<u32>,
}

pub fn get_edge_indices(width: usize, height: usize) -> EdgeIndices {
    let mut north = vec![0u32; width];
    let mut south = vec![0u32; width];
    let mut east = vec![0u32; height];
    let mut west = vec![0u32; height];

    for (i, item) in north.iter_mut().enumerate().take(width) {
        *item = i as u32;
    }
    for (i, item) in south.iter_mut().enumerate().take(width) {
        *item = (width * height - 1 - i) as u32;
    }
    for (i, item) in east.iter_mut().enumerate().take(height) {
        *item = ((i + 1) * width - 1) as u32;
    }
    for (i, item) in west.iter_mut().enumerate().take(height) {
        *item = ((height - i - 1) * width) as u32;
    }

    EdgeIndices {
        west_indices_south_to_north: west,
        south_indices_east_to_west: south,
        east_indices_north_to_south: east,
        north_indices_west_to_east: north,
    }
}

pub fn get_regular_grid_indices(width: usize, height: usize) -> Vec<u32> {
    let mut indices = Vec::with_capacity((width - 1) * (height - 1) * 6);
    let mut index = 0;
    for _ in 0..height - 1 {
        for _ in 0..width - 1 {
            let upper_left = index;
            let lower_left = upper_left + (width as u32);
            let lower_right = lower_left + 1;
            let upper_right = upper_left + 1;

            indices.push(upper_left);
            indices.push(lower_left);
            indices.push(upper_right);
            indices.push(upper_right);
            indices.push(lower_left);
            indices.push(lower_right);

            index += 1;
        }
        index += 1;
    }
    indices
}

#[allow(clippy::too_many_arguments)]
fn interpolate_height(
    source_heights: &HeightmapBuffer,
    elements_per_height: usize,
    element_multiplier: f64,
    stride: usize,
    is_big_endian: bool,
    source_rect: &GeoRectangle,
    width: usize,
    height: usize,
    longitude: f64,
    latitude: f64,
) -> f64 {
    let from_west = ((longitude - source_rect.west) * ((width - 1) as f64)) / (source_rect.east - source_rect.west);
    let from_south = ((latitude - source_rect.south) * ((height - 1) as f64)) / (source_rect.north - source_rect.south);

    let mut west_int = from_west.floor() as isize;
    let mut east_int = west_int + 1;
    if east_int >= width as isize {
        east_int = (width - 1) as isize;
        west_int = (width - 2) as isize;
    }

    let mut south_int = from_south.floor() as isize;
    let mut north_int = south_int + 1;
    if north_int >= height as isize {
        north_int = (height - 1) as isize;
        south_int = (height - 2) as isize;
    }

    let dx = from_west - (west_int as f64);
    let dy = from_south - (south_int as f64);

    let south_idx = (height - 1 - (south_int as usize)) * width;
    let north_idx = (height - 1 - (north_int as usize)) * width;

    let sw = get_height(source_heights, elements_per_height, element_multiplier, stride, is_big_endian, south_idx + (west_int as usize));
    let se = get_height(source_heights, elements_per_height, element_multiplier, stride, is_big_endian, south_idx + (east_int as usize));
    let nw = get_height(source_heights, elements_per_height, element_multiplier, stride, is_big_endian, north_idx + (west_int as usize));
    let ne = get_height(source_heights, elements_per_height, element_multiplier, stride, is_big_endian, north_idx + (east_int as usize));

    triangle_interpolate_height(dx, dy, sw, se, nw, ne)
}

#[allow(clippy::too_many_arguments)]
fn interpolate_mesh_height(
    buffer: &[f32],
    encoding: &TerrainEncoding,
    height_offset: f64,
    height_scale: f64,
    source_rect: &GeoRectangle,
    width: usize,
    height: usize,
    longitude: f64,
    latitude: f64,
) -> f64 {
    let from_west = ((longitude - source_rect.west) * ((width - 1) as f64)) / (source_rect.east - source_rect.west);
    let from_south = ((latitude - source_rect.south) * ((height - 1) as f64)) / (source_rect.north - source_rect.south);

    let mut west_int = from_west.floor() as isize;
    let mut east_int = west_int + 1;
    if east_int >= width as isize {
        east_int = (width - 1) as isize;
        west_int = (width - 2) as isize;
    }

    let mut south_int = from_south.floor() as isize;
    let mut north_int = south_int + 1;
    if north_int >= height as isize {
        north_int = (height - 1) as isize;
        south_int = (height - 2) as isize;
    }

    let dx = from_west - (west_int as f64);
    let dy = from_south - (south_int as f64);

    let south_idx = (height - 1 - (south_int as usize)) * width;
    let north_idx = (height - 1 - (north_int as usize)) * width;

    let sw = (encoding.decode_height(buffer, south_idx + (west_int as usize)) - height_offset) / height_scale;
    let se = (encoding.decode_height(buffer, south_idx + (east_int as usize)) - height_offset) / height_scale;
    let nw = (encoding.decode_height(buffer, north_idx + (west_int as usize)) - height_offset) / height_scale;
    let ne = (encoding.decode_height(buffer, north_idx + (east_int as usize)) - height_offset) / height_scale;

    triangle_interpolate_height(dx, dy, sw, se, nw, ne)
}

fn triangle_interpolate_height(
    dx: f64,
    dy: f64,
    southwest_height: f64,
    southeast_height: f64,
    northwest_height: f64,
    northeast_height: f64,
) -> f64 {
    if dy < dx {
        southwest_height + dx * (southeast_height - southwest_height) + dy * (northeast_height - southeast_height)
    } else {
        southwest_height + dx * (northeast_height - northwest_height) + dy * (northwest_height - southwest_height)
    }
}

fn get_height(
    heights: &HeightmapBuffer,
    elements_per_height: usize,
    element_multiplier: f64,
    stride: usize,
    is_big_endian: bool,
    index: usize,
) -> f64 {
    let start = index * stride;
    let mut height = 0.0;

    if is_big_endian {
        for i in 0..elements_per_height {
            height = height * element_multiplier + heights.get(start + i);
        }
    } else {
        for i in (0..elements_per_height).rev() {
            height = height * element_multiplier + heights.get(start + i);
        }
    }

    height
}

#[allow(clippy::too_many_arguments)]
fn set_height(
    heights: &mut HeightmapBuffer,
    elements_per_height: usize,
    element_multiplier: f64,
    mut divisor: f64,
    stride: usize,
    is_big_endian: bool,
    index: usize,
    mut height: f64,
) {
    let start = index * stride;
    if is_big_endian {
        for i in 0..elements_per_height.saturating_sub(1) {
            let val = (height / divisor).floor();
            heights.set(start + i, val);
            height -= val * divisor;
            divisor /= element_multiplier;
        }
        heights.set(start + elements_per_height - 1, height);
    } else {
        for i in (1..elements_per_height).rev() {
            let val = (height / divisor).floor();
            heights.set(start + i, val);
            height -= val * divisor;
            divisor /= element_multiplier;
        }
        heights.set(start, height);
    }
}
