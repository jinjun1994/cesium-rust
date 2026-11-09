use cesium_renderer::implicit_subdivision_scheme::ImplicitSubdivisionScheme;
use cesium_renderer::implicit_tile_coordinates::ImplicitTileCoordinates;

fn quadtree_coordinates(level: u32, x: u32, y: u32, subtree_levels: Option<u32>) -> ImplicitTileCoordinates {
    ImplicitTileCoordinates::quadtree(level, x, y, subtree_levels).unwrap()
}

fn octree_coordinates(level: u32, x: u32, y: u32, z: u32, subtree_levels: Option<u32>) -> ImplicitTileCoordinates {
    ImplicitTileCoordinates::octree(level, x, y, z, subtree_levels).unwrap()
}

#[test]
fn constructs_quadtree_and_octree() {
    let q = quadtree_coordinates(4, 3, 2, Some(6));
    assert_eq!(q.subdivision_scheme, ImplicitSubdivisionScheme::Quadtree);
    assert_eq!(q.subtree_levels, 6);
    assert_eq!(q.level, 4);
    assert_eq!(q.x, 3);
    assert_eq!(q.y, 2);
    assert_eq!(q.z, 0);

    let o = octree_coordinates(4, 3, 2, 1, Some(6));
    assert_eq!(o.subdivision_scheme, ImplicitSubdivisionScheme::Octree);
    assert_eq!(o.subtree_levels, 6);
    assert_eq!(o.level, 4);
    assert_eq!(o.x, 3);
    assert_eq!(o.y, 2);
    assert_eq!(o.z, 1);
}

#[test]
fn constructor_throws_with_invalid_inputs() {
    // out of range x
    assert!(ImplicitTileCoordinates::quadtree(0, 4, 0, None).is_err());
    // out of range y
    assert!(ImplicitTileCoordinates::quadtree(0, 0, 4, None).is_err());
    // out of range z for octree
    assert!(ImplicitTileCoordinates::octree(0, 0, 0, 4, None).is_err());
}

#[test]
fn get_descendant_coordinates_validation_and_behavior() {
    let q0 = quadtree_coordinates(0, 0, 0, Some(2));
    let o0 = octree_coordinates(0, 0, 0, 0, Some(2));
    let q_mismatch = quadtree_coordinates(0, 0, 0, Some(3));

    // mismatched subdivisionScheme
    assert!(q0.get_descendant_coordinates(&o0).is_err());
    // mismatched subtreeLevels
    assert!(q0.get_descendant_coordinates(&q_mismatch).is_err());

    // quadtree descendant
    assert!(q0.get_descendant_coordinates(&quadtree_coordinates(0, 0, 0, None)).unwrap().is_equal(&quadtree_coordinates(0, 0, 0, None)));
    assert!(q0.get_descendant_coordinates(&quadtree_coordinates(1, 1, 1, None)).unwrap().is_equal(&quadtree_coordinates(1, 1, 1, None)));
    assert!(quadtree_coordinates(1, 1, 1, None).get_descendant_coordinates(&quadtree_coordinates(2, 3, 3, None)).unwrap().is_equal(&quadtree_coordinates(3, 7, 7, None)));

    // octree descendant
    assert!(o0.get_descendant_coordinates(&octree_coordinates(0, 0, 0, 0, None)).unwrap().is_equal(&octree_coordinates(0, 0, 0, 0, None)));
    assert!(o0.get_descendant_coordinates(&octree_coordinates(1, 1, 1, 1, None)).unwrap().is_equal(&octree_coordinates(1, 1, 1, 1, None)));
    assert!(octree_coordinates(1, 1, 1, 1, None).get_descendant_coordinates(&octree_coordinates(2, 3, 3, 3, None)).unwrap().is_equal(&octree_coordinates(3, 7, 7, 7, None)));
}

