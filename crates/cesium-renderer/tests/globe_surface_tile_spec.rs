use std::collections::HashMap;

use cesium_math::Cartesian4;
use cesium_renderer::globe_surface_tile::{
    GlobeSurfaceTile, MeshCreationOutcome, SurfaceImageryAttachment, SurfaceTerrainData,
    TerrainProviderActions, TerrainRequestOutcome, TerrainState, WaterMaskData,
};
use cesium_renderer::imagery_pipeline_stage::ImageryState;
use cesium_renderer::quadtree_tile_load_state::QuadtreeTileLoadState;

#[derive(Default)]
struct TestTerrainProvider {
    available: HashMap<(u32, u32, u32), bool>,
    geometry: HashMap<(u32, u32, u32), TerrainRequestOutcome>,
    mesh: HashMap<(u32, u32, u32), MeshCreationOutcome>,
    allow_upsample: bool,
}

impl TerrainProviderActions for TestTerrainProvider {
    fn get_tile_data_available(&self, x: u32, y: u32, level: u32) -> Option<bool> {
        self.available.get(&(x, y, level)).copied()
    }

    fn request_tile_geometry(&mut self, x: u32, y: u32, level: u32) -> TerrainRequestOutcome {
        self.geometry
            .get(&(x, y, level))
            .cloned()
            .unwrap_or(TerrainRequestOutcome::Loaded(SurfaceTerrainData {
                created_by_upsampling: false,
                water_mask: None,
                vertex_count: 16,
                minimum_height: 0.0,
                maximum_height: 100.0,
            }))
    }

    fn create_mesh(
        &mut self,
        x: u32,
        y: u32,
        level: u32,
        data: &SurfaceTerrainData,
    ) -> MeshCreationOutcome {
        self.mesh
            .get(&(x, y, level))
            .cloned()
            .unwrap_or(MeshCreationOutcome::Ready {
                vertex_count: data.vertex_count,
            })
    }

    fn upsample(
        &mut self,
        parent_data: &SurfaceTerrainData,
        _x: u32,
        _y: u32,
        _level: u32,
    ) -> Option<SurfaceTerrainData> {
        if !self.allow_upsample {
            return None;
        }
        Some(SurfaceTerrainData {
            created_by_upsampling: true,
            water_mask: None,
            vertex_count: parent_data.vertex_count,
            minimum_height: parent_data.minimum_height,
            maximum_height: parent_data.maximum_height,
        })
    }
}

#[test]
fn starts_in_start_and_fails_when_unavailable_or_request_fails() {
    let mut tile = GlobeSurfaceTile::new(0, 0, 0);
    assert_eq!(tile.state, QuadtreeTileLoadState::Start);
    assert_eq!(tile.terrain_state, TerrainState::Unloaded);

    let mut provider = TestTerrainProvider::default();
    provider.available.insert((0, 0, 0), false);
    tile.process_state_machine(None, &mut provider, 99);
    assert_eq!(tile.state, QuadtreeTileLoadState::Failed);
    assert_eq!(tile.terrain_state, TerrainState::Failed);

    let mut tile2 = GlobeSurfaceTile::new(1, 0, 0);
    let mut provider2 = TestTerrainProvider::default();
    provider2.available.insert((1, 0, 0), true);
    provider2.geometry.insert(
        (1, 0, 0),
        TerrainRequestOutcome::Failed("RuntimeError: requestTileGeometry failed as requested.".into()),
    );
    tile2.process_state_machine(None, &mut provider2, 99);
    assert_eq!(tile2.state, QuadtreeTileLoadState::Failed);
    assert_eq!(tile2.terrain_state, TerrainState::Failed);
    assert!(tile2
        .last_error
        .as_deref()
        .is_some_and(|e| e.contains("requestTileGeometry failed")));
}

#[test]
fn upsamples_from_parent_and_respects_fresh_imagery_or_terrain() {
    let mut root = GlobeSurfaceTile::new(0, 0, 0);
    let mut sw = GlobeSurfaceTile::new(0, 1, 1);
    let mut provider = TestTerrainProvider {
        allow_upsample: true,
        ..Default::default()
    };
    provider.available.insert((0, 0, 0), true);
    provider.available.insert((0, 1, 1), false);

    root.process_state_machine(None, &mut provider, 99);
    assert_eq!(root.state, QuadtreeTileLoadState::Done);
    assert!(!root.upsampled_from_parent);

    // SW has no fresh imagery -> upsampled_from_parent = true.
    sw.imagery.push(SurfaceImageryAttachment {
        state: ImageryState::Failed,
        upsampled_from_parent: true,
    });
    sw.process_state_machine(Some(&mut root), &mut provider, 99);
    assert_eq!(sw.state, QuadtreeTileLoadState::Done);
    assert!(sw
        .terrain_data
        .as_ref()
        .is_some_and(SurfaceTerrainData::was_created_by_upsampling));
    assert!(sw.upsampled_from_parent);

    // SW with fresh imagery -> upsampled_from_parent = false.
    let mut sw_fresh_img = GlobeSurfaceTile::new(0, 1, 1);
    sw_fresh_img.imagery.push(SurfaceImageryAttachment {
        state: ImageryState::Ready,
        upsampled_from_parent: false,
    });
    sw_fresh_img.process_state_machine(Some(&mut root), &mut provider, 99);
    assert_eq!(sw_fresh_img.state, QuadtreeTileLoadState::Done);
    assert!(!sw_fresh_img.upsampled_from_parent);
}

