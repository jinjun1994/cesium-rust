use cesium_math::{
    Cartesian3, CreateMeshOptions, GeographicTilingScheme,
    QuantizedMeshTerrainData, QuantizedMeshTerrainDataOptions, Sphere,
};

fn find_vertex_with_coordinates(u_buffer: &[u16], v_buffer: &[u16], u: f64, v: f64) -> isize {
    let u_target = (u * 32767.0).round() as i32;
    let v_target = (v * 32767.0).round() as i32;
    for (i, (&ub, &vb)) in u_buffer.iter().zip(v_buffer.iter()).enumerate() {
        if (ub as i32 - u_target).abs() <= 1 && (vb as i32 - v_target).abs() <= 1 {
            return i as isize;
        }
    }
    -1
}

fn has_triangle(ib: &[u32], i0: isize, i1: isize, i2: isize) -> bool {
    if i0 < 0 || i1 < 0 || i2 < 0 {
        return false;
    }
    let (u0, u1, u2) = (i0 as u32, i1 as u32, i2 as u32);
    for chunk in ib.as_chunks::<3>().0 {
        if (chunk[0] == u0 && chunk[1] == u1 && chunk[2] == u2)
            || (chunk[0] == u1 && chunk[1] == u2 && chunk[2] == u0)
            || (chunk[0] == u2 && chunk[1] == u0 && chunk[2] == u1)
        {
            return true;
        }
    }
    false
}

#[test]
fn test_create_mesh_and_exaggeration() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut data = QuantizedMeshTerrainData::new(QuantizedMeshTerrainDataOptions {
        minimum_height: Some(0.0),
        maximum_height: Some(4.0),
        quantized_vertices: Some(vec![
            0, 0, 32767, 32767, // u
            0, 32767, 0, 32767, // v
            8191, 16383, 24575, 32767, // heights
        ]),
        indices: Some(vec![0, 3, 1, 0, 2, 3]),
        bounding_sphere: Some(Sphere::new(Cartesian3::new(1.0, 2.0, 3.0), 10000.0)),
        horizon_occlusion_point: Some(Cartesian3::ZERO),
        west_indices: Some(vec![0, 1]),
        south_indices: Some(vec![0, 2]),
        east_indices: Some(vec![2, 3]),
        north_indices: Some(vec![1, 3]),
        west_skirt_height: Some(1.0),
        south_skirt_height: Some(1.0),
        east_skirt_height: Some(1.0),
        north_skirt_height: Some(1.0),
        child_tile_mask: Some(15),
        ..Default::default()
    })
    .unwrap();

    let mesh = data
        .create_mesh(CreateMeshOptions {
            tiling_scheme: &tiling_scheme,
            x: 0,
            y: 0,
            level: 0,
            exaggeration: 2.0,
            exaggeration_relative_height: 0.0,
            throttle: false,
        })
        .unwrap()
        .expect("mesh created");

    // 4 regular vertices + 8 skirt vertices = 12 vertices
    assert_eq!(mesh.vertices.len(), 12 * mesh.encoding.stride);
    // 2 regular triangles + 8 skirt triangles = 10 * 3 = 30 indices
    assert_eq!(mesh.indices.len(), 10 * 3);
    assert_eq!(mesh.minimum_height, 0.0);
    assert_eq!(mesh.maximum_height, 4.0);
    assert_eq!(mesh.encoding.exaggeration, 2.0);
}

