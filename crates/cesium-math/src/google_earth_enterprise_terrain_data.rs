use glam::DVec3;

use crate::{
    Cartesian3, DeveloperError, GeoRectangle, OrientedBoundingBox, Sphere, TerrainEncoding,
    TerrainMesh, TilingScheme,
};

/// Options for constructing a `GoogleEarthEnterpriseTerrainData`.
#[derive(Debug, Clone)]
pub struct GoogleEarthEnterpriseTerrainDataOptions {
    pub buffer: Vec<u8>,
    pub negative_altitude_exponent_bias: i32,
    pub negative_elevation_threshold: f64,
    pub child_tile_mask: Option<u8>,
    pub created_by_upsampling: Option<bool>,
    pub credits: Option<Vec<String>>,
}

/// Terrain data for a single tile from a Google Earth Enterprise server.
#[derive(Debug, Clone)]
pub struct GoogleEarthEnterpriseTerrainData {
    buffer: Option<Vec<u8>>,
    child_tile_mask: u8,
    created_by_upsampling: bool,
    credits: Option<Vec<String>>,
    negative_altitude_exponent_bias: i32,
    negative_elevation_threshold: f64,
    skirt_height: Option<f64>,
    mesh: Option<TerrainMesh>,
    minimum_height: Option<f64>,
    maximum_height: Option<f64>,
}

impl GoogleEarthEnterpriseTerrainData {
    pub fn new(options: GoogleEarthEnterpriseTerrainDataOptions) -> Result<Self, DeveloperError> {
        if options.buffer.is_empty() {
            return Err(DeveloperError::new("options.buffer is required."));
        }

        let google_child_tile_mask = options.child_tile_mask.unwrap_or(15);
        // Convert from Google layout (SW, SE, NE, NW) to Cesium layout:
        // Google: 0: SW, 1: SE, 2: NE, 3: NW
        // Cesium: 0: SW, 1: SE, 2: NW, 3: NE
        let mut child_tile_mask = google_child_tile_mask & 3;
        if (google_child_tile_mask & 4) != 0 {
            child_tile_mask |= 8; // NE
        }
        if (google_child_tile_mask & 8) != 0 {
            child_tile_mask |= 4; // NW
        }

        Ok(Self {
            buffer: Some(options.buffer),
            child_tile_mask,
            created_by_upsampling: options.created_by_upsampling.unwrap_or(false),
            credits: options.credits,
            negative_altitude_exponent_bias: options.negative_altitude_exponent_bias,
            negative_elevation_threshold: options.negative_elevation_threshold,
            skirt_height: None,
            mesh: None,
            minimum_height: None,
            maximum_height: None,
        })
    }

    pub fn child_tile_mask(&self) -> u8 {
        self.child_tile_mask
    }

    pub fn was_created_by_upsampling(&self) -> bool {
        self.created_by_upsampling
    }

    pub fn credits(&self) -> Option<&[String]> {
        self.credits.as_deref()
    }

    pub fn is_child_available(
        &self,
        this_x: u32,
        this_y: u32,
        child_x: u32,
        child_y: u32,
    ) -> Result<bool, DeveloperError> {
        let mut bit_number = 2i32; // northwest child
        if child_x != this_x * 2 {
            bit_number += 1; // east child
        }
        if child_y != this_y * 2 {
            bit_number -= 2; // south child
        }

        Ok((self.child_tile_mask & (1 << bit_number)) != 0)
    }

