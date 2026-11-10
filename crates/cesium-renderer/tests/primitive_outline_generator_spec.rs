use cesium_renderer::primitive_outline_generator::{
    PrimitiveOutlineGenerator, PrimitiveOutlineGeneratorOptions,
};

#[test]
fn test_throws_for_undefined_arguments() {
    // throws for undefined triangleIndices
    let err = PrimitiveOutlineGenerator::try_new(PrimitiveOutlineGeneratorOptions {
        triangle_indices: None,
        outline_indices: Some(vec![0, 1]),
        original_vertex_count: Some(6),
    })
    .unwrap_err();
    assert_eq!(err.message, "options.triangleIndices is required.");

    // throws for undefined outlineIndices
    let err = PrimitiveOutlineGenerator::try_new(PrimitiveOutlineGeneratorOptions {
        triangle_indices: Some(vec![0, 1, 2]),
        outline_indices: None,
        original_vertex_count: Some(6),
    })
    .unwrap_err();
    assert_eq!(err.message, "options.outlineIndices is required.");

    // throws for undefined originalVertexCount
    let err = PrimitiveOutlineGenerator::try_new(PrimitiveOutlineGeneratorOptions {
        triangle_indices: Some(vec![0, 1, 2]),
        outline_indices: Some(vec![0, 1]),
        original_vertex_count: None,
    })
    .unwrap_err();
    assert_eq!(err.message, "options.originalVertexCount is required.");
}

#[test]
fn test_generates_outline_coordinates_for_simple_quad() {
    // Quad with 4 vertices, 2 triangles:
    // 3---2
    // | / |
    // 0---1
    // Triangles: (0, 1, 2), (0, 2, 3)
    // Outline: 0-1, 1-2, 2-3 (3 sides)
    let triangle_indices = vec![0, 1, 2, 0, 2, 3];
    let outline_indices = vec![0, 1, 1, 2, 2, 3];
    let original_vertex_count = 4;

    let generator = PrimitiveOutlineGenerator::new(PrimitiveOutlineGeneratorOptions {
        triangle_indices: Some(triangle_indices),
        outline_indices: Some(outline_indices),
        original_vertex_count: Some(original_vertex_count),
    });

    // Vertex 0 is duplicated because 0 has edge (0-1) in first triangle and no edge in second
    assert_eq!(generator.extra_vertices.len(), 1);
    assert_eq!(generator.extra_vertices[0], 0);

    // Total vertices: 4 + 1 = 5
    assert_eq!(generator.outline_coordinates.len(), 5 * 3);

    // Attribute update test
    let positions = vec![
        0.0, 0.0, 0.0, // 0
        1.0, 0.0, 0.0, // 1
        1.0, 1.0, 0.0, // 2
        0.0, 1.0, 0.0, // 3
    ];
    let updated_positions = generator.update_attribute(&positions);
    assert_eq!(updated_positions.len(), 5 * 3);
    assert_eq!(&updated_positions[12..15], &[0.0, 0.0, 0.0]); // copy of vertex 0
}

#[test]
fn test_generates_outline_for_grid_geometry() {
    // 6---7---8
    // | \ | / |
    // 3---4---5
    // | / | \ |
    // 0---1---2
    let indices = vec![
        0, 1, 4,
        0, 4, 3,
        1, 2, 4,
        4, 2, 5,
        3, 4, 6,
        6, 4, 7,
        4, 5, 8,
        4, 8, 7,
    ];
    // Exterior perimeter
    let edges = vec![
        0, 1,
        1, 2,
        2, 5,
        5, 8,
        8, 7,
        7, 6,
        6, 3,
    ];

    let generator = PrimitiveOutlineGenerator::new(PrimitiveOutlineGeneratorOptions {
        triangle_indices: Some(indices),
        outline_indices: Some(edges),
        original_vertex_count: Some(9),
    });

    assert!(!generator.outline_coordinates.is_empty());
    assert_eq!(
        generator.outline_coordinates.len(),
        (9 + generator.extra_vertices.len()) * 3
    );
}

#[test]
fn test_create_mip_level() {
    let m1 = PrimitiveOutlineGenerator::create_mip_level(1);
    assert_eq!(m1, vec![12]);

    let m2 = PrimitiveOutlineGenerator::create_mip_level(2);
    assert_eq!(m2, vec![0, 24]);

    let m4 = PrimitiveOutlineGenerator::create_mip_level(4);
    assert_eq!(m4, vec![0, 0, 0, 48]);

    let m8 = PrimitiveOutlineGenerator::create_mip_level(8);
    assert_eq!(m8, vec![0, 0, 0, 0, 0, 0, 0, 96]);

    let m16 = PrimitiveOutlineGenerator::create_mip_level(16);
    assert_eq!(m16[15], 192);
}

#[test]
fn test_generates_outline_for_tricky_geometry() {
    // 1-2     5-4
    //  \|     |/
    //   0-----3
    //    \   /
    //     \ /
    //      6
    //     / \
    //    /   \
    //   7-----8
    let indices = vec![
        0, 2, 1,
        3, 4, 5,
        6, 7, 8,
        0, 6, 3,
    ];
    let edges = vec![
        0, 1,
        0, 2,
        3, 5,
        3, 4,
        6, 7,
        6, 8,
        0, 3,
        0, 6,
        3, 6,
    ];
    let original_vertex_count = 9;

    let generator = PrimitiveOutlineGenerator::new(PrimitiveOutlineGeneratorOptions {
        triangle_indices: Some(indices.clone()),
        outline_indices: Some(edges),
        original_vertex_count: Some(original_vertex_count),
    });

    let new_vertex_count = original_vertex_count + generator.extra_vertices.len();
    assert_eq!(generator.outline_coordinates.len(), new_vertex_count * 3);

    // Verify coordinates are values in {0.0, 1.0, 2.0}
    for &coord in &generator.outline_coordinates {
        assert!(coord == 0.0 || coord == 1.0 || coord == 2.0);
    }

    // Verify indices updated correctly
    for &idx in &generator.updated_triangle_indices {
        assert!((idx as usize) < new_vertex_count);
    }
}