#[test]
fn test_upsample_simple_quad() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut data = QuantizedMeshTerrainData::new(QuantizedMeshTerrainDataOptions {
        minimum_height: Some(0.0),
        maximum_height: Some(4.0),
        quantized_vertices: Some(vec![
            0, 0, 32767, 32767, // u
            0, 32767, 0, 32767, // v
            8191, 16383, 24575, 32767, // heights
        ]),
        indices: Some(vec![0, 3, 1, 0, 2, 3]),
        bounding_sphere: Some(Sphere::new(Cartesian3::ZERO, 10000.0)),
        horizon_occlusion_point: Some(Cartesian3::ZERO),
        west_indices: Some(vec![]),
        south_indices: Some(vec![]),
        east_indices: Some(vec![]),
        north_indices: Some(vec![]),
        west_skirt_height: Some(1.0),
        south_skirt_height: Some(1.0),
        east_skirt_height: Some(1.0),
        north_skirt_height: Some(1.0),
        child_tile_mask: Some(15),
        ..Default::default()
    })
    .unwrap();

    data.create_mesh(CreateMeshOptions {
        tiling_scheme: &tiling_scheme,
        x: 0,
        y: 0,
        level: 0,
        exaggeration: 1.0,
        exaggeration_relative_height: 0.0,
        throttle: false,
    })
    .unwrap();

    let sw = data.upsample(Some(&tiling_scheme), Some(0), Some(0), Some(0), Some(0), Some(0), Some(1)).unwrap();
    let se = data.upsample(Some(&tiling_scheme), Some(0), Some(0), Some(0), Some(1), Some(0), Some(1)).unwrap();
    let nw = data.upsample(Some(&tiling_scheme), Some(0), Some(0), Some(0), Some(0), Some(1), Some(1)).unwrap();
    let ne = data.upsample(Some(&tiling_scheme), Some(0), Some(0), Some(0), Some(1), Some(1), Some(1)).unwrap();

    let children = [sw, se, nw, ne];
    for child in &children {
        let u_buffer = child.u_values();
        let v_buffer = child.v_values();
        let ib = child.indices();

        assert_eq!(u_buffer.len(), 4);
        assert_eq!(v_buffer.len(), 4);
        assert_eq!(child.height_values().len(), 4);
        assert_eq!(ib.len(), 6);

        let sw_idx = find_vertex_with_coordinates(u_buffer, v_buffer, 0.0, 0.0);
        assert_ne!(sw_idx, -1);
        let nw_idx = find_vertex_with_coordinates(u_buffer, v_buffer, 0.0, 1.0);
        assert_ne!(nw_idx, -1);
        let se_idx = find_vertex_with_coordinates(u_buffer, v_buffer, 1.0, 0.0);
        assert_ne!(se_idx, -1);
        let ne_idx = find_vertex_with_coordinates(u_buffer, v_buffer, 1.0, 1.0);
        assert_ne!(ne_idx, -1);

        let nw_to_se = has_triangle(ib, sw_idx, se_idx, nw_idx) && has_triangle(ib, nw_idx, se_idx, ne_idx);
        let sw_to_ne = has_triangle(ib, sw_idx, ne_idx, nw_idx) && has_triangle(ib, sw_idx, se_idx, ne_idx);
        assert!(nw_to_se || sw_to_ne);
    }
}

#[test]
fn test_oct_encoded_normals_upsample() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut data = QuantizedMeshTerrainData::new(QuantizedMeshTerrainDataOptions {
        minimum_height: Some(0.0),
        maximum_height: Some(4.0),
        quantized_vertices: Some(vec![
            0, 0, 32767, 32767,
            0, 32767, 0, 32767,
            8191, 16383, 24575, 32767,
        ]),
        encoded_normals: Some(vec![
            255, 0,
            255, 255,
            255, 0,
            255, 255,
        ]),
        indices: Some(vec![0, 3, 1, 0, 2, 3]),
        bounding_sphere: Some(Sphere::new(Cartesian3::ZERO, 10000.0)),
        horizon_occlusion_point: Some(Cartesian3::ZERO),
        west_indices: Some(vec![]),
        south_indices: Some(vec![]),
        east_indices: Some(vec![]),
        north_indices: Some(vec![]),
        west_skirt_height: Some(1.0),
        south_skirt_height: Some(1.0),
        east_skirt_height: Some(1.0),
        north_skirt_height: Some(1.0),
        child_tile_mask: Some(15),
        ..Default::default()
    })
    .unwrap();

    data.create_mesh(CreateMeshOptions {
        tiling_scheme: &tiling_scheme,
        x: 0,
        y: 0,
        level: 0,
        exaggeration: 1.0,
        exaggeration_relative_height: 0.0,
        throttle: false,
    })
    .unwrap();

    let sw = data.upsample(Some(&tiling_scheme), Some(0), Some(0), Some(0), Some(0), Some(0), Some(1)).unwrap();
    let normals = sw.encoded_normals().expect("normals present");
    assert_eq!(normals.len(), 8);
}

