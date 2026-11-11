use std::collections::HashMap;

use cesium_math::{
    Cartesian3, GeoRectangle, Matrix4, OrientedBoundingBox, Sphere, TerrainEncoding, TerrainMesh,
    TerrainPicker,
};
use cesium_renderer::terrain_fill_mesh::{TerrainFillMesh, TileCoord};

fn create_mock_mesh(height: f64) -> TerrainMesh {
    TerrainMesh {
        center: Cartesian3::ZERO,
        vertices: vec![0.0; 12],
        stride: 3,
        indices: vec![0, 1, 2],
        index_count_without_skirts: 3,
        vertex_count_without_skirts: 3,
        minimum_height: height,
        maximum_height: height + 10.0,
        rectangle: GeoRectangle::from_degrees(0.0, 0.0, 1.0, 1.0),
        bounding_sphere3d: Sphere::new(Cartesian3::ZERO, 10.0),
        occludee_point_in_scaled_space: Cartesian3::ZERO,
        oriented_bounding_box: Some(OrientedBoundingBox::default()),
        encoding: TerrainEncoding::default(),
        west_indices_south_to_north: vec![0],
        south_indices_east_to_west: vec![1],
        east_indices_north_to_south: vec![2],
        north_indices_west_to_east: vec![0],
        transform: Matrix4::IDENTITY,
        last_pick_scene_mode: None,
        terrain_picker: TerrainPicker::new(vec![0.0; 12], vec![0, 1, 2], TerrainEncoding::default()),
    }
}

#[test]
fn test_update_fill_tiles_does_nothing_if_no_rendered_tiles_are_provided() {
    let mut fill_meshes = HashMap::new();
    let loaded_meshes = HashMap::new();

    TerrainFillMesh::update_fill_tiles(&[], &loaded_meshes, &mut fill_meshes);
    assert!(fill_meshes.is_empty());
}

#[test]
fn test_propagates_edges_and_corners_to_unloaded_tile() {
    let center = TileCoord::new(2, 2, 2);
    let west = TileCoord::new(2, 1, 2);
    let south = TileCoord::new(2, 2, 1);
    let east = TileCoord::new(2, 3, 2);
    let north = TileCoord::new(2, 2, 3);
    let southwest = TileCoord::new(2, 1, 1);
    let southeast = TileCoord::new(2, 3, 1);
    let northwest = TileCoord::new(2, 1, 3);
    let northeast = TileCoord::new(2, 3, 3);

    let mut loaded_meshes = HashMap::new();
    loaded_meshes.insert(west, create_mock_mesh(15.0));
    loaded_meshes.insert(south, create_mock_mesh(31.0));
    loaded_meshes.insert(east, create_mock_mesh(19.0));
    loaded_meshes.insert(north, create_mock_mesh(3.0));
    loaded_meshes.insert(southwest, create_mock_mesh(29.0));
    loaded_meshes.insert(southeast, create_mock_mesh(33.0));
    loaded_meshes.insert(northwest, create_mock_mesh(1.0));
    loaded_meshes.insert(northeast, create_mock_mesh(5.0));

    let rendered_tiles = vec![
        center, west, south, east, north, southwest, southeast, northwest, northeast,
    ];

    let mut fill_meshes = HashMap::new();
    TerrainFillMesh::update_fill_tiles(&rendered_tiles, &loaded_meshes, &mut fill_meshes);

    let fill = fill_meshes.get(&center).expect("center fill mesh should exist");
    assert_eq!(fill.west_tiles, vec![west]);
    assert_eq!(fill.west_meshes.len(), 1);
    assert_eq!(fill.west_meshes[0].minimum_height, 15.0);

    assert_eq!(fill.south_tiles, vec![south]);
    assert_eq!(fill.south_meshes.len(), 1);
    assert_eq!(fill.south_meshes[0].minimum_height, 31.0);

    assert_eq!(fill.east_tiles, vec![east]);
    assert_eq!(fill.east_meshes.len(), 1);
    assert_eq!(fill.east_meshes[0].minimum_height, 19.0);

    assert_eq!(fill.north_tiles, vec![north]);
    assert_eq!(fill.north_meshes.len(), 1);
    assert_eq!(fill.north_meshes[0].minimum_height, 3.0);

    assert_eq!(fill.southwest_tile, Some(southwest));
    assert_eq!(fill.southwest_mesh.as_ref().unwrap().minimum_height, 29.0);

    assert_eq!(fill.southeast_tile, Some(southeast));
    assert_eq!(fill.southeast_mesh.as_ref().unwrap().minimum_height, 33.0);

    assert_eq!(fill.northwest_tile, Some(northwest));
    assert_eq!(fill.northwest_mesh.as_ref().unwrap().minimum_height, 1.0);

    assert_eq!(fill.northeast_tile, Some(northeast));
    assert_eq!(fill.northeast_mesh.as_ref().unwrap().minimum_height, 5.0);
}

#[test]
fn test_terrain_fill_mesh_update_synthesizes_mesh() {
    let mut fill = TerrainFillMesh::new(TileCoord::new(1, 0, 0));
    let mock_mesh = create_mock_mesh(42.0);
    fill.west_meshes = vec![mock_mesh.clone()];
    fill.changed_this_frame = true;

    assert!(fill.mesh.is_none());
    fill.update();
    assert!(fill.mesh.is_some());
    assert_eq!(fill.mesh.unwrap().minimum_height, 42.0);
    assert!(!fill.changed_this_frame);
}

#[test]
fn test_terrain_fill_mesh_destroy() {
    let mut fill = TerrainFillMesh::new(TileCoord::new(1, 0, 0));
    fill.west_meshes = vec![create_mock_mesh(10.0)];
    fill.update();
    assert!(fill.mesh.is_some());

    fill.destroy();
    assert!(fill.is_destroyed);
    assert!(fill.mesh.is_none());
    assert!(fill.west_meshes.is_empty());
}