#[test]
fn get_ancestor_coordinates_validation_and_behavior() {
    let q0 = quadtree_coordinates(0, 0, 0, None);
    // ancestor cannot be above tileset root
    assert!(q0.get_ancestor_coordinates(1).is_err());

    // quadtree ancestors
    assert!(quadtree_coordinates(0, 0, 0, None).get_ancestor_coordinates(0).unwrap().is_equal(&quadtree_coordinates(0, 0, 0, None)));
    assert!(quadtree_coordinates(1, 0, 0, None).get_ancestor_coordinates(1).unwrap().is_equal(&quadtree_coordinates(0, 0, 0, None)));
    assert!(quadtree_coordinates(1, 1, 1, None).get_ancestor_coordinates(1).unwrap().is_equal(&quadtree_coordinates(0, 0, 0, None)));
    assert!(quadtree_coordinates(2, 3, 3, None).get_ancestor_coordinates(1).unwrap().is_equal(&quadtree_coordinates(1, 1, 1, None)));
    assert!(quadtree_coordinates(2, 3, 3, None).get_ancestor_coordinates(2).unwrap().is_equal(&quadtree_coordinates(0, 0, 0, None)));

    // octree ancestors
    assert!(octree_coordinates(0, 0, 0, 0, None).get_ancestor_coordinates(0).unwrap().is_equal(&octree_coordinates(0, 0, 0, 0, None)));
    assert!(octree_coordinates(1, 0, 0, 0, None).get_ancestor_coordinates(1).unwrap().is_equal(&octree_coordinates(0, 0, 0, 0, None)));
    assert!(octree_coordinates(1, 1, 1, 1, None).get_ancestor_coordinates(1).unwrap().is_equal(&octree_coordinates(0, 0, 0, 0, None)));
    assert!(octree_coordinates(2, 3, 3, 3, None).get_ancestor_coordinates(1).unwrap().is_equal(&octree_coordinates(1, 1, 1, 1, None)));
    assert!(octree_coordinates(2, 3, 3, 3, None).get_ancestor_coordinates(2).unwrap().is_equal(&octree_coordinates(0, 0, 0, 0, None)));
}

#[test]
fn get_offset_coordinates_validation_and_behavior() {
    let q1 = quadtree_coordinates(1, 0, 0, None);
    let q0 = quadtree_coordinates(0, 0, 0, None);
    let q2_3_3 = quadtree_coordinates(2, 3, 3, None);
    let o0 = octree_coordinates(0, 0, 0, 0, None);

    // descendant is above ancestor
    assert!(q1.get_offset_coordinates(&q0).is_err());
    // descendant is not actually a descendant
    assert!(q1.get_offset_coordinates(&q2_3_3).is_err());
    // mismatched scheme
    assert!(q0.get_offset_coordinates(&o0).is_err());

    // quadtree offset
    assert!(q0.get_offset_coordinates(&quadtree_coordinates(0, 0, 0, None)).unwrap().is_equal(&quadtree_coordinates(0, 0, 0, None)));
    assert!(q0.get_offset_coordinates(&quadtree_coordinates(1, 1, 1, None)).unwrap().is_equal(&quadtree_coordinates(1, 1, 1, None)));
    assert!(q0.get_offset_coordinates(&quadtree_coordinates(2, 3, 3, None)).unwrap().is_equal(&quadtree_coordinates(2, 3, 3, None)));
    assert!(quadtree_coordinates(1, 1, 1, None).get_offset_coordinates(&quadtree_coordinates(2, 2, 2, None)).unwrap().is_equal(&quadtree_coordinates(1, 0, 0, None)));

    // octree offset
    assert!(o0.get_offset_coordinates(&octree_coordinates(0, 0, 0, 0, None)).unwrap().is_equal(&octree_coordinates(0, 0, 0, 0, None)));
    assert!(o0.get_offset_coordinates(&octree_coordinates(1, 1, 1, 1, None)).unwrap().is_equal(&octree_coordinates(1, 1, 1, 1, None)));
    assert!(o0.get_offset_coordinates(&octree_coordinates(2, 3, 3, 3, None)).unwrap().is_equal(&octree_coordinates(2, 3, 3, 3, None)));
    assert!(octree_coordinates(1, 1, 1, 1, None).get_offset_coordinates(&octree_coordinates(2, 2, 2, 2, None)).unwrap().is_equal(&octree_coordinates(1, 0, 0, 0, None)));
}

