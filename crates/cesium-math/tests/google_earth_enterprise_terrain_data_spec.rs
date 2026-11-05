use std::f64::consts::PI;

use cesium_math::{
    GeographicTilingScheme, GeoRectangle, GoogleEarthEnterpriseTerrainData,
    GoogleEarthEnterpriseTerrainDataOptions,
};

fn create_test_buffer(tiling_scheme: &GeographicTilingScheme, x: u32, y: u32, level: u32) -> Vec<u8> {
    let rect = tiling_scheme.tile_xy_to_rectangle(x, y, level).unwrap();
    let center = rect.center();
    let sw_lon = rect.west;
    let sw_lat = rect.south;
    let step_x = (rect.width() * 0.5 * 180.0 / PI) / 180.0;
    let step_y = (rect.height() * 0.5 * 180.0 / PI) / 180.0;
    let to_earth_radii = 1.0f32 / 6371010.0f32;

    let point_size = 2 + 4; // 2 u8s + 1 f32
    let face_size = 3 * 2; // 3 u16s
    let quad_size = 4 * 8 + 3 * 4 + 4 * point_size + 2 * face_size; // 32 + 12 + 24 + 12 = 80
    let total_size = 4 * (quad_size + 4);

    let mut buf = vec![0u8; total_size];
    let mut offset = 0usize;

    for i in 0..4 {
        buf[offset..offset + 4].copy_from_slice(&(quad_size as u32).to_le_bytes());
        offset += 4;

        let mut x_origin = sw_lon;
        let mut y_origin = sw_lat;
        let mut alt_start = 0.0f32;

        if (i & 2) != 0 {
            // Top row
            if (i & 1) == 0 {
                x_origin = center.longitude;
                alt_start = 10.0;
            }
            y_origin = center.latitude;
        } else if (i & 1) != 0 {
            x_origin = center.longitude;
            alt_start = 10.0;
        }

        buf[offset..offset + 8].copy_from_slice(&(x_origin * 180.0 / PI / 180.0).to_le_bytes());
        offset += 8;
        buf[offset..offset + 8].copy_from_slice(&(y_origin * 180.0 / PI / 180.0).to_le_bytes());
        offset += 8;
        buf[offset..offset + 8].copy_from_slice(&step_x.to_le_bytes());
        offset += 8;
        buf[offset..offset + 8].copy_from_slice(&step_y.to_le_bytes());
        offset += 8;

        // numPoints (4)
        buf[offset..offset + 4].copy_from_slice(&4i32.to_le_bytes());
        offset += 4;
        // numFaces (2)
        buf[offset..offset + 4].copy_from_slice(&2i32.to_le_bytes());
        offset += 4;
        // level (0)
        buf[offset..offset + 4].copy_from_slice(&0i32.to_le_bytes());
        offset += 4;

        // Points
        for j in 0..4 {
            let mut x_pos = 0u8;
            let mut y_pos = 0u8;
            let mut alt = alt_start;
            if (j & 1) != 0 {
                x_pos += 1;
                alt += 10.0;
            }
            if (j & 2) != 0 {
                y_pos += 1;
            }
            buf[offset] = x_pos;
            buf[offset + 1] = y_pos;
            offset += 2;
            buf[offset..offset + 4].copy_from_slice(&(alt * to_earth_radii).to_le_bytes());
            offset += 4;
        }

        // Faces: [0, 1, 2, 1, 3, 2]
        let indices: [u16; 6] = [0, 1, 2, 1, 3, 2];
        for idx in indices {
            buf[offset..offset + 2].copy_from_slice(&idx.to_le_bytes());
            offset += 2;
        }
    }

    buf
}

