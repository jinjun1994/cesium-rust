use std::sync::{Arc, RwLock};
use cesium_renderer::implicit_3d_tile_content::{
    Implicit3DTileContent, ImplicitPlaceholderTile,
};
use cesium_renderer::implicit_subdivision_scheme::ImplicitSubdivisionScheme;
use cesium_renderer::implicit_tile_coordinates::ImplicitTileCoordinates;
use cesium_renderer::implicit_tileset::ImplicitTileset;

fn create_test_tileset() -> ImplicitTileset {
    let json = serde_json::json!({
        "geometricError": 800.0,
        "refine": "ADD",
        "boundingVolume": {
            "box": [0.0, 0.0, 0.0, 256.0, 0.0, 0.0, 0.0, 256.0, 0.0, 0.0, 0.0, 256.0]
        },
        "content": {
            "uri": "https://example.com/{level}/{x}/{y}.b3dm"
        },
        "implicitTiling": {
            "subdivisionScheme": "QUADTREE",
            "subtreeLevels": 2,
            "availableLevels": 2,
            "subtrees": {
                "uri": "https://example.com/{level}/{x}/{y}.subtree"
            }
        }
    });
    ImplicitTileset::from_tile_json(&json).expect("valid tileset json")
}

fn create_placeholder(
    tileset: &ImplicitTileset,
    coords: ImplicitTileCoordinates,
) -> ImplicitPlaceholderTile {
    ImplicitPlaceholderTile::new(coords, tileset.clone(), None, None, 400.0)
}

fn gather_tiles_preorder(
    tile_arc: &Arc<RwLock<ImplicitPlaceholderTile>>,
    min_level: u32,
    max_level: u32,
    result: &mut Vec<Arc<RwLock<ImplicitPlaceholderTile>>>,
) {
    let level = tile_arc.read().unwrap().implicit_coordinates.level;
    if level >= min_level && level <= max_level {
        result.push(tile_arc.clone());
    }

    let children = tile_arc.write().unwrap().get_children().to_vec();
    for child in &children {
        gather_tiles_preorder(child, min_level, max_level, result);
    }
}

#[test]
fn loads_subtree_using_json() {
    let tileset = create_test_tileset();
    let root_coords = ImplicitTileCoordinates::new(
        ImplicitSubdivisionScheme::Quadtree,
        2,
        0,
        0,
        0,
        0,
    )
    .unwrap();
    let mut placeholder = create_placeholder(&tileset, root_coords);

    let quadtree_json = serde_json::json!({
        "tileAvailability": { "constant": 1 },
        "contentAvailability": { "constant": 1 },
        "childSubtreeAvailability": { "constant": 0 }
    });

    let content = Implicit3DTileContent::from_subtree_json(
        &mut placeholder,
        Some(&quadtree_json),
        None,
    )
    .expect("from_subtree_json success");

    assert!(content.is_ready());
    assert_eq!(content.features_length(), 0);

    let subtree_root = placeholder.get_children()[0].clone();
    let mut tiles = Vec::new();
    gather_tiles_preorder(&subtree_root, 0, 1, &mut tiles);

    // root + 4 children = 5
    let expected_children_counts = [4, 0, 0, 0, 0];
    assert_eq!(tiles.len(), expected_children_counts.len());
    for (i, t) in tiles.iter().enumerate() {
        assert_eq!(
            t.write().unwrap().get_children().len(),
            expected_children_counts[i]
        );
    }
}

#[test]
fn expands_subtree_with_bitstream() {
    let tileset = create_test_tileset();
    let root_coords = ImplicitTileCoordinates::new(
        ImplicitSubdivisionScheme::Quadtree,
        2,
        0,
        0,
        0,
        0,
    )
    .unwrap();
    let mut placeholder = create_placeholder(&tileset, root_coords);

    // Simulate bitstream buffer with descriptor 11010
    let buffer = [0b00001011u8, 0, 0, 0];
    Implicit3DTileContent::from_subtree_json(&mut placeholder, None, Some(&buffer))
        .expect("from_subtree_json with buffer");

    let subtree_root = placeholder.get_children()[0].clone();
    let mut tiles = Vec::new();
    gather_tiles_preorder(&subtree_root, 0, 2, &mut tiles);

    let expected_children_counts = [2, 4, 0, 0, 0, 0, 4, 0, 0, 0, 0];
    assert_eq!(tiles.len(), expected_children_counts.len());
    for (i, t) in tiles.iter().enumerate() {
        assert_eq!(
            t.write().unwrap().get_children().len(),
            expected_children_counts[i]
        );
    }
}