#[test]
fn get_child_coordinates_behavior() {
    let q = quadtree_coordinates(1, 0, 0, None);
    assert!(q.get_child_coordinates(10).is_err());

    assert!(q.get_child_coordinates(0).unwrap().is_equal(&quadtree_coordinates(2, 0, 0, None)));
    assert!(q.get_child_coordinates(1).unwrap().is_equal(&quadtree_coordinates(2, 1, 0, None)));
    assert!(q.get_child_coordinates(2).unwrap().is_equal(&quadtree_coordinates(2, 0, 1, None)));
    assert!(q.get_child_coordinates(3).unwrap().is_equal(&quadtree_coordinates(2, 1, 1, None)));

    let o = octree_coordinates(1, 0, 1, 1, None);
    assert!(o.get_child_coordinates(0).unwrap().is_equal(&octree_coordinates(2, 0, 2, 2, None)));
    assert!(o.get_child_coordinates(1).unwrap().is_equal(&octree_coordinates(2, 1, 2, 2, None)));
    assert!(o.get_child_coordinates(2).unwrap().is_equal(&octree_coordinates(2, 0, 3, 2, None)));
    assert!(o.get_child_coordinates(3).unwrap().is_equal(&octree_coordinates(2, 1, 3, 2, None)));
    assert!(o.get_child_coordinates(4).unwrap().is_equal(&octree_coordinates(2, 0, 2, 3, None)));
    assert!(o.get_child_coordinates(5).unwrap().is_equal(&octree_coordinates(2, 1, 2, 3, None)));
    assert!(o.get_child_coordinates(6).unwrap().is_equal(&octree_coordinates(2, 0, 3, 3, None)));
    assert!(o.get_child_coordinates(7).unwrap().is_equal(&octree_coordinates(2, 1, 3, 3, None)));
}

#[test]
fn get_subtree_and_parent_subtree_coordinates() {
    // getSubtreeCoordinates
    assert!(quadtree_coordinates(0, 0, 0, None).get_subtree_coordinates().unwrap().is_equal(&quadtree_coordinates(0, 0, 0, None)));
    assert!(quadtree_coordinates(1, 1, 1, None).get_subtree_coordinates().unwrap().is_equal(&quadtree_coordinates(0, 0, 0, None)));
    assert!(quadtree_coordinates(2, 3, 3, None).get_subtree_coordinates().unwrap().is_equal(&quadtree_coordinates(2, 3, 3, None)));
    assert!(quadtree_coordinates(3, 7, 7, None).get_subtree_coordinates().unwrap().is_equal(&quadtree_coordinates(2, 3, 3, None)));

    assert!(octree_coordinates(0, 0, 0, 0, None).get_subtree_coordinates().unwrap().is_equal(&octree_coordinates(0, 0, 0, 0, None)));
    assert!(octree_coordinates(1, 1, 1, 1, None).get_subtree_coordinates().unwrap().is_equal(&octree_coordinates(0, 0, 0, 0, None)));
    assert!(octree_coordinates(2, 3, 3, 3, None).get_subtree_coordinates().unwrap().is_equal(&octree_coordinates(2, 3, 3, 3, None)));
    assert!(octree_coordinates(3, 7, 7, 7, None).get_subtree_coordinates().unwrap().is_equal(&octree_coordinates(2, 3, 3, 3, None)));

    // getParentSubtreeCoordinates invalid inputs
    assert!(quadtree_coordinates(0, 0, 0, None).get_parent_subtree_coordinates().is_err());
    assert!(quadtree_coordinates(1, 1, 1, None).get_parent_subtree_coordinates().is_err());

    // quadtree getParentSubtreeCoordinates
    assert!(quadtree_coordinates(2, 0, 0, None).get_parent_subtree_coordinates().unwrap().is_equal(&quadtree_coordinates(0, 0, 0, None)));
    assert!(quadtree_coordinates(2, 3, 3, None).get_parent_subtree_coordinates().unwrap().is_equal(&quadtree_coordinates(0, 0, 0, None)));
    assert!(quadtree_coordinates(3, 7, 7, None).get_parent_subtree_coordinates().unwrap().is_equal(&quadtree_coordinates(0, 0, 0, None)));
    assert!(quadtree_coordinates(4, 0, 0, None).get_parent_subtree_coordinates().unwrap().is_equal(&quadtree_coordinates(2, 0, 0, None)));
    assert!(quadtree_coordinates(4, 15, 15, None).get_parent_subtree_coordinates().unwrap().is_equal(&quadtree_coordinates(2, 3, 3, None)));

    // octree getParentSubtreeCoordinates
    assert!(octree_coordinates(2, 0, 0, 0, None).get_parent_subtree_coordinates().unwrap().is_equal(&octree_coordinates(0, 0, 0, 0, None)));
    assert!(octree_coordinates(2, 3, 3, 3, None).get_parent_subtree_coordinates().unwrap().is_equal(&octree_coordinates(0, 0, 0, 0, None)));
    assert!(octree_coordinates(3, 7, 7, 7, None).get_parent_subtree_coordinates().unwrap().is_equal(&octree_coordinates(0, 0, 0, 0, None)));
    assert!(octree_coordinates(4, 0, 0, 0, None).get_parent_subtree_coordinates().unwrap().is_equal(&octree_coordinates(2, 0, 0, 0, None)));
    assert!(octree_coordinates(4, 15, 15, 15, None).get_parent_subtree_coordinates().unwrap().is_equal(&octree_coordinates(2, 3, 3, 3, None)));
}

