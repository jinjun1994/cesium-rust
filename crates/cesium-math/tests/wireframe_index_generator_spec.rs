use cesium_math::{PrimitiveType, WireframeIndexGenerator};

#[test]
fn test_create_wireframe_indices_returns_none_for_non_triangles() {
    assert!(WireframeIndexGenerator::create_wireframe_indices(PrimitiveType::Points, 6, None)
        .is_none());
    assert!(WireframeIndexGenerator::create_wireframe_indices(PrimitiveType::Lines, 6, None)
        .is_none());
    assert!(
        WireframeIndexGenerator::create_wireframe_indices(PrimitiveType::LineStrip, 6, None)
            .is_none()
    );
    assert!(
        WireframeIndexGenerator::create_wireframe_indices(PrimitiveType::LineLoop, 6, None)
            .is_none()
    );
}

#[test]
fn test_create_wireframe_indices_works_for_triangles() {
    let expected = vec![0, 1, 1, 2, 2, 0, 3, 4, 4, 5, 5, 3];
    let result =
        WireframeIndexGenerator::create_wireframe_indices(PrimitiveType::Triangles, 6, None)
            .expect("wireframe indices");
    assert_eq!(result.to_vec_u32(), expected);
}

#[test]
fn test_create_wireframe_indices_works_for_triangles_from_indices() {
    let indices = [1, 0, 2, 4, 5, 3];
    let expected = vec![1, 0, 0, 2, 2, 1, 4, 5, 5, 3, 3, 4];
    let result = WireframeIndexGenerator::create_wireframe_indices(
        PrimitiveType::Triangles,
        6,
        Some(&indices),
    )
    .expect("wireframe indices");
    assert_eq!(result.to_vec_u32(), expected);
}

#[test]
fn test_create_wireframe_indices_works_for_triangle_strip() {
    let expected = vec![
        0, 1, // First edge of the strip
        1, 2, 2, 0, // The next two edges of the first triangle
        2, 3, 3, 1, // The next two edges of the third triangle
        3, 4, 4, 2, // And so on...
        4, 5, 5, 3,
    ];
    let result =
        WireframeIndexGenerator::create_wireframe_indices(PrimitiveType::TriangleStrip, 6, None)
            .expect("wireframe indices");
    assert_eq!(result.to_vec_u32(), expected);
}

#[test]
fn test_create_wireframe_indices_works_for_triangle_strip_from_indices() {
    let indices = [1, 0, 2, 4, 5, 3];
    let expected = vec![
        1, 0, // First edge of the strip
        0, 2, 2, 1, // The next two edges of the first triangle
        2, 4, 4, 0, // The next two edges of the third triangle
        4, 5, 5, 2, // And so on...
        5, 3, 3, 4,
    ];
    let result = WireframeIndexGenerator::create_wireframe_indices(
        PrimitiveType::TriangleStrip,
        6,
        Some(&indices),
    )
    .expect("wireframe indices");
    assert_eq!(result.to_vec_u32(), expected);
}

#[test]
fn test_create_wireframe_indices_works_for_triangle_fan() {
    let expected = vec![
        0, 1, // First edge of the fan
        1, 2, 2, 0, // The next two edges of the first triangle
        2, 3, 3, 0, // The next two edges of the third triangle
        3, 4, 4, 0, // And so on...
        4, 5, 5, 0,
    ];
    let result =
        WireframeIndexGenerator::create_wireframe_indices(PrimitiveType::TriangleFan, 6, None)
            .expect("wireframe indices");
    assert_eq!(result.to_vec_u32(), expected);
}

#[test]
fn test_create_wireframe_indices_works_for_triangle_fan_from_indices() {
    let indices = [1, 0, 2, 4, 5, 3];
    let expected = vec![
        1, 0, // First edge of the fan
        0, 2, 2, 1, // The next two edges of the first triangle
        2, 4, 4, 1, // The next two edges of the third triangle
        4, 5, 5, 1, // And so on...
        5, 3, 3, 1,
    ];
    let result = WireframeIndexGenerator::create_wireframe_indices(
        PrimitiveType::TriangleFan,
        6,
        Some(&indices),
    )
    .expect("wireframe indices");
    assert_eq!(result.to_vec_u32(), expected);
}

#[test]
fn test_get_wireframe_indices_count_returns_original_count_for_non_triangles() {
    let original_count = 6;
    assert_eq!(
        WireframeIndexGenerator::get_wireframe_indices_count(
            PrimitiveType::Points,
            original_count
        ),
        original_count
    );
    assert_eq!(
        WireframeIndexGenerator::get_wireframe_indices_count(PrimitiveType::Lines, original_count),
        original_count
    );
    assert_eq!(
        WireframeIndexGenerator::get_wireframe_indices_count(
            PrimitiveType::LineStrip,
            original_count
        ),
        original_count
    );
    assert_eq!(
        WireframeIndexGenerator::get_wireframe_indices_count(
            PrimitiveType::LineLoop,
            original_count
        ),
        original_count
    );
}

#[test]
fn test_get_wireframe_indices_count_works() {
    let original_count = 6;
    assert_eq!(
        WireframeIndexGenerator::get_wireframe_indices_count(
            PrimitiveType::Triangles,
            original_count
        ),
        12
    );
    assert_eq!(
        WireframeIndexGenerator::get_wireframe_indices_count(
            PrimitiveType::TriangleStrip,
            original_count
        ),
        18
    );
    assert_eq!(
        WireframeIndexGenerator::get_wireframe_indices_count(
            PrimitiveType::TriangleFan,
            original_count
        ),
        18
    );
}