#[test]
fn test_child_availability() {
    // Default child mask (15) -> all available
    let data = GoogleEarthEnterpriseTerrainData::new(GoogleEarthEnterpriseTerrainDataOptions {
        buffer: vec![1, 2, 3, 4],
        negative_altitude_exponent_bias: 32,
        negative_elevation_threshold: 1e-12,
        child_tile_mask: None,
        created_by_upsampling: None,
        credits: None,
    })
    .unwrap();

    assert!(data.is_child_available(10, 20, 20, 40).unwrap()); // NW
    assert!(data.is_child_available(10, 20, 21, 40).unwrap()); // NE
    assert!(data.is_child_available(10, 20, 20, 41).unwrap()); // SW
    assert!(data.is_child_available(10, 20, 21, 41).unwrap()); // SE

    // Only Southwest child (1)
    let data_sw = GoogleEarthEnterpriseTerrainData::new(GoogleEarthEnterpriseTerrainDataOptions {
        buffer: vec![1, 2, 3, 4],
        negative_altitude_exponent_bias: 32,
        negative_elevation_threshold: 1e-12,
        child_tile_mask: Some(1),
        created_by_upsampling: None,
        credits: None,
    })
    .unwrap();
    assert!(!data_sw.is_child_available(10, 20, 20, 40).unwrap());
    assert!(!data_sw.is_child_available(10, 20, 21, 40).unwrap());
    assert!(data_sw.is_child_available(10, 20, 20, 41).unwrap());
    assert!(!data_sw.is_child_available(10, 20, 21, 41).unwrap());

    // Only Southeast child (2)
    let data_se = GoogleEarthEnterpriseTerrainData::new(GoogleEarthEnterpriseTerrainDataOptions {
        buffer: vec![1, 2, 3, 4],
        negative_altitude_exponent_bias: 32,
        negative_elevation_threshold: 1e-12,
        child_tile_mask: Some(2),
        created_by_upsampling: None,
        credits: None,
    })
    .unwrap();
    assert!(!data_se.is_child_available(10, 20, 20, 40).unwrap());
    assert!(!data_se.is_child_available(10, 20, 21, 40).unwrap());
    assert!(!data_se.is_child_available(10, 20, 20, 41).unwrap());
    assert!(data_se.is_child_available(10, 20, 21, 41).unwrap());

    // Only Northeast child (4)
    let data_ne = GoogleEarthEnterpriseTerrainData::new(GoogleEarthEnterpriseTerrainDataOptions {
        buffer: vec![1, 2, 3, 4],
        negative_altitude_exponent_bias: 32,
        negative_elevation_threshold: 1e-12,
        child_tile_mask: Some(4),
        created_by_upsampling: None,
        credits: None,
    })
    .unwrap();
    assert!(!data_ne.is_child_available(10, 20, 20, 40).unwrap());
    assert!(data_ne.is_child_available(10, 20, 21, 40).unwrap());
    assert!(!data_ne.is_child_available(10, 20, 20, 41).unwrap());
    assert!(!data_ne.is_child_available(10, 20, 21, 41).unwrap());

    // Only Northwest child (8)
    let data_nw = GoogleEarthEnterpriseTerrainData::new(GoogleEarthEnterpriseTerrainDataOptions {
        buffer: vec![1, 2, 3, 4],
        negative_altitude_exponent_bias: 32,
        negative_elevation_threshold: 1e-12,
        child_tile_mask: Some(8),
        created_by_upsampling: None,
        credits: None,
    })
    .unwrap();
    assert!(data_nw.is_child_available(10, 20, 20, 40).unwrap());
    assert!(!data_nw.is_child_available(10, 20, 21, 40).unwrap());
    assert!(!data_nw.is_child_available(10, 20, 20, 41).unwrap());
    assert!(!data_nw.is_child_available(10, 20, 21, 41).unwrap());
}

#[test]
fn test_create_mesh_and_upsample() {
    let tiling_scheme = GeographicTilingScheme::default();
    let buffer = create_test_buffer(&tiling_scheme, 0, 0, 0);

    let mut data = GoogleEarthEnterpriseTerrainData::new(GoogleEarthEnterpriseTerrainDataOptions {
        buffer,
        negative_altitude_exponent_bias: 32,
        negative_elevation_threshold: 1e-12,
        child_tile_mask: Some(15),
        created_by_upsampling: Some(false),
        credits: None,
    })
    .unwrap();

    assert!(!data.was_created_by_upsampling());

    let mesh = data.create_mesh(&tiling_scheme, 0, 0, 0, None, None).unwrap();
    assert_eq!(mesh.vertices.len(), 48); // 4 quads * 4 vertices * 3 coordinates
    assert_eq!(mesh.indices.len(), 24); // 4 quads * 2 faces * 3 indices

    // Upsample across > 1 level fails
    assert!(data.upsample(&tiling_scheme, 0, 0, 0, 0, 0, 2).is_err());

    // Upsample to single child succeeds
    let child = data.upsample(&tiling_scheme, 0, 0, 0, 0, 0, 1).unwrap();
    assert!(child.was_created_by_upsampling());
}

#[test]
fn test_interpolate_height() {
    let tiling_scheme = GeographicTilingScheme::default();
    let buffer = create_test_buffer(&tiling_scheme, 0, 0, 0);

    let mut data = GoogleEarthEnterpriseTerrainData::new(GoogleEarthEnterpriseTerrainDataOptions {
        buffer,
        negative_altitude_exponent_bias: 32,
        negative_elevation_threshold: 1e-12,
        child_tile_mask: Some(15),
        created_by_upsampling: None,
        credits: None,
    })
    .unwrap();

    let rect = GeoRectangle::degrees(0.0, 0.0, 10.0, 10.0).unwrap();
    assert!(data.interpolate_height(&rect, 0.0, 0.0).is_err());

    let _ = data.create_mesh(&tiling_scheme, 0, 0, 0, None, None).unwrap();
    let h = data.interpolate_height(&rect, (rect.west + rect.east) * 0.5, (rect.south + rect.north) * 0.5).unwrap();
    assert!(h >= 0.0);
}