#[test]
fn is_ancestor_and_predicates() {
    let q0 = quadtree_coordinates(0, 0, 0, None);
    let q1 = quadtree_coordinates(1, 1, 1, None);
    let q3 = quadtree_coordinates(3, 7, 7, None);
    let q1_0 = quadtree_coordinates(1, 0, 0, None);
    let q2_3_3 = quadtree_coordinates(2, 3, 3, None);

    assert!(!q0.is_ancestor(&q0).unwrap());
    assert!(q0.is_ancestor(&q1).unwrap());
    assert!(!q1.is_ancestor(&q0).unwrap());
    assert!(q0.is_ancestor(&q3).unwrap());
    assert!(!q1_0.is_ancestor(&q2_3_3).unwrap());

    // isImplicitTilesetRoot
    assert!(quadtree_coordinates(0, 0, 0, None).is_implicit_tileset_root());
    assert!(!quadtree_coordinates(1, 0, 0, None).is_implicit_tileset_root());

    // isSubtreeRoot
    assert!(quadtree_coordinates(0, 0, 0, None).is_subtree_root());
    assert!(!quadtree_coordinates(1, 0, 0, None).is_subtree_root());
    assert!(quadtree_coordinates(2, 0, 0, None).is_subtree_root());
    assert!(!quadtree_coordinates(3, 0, 0, None).is_subtree_root());

    // isBottomOfSubtree
    assert!(!quadtree_coordinates(0, 0, 0, None).is_bottom_of_subtree());
    assert!(quadtree_coordinates(1, 0, 0, None).is_bottom_of_subtree());
    assert!(!quadtree_coordinates(2, 0, 0, None).is_bottom_of_subtree());
    assert!(quadtree_coordinates(3, 0, 0, None).is_bottom_of_subtree());
}

#[test]
fn morton_and_tile_index_calculations() {
    // childIndex
    assert_eq!(quadtree_coordinates(4, 3, 2, None).child_index(), 1);
    assert_eq!(octree_coordinates(4, 3, 2, 1, None).child_index(), 5);

    // mortonIndex
    assert_eq!(quadtree_coordinates(4, 5, 11, None).morton_index(), 155);
    assert_eq!(octree_coordinates(6, 7, 15, 32, None).morton_index(), 132315);

    // fromMortonIndex
    let q_from_morton = ImplicitTileCoordinates::from_morton_index(ImplicitSubdivisionScheme::Quadtree, 6, 3, 42);
    assert!(q_from_morton.is_equal(&quadtree_coordinates(3, 0, 7, Some(6))));

    let o_from_morton = ImplicitTileCoordinates::from_morton_index(ImplicitSubdivisionScheme::Octree, 6, 2, 43);
    assert!(o_from_morton.is_equal(&octree_coordinates(2, 3, 1, 2, Some(6))));

    // tileIndex
    assert_eq!(quadtree_coordinates(4, 5, 11, None).tile_index(), 240);
    assert_eq!(octree_coordinates(6, 7, 15, 32, None).tile_index(), 169764);

    // fromTileIndex
    let q_from_tile = ImplicitTileCoordinates::from_tile_index(ImplicitSubdivisionScheme::Quadtree, 6, 63);
    assert!(q_from_tile.is_equal(&quadtree_coordinates(3, 0, 7, Some(6))));

    let o_from_tile = ImplicitTileCoordinates::from_tile_index(ImplicitSubdivisionScheme::Octree, 6, 52);
    assert!(o_from_tile.is_equal(&octree_coordinates(2, 3, 1, 2, Some(6))));
}

#[test]
fn get_template_values_quadtree_and_octree() {
    let q_map = quadtree_coordinates(4, 3, 2, Some(6)).get_template_values();
    assert_eq!(q_map.get("level").unwrap(), "4");
    assert_eq!(q_map.get("x").unwrap(), "3");
    assert_eq!(q_map.get("y").unwrap(), "2");
    assert!(!q_map.contains_key("z"));

    let o_map = octree_coordinates(4, 3, 2, 1, Some(6)).get_template_values();
    assert_eq!(o_map.get("level").unwrap(), "4");
    assert_eq!(o_map.get("x").unwrap(), "3");
    assert_eq!(o_map.get("y").unwrap(), "2");
    assert_eq!(o_map.get("z").unwrap(), "1");
}
