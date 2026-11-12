use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{Color, Ellipsoid, SceneMode};
use cesium_renderer::commands::Pass;
use cesium_renderer::globe::Globe;
use cesium_renderer::globe_surface_tile::GlobeSurfaceTile;
use cesium_renderer::globe_surface_tile_provider::ImageryLayerRecord;
use cesium_renderer::quadtree_tile_load_state::QuadtreeTileLoadState;

#[test]
fn imagery_layers_updated_event_fires_on_add_hide_show_raise_remove() {
    let mut globe = Globe::new(Ellipsoid::default());
    let event_count = Rc::new(RefCell::new(0_usize));
    let counter_clone = event_count.clone();

    globe.imagery_layers_updated_event.add_listener(move |()| {
        *counter_clone.borrow_mut() += 1;
    });

    globe.add_imagery_layer(ImageryLayerRecord::new(1));
    globe.add_imagery_layer(ImageryLayerRecord::new(2));
    assert_eq!(*event_count.borrow(), 2);

    globe.set_layer_show(1, false);
    assert_eq!(*event_count.borrow(), 3);

    globe.set_layer_show(1, true);
    assert_eq!(*event_count.borrow(), 4);

    globe.raise_imagery_layer(1);
    assert_eq!(*event_count.borrow(), 5);

    globe.remove_imagery_layer(1);
    assert_eq!(*event_count.borrow(), 6);
}

#[test]
fn terrain_provider_changed_fires_and_updates_state() {
    let mut globe = Globe::new(Ellipsoid::default());
    let changed_provider = Rc::new(RefCell::new(String::new()));
    let provider_clone = changed_provider.clone();

    globe.terrain_provider_changed.add_listener(move |name| {
        *provider_clone.borrow_mut() = name.clone();
    });

    globe.set_terrain_provider("CesiumTerrainProvider");
    assert_eq!(*changed_provider.borrow(), "CesiumTerrainProvider");
    assert_eq!(globe.terrain_provider, "CesiumTerrainProvider");
    assert_eq!(globe.surface().terrain_provider_name, "CesiumTerrainProvider");
}

#[test]
fn tiles_loaded_tracks_load_queues() {
    let mut globe = Globe::new(Ellipsoid::default());
    assert!(globe.tiles_loaded());

    globe.tile_load_queue_high = 2;
    assert!(!globe.tiles_loaded());
    globe.tile_load_queue_high = 0;
    assert!(globe.tiles_loaded());

    globe.tile_load_queue_medium = 2;
    assert!(!globe.tiles_loaded());
    globe.tile_load_queue_medium = 0;
    assert!(globe.tiles_loaded());

    globe.tile_load_queue_low = 2;
    assert!(!globe.tiles_loaded());
    globe.tile_load_queue_low = 0;
    assert!(globe.tiles_loaded());

    // Setting new terrain provider marks queue non-empty.
    globe.set_terrain_provider("NewTerrainProvider");
    assert!(!globe.tiles_loaded());
    globe.tile_load_queue_medium = 0;
    assert!(globe.tiles_loaded());
}

#[test]
fn lighting_and_atmosphere_flags_configure_rendering() {
    let mut globe = Globe::new(Ellipsoid::default());
    globe.enable_lighting = true;
    globe.dynamic_atmosphere_lighting = true;
    globe.dynamic_atmosphere_lighting_from_sun = true;
    globe.show_water_effect = false;
    globe.lambert_diffuse_multiplier = 2.5;
    globe.vertex_shadow_darkness = 0.5;

    let mut tile = GlobeSurfaceTile::new(0, 0, 0);
    tile.state = QuadtreeTileLoadState::Done;
    tile.mesh_vertex_count = 24;
    globe.add_tile(tile);

    let cmds = globe.update(SceneMode::Scene3D);
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].pass, Some(Pass::Globe));
    assert_eq!(cmds[0].count, Some(24));
    assert!(globe.surface().enable_lighting);
    assert!(!globe.surface().show_water_effect);

    // show = false suppresses commands
    globe.show = false;
    assert!(globe.update(SceneMode::Scene3D).is_empty());
    globe.show = true;
    assert_eq!(globe.update(SceneMode::Scene3D).len(), 1);

    // Translucent globe passes
    globe.base_color = Color::new(0.0, 0.0, 0.5, 0.5);
    let cmds = globe.update(SceneMode::Scene3D);
    assert_eq!(cmds[0].pass, Some(Pass::Translucent));

    globe.destroy();
    assert!(globe.is_destroyed());
    assert!(globe.update(SceneMode::Scene3D).is_empty());
}