#[test]
fn derives_children_only_when_requested() {
    let tileset = create_test_tileset();
    let root_coords = ImplicitTileCoordinates::new(
        ImplicitSubdivisionScheme::Quadtree,
        2,
        0,
        0,
        0,
        0,
    )
    .unwrap();
    let mut placeholder = create_placeholder(&tileset, root_coords);

    let buffer = [0b00001011u8, 0, 0, 0];
    Implicit3DTileContent::from_subtree_json(&mut placeholder, None, Some(&buffer)).unwrap();

    let subtree_root_arc = placeholder.get_children()[0].clone();
    {
        let subtree_root = subtree_root_arc.read().unwrap();
        assert_eq!(subtree_root.cached_children_count(), 0);
    }

    let children = subtree_root_arc.write().unwrap().get_children().to_vec();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].read().unwrap().cached_children_count(), 0);
    assert_eq!(children[1].read().unwrap().cached_children_count(), 0);
}

#[test]
fn derives_child_subtree_placeholders_only_when_requested() {
    let tileset = create_test_tileset();
    let root_coords = ImplicitTileCoordinates::new(
        ImplicitSubdivisionScheme::Quadtree,
        2,
        0,
        0,
        0,
        0,
    )
    .unwrap();
    let mut placeholder = create_placeholder(&tileset, root_coords);

    let buffer = [0b00001011u8, 0, 0, 0];
    Implicit3DTileContent::from_subtree_json(&mut placeholder, None, Some(&buffer)).unwrap();

    let subtree_root = placeholder.get_children()[0].clone();
    let bottom_tile = subtree_root.write().unwrap().get_children()[0].clone();
    assert_eq!(bottom_tile.read().unwrap().cached_children_count(), 0);

    let placeholder_tiles = bottom_tile.write().unwrap().get_children().to_vec();
    assert_eq!(placeholder_tiles.len(), 4);

    for p in &placeholder_tiles {
        let p_lock = p.read().unwrap();
        assert!(p_lock.content_uri.as_ref().unwrap().contains(".subtree"));
        assert_eq!(p_lock.cached_children_count(), 0);
    }
}

#[test]
fn does_not_derive_children_of_a_destroyed_tile() {
    let tileset = create_test_tileset();
    let root_coords = ImplicitTileCoordinates::new(
        ImplicitSubdivisionScheme::Quadtree,
        2,
        0,
        0,
        0,
        0,
    )
    .unwrap();
    let mut placeholder = create_placeholder(&tileset, root_coords);

    let buffer = [0b00001011u8, 0, 0, 0];
    Implicit3DTileContent::from_subtree_json(&mut placeholder, None, Some(&buffer)).unwrap();

    let subtree_root = placeholder.get_children()[0].clone();
    subtree_root.write().unwrap().destroy();
    assert!(subtree_root.write().unwrap().get_children().is_empty());
}

#[test]
fn sets_tile_coordinates_on_each_tile() {
    let tileset = create_test_tileset();
    let root_coords = ImplicitTileCoordinates::new(
        ImplicitSubdivisionScheme::Quadtree,
        2,
        0,
        0,
        0,
        0,
    )
    .unwrap();
    let mut placeholder = create_placeholder(&tileset, root_coords);

    let buffer = [0b00001011u8, 0, 0, 0];
    Implicit3DTileContent::from_subtree_json(&mut placeholder, None, Some(&buffer)).unwrap();

    let expected_coordinates = [
        [0, 0, 0],
        [1, 0, 0],
        [2, 0, 0],
        [2, 1, 0],
        [2, 0, 1],
        [2, 1, 1],
        [1, 0, 1],
        [2, 0, 2],
        [2, 1, 2],
        [2, 0, 3],
        [2, 1, 3],
    ];

    let subtree_root = placeholder.get_children()[0].clone();
    let mut tiles = Vec::new();
    gather_tiles_preorder(&subtree_root, 0, 2, &mut tiles);

    assert_eq!(tiles.len(), expected_coordinates.len());
    for (i, t) in tiles.iter().enumerate() {
        let expected = expected_coordinates[i];
        let coords = t.read().unwrap().implicit_coordinates;
        assert_eq!(coords.level, expected[0]);
        assert_eq!(coords.x, expected[1]);
        assert_eq!(coords.y, expected[2]);
    }
}