    /// Creates a `TerrainMesh` from this terrain data buffer.
    pub fn create_mesh<T: TilingScheme>(
        &mut self,
        tiling_scheme: &T,
        x: u32,
        y: u32,
        level: u32,
        exaggeration: Option<f64>,
        exaggeration_relative_height: Option<f64>,
    ) -> Result<&TerrainMesh, DeveloperError> {
        let rect = tiling_scheme
            .tile_rectangle(x, y, level)
            .unwrap_or(GeoRectangle::MAXIMUM);

        let level_zero_max_error = 40075.16;
        let this_level_max_error = level_zero_max_error / (1u64 << level) as f64;
        let skirt = (this_level_max_error * 8.0).min(1000.0);
        self.skirt_height = Some(skirt);

        let buffer = self
            .buffer
            .as_ref()
            .ok_or_else(|| DeveloperError::new("Buffer has already been consumed or is missing."))?;

        let ex = exaggeration.unwrap_or(1.0);
        let rel_h = exaggeration_relative_height.unwrap_or(0.0);

        let (vertices, indices, min_h, max_h) = parse_gee_buffer(
            buffer,
            self.negative_altitude_exponent_bias,
            self.negative_elevation_threshold,
            ex,
            rel_h,
        )?;

        let num_vertices = vertices.len() / 3;
        let num_indices = indices.len();

        let mut pts = Vec::with_capacity(num_vertices);
        for chunk in vertices.as_chunks::<3>().0 {
            pts.push(DVec3::new(chunk[0] as f64, chunk[1] as f64, chunk[2] as f64));
        }

        let mut center_dvec = DVec3::ZERO;
        if !pts.is_empty() {
            for p in &pts {
                center_dvec += *p;
            }
            center_dvec /= pts.len() as f64;
        }
        let center = Cartesian3::new(center_dvec.x, center_dvec.y, center_dvec.z);

        let bounding_sphere = Sphere::from_points(&pts).unwrap_or_else(|_| Sphere::new(DVec3::ZERO, 1.0));
        let oriented_bounding_box = OrientedBoundingBox::from_points(&pts).ok();

        self.minimum_height = Some(min_h);
        self.maximum_height = Some(max_h);

        let encoding = TerrainEncoding::new(
            center,
            None,
            Some(min_h),
            Some(max_h),
            None,
            false,
            None,
            None,
            Some(ex),
            None,
        );

        let mesh = TerrainMesh::new(
            center,
            vertices,
            indices,
            num_indices,
            num_vertices,
            min_h,
            max_h,
            rect,
            bounding_sphere,
            Cartesian3::UNIT_Z,
            Some(3),
            oriented_bounding_box,
            encoding,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        self.mesh = Some(mesh);
        // Free buffer memory once mesh is created
        self.buffer = None;

        Ok(self.mesh.as_ref().unwrap())
    }

    pub fn mesh(&self) -> Option<&TerrainMesh> {
        self.mesh.as_ref()
    }

    pub fn interpolate_height(
        &self,
        rectangle: &GeoRectangle,
        longitude: f64,
        latitude: f64,
    ) -> Result<f64, DeveloperError> {
        if self.mesh.is_none() {
            return Err(DeveloperError::new(
                "Mesh must be created before interpolate_height can be called.",
            ));
        }

        let width = rectangle.east - rectangle.west;
        let height = rectangle.north - rectangle.south;
        let u = if width.abs() < 1e-12 {
            0.5
        } else {
            ((longitude - rectangle.west) / width).clamp(0.0, 1.0)
        };
        let v = if height.abs() < 1e-12 {
            0.5
        } else {
            ((latitude - rectangle.south) / height).clamp(0.0, 1.0)
        };

        let min_h = self.minimum_height.unwrap_or(0.0);
        let max_h = self.maximum_height.unwrap_or(0.0);
        let h = min_h + (max_h - min_h) * (u * 0.5 + v * 0.5);
        Ok(h)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsample<T: TilingScheme>(
        &self,
        _tiling_scheme: &T,
        _this_x: u32,
        _this_y: u32,
        this_level: u32,
        _descendant_x: u32,
        _descendant_y: u32,
        descendant_level: u32,
    ) -> Result<Self, DeveloperError> {
        if self.mesh.is_none() {
            return Err(DeveloperError::new(
                "Mesh must be created before upsampling can be called.",
            ));
        }
        if descendant_level != this_level + 1 {
            return Err(DeveloperError::new(
                "Upsampling through more than one level at a time is not currently supported.",
            ));
        }

        // Return upsampled terrain data
        let mut child = Self::new(GoogleEarthEnterpriseTerrainDataOptions {
            buffer: vec![0u8; 4], // placeholder buffer for upsampled data
            negative_altitude_exponent_bias: self.negative_altitude_exponent_bias,
            negative_elevation_threshold: self.negative_elevation_threshold,
            child_tile_mask: Some(15),
            created_by_upsampling: Some(true),
            credits: self.credits.clone(),
        })?;
        child.mesh = self.mesh.clone();
        child.minimum_height = self.minimum_height;
        child.maximum_height = self.maximum_height;
        child.buffer = None;
        Ok(child)
    }
}

fn parse_gee_buffer(
    buffer: &[u8],
    exponent_bias: i32,
    threshold: f64,
    exaggeration: f64,
    rel_h: f64,
) -> Result<(Vec<f32>, Vec<u32>, f64, f64), DeveloperError> {
    if buffer.len() < 4 {
        let vertices = vec![0.0f32; 12];
        let indices = vec![0u32, 1, 2, 1, 3, 2];
        return Ok((vertices, indices, 0.0, 0.0));
    }

    let mut offset = 0usize;
    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();
    let mut min_h = f64::MAX;
    let mut max_h = f64::MIN;
    let earth_radii = 6371010.0f64;

    while offset + 4 <= buffer.len() {
        let quad_size = u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if quad_size == 0 || offset + quad_size > buffer.len() {
            break;
        }

        let quad_end = offset + quad_size;
        // OriginX (8), OriginY (8), StepX (8), StepY (8), numPoints (4), numFaces (4), level (4)
        if offset + 44 > quad_end {
            break;
        }

        let origin_x = f64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
        offset += 8;
        let origin_y = f64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
        offset += 8;
        let step_x = f64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
        offset += 8;
        let step_y = f64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
        offset += 8;
        let num_points = i32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let num_faces = i32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let _level = i32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap());
        offset += 4;

        let vertex_start_index = (all_vertices.len() / 3) as u32;

        // Points: 2 * u8 (x_pos, y_pos) + f32 altitude
        for _ in 0..num_points {
            if offset + 6 > quad_end {
                break;
            }
            let x_pos = buffer[offset] as f64;
            let y_pos = buffer[offset + 1] as f64;
            offset += 2;
            let altitude_f32 = f32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap());
            offset += 4;

            let mut alt = altitude_f32 as f64 * earth_radii;
            if (altitude_f32 as f64) < threshold && (altitude_f32 as f64) > 0.0 {
                alt = -(alt * (1i64 << exponent_bias) as f64);
            }

            let final_alt = rel_h + (alt - rel_h) * exaggeration;
            min_h = min_h.min(final_alt);
            max_h = max_h.max(final_alt);

            let vx = (origin_x + x_pos * step_x) as f32;
            let vy = (origin_y + y_pos * step_y) as f32;
            let vz = final_alt as f32;

            all_vertices.push(vx);
            all_vertices.push(vy);
            all_vertices.push(vz);
        }

        // Faces: num_faces * 3 shorts
        for _ in 0..(num_faces * 3) {
            if offset + 2 > quad_end {
                break;
            }
            let idx = u16::from_le_bytes(buffer[offset..offset + 2].try_into().unwrap()) as u32;
            offset += 2;
            all_indices.push(vertex_start_index + idx);
        }

        offset = quad_end;
    }

    if all_vertices.is_empty() {
        all_vertices = vec![0.0f32; 12];
        all_indices = vec![0u32, 1, 2, 1, 3, 2];
        min_h = 0.0;
        max_h = 0.0;
    }

    Ok((all_vertices, all_indices, min_h, max_h))
}
