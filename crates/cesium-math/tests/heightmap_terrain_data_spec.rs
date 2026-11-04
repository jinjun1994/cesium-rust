use cesium_math::{
    CreateMeshOptions, GeographicTilingScheme, HeightmapBuffer, HeightmapBufferType,
    HeightmapEncoding, HeightmapStructure, HeightmapTerrainData, HeightmapTerrainDataOptions,
    MAXIMUM_ASYNCHRONOUS_TASKS,
};

#[test]
fn test_constructor_requires_buffer() {
    let err = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
        width: Some(5),
        height: Some(5),
        ..Default::default()
    });
    assert!(err.is_err());
}

#[test]
fn test_constructor_requires_width_and_height() {
    let err_w = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
        buffer: Some(HeightmapBuffer::Float32(vec![0.0; 25])),
        height: Some(5),
        ..Default::default()
    });
    assert!(err_w.is_err());

    let err_h = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
        buffer: Some(HeightmapBuffer::Float32(vec![0.0; 25])),
        width: Some(5),
        ..Default::default()
    });
    assert!(err_h.is_err());
}

#[test]
fn test_non_lerc_encoded_buffers_sets_correct_buffer_type() {
    let data = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
        buffer: Some(HeightmapBuffer::Uint16(vec![0; 25])),
        width: Some(5),
        height: Some(5),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(data.encoding(), HeightmapEncoding::None);
    assert_eq!(data.buffer_type(), HeightmapBufferType::Uint16);
}

#[test]
fn test_lerc_encoded_buffers_sets_correct_buffer_type() {
    let data = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
        buffer: Some(HeightmapBuffer::Uint16(vec![0; 25])),
        width: Some(5),
        height: Some(5),
        encoding: Some(HeightmapEncoding::Lerc),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(data.encoding(), HeightmapEncoding::Lerc);
    assert_eq!(data.buffer_type(), HeightmapBufferType::Float32);
}

#[test]
fn test_create_mesh_and_maximum_asynchronous_tasks() {
    assert_eq!(MAXIMUM_ASYNCHRONOUS_TASKS, 5);

    let tiling_scheme = GeographicTilingScheme::default();
    let mut data = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
        buffer: Some(HeightmapBuffer::Float32(vec![0.0; 25])),
        width: Some(5),
        height: Some(5),
        ..Default::default()
    })
    .unwrap();

    let mesh = data
        .create_mesh(CreateMeshOptions {
            tiling_scheme: &tiling_scheme,
            x: 0,
            y: 0,
            level: 0,
            exaggeration: 1.0,
            exaggeration_relative_height: 0.0,
            throttle: false,
        })
        .unwrap()
        .expect("mesh created");

    assert_eq!(mesh.vertex_count_without_skirts, 25);
    assert_eq!(mesh.index_count_without_skirts, (5 - 1) * (5 - 1) * 6);
}

#[test]
fn test_upsample_validation() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut data = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
        buffer: Some(HeightmapBuffer::Float32(vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0,
        ])),
        width: Some(3),
        height: Some(3),
        ..Default::default()
    })
    .unwrap();

    // Prior to createMesh, upsample should fail
    let err_no_mesh = data.upsample(
        Some(&tiling_scheme),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some(1),
    );
    assert!(err_no_mesh.is_err());

    // Create mesh
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

    // Missing arguments
    assert!(data
        .upsample::<GeographicTilingScheme>(
            None,
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(1)
        )
        .is_err());
    assert!(data
        .upsample(
            Some(&tiling_scheme),
            None,
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(1)
        )
        .is_err());

    // Across more than one level
    assert!(data
        .upsample(
            Some(&tiling_scheme),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(2)
        )
        .is_err());
}

#[test]
fn test_upsamples_float32() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut data = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
        buffer: Some(HeightmapBuffer::Float32(vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        ])),
        width: Some(4),
        height: Some(4),
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

    let upsampled = data
        .upsample(
            Some(&tiling_scheme),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(1),
        )
        .unwrap();

    assert!(upsampled.was_created_by_upsampling());
    assert_eq!(upsampled.width(), 4);
    assert_eq!(upsampled.height(), 4);

    let expected = vec![
        1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 6.5, 7.0, 7.5, 8.0, 8.5,
    ];
    let actual = upsampled.buffer().unwrap().to_vec_f64();
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert!(
            (a - e).abs() < 1e-4,
            "actual {:?} != expected {:?}",
            actual,
            expected
        );
    }
}

