use cesium_math::Intersections2D;

fn assert_close(a: f64, b: f64, eps: f64) {
    assert!((a - b).abs() <= eps, "expected {b}, got {a}");
}

#[test]
fn test_clip_eliminates_triangle_entirely_wrong_side() {
    let result = Intersections2D::clip_triangle_at_axis_aligned_threshold(
        0.1, false, 0.2, 0.3, 0.4,
    );
    assert_eq!(result.len(), 0);
}

#[test]
fn test_clip_keeps_triangle_entirely_correct_side() {
    let result = Intersections2D::clip_triangle_at_axis_aligned_threshold(
        0.1, true, 0.2, 0.3, 0.4,
    );
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], 0.0);
    assert_eq!(result[1], 1.0);
    assert_eq!(result[2], 2.0);
}

#[test]
fn test_clip_point_0_wrong_side_and_above() {
    let result = Intersections2D::clip_triangle_at_axis_aligned_threshold(
        0.5, false, 0.6, 0.4, 0.2,
    );
    assert_eq!(result.len(), 10);
    assert_eq!(result[0], 1.0);
    assert_eq!(result[1], 2.0);
    assert_eq!(result[2], -1.0);
    assert_eq!(result[3], 0.0);
    assert_eq!(result[4], 2.0);
    assert_close(result[5], 0.25, 1e-14);
    assert_eq!(result[6], -1.0);
    assert_eq!(result[7], 0.0);
    assert_eq!(result[8], 1.0);
    assert_close(result[9], 0.5, 1e-14);
}

#[test]
fn test_clip_point_0_wrong_side_and_below() {
    let result = Intersections2D::clip_triangle_at_axis_aligned_threshold(
        0.5, true, 0.4, 0.6, 0.8,
    );
    assert_eq!(result.len(), 10);
    assert_eq!(result[0], 1.0);
    assert_eq!(result[1], 2.0);
    assert_eq!(result[2], -1.0);
    assert_eq!(result[3], 0.0);
    assert_eq!(result[4], 2.0);
    assert_close(result[5], 0.25, 1e-14);
    assert_eq!(result[6], -1.0);
    assert_eq!(result[7], 0.0);
    assert_eq!(result[8], 1.0);
    assert_close(result[9], 0.5, 1e-14);
}

#[test]
fn test_clip_point_1_wrong_side_and_above() {
    let result = Intersections2D::clip_triangle_at_axis_aligned_threshold(
        0.5, false, 0.2, 0.6, 0.4,
    );
    assert_eq!(result.len(), 10);
    assert_eq!(result[0], 2.0);
    assert_eq!(result[1], 0.0);
    assert_eq!(result[2], -1.0);
    assert_eq!(result[3], 1.0);
    assert_eq!(result[4], 0.0);
    assert_close(result[5], 0.25, 1e-14);
    assert_eq!(result[6], -1.0);
    assert_eq!(result[7], 1.0);
    assert_eq!(result[8], 2.0);
    assert_close(result[9], 0.5, 1e-14);
}

#[test]
fn test_clip_point_1_wrong_side_and_below() {
    let result = Intersections2D::clip_triangle_at_axis_aligned_threshold(
        0.5, true, 0.8, 0.4, 0.6,
    );
    assert_eq!(result.len(), 10);
    assert_eq!(result[0], 2.0);
    assert_eq!(result[1], 0.0);
    assert_eq!(result[2], -1.0);
    assert_eq!(result[3], 1.0);
    assert_eq!(result[4], 0.0);
    assert_close(result[5], 0.25, 1e-14);
    assert_eq!(result[6], -1.0);
    assert_eq!(result[7], 1.0);
    assert_eq!(result[8], 2.0);
    assert_close(result[9], 0.5, 1e-14);
}

