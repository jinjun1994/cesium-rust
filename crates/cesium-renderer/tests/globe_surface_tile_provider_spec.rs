use cesium_math::{Cartesian3, Color, SceneMode};
use cesium_renderer::commands::Pass;
use cesium_renderer::globe_surface_tile::GlobeSurfaceTile;
use cesium_renderer::globe_surface_tile_provider::{
    GlobeSurfaceTileProvider, GlobeSurfaceTileProviderOptions, ImageryLayerRecord,
};
use cesium_renderer::quadtree_tile_load_state::QuadtreeTileLoadState;

#[test]
fn construction_validates_required_options() {
    assert!(GlobeSurfaceTileProvider::new(GlobeSurfaceTileProviderOptions {
        terrain_provider_name: None,
        ..Default::default()
    })
    .is_err());

    assert!(GlobeSurfaceTileProvider::new(GlobeSurfaceTileProviderOptions {
        imagery_layers_present: false,
        ..Default::default()
    })
    .is_err());

    assert!(GlobeSurfaceTileProvider::new(GlobeSurfaceTileProviderOptions {
        surface_shader_set_present: false,
        ..Default::default()
    })
    .is_err());

    assert!(GlobeSurfaceTileProvider::new(GlobeSurfaceTileProviderOptions {
        vector_provider_present: false,
        ..Default::default()
    })
    .is_err());

    assert!(GlobeSurfaceTileProvider::new(GlobeSurfaceTileProviderOptions::default()).is_ok());
}

#[test]
fn layer_updating_adds_removes_reorders_and_avoids_duplicate_skeletons() {
    let mut provider =
        GlobeSurfaceTileProvider::new(GlobeSurfaceTileProviderOptions::default()).unwrap();
    let mut tile = GlobeSurfaceTile::new(0, 0, 0);
    tile.state = QuadtreeTileLoadState::Done;
    tile.mesh_vertex_count = 12;
    provider.add_tile(tile, Cartesian3::ZERO);

    provider.add_imagery_layer(ImageryLayerRecord::new(1));
    provider.add_imagery_layer(ImageryLayerRecord::new(2));
    // Adding layer 1 again does not duplicate skeletons.
    provider.add_imagery_layer(ImageryLayerRecord::new(1));

    assert_eq!(provider.tiles()[0].imagery_layer_ids, vec![1, 2]);

    // Raise layer 1 to top -> order becomes [2, 1] on both provider and tile.
    provider.raise_layer_to_top(1);
    assert_eq!(provider.tiles()[0].imagery_layer_ids, vec![2, 1]);

    // Remove layer 2 -> only layer 1 remains.
    provider.remove_imagery_layer(2);
    assert_eq!(provider.tiles()[0].imagery_layer_ids, vec![1]);

    // Remove layer 1 -> empty.
    provider.remove_imagery_layer(1);
    assert!(provider.tiles()[0].imagery_layer_ids.is_empty());
}

#[test]
fn buckets_tiles_by_texture_count_and_emits_globe_or_translucent_commands() {
    let mut provider =
        GlobeSurfaceTileProvider::new(GlobeSurfaceTileProviderOptions::default()).unwrap();
    provider.add_imagery_layer(ImageryLayerRecord::new(10));
    provider.add_imagery_layer(ImageryLayerRecord::new(20));
    provider.set_layer_show(20, false);

    let mut tile = GlobeSurfaceTile::new(0, 0, 0);
    tile.state = QuadtreeTileLoadState::Done;
    tile.mesh_vertex_count = 36;
    let idx = provider.add_tile(tile, Cartesian3::new(6_378_137.0, 0.0, 0.0));

    provider.begin_update();
    provider.show_tile_this_frame(idx);
    // Only layer 10 is shown -> bucket 1.
    assert_eq!(provider.tiles_to_render_by_texture_count()[1], vec![idx]);
    let cmds = provider.end_update(SceneMode::Scene3D);
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].pass, Some(Pass::Globe));
    assert_eq!(cmds[0].count, Some(36));

    // Translucent base color switches pass to Translucent.
    provider.base_color = Color::new(0.0, 0.0, 0.5, 0.5);
    provider.begin_update();
    provider.show_tile_this_frame(idx);
    let cmds = provider.end_update(SceneMode::Scene3D);
    assert_eq!(cmds[0].pass, Some(Pass::Translucent));

    // Morphing suppresses draw commands.
    provider.begin_update();
    provider.show_tile_this_frame(idx);
    assert!(provider.end_update(SceneMode::Morphing).is_empty());
}

#[test]
fn compute_distance_to_tile_and_destroy() {
    let mut provider =
        GlobeSurfaceTileProvider::new(GlobeSurfaceTileProviderOptions::default()).unwrap();
    let dist = provider.compute_distance_to_tile(
        Cartesian3::new(10.0, 0.0, 0.0),
        Cartesian3::new(0.0, 0.0, 0.0),
    );
    assert!((dist - 10.0).abs() < 1e-12);

    provider.destroy();
    assert!(provider.is_destroyed());
    assert!(provider.tiles().is_empty());
}
