use serde_json::json;
use cesium_renderer::implicit_subdivision_scheme::ImplicitSubdivisionScheme;
use cesium_renderer::implicit_subtree::ImplicitSubtree;
use cesium_renderer::implicit_tile_coordinates::ImplicitTileCoordinates;
use cesium_renderer::implicit_tileset::ImplicitTileset;

fn create_quadtree_tileset() -> ImplicitTileset {
    let json = json!({
        "geometricError": 500.0,
        "refine": "ADD",
        "boundingVolume": {
            "region": [0.0, 0.0, 0.13, 0.13, 0.0, 1000.0]
        },
        "content": {
            "uri": "https://example.com/{level}/{x}/{y}.b3dm"
        },
        "implicitTiling": {
            "subdivisionScheme": "QUADTREE",
            "subtreeLevels": 2,
            "availableLevels": 4,
            "subtrees": {
                "uri": "https://example.com/{level}/{x}/{y}.subtree"
            }
        }
    });
    ImplicitTileset::from_tile_json(&json).unwrap()
}

#[test]
fn from_subtree_json_throws_without_required_arguments() {
    let tileset = create_quadtree_tileset();
    let coords = ImplicitTileCoordinates::new(
        ImplicitSubdivisionScheme::Quadtree,
        2,
        0,
        0,
        0,
        0,
    )
    .unwrap();
    let dummy_json = json!({});

    assert!(ImplicitSubtree::from_subtree_json(
        None,
        Some(&dummy_json),
        None,
        Some(&tileset),
        Some(&coords),
    )
    .is_err());

    assert!(ImplicitSubtree::from_subtree_json(
        Some("https://example.com/test.subtree"),
        Some(&dummy_json),
        None,
        None,
        Some(&coords),
    )
    .is_err());

    assert!(ImplicitSubtree::from_subtree_json(
        Some("https://example.com/test.subtree"),
        Some(&dummy_json),
        None,
        Some(&tileset),
        None,
    )
    .is_err());

    assert!(ImplicitSubtree::from_subtree_json(
        Some("https://example.com/test.subtree"),
        None,
        None,
        Some(&tileset),
        Some(&coords),
    )
    .is_err());
}

#[test]
fn sets_the_implicit_coordinates_of_the_subtree_root() {
    let tileset = create_quadtree_tileset();
    let coords = ImplicitTileCoordinates::new(
        ImplicitSubdivisionScheme::Quadtree,
        2,
        0,
        0,
        0,
        0,
    )
    .unwrap();
    let buffer = [0b00001011u8];

    let subtree = ImplicitSubtree::from_subtree_json(
        Some("https://example.com/test.subtree"),
        None,
        Some(&buffer),
        Some(&tileset),
        Some(&coords),
    )
    .unwrap();

    assert!(subtree.implicit_coordinates.is_equal(&coords));
}

#[test]
fn gets_availability_from_buffer_and_coordinates() {
    let tileset = create_quadtree_tileset();
    let coords = ImplicitTileCoordinates::new(
        ImplicitSubdivisionScheme::Quadtree,
        2,
        0,
        0,
        0,
        0,
    )
    .unwrap();
    let buffer = [0b00001011u8];

    let subtree = ImplicitSubtree::from_subtree_json(
        Some("https://example.com/test.subtree"),
        None,
        Some(&buffer),
        Some(&tileset),
        Some(&coords),
    )
    .unwrap();

    // tileAvailability "11010" -> index 0=true, 1=true, 2=false, 3=true, 4=false
    let expected_tile_avail = [true, true, false, true, false];
    for (i, &exp) in expected_tile_avail.iter().enumerate() {
        assert_eq!(subtree.tile_is_available_at_index(i), exp);

        let coord = ImplicitTileCoordinates::from_tile_index(
            ImplicitSubdivisionScheme::Quadtree,
            2,
            i,
        );
        assert_eq!(subtree.tile_is_available_at_coordinates(&coord), exp);
    }

    // contentAvailability "11000" -> index 0=true, 1=true, 2=false, 3=false, 4=false
    let expected_content_avail = [true, true, false, false, false];
    for (i, &exp) in expected_content_avail.iter().enumerate() {
        assert_eq!(subtree.content_is_available_at_index(i, 0), exp);

        let coord = ImplicitTileCoordinates::from_tile_index(
            ImplicitSubdivisionScheme::Quadtree,
            2,
            i,
        );
        assert_eq!(subtree.content_is_available_at_coordinates(&coord, 0), exp);
    }

    // childSubtreeAvailability "1111000010100000"
    let child_descriptor = "1111000010100000";
    for (i, ch) in child_descriptor.chars().enumerate() {
        let exp = ch == '1';
        assert_eq!(subtree.child_subtree_is_available_at_index(i), exp);

        let coord = ImplicitTileCoordinates::from_morton_index(
            ImplicitSubdivisionScheme::Quadtree,
            2,
            2,
            i,
        );
        assert_eq!(subtree.child_subtree_is_available_at_coordinates(&coord), exp);
    }
}

#[test]
fn gets_availability_from_constant_json() {
    let tileset = create_quadtree_tileset();
    let coords = ImplicitTileCoordinates::new(
        ImplicitSubdivisionScheme::Quadtree,
        2,
        0,
        0,
        0,
        0,
    )
    .unwrap();

    let json = json!({
        "tileAvailability": { "constant": 1 },
        "contentAvailability": { "constant": 1 },
        "childSubtreeAvailability": { "constant": 0 }
    });

    let subtree = ImplicitSubtree::from_subtree_json(
        Some("https://example.com/test.subtree"),
        Some(&json),
        None,
        Some(&tileset),
        Some(&coords),
    )
    .unwrap();

    for i in 0..5 {
        assert!(subtree.tile_is_available_at_index(i));
        assert!(subtree.content_is_available_at_index(i, 0));
    }
    for i in 0..16 {
        assert!(!subtree.child_subtree_is_available_at_index(i));
    }
}