#[test]
fn test_clip_point_2_wrong_side_and_above() {
    let result = Intersections2D::clip_triangle_at_axis_aligned_threshold(
        0.5, false, 0.4, 0.2, 0.6,
    );
    assert_eq!(result.len(), 10);
    assert_eq!(result[0], 0.0);
    assert_eq!(result[1], 1.0);
    assert_eq!(result[2], -1.0);
    assert_eq!(result[3], 2.0);
    assert_eq!(result[4], 1.0);
    assert_close(result[5], 0.25, 1e-14);
    assert_eq!(result[6], -1.0);
    assert_eq!(result[7], 2.0);
    assert_eq!(result[8], 0.0);
    assert_close(result[9], 0.5, 1e-14);
}

#[test]
fn test_clip_point_2_wrong_side_and_below() {
    let result = Intersections2D::clip_triangle_at_axis_aligned_threshold(
        0.5, true, 0.6, 0.8, 0.4,
    );
    assert_eq!(result.len(), 10);
    assert_eq!(result[0], 0.0);
    assert_eq!(result[1], 1.0);
    assert_eq!(result[2], -1.0);
    assert_eq!(result[3], 2.0);
    assert_eq!(result[4], 1.0);
    assert_close(result[5], 0.25, 1e-14);
    assert_eq!(result[6], -1.0);
    assert_eq!(result[7], 2.0);
    assert_eq!(result[8], 0.0);
    assert_close(result[9], 0.5, 1e-14);
}

#[test]
fn test_clip_only_point_0_right_side_and_below() {
    let result = Intersections2D::clip_triangle_at_axis_aligned_threshold(
        0.5, false, 0.4, 0.6, 0.8,
    );
    assert_eq!(result.len(), 9);
    assert_eq!(result[0], 0.0);
    assert_eq!(result[1], -1.0);
    assert_eq!(result[2], 1.0);
    assert_eq!(result[3], 0.0);
    assert_close(result[4], 0.5, 1e-14);
    assert_eq!(result[5], -1.0);
    assert_eq!(result[6], 2.0);
    assert_eq!(result[7], 0.0);
    assert_close(result[8], 0.75, 1e-14);
}

#[test]
fn test_clip_only_point_0_right_side_and_above() {
    let result = Intersections2D::clip_triangle_at_axis_aligned_threshold(
        0.5, true, 0.6, 0.4, 0.2,
    );
    assert_eq!(result.len(), 9);
    assert_eq!(result[0], 0.0);
    assert_eq!(result[1], -1.0);
    assert_eq!(result[2], 1.0);
    assert_eq!(result[3], 0.0);
    assert_close(result[4], 0.5, 1e-14);
    assert_eq!(result[5], -1.0);
    assert_eq!(result[6], 2.0);
    assert_eq!(result[7], 0.0);
    assert_close(result[8], 0.75, 1e-14);
}

#[test]
fn test_clip_only_point_1_right_side_and_below() {
    let result = Intersections2D::clip_triangle_at_axis_aligned_threshold(
        0.5, false, 0.8, 0.4, 0.6,
    );
    assert_eq!(result.len(), 9);
    assert_eq!(result[0], 1.0);
    assert_eq!(result[1], -1.0);
    assert_eq!(result[2], 2.0);
    assert_eq!(result[3], 1.0);
    assert_close(result[4], 0.5, 1e-14);
    assert_eq!(result[5], -1.0);
    assert_eq!(result[6], 0.0);
    assert_eq!(result[7], 1.0);
    assert_close(result[8], 0.75, 1e-14);
}

#[test]
fn test_clip_only_point_1_right_side_and_above() {
    let result = Intersections2D::clip_triangle_at_axis_aligned_threshold(
        0.5, true, 0.2, 0.6, 0.4,
    );
    assert_eq!(result.len(), 9);
    assert_eq!(result[0], 1.0);
    assert_eq!(result[1], -1.0);
    assert_eq!(result[2], 2.0);
    assert_eq!(result[3], 1.0);
    assert_close(result[4], 0.5, 1e-14);
    assert_eq!(result[5], -1.0);
    assert_eq!(result[6], 0.0);
    assert_eq!(result[7], 1.0);
    assert_close(result[8], 0.75, 1e-14);
}

