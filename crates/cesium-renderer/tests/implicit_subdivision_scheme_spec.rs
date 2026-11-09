use cesium_renderer::implicit_subdivision_scheme::ImplicitSubdivisionScheme;

#[test]
fn get_branching_factor_returns_the_right_branching_factor() {
    let tree_types = ["OCTREE", "QUADTREE"];
    let branching_factors = [8, 4];

    for i in 0..tree_types.len() {
        assert_eq!(
            ImplicitSubdivisionScheme::get_branching_factor(tree_types[i]).unwrap(),
            branching_factors[i]
        );
    }

    assert_eq!(ImplicitSubdivisionScheme::Octree.branching_factor(), 8);
    assert_eq!(ImplicitSubdivisionScheme::Quadtree.branching_factor(), 4);
    assert_eq!(ImplicitSubdivisionScheme::Octree.as_str(), "OCTREE");
    assert_eq!(ImplicitSubdivisionScheme::Quadtree.as_str(), "QUADTREE");
}

#[test]
fn throws_for_invalid_value() {
    let res = ImplicitSubdivisionScheme::get_branching_factor("BINARY_TREE");
    assert!(res.is_err());
    assert!(res.unwrap_err().message.contains("subdivisionScheme is not a valid value"));
}
