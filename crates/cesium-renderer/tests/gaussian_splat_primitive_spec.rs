use cesium_math::{Cartesian3, SceneMode};
use cesium_renderer::commands::Pass;
use cesium_renderer::gaussian_splat_3d_tile_content::{
    GaussianSplat3DTileContent, GaussianSplat3DTileContentOptions,
};
use cesium_renderer::gaussian_splat_primitive::{
    GaussianSplatFrameContext, GaussianSplatPrimitive, GaussianSplatPrimitiveOptions,
    GaussianSplatSortingState, SnapshotState,
};

fn make_splat_content(positions: &[[f32; 3]]) -> GaussianSplat3DTileContent {
    let count = positions.len();
    let flat_pos: Vec<f32> = positions.iter().flat_map(|p| p.iter().copied()).collect();
    let rotations = [0.0_f32, 0.0, 0.0, 1.0].repeat(count);
    let scales = [1.0_f32, 1.0, 1.0].repeat(count);
    let colors = [1.0_f32, 1.0, 1.0, 1.0].repeat(count);

    GaussianSplat3DTileContent::new(GaussianSplat3DTileContentOptions {
        positions: flat_pos,
        rotations,
        scales,
        colors,
        ..Default::default()
    })
}

#[test]
fn builds_snapshot_and_emits_sorted_draw_command_after_stability() {
    let mut primitive = GaussianSplatPrimitive::default();
    let c1 = make_splat_content(&[[0.0, 0.0, -5.0], [0.0, 0.0, -20.0]]);
    let c2 = make_splat_content(&[[0.0, 0.0, -10.0]]);
    primitive.set_tiles_content(vec![c1, c2]);

    let mut frame = GaussianSplatFrameContext {
        frame_number: 1,
        camera_position: Cartesian3::ZERO,
        camera_direction: Cartesian3::new(0.0, 0.0, -1.0),
        passes_render: true,
        scene_mode: SceneMode::Scene3D,
    };

    // Frame 1: dirty reset.
    assert!(primitive.update(&frame).is_empty());
    assert!(!primitive.is_stable());

    // Frame 2: stable_frames = 1.
    frame.frame_number = 2;
    assert!(primitive.update(&frame).is_empty());
    assert!(!primitive.is_stable());

    // Frame 3: stable_frames = 2 -> builds snapshot, sorts, emits draw command.
    frame.frame_number = 3;
    let cmds = primitive.update(&frame);
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].pass, Some(Pass::GaussianSplats));
    assert_eq!(cmds[0].instance_count, 3);
    assert!(primitive.is_stable());
    assert_eq!(primitive.sorting_state(), GaussianSplatSortingState::Sorted);

    let snap = primitive.active_snapshot().expect("active snapshot");
    assert_eq!(snap.state, SnapshotState::Ready);
    // Depth along (0,0,-1) from origin: idx 1 (-20 -> +20), idx 2 (-10 -> +10), idx 0 (-5 -> +5).
    assert_eq!(snap.indexes, vec![1, 2, 0]);
}

#[test]
fn retries_texture_ready_when_sorter_becomes_available() {
    let mut primitive = GaussianSplatPrimitive::new(GaussianSplatPrimitiveOptions {
        sorter_available: false,
        ..Default::default()
    });
    let content = make_splat_content(&[[0.0, 0.0, -1.0], [0.0, 0.0, -5.0]]);
    primitive.set_tiles_content(vec![content]);

    let mut frame = GaussianSplatFrameContext::default();
    for f in 1..=3 {
        frame.frame_number = f;
        assert!(primitive.update(&frame).is_empty());
    }

    assert_eq!(primitive.sorting_state(), GaussianSplatSortingState::Waiting);
    let pending = primitive.pending_snapshot().expect("pending snapshot");
    assert_eq!(pending.state, SnapshotState::TextureReady);
    assert!(primitive.active_snapshot().is_none());

    // Sorter becomes available on frame 4.
    primitive.sorter_available = true;
    frame.frame_number = 4;
    let cmds = primitive.update(&frame);
    assert_eq!(cmds.len(), 1);
    assert_eq!(primitive.sorting_state(), GaussianSplatSortingState::Sorted);
    assert!(primitive.pending_snapshot().is_none());
    assert_eq!(
        primitive.active_snapshot().expect("active").indexes,
        vec![1, 0]
    );
}

#[test]
fn steady_sort_respects_frame_interval_and_camera_delta() {
    let mut primitive = GaussianSplatPrimitive::default();
    let content = make_splat_content(&[[0.0, 0.0, -1.0], [0.0, 0.0, -5.0]]);
    primitive.set_tiles_content(vec![content]);

    let mut frame = GaussianSplatFrameContext {
        frame_number: 1,
        camera_position: Cartesian3::ZERO,
        camera_direction: Cartesian3::new(0.0, 0.0, -1.0),
        passes_render: true,
        scene_mode: SceneMode::Scene3D,
    };
    for f in 1..=3 {
        frame.frame_number = f;
        primitive.update(&frame);
    }

    // Within min frame interval (3 frames): even big move does not trigger steady sort.
    frame.frame_number = 4;
    frame.camera_position = Cartesian3::new(0.0, 0.0, -10.0);
    assert!(!primitive.needs_steady_sort(&frame));

    // Past min frame interval, tiny move below threshold: no re-sort.
    frame.frame_number = 7;
    frame.camera_position = Cartesian3::new(0.1, 0.0, 0.0);
    assert!(!primitive.needs_steady_sort(&frame));

    // Past min frame interval, large move flips camera past both splats -> re-sorts.
    frame.camera_position = Cartesian3::new(0.0, 0.0, -10.0);
    frame.camera_direction = Cartesian3::new(0.0, 0.0, 1.0);
    assert!(primitive.needs_steady_sort(&frame));
    primitive.update(&frame);
    assert_eq!(
        primitive.active_snapshot().expect("active").indexes,
        vec![0, 1]
    );
}

#[test]
fn show_false_morphing_and_destroy_suppress_commands() {
    let mut primitive = GaussianSplatPrimitive::default();
    let content = make_splat_content(&[[0.0, 0.0, -1.0]]);
    primitive.set_tiles_content(vec![content]);

    let mut frame = GaussianSplatFrameContext::default();
    for f in 1..=3 {
        frame.frame_number = f;
        primitive.update(&frame);
    }
    assert_eq!(primitive.draw_commands().len(), 1);

    primitive.show = false;
    frame.frame_number = 4;
    assert!(primitive.update(&frame).is_empty());

    primitive.show = true;
    frame.scene_mode = SceneMode::Morphing;
    frame.frame_number = 5;
    assert!(primitive.update(&frame).is_empty());

    frame.scene_mode = SceneMode::Scene3D;
    primitive.destroy();
    assert!(primitive.is_destroyed());
    frame.frame_number = 6;
    assert!(primitive.update(&frame).is_empty());
    assert!(primitive.active_snapshot().is_none());
}
