use cesium_math::{Cartesian3, Matrix4};
use cesium_renderer::spatial_node::SpatialNode;
use cesium_renderer::voxel_box_shape::VoxelBoxShape;

fn get_basic_box_shape() -> VoxelBoxShape {
    let mut shape = VoxelBoxShape::new();
    let model_matrix = Matrix4::IDENTITY;
    let min_bounds = VoxelBoxShape::DEFAULT_MIN_BOUNDS;
    let max_bounds = VoxelBoxShape::DEFAULT_MAX_BOUNDS;
    shape.update(Some(model_matrix), Some(min_bounds), Some(max_bounds), None, None).unwrap();
    shape
}

#[test]
fn test_spatial_node_constructs() {
    let shape = get_basic_box_shape();
    let dimensions = Cartesian3::new(2.0, 3.0, 4.0);
    let node = SpatialNode::new(2, 1, 2, 3, &shape, dimensions);

    assert_eq!(node.level, 2);
    assert_eq!(node.x, 1);
    assert_eq!(node.y, 2);
    assert_eq!(node.z, 3);
    assert_eq!(node.screen_space_error, 0.0);
    assert!(node.children.is_none());
}

#[test]
fn test_spatial_node_returns_coordinates_of_child() {
    let shape = get_basic_box_shape();
    let dimensions = Cartesian3::new(2.0, 3.0, 4.0);
    let mut node = SpatialNode::new(2, 1, 2, 3, &shape, dimensions);

    assert!(node.children.is_none());
    node.construct_child_nodes(&shape, dimensions);

    let children = node.children.as_ref().expect("children should be present");
    assert_eq!(children.len(), 8);

    let expected_child_coordinates = [
        [3, 2, 4, 6],
        [3, 3, 4, 6],
        [3, 2, 5, 6],
        [3, 3, 5, 6],
        [3, 2, 4, 7],
        [3, 3, 4, 7],
        [3, 2, 5, 7],
        [3, 3, 5, 7],
    ];

    let actual_child_coordinates: Vec<[u32; 4]> = children
        .iter()
        .map(|child| [child.level, child.x, child.y, child.z])
        .collect();

    assert_eq!(actual_child_coordinates, expected_child_coordinates);
}

#[test]
fn test_spatial_node_computes_screen_space_error() {
    let shape = get_basic_box_shape();
    let dimensions = Cartesian3::new(2.0, 3.0, 4.0);
    let mut node = SpatialNode::new(0, 0, 0, 0, &shape, dimensions);

    let camera_position = Cartesian3::new(0.0, 0.0, 2.0);
    node.compute_screen_space_error(camera_position, 1.0);

    // Box extent is [-1, 1] in z, so distance from (0, 0, 2) to box surface at z=1 is 1.0.
    // Half axes have scale 1.0, maximum scale = 2.0. Minimum dimension component = 2.0.
    // Approximate voxel size = 2.0 / 2.0 = 1.0.
    // screenSpaceError = 1.0 * (1.0 / 1.0) = 1.0.
    assert!((node.screen_space_error - 1.0).abs() < 1e-10);
}

#[test]
fn test_spatial_node_adds_a_keyframe_node() {
    let shape = get_basic_box_shape();
    let dimensions = Cartesian3::new(2.0, 3.0, 4.0);
    let mut node = SpatialNode::new(0, 0, 0, 0, &shape, dimensions);

    let starting_keyframes = node.keyframe_nodes.len();
    node.create_keyframe_node(0);
    assert_eq!(node.keyframe_nodes.len() - starting_keyframes, 1);
    assert_eq!(node.keyframe_nodes[0].keyframe, 0);

    // Adding existing keyframe is idempotent
    node.create_keyframe_node(0);
    assert_eq!(node.keyframe_nodes.len(), 1);

    // Adding another keyframe maintains sorted order
    node.create_keyframe_node(-1);
    node.create_keyframe_node(5);
    assert_eq!(node.keyframe_nodes.len(), 3);
    assert_eq!(node.keyframe_nodes[0].keyframe, -1);
    assert_eq!(node.keyframe_nodes[1].keyframe, 0);
    assert_eq!(node.keyframe_nodes[2].keyframe, 5);
}

#[test]
fn test_spatial_node_visited_check() {
    let shape = get_basic_box_shape();
    let dimensions = Cartesian3::new(2.0, 2.0, 2.0);
    let mut node = SpatialNode::new(1, 0, 0, 0, &shape, dimensions);

    assert!(!node.is_visited(42));
    node.visited_frame_number = 42;
    assert!(node.is_visited(42));
    assert!(!node.is_visited(43));
}