#[test]
fn test_interpolate_height() {
    let tiling_scheme = GeographicTilingScheme::default();
    let rectangle = tiling_scheme.tile_rectangle(7, 6, 5).unwrap();

    let mesh = QuantizedMeshTerrainData::new(QuantizedMeshTerrainDataOptions {
        minimum_height: Some(-16384.0),
        maximum_height: Some(16383.0),
        quantized_vertices: Some(vec![
            0, 0, 32767, 32767,
            0, 32767, 0, 32767,
            16384, 0, 32767, 16384,
        ]),
        indices: Some(vec![0, 3, 1, 0, 2, 3]),
        bounding_sphere: Some(Sphere::new(Cartesian3::ZERO, 10000.0)),
        horizon_occlusion_point: Some(Cartesian3::ZERO),
        west_indices: Some(vec![0, 1]),
        south_indices: Some(vec![0, 1]),
        east_indices: Some(vec![2, 3]),
        north_indices: Some(vec![1, 3]),
        west_skirt_height: Some(1.0),
        south_skirt_height: Some(1.0),
        east_skirt_height: Some(1.0),
        north_skirt_height: Some(1.0),
        child_tile_mask: Some(15),
        ..Default::default()
    })
    .unwrap();

    // Position outside mesh should clamp
    let clamped_result = mesh.interpolate_height(&rectangle, 0.0, 0.0);
    let border_result = mesh.interpolate_height(&rectangle, rectangle.east, rectangle.south);
    assert_eq!(clamped_result, border_result);

    // Northwest quadrant: negative height
    let lon_nw = rectangle.west + (rectangle.east - rectangle.west) * 0.25;
    let lat_nw = rectangle.south + (rectangle.north - rectangle.south) * 0.75;
    let res_nw = mesh.interpolate_height(&rectangle, lon_nw, lat_nw).unwrap();
    assert!(res_nw < 0.0);

    // Southeast quadrant: positive height
    let lon_se = rectangle.west + (rectangle.east - rectangle.west) * 0.75;
    let lat_se = rectangle.south + (rectangle.north - rectangle.south) * 0.25;
    let res_se = mesh.interpolate_height(&rectangle, lon_se, lat_se).unwrap();
    assert!(res_se > 0.0);

    // Center diagonal: 0.0
    let lon_mid = rectangle.west + (rectangle.east - rectangle.west) * 0.5;
    let lat_mid = rectangle.south + (rectangle.north - rectangle.south) * 0.5;
    let res_mid = mesh.interpolate_height(&rectangle, lon_mid, lat_mid).unwrap();
    assert!(res_mid.abs() < 1e-4);
}

#[test]
fn test_is_child_available() {
    let data = QuantizedMeshTerrainData::new(QuantizedMeshTerrainDataOptions {
        minimum_height: Some(0.0),
        maximum_height: Some(10.0),
        quantized_vertices: Some(vec![0; 12]),
        indices: Some(vec![0, 1, 2]),
        bounding_sphere: Some(Sphere::new(Cartesian3::ZERO, 1000.0)),
        horizon_occlusion_point: Some(Cartesian3::ZERO),
        west_indices: Some(vec![]),
        south_indices: Some(vec![]),
        east_indices: Some(vec![]),
        north_indices: Some(vec![]),
        west_skirt_height: Some(1.0),
        south_skirt_height: Some(1.0),
        east_skirt_height: Some(1.0),
        north_skirt_height: Some(1.0),
        child_tile_mask: Some(1), // Southwest only
        ..Default::default()
    })
    .unwrap();

    // Requires params
    assert!(data.is_child_available(None, Some(0), Some(0), Some(0)).is_err());
    assert!(data.is_child_available(Some(0), None, Some(0), Some(0)).is_err());
    assert!(data.is_child_available(Some(0), Some(0), None, Some(0)).is_err());
    assert!(data.is_child_available(Some(0), Some(0), Some(0), None).is_err());

    // Only southwest (child_x: 20, child_y: 41) is available
    assert!(!data.is_child_available(Some(10), Some(20), Some(20), Some(40)).unwrap());
    assert!(!data.is_child_available(Some(10), Some(20), Some(21), Some(40)).unwrap());
    assert!(data.is_child_available(Some(10), Some(20), Some(20), Some(41)).unwrap());
    assert!(!data.is_child_available(Some(10), Some(20), Some(21), Some(41)).unwrap());
}

#[test]
fn test_constructor_validation() {
    assert!(QuantizedMeshTerrainData::new(QuantizedMeshTerrainDataOptions::default()).is_err());
    assert!(QuantizedMeshTerrainData::new(QuantizedMeshTerrainDataOptions {
        quantized_vertices: Some(vec![0; 12]),
        ..Default::default()
    }).is_err());
}
