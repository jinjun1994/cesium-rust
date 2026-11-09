use std::sync::Arc;
use cesium_renderer::implicit_subdivision_scheme::ImplicitSubdivisionScheme;
use cesium_renderer::implicit_subtree::ImplicitSubtree;
use cesium_renderer::implicit_subtree_cache::{
    ImplicitSubtreeCache, ImplicitSubtreeCacheOptions,
};
use cesium_renderer::implicit_tile_coordinates::ImplicitTileCoordinates;

fn make_dummy_subtree() -> Arc<ImplicitSubtree> {
    Arc::new(ImplicitSubtree::default())
}

#[test]
fn constructs_with_default_options() {
    let cache = ImplicitSubtreeCache::new(None);
    assert_eq!(cache.maximum_subtree_count(), 0);
    assert_eq!(cache.subtree_request_counter(), 0);
    assert_eq!(cache.queue_len(), 0);
}

#[test]
fn can_add_a_subtree() {
    let mut cache = ImplicitSubtreeCache::new(None);
    let coords = ImplicitTileCoordinates::new(
        ImplicitSubdivisionScheme::Octree,
        2,
        0,
        0,
        0,
        0,
    )
    .unwrap();
    let subtree = make_dummy_subtree();

    cache.add_subtree(subtree, coords).expect("add subtree success");
    assert_eq!(cache.subtree_request_counter(), 1);
    assert_eq!(cache.queue_len(), 1);
}

#[test]
fn add_subtree_throws_if_parent_does_not_exist() {
    let mut cache = ImplicitSubtreeCache::new(None);
    let coords = ImplicitTileCoordinates::new(
        ImplicitSubdivisionScheme::Octree,
        2,
        2,
        0,
        1,
        0,
    )
    .unwrap();
    let subtree = make_dummy_subtree();

    let res = cache.add_subtree(subtree, coords);
    assert!(res.is_err());
    assert!(res.unwrap_err().message.contains("parent node needs to exist"));
}

#[test]
fn add_subtree_trims_cache_as_needed() {
    let mut cache = ImplicitSubtreeCache::new(Some(ImplicitSubtreeCacheOptions {
        maximum_subtree_count: Some(3),
    }));

    let octree_coords = [
        (0, 0, 0, 0),
        (2, 1, 0, 0),
        (2, 0, 1, 0),
        (2, 0, 0, 1),
    ];

    for (level, x, y, z) in octree_coords {
        let coord = ImplicitTileCoordinates::new(
            ImplicitSubdivisionScheme::Octree,
            2,
            level,
            x,
            y,
            z,
        )
        .unwrap();
        let subtree = make_dummy_subtree();
        cache.add_subtree(subtree, coord).expect("success");
    }

    assert_eq!(cache.subtree_request_counter(), 4);
    assert_eq!(cache.queue_len(), 3);
}