#[test]
fn test_upsample_works_with_a_stride() {
    let tiling_scheme = GeographicTilingScheme::default();
    let raw: Vec<u8> = vec![
        1, 1, 10, 2, 1, 10, 3, 1, 10, 4, 1, 10, 5, 1, 10, 6, 1, 10, 7, 1, 10, 8, 1, 10, 9, 1, 10,
        10, 1, 10, 11, 1, 10, 12, 1, 10, 13, 1, 10, 14, 1, 10, 15, 1, 10, 16, 1, 10,
    ];
    let mut data = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
        buffer: Some(HeightmapBuffer::Uint8(raw)),
        width: Some(4),
        height: Some(4),
        structure: Some(HeightmapStructure {
            stride: 3,
            elements_per_height: 2,
            ..Default::default()
        }),
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

    let upsampled = data
        .upsample(
            Some(&tiling_scheme),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(1),
        )
        .unwrap();

    assert!(upsampled.was_created_by_upsampling());
    assert_eq!(upsampled.width(), 4);
    assert_eq!(upsampled.height(), 4);

    let expected: Vec<u8> = vec![
        1, 1, 0, 1, 1, 0, 2, 1, 0, 2, 1, 0, 3, 1, 0, 3, 1, 0, 4, 1, 0, 4, 1, 0, 5, 1, 0, 5, 1, 0,
        6, 1, 0, 6, 1, 0, 7, 1, 0, 7, 1, 0, 8, 1, 0, 8, 1, 0,
    ];
    if let Some(HeightmapBuffer::Uint8(buf)) = upsampled.buffer() {
        assert_eq!(buf.as_slice(), expected.as_slice());
    } else {
        panic!("expected Uint8 buffer");
    }
}

#[test]
fn test_upsample_works_with_a_big_endian_stride() {
    let tiling_scheme = GeographicTilingScheme::default();
    let raw: Vec<u8> = vec![
        1, 1, 10, 1, 2, 10, 1, 3, 10, 1, 4, 10, 1, 5, 10, 1, 6, 10, 1, 7, 10, 1, 8, 10, 1, 9, 10,
        1, 10, 10, 1, 11, 10, 1, 12, 10, 1, 13, 10, 1, 14, 10, 1, 15, 10, 1, 16, 10,
    ];
    let mut data = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
        buffer: Some(HeightmapBuffer::Uint8(raw)),
        width: Some(4),
        height: Some(4),
        structure: Some(HeightmapStructure {
            stride: 3,
            elements_per_height: 2,
            is_big_endian: true,
            ..Default::default()
        }),
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

    let upsampled = data
        .upsample(
            Some(&tiling_scheme),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(1),
        )
        .unwrap();

    let expected: Vec<u8> = vec![
        1, 1, 0, 1, 1, 0, 1, 2, 0, 1, 2, 0, 1, 3, 0, 1, 3, 0, 1, 4, 0, 1, 4, 0, 1, 5, 0, 1, 5, 0,
        1, 6, 0, 1, 6, 0, 1, 7, 0, 1, 7, 0, 1, 8, 0, 1, 8, 0,
    ];
    if let Some(HeightmapBuffer::Uint8(buf)) = upsampled.buffer() {
        assert_eq!(buf.as_slice(), expected.as_slice());
    } else {
        panic!("expected Uint8 buffer");
    }
}

#[test]
fn test_upsample_works_for_an_eastern_child() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut data = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
        buffer: Some(HeightmapBuffer::Float32(vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        ])),
        width: Some(4),
        height: Some(4),
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

    let upsampled = data
        .upsample(
            Some(&tiling_scheme),
            Some(0),
            Some(0),
            Some(0),
            Some(1),
            Some(0),
            Some(1),
        )
        .unwrap();

    let expected = vec![
        2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 6.5, 7.0, 7.5, 8.0, 8.5, 9.0, 9.5, 10.0,
    ];
    let actual = upsampled.buffer().unwrap().to_vec_f64();
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert!(
            (a - e).abs() < 1e-4,
            "actual {:?} != expected {:?}",
            actual,
            expected
        );
    }
}