#[test]
fn handles_all_land_all_water_multibyte_and_upsampled_water_masks() {
    let mut provider = TestTerrainProvider::default();

    // All land -> None
    let mut land = GlobeSurfaceTile::new(0, 0, 0);
    provider.geometry.insert(
        (0, 0, 0),
        TerrainRequestOutcome::Loaded(SurfaceTerrainData {
            created_by_upsampling: false,
            water_mask: Some(WaterMaskData::AllLand),
            vertex_count: 4,
            minimum_height: 0.0,
            maximum_height: 10.0,
        }),
    );
    land.process_state_machine(None, &mut provider, 777);
    assert!(land.water_mask_texture.is_none());

    // All water -> shared texture
    let mut water1 = GlobeSurfaceTile::new(0, 0, 0);
    let mut water2 = GlobeSurfaceTile::new(0, 1, 1);
    provider.geometry.insert(
        (0, 0, 0),
        TerrainRequestOutcome::Loaded(SurfaceTerrainData {
            created_by_upsampling: false,
            water_mask: Some(WaterMaskData::AllWater),
            vertex_count: 4,
            minimum_height: 0.0,
            maximum_height: 0.0,
        }),
    );
    provider.geometry.insert(
        (0, 1, 1),
        TerrainRequestOutcome::Loaded(SurfaceTerrainData {
            created_by_upsampling: false,
            water_mask: Some(WaterMaskData::AllWater),
            vertex_count: 4,
            minimum_height: 0.0,
            maximum_height: 0.0,
        }),
    );
    water1.process_state_machine(None, &mut provider, 777);
    water2.process_state_machine(Some(&mut water1), &mut provider, 777);
    assert_eq!(water1.water_mask_texture, water2.water_mask_texture);
    assert!(water1
        .water_mask_texture
        .as_ref()
        .is_some_and(|t| t.is_shared_all_water && t.id == 777));

    // Multi-byte on parent, none on SW child -> upsampled water mask translation & scale
    let mut parent = GlobeSurfaceTile::new(0, 0, 0);
    let mut sw = GlobeSurfaceTile::new(0, 1, 1);
    provider.geometry.insert(
        (0, 0, 0),
        TerrainRequestOutcome::Loaded(SurfaceTerrainData {
            created_by_upsampling: false,
            water_mask: Some(WaterMaskData::MultiByte {
                width: 2,
                height: 2,
                data: vec![0, 255, 255, 0],
            }),
            vertex_count: 4,
            minimum_height: 0.0,
            maximum_height: 10.0,
        }),
    );
    provider.geometry.insert(
        (0, 1, 1),
        TerrainRequestOutcome::Loaded(SurfaceTerrainData {
            created_by_upsampling: false,
            water_mask: None,
            vertex_count: 4,
            minimum_height: 0.0,
            maximum_height: 10.0,
        }),
    );
    parent.process_state_machine(None, &mut provider, 777);
    sw.process_state_machine(Some(&mut parent), &mut provider, 777);
    assert!(sw.water_mask_texture.is_some());
    assert_eq!(
        sw.water_mask_translation_and_scale,
        Cartesian4::new(0.0, 0.0, 0.5, 0.5)
    );
}

#[test]
fn eligible_for_unloading_reflects_in_flight_states() {
    let mut tile = GlobeSurfaceTile::new(0, 0, 0);
    assert!(tile.eligible_for_unloading());

    let mut pending_geo = TestTerrainProvider::default();
    pending_geo
        .geometry
        .insert((0, 0, 0), TerrainRequestOutcome::Pending);
    tile.process_state_machine(None, &mut pending_geo, 99);
    assert_eq!(tile.terrain_state, TerrainState::Receiving);
    assert!(!tile.eligible_for_unloading());

    let mut tile2 = GlobeSurfaceTile::new(0, 0, 0);
    let mut pending_mesh = TestTerrainProvider::default();
    pending_mesh
        .mesh
        .insert((0, 0, 0), MeshCreationOutcome::Pending);
    tile2.process_state_machine(None, &mut pending_mesh, 99);
    assert_eq!(tile2.terrain_state, TerrainState::Transforming);
    assert!(!tile2.eligible_for_unloading());

    let mut tile3 = GlobeSurfaceTile::new(0, 0, 0);
    tile3.imagery.push(SurfaceImageryAttachment {
        state: ImageryState::Transitioning,
        upsampled_from_parent: false,
    });
    assert!(!tile3.eligible_for_unloading());

    tile3.free_resources();
    assert!(tile3.eligible_for_unloading());
    assert_eq!(tile3.state, QuadtreeTileLoadState::Start);
}