#[test]
fn test_clip_only_point_2_right_side_and_below() {
    let result = Intersections2D::clip_triangle_at_axis_aligned_threshold(
        0.5, false, 0.6, 0.8, 0.4,
    );
    assert_eq!(result.len(), 9);
    assert_eq!(result[0], 2.0);
    assert_eq!(result[1], -1.0);
    assert_eq!(result[2], 0.0);
    assert_eq!(result[3], 2.0);
    assert_close(result[4], 0.5, 1e-14);
    assert_eq!(result[5], -1.0);
    assert_eq!(result[6], 1.0);
    assert_eq!(result[7], 2.0);
    assert_close(result[8], 0.75, 1e-14);
}

#[test]
fn test_clip_only_point_2_right_side_and_above() {
    let result = Intersections2D::clip_triangle_at_axis_aligned_threshold(
        0.5, true, 0.4, 0.2, 0.6,
    );
    assert_eq!(result.len(), 9);
    assert_eq!(result[0], 2.0);
    assert_eq!(result[1], -1.0);
    assert_eq!(result[2], 0.0);
    assert_eq!(result[3], 2.0);
    assert_close(result[4], 0.5, 1e-14);
    assert_eq!(result[5], -1.0);
    assert_eq!(result[6], 1.0);
    assert_eq!(result[7], 2.0);
    assert_close(result[8], 0.75, 1e-14);
}

#[test]
fn test_compute_barycentric_coordinates_vertex() {
    let ll = Intersections2D::compute_barycentric_coordinates(
        0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0,
    );
    assert_close(ll.x, 1.0, 1e-15);
    assert_close(ll.y, 0.0, 1e-15);
    assert_close(ll.z, 0.0, 1e-15);

    let lr = Intersections2D::compute_barycentric_coordinates(
        1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0,
    );
    assert_close(lr.x, 0.0, 1e-15);
    assert_close(lr.y, 1.0, 1e-15);
    assert_close(lr.z, 0.0, 1e-15);

    let ul = Intersections2D::compute_barycentric_coordinates(
        0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0,
    );
    assert_close(ul.x, 0.0, 1e-15);
    assert_close(ul.y, 0.0, 1e-15);
    assert_close(ul.z, 1.0, 1e-15);
}

#[test]
fn test_compute_barycentric_coordinates_barycenter() {
    let result = Intersections2D::compute_barycentric_coordinates(
        0.0, 0.0, 0.0, 1.0, -1.0, -0.5, 1.0, -0.5,
    );
    assert_close(result.x, 1.0 / 3.0, 1e-15);
    assert_close(result.y, 1.0 / 3.0, 1e-15);
    assert_close(result.z, 1.0 / 3.0, 1e-15);
}

#[test]
fn test_compute_barycentric_coordinates_edges() {
    let e12 = Intersections2D::compute_barycentric_coordinates(
        1.5, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 2.0,
    );
    assert_close(e12.x, 0.5, 1e-15);
    assert_close(e12.y, 0.5, 1e-15);
    assert_close(e12.z, 0.0, 1e-15);

    let e23 = Intersections2D::compute_barycentric_coordinates(
        1.5, 1.5, 1.0, 1.0, 2.0, 1.0, 1.0, 2.0,
    );
    assert_close(e23.x, 0.0, 1e-15);
    assert_close(e23.y, 0.5, 1e-15);
    assert_close(e23.z, 0.5, 1e-15);

    let e31 = Intersections2D::compute_barycentric_coordinates(
        1.0, 1.5, 1.0, 1.0, 2.0, 1.0, 1.0, 2.0,
    );
    assert_close(e31.x, 0.5, 1e-15);
    assert_close(e31.y, 0.0, 1e-15);
    assert_close(e31.z, 0.5, 1e-15);
}

#[test]
fn test_compute_barycentric_coordinates_outside() {
    let result1 = Intersections2D::compute_barycentric_coordinates(
        0.5, 0.5, 1.0, 1.0, 2.0, 1.0, 1.0, 2.0,
    );
    assert!(result1.x > 0.0);
    assert!(result1.y < 0.0);
    assert!(result1.z < 0.0);

    let result2 = Intersections2D::compute_barycentric_coordinates(
        2.1, 0.99, 1.0, 1.0, 2.0, 1.0, 1.0, 2.0,
    );
    assert!(result2.x < 0.0);
    assert!(result2.y > 0.0);
    assert!(result2.z < 0.0);

    let result3 = Intersections2D::compute_barycentric_coordinates(
        0.99, 2.1, 1.0, 1.0, 2.0, 1.0, 1.0, 2.0,
    );
    assert!(result3.x < 0.0);
    assert!(result3.y < 0.0);
    assert!(result3.z > 0.0);
}

