use cesium_renderer::cesium_3d_tile::Cesium3DTileRefine;
use cesium_renderer::cesium_3d_tileset_traversal::{
    Cesium3DTilesetBaseTraversal, Cesium3DTilesetMostDetailedTraversal, Cesium3DTilesetSkipTraversal,
    Cesium3DTilesetTraversal, Cesium3DTilesetTraversalHelper, TraversalFrameState,
    TraversalTileNode, TraversalTilesetContext,
};

fn build_test_hierarchy() -> TraversalTilesetContext {
    let mut ctx = TraversalTilesetContext::new(16.0);
    // Root: geometricError = 64.0, distance = 1000.0 -> sse = 64.0 > 16.0
    let mut root = TraversalTileNode::new(0, 64.0, Cesium3DTileRefine::Replace);
    root.distance_to_camera = 1000.0;

    // Child 0: geometricError = 16.0, distance = 1000.0 -> sse = 16.0 <= 16.0 (meets SSE)
    let mut c0 = TraversalTileNode::new(1, 16.0, Cesium3DTileRefine::Replace);
    c0.distance_to_camera = 1000.0;

    // Child 1: geometricError = 32.0, distance = 1000.0 -> sse = 32.0 > 16.0
    let mut c1 = TraversalTileNode::new(2, 32.0, Cesium3DTileRefine::Replace);
    c1.distance_to_camera = 1000.0;

    // Grandchild 1_0 (leaf): geometricError = 8.0, distance = 1000.0 -> sse = 8.0 <= 16.0
    let mut gc = TraversalTileNode::new(3, 8.0, Cesium3DTileRefine::Replace);
    gc.distance_to_camera = 1000.0;

    c1.children.push(3);
    root.children.push(1);
    root.children.push(2);

    ctx.add_node(root);
    ctx.add_node(c0);
    ctx.add_node(c1);
    ctx.add_node(gc);

    ctx
}

#[test]
fn base_traversal_conforms_to_interface_and_selects_by_sse() {
    let traversal = Cesium3DTilesetBaseTraversal;
    assert_eq!(traversal.traversal_name(), "Cesium3DTilesetBaseTraversal");

    let mut ctx = build_test_hierarchy();
    let frame_state = TraversalFrameState::default();

    traversal.select_tiles(&mut ctx, &frame_state);
    // Node 1 meets SSE, Node 2 refines to Node 3.
    // Selected tiles should be [1, 3]
    assert_eq!(ctx.selected_tiles.len(), 2);
    assert!(ctx.selected_tiles.contains(&1));
    assert!(ctx.selected_tiles.contains(&3));
}

#[test]
fn most_detailed_traversal_conforms_to_interface_and_selects_all_leaves() {
    let traversal = Cesium3DTilesetMostDetailedTraversal;
    assert_eq!(traversal.traversal_name(), "Cesium3DTilesetMostDetailedTraversal");

    let mut ctx = build_test_hierarchy();
    let frame_state = TraversalFrameState::default();

    traversal.select_tiles(&mut ctx, &frame_state);
    // Leaf nodes are 1 and 3.
    assert_eq!(ctx.selected_tiles.len(), 2);
    assert!(ctx.selected_tiles.contains(&1));
    assert!(ctx.selected_tiles.contains(&3));
}

#[test]
fn skip_traversal_conforms_to_interface() {
    let traversal = Cesium3DTilesetSkipTraversal;
    assert_eq!(traversal.traversal_name(), "Cesium3DTilesetSkipTraversal");

    let mut ctx = build_test_hierarchy();
    let frame_state = TraversalFrameState::default();

    traversal.select_tiles(&mut ctx, &frame_state);
    assert!(!ctx.selected_tiles.is_empty());
}

#[test]
fn traversal_helpers_sort_and_test_traversability() {
    let mut a = TraversalTileNode::new(0, 10.0, Cesium3DTileRefine::Replace);
    a.distance_to_camera = 100.0;
    a.children.push(1);

    let mut b = TraversalTileNode::new(1, 10.0, Cesium3DTileRefine::Replace);
    b.distance_to_camera = 200.0;

    assert!(Cesium3DTilesetTraversalHelper::can_traverse(&a));
    assert!(!Cesium3DTilesetTraversalHelper::can_traverse(&b));

    // Sort order: farthest child first (b > a)
    let ord = Cesium3DTilesetTraversalHelper::sort_children_by_distance_to_camera(&a, &b);
    assert_eq!(ord, std::cmp::Ordering::Greater);
}