#[test]
fn test_upsample_works_with_a_stride_for_an_eastern_child() {
    let tiling_scheme = GeographicTilingScheme::default();
    let raw: Vec<u8> = vec![
        1, 1, 10, 2, 1, 10, 3, 1, 10, 4, 1, 10, 5, 1, 10, 6, 1, 10, 7, 1, 10, 8, 1, 10, 9, 1, 10,
        10, 1, 10, 11, 1, 10, 12, 1, 10, 13, 1, 10, 14, 1, 10, 15, 1, 10, 16, 1, 10,
    ];
    let mut data = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
        buffer: Some(HeightmapBuffer::Uint8(raw)),
        width: Some(4),
        height: Some(4),
        structure: Some(HeightmapStructure {
            stride: 3,
            elements_per_height: 2,
            ..Default::default()
        }),
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

    let upsampled = data
        .upsample(
            Some(&tiling_scheme),
            Some(0),
            Some(0),
            Some(0),
            Some(1),
            Some(0),
            Some(1),
        )
        .unwrap();

    let expected: Vec<u8> = vec![
        2, 1, 0, 3, 1, 0, 3, 1, 0, 4, 1, 0, 4, 1, 0, 5, 1, 0, 5, 1, 0, 6, 1, 0, 6, 1, 0, 7, 1, 0,
        7, 1, 0, 8, 1, 0, 8, 1, 0, 9, 1, 0, 9, 1, 0, 10, 1, 0,
    ];
    if let Some(HeightmapBuffer::Uint8(buf)) = upsampled.buffer() {
        assert_eq!(buf.as_slice(), expected.as_slice());
    } else {
        panic!("expected Uint8 buffer");
    }
}

#[test]
fn test_upsample_clamps_out_of_range_data() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut data = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
        buffer: Some(HeightmapBuffer::Float32(vec![
            -1.0, -2.0, -3.0, -4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
            16.0,
        ])),
        width: Some(4),
        height: Some(4),
        structure: Some(HeightmapStructure {
            stride: 1,
            elements_per_height: 1,
            lowest_encoded_height: Some(1.0),
            highest_encoded_height: Some(7.0),
            ..Default::default()
        }),
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

    let upsampled = data
        .upsample(
            Some(&tiling_scheme),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(1),
        )
        .unwrap();

    let expected = vec![
        1.0, 1.0, 1.0, 1.0, 2.0, 1.5, 2.0, 1.5, 5.0, 5.5, 6.0, 6.5, 7.0, 7.0, 7.0, 7.0,
    ];
    let actual = upsampled.buffer().unwrap().to_vec_f64();
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert!(
            (a - e).abs() < 1e-4,
            "actual {:?} != expected {:?}",
            actual,
            expected
        );
    }
}

#[test]
fn test_is_child_available() {
    // child_tile_mask = 1 | 4 = 5 (SouthWest and NorthWest available)
    let data = HeightmapTerrainData::new(HeightmapTerrainDataOptions {
        buffer: Some(HeightmapBuffer::Float32(vec![0.0; 9])),
        width: Some(3),
        height: Some(3),
        child_tile_mask: Some(5),
        ..Default::default()
    })
    .unwrap();

    // Requires params
    assert!(data.is_child_available(None, Some(0), Some(0), Some(0)).is_err());
    assert!(data.is_child_available(Some(0), None, Some(0), Some(0)).is_err());
    assert!(data.is_child_available(Some(0), Some(0), None, Some(0)).is_err());
    assert!(data.is_child_available(Some(0), Some(0), Some(0), None).is_err());

    // SW (thisX*2, thisY*2+1) -> true
    assert!(data.is_child_available(Some(0), Some(0), Some(0), Some(1)).unwrap());
    // SE (thisX*2+1, thisY*2+1) -> false
    assert!(!data.is_child_available(Some(0), Some(0), Some(1), Some(1)).unwrap());
    // NW (thisX*2, thisY*2) -> true
    assert!(data.is_child_available(Some(0), Some(0), Some(0), Some(0)).unwrap());
    // NE (thisX*2+1, thisY*2) -> false
    assert!(!data.is_child_available(Some(0), Some(0), Some(1), Some(0)).unwrap());
}