#[test]
fn test_compute_line_segment_intersection() {
    let intersection0 = Intersections2D::compute_line_segment_line_segment_intersection(
        0.0, 0.0, 0.0, 2.0, -1.0, 1.0, 1.0, 1.0,
    )
    .unwrap();
    assert_close(intersection0.x, 0.0, 1e-15);
    assert_close(intersection0.y, 1.0, 1e-15);

    let intersection1 = Intersections2D::compute_line_segment_line_segment_intersection(
        0.0, 0.0, 10.0, 5.0, 0.0, 5.0, 10.0, 0.0,
    )
    .unwrap();
    assert_close(intersection1.x, 5.0, 1e-15);
    assert_close(intersection1.y, 2.5, 1e-15);

    let intersection2 = Intersections2D::compute_line_segment_line_segment_intersection(
        0.0, -5.0, 4.0, 3.0, -2.0, 1.0, 4.0, -2.0,
    )
    .unwrap();
    assert_close(intersection2.x, 2.0, 1e-15);
    assert_close(intersection2.y, -1.0, 1e-15);
}

#[test]
fn test_compute_line_segment_intersection_on_vertex() {
    let intersection0 = Intersections2D::compute_line_segment_line_segment_intersection(
        0.0, 0.0, 0.0, 2.0, -1.0, 0.0, 1.0, 0.0,
    )
    .unwrap();
    assert_close(intersection0.x, 0.0, 1e-15);
    assert_close(intersection0.y, 0.0, 1e-15);

    let intersection1 = Intersections2D::compute_line_segment_line_segment_intersection(
        0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 2.0, 0.0,
    )
    .unwrap();
    assert_close(intersection1.x, 1.0, 1e-15);
    assert_close(intersection1.y, 1.0, 1e-15);

    let intersection2 = Intersections2D::compute_line_segment_line_segment_intersection(
        0.0, 0.0, 4.0, 3.0, 5.0, 0.0, 4.0, 3.0,
    )
    .unwrap();
    assert_close(intersection2.x, 4.0, 1e-15);
    assert_close(intersection2.y, 3.0, 1e-15);
}

#[test]
fn test_compute_line_segment_intersection_non_intersecting() {
    let intersection0 = Intersections2D::compute_line_segment_line_segment_intersection(
        0.0, 0.0, 0.0, 5.0, 0.1, 4.8, 5.0, 0.0,
    );
    assert!(intersection0.is_none());

    let intersection1 = Intersections2D::compute_line_segment_line_segment_intersection(
        10.0, 0.0, 0.0, -10.0, 0.0, 0.0, -8.0, -8.0,
    );
    assert!(intersection1.is_none());
}

#[test]
fn test_compute_line_segment_intersection_parallel() {
    let intersection0 = Intersections2D::compute_line_segment_line_segment_intersection(
        0.0, 0.0, 0.0, 2.0, 1.0, 1.0, 1.0, 4.0,
    );
    assert!(intersection0.is_none());

    let intersection1 = Intersections2D::compute_line_segment_line_segment_intersection(
        1.0, 1.0, 4.0, 4.0, 0.0, 0.0, 3.0, 3.0,
    );
    assert!(intersection1.is_none());
}

#[test]
fn test_compute_line_segment_intersection_coincident() {
    let intersection0 = Intersections2D::compute_line_segment_line_segment_intersection(
        0.0, 0.0, 0.0, 2.0, 0.0, 1.0, 0.0, 4.0,
    );
    assert!(intersection0.is_none());

    let intersection1 = Intersections2D::compute_line_segment_line_segment_intersection(
        0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 2.0,
    );
    assert!(intersection1.is_none());
}
