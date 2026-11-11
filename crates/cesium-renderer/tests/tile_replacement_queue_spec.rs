use std::rc::Rc;

use cesium_math::GeographicTilingScheme;
use cesium_renderer::quadtree_tile::{AnyTilingScheme, QuadtreeTile, QuadtreeTileOptions};
use cesium_renderer::tile_replacement_queue::TileReplacementQueue;

struct TestContext {
    queue: TileReplacementQueue,
    one: Rc<std::cell::RefCell<QuadtreeTile>>,
    two: Rc<std::cell::RefCell<QuadtreeTile>>,
    three: Rc<std::cell::RefCell<QuadtreeTile>>,
    four: Rc<std::cell::RefCell<QuadtreeTile>>,
    not_eligible_for_unloading: Rc<std::cell::RefCell<QuadtreeTile>>,
}

fn setup() -> TestContext {
    let tiling_scheme = AnyTilingScheme::Geographic(GeographicTilingScheme::default());
    let queue = TileReplacementQueue::new();
    let one = QuadtreeTile::new(QuadtreeTileOptions {
        x: 0,
        y: 0,
        level: 0,
        tiling_scheme: tiling_scheme.clone(),
        parent: None,
    })
    .unwrap();
    let two = QuadtreeTile::new(QuadtreeTileOptions {
        x: 1,
        y: 0,
        level: 0,
        tiling_scheme: tiling_scheme.clone(),
        parent: None,
    })
    .unwrap();
    let three = QuadtreeTile::new(QuadtreeTileOptions {
        x: 0,
        y: 0,
        level: 1,
        tiling_scheme: tiling_scheme.clone(),
        parent: None,
    })
    .unwrap();
    let four = QuadtreeTile::new(QuadtreeTileOptions {
        x: 2,
        y: 1,
        level: 1,
        tiling_scheme: tiling_scheme.clone(),
        parent: None,
    })
    .unwrap();
    let not_eligible_for_unloading = QuadtreeTile::new(QuadtreeTileOptions {
        x: 2,
        y: 1,
        level: 1,
        tiling_scheme,
        parent: None,
    })
    .unwrap();
    not_eligible_for_unloading.borrow_mut().eligible_for_unloading = false;

    TestContext {
        queue,
        one,
        two,
        three,
        four,
        not_eligible_for_unloading,
    }
}

#[test]
fn prevents_tiles_added_afterward_from_being_trimmed() {
    let mut ctx = setup();
    ctx.queue.mark_tile_rendered(&ctx.one);
    ctx.queue.mark_tile_rendered(&ctx.two);
    ctx.queue.mark_start_of_render_frame();

    ctx.queue.mark_tile_rendered(&ctx.three);

    ctx.queue.trim_tiles(0);

    assert_eq!(ctx.queue.count(), 1);
    assert!(Rc::ptr_eq(&ctx.queue.head().unwrap(), &ctx.three));
}

#[test]
fn prevents_all_tiles_from_being_trimmed_if_called_on_an_empty_queue() {
    let mut ctx = setup();
    ctx.queue.mark_start_of_render_frame();

    ctx.queue.mark_tile_rendered(&ctx.one);
    ctx.queue.mark_tile_rendered(&ctx.two);
    ctx.queue.mark_tile_rendered(&ctx.three);

    ctx.queue.trim_tiles(0);
    assert_eq!(ctx.queue.count(), 3);
}

#[test]
fn adjusts_properly_when_last_tile_in_previous_frame_is_moved_to_the_head() {
    let mut ctx = setup();
    ctx.queue.mark_tile_rendered(&ctx.one);
    ctx.queue.mark_tile_rendered(&ctx.two);
    ctx.queue.mark_tile_rendered(&ctx.three);

    ctx.queue.mark_start_of_render_frame();

    ctx.queue.mark_tile_rendered(&ctx.three);

    ctx.queue.trim_tiles(0);
    assert_eq!(ctx.queue.count(), 1);
    assert!(Rc::ptr_eq(&ctx.queue.head().unwrap(), &ctx.three));
}

#[test]
fn adjusts_properly_when_all_tiles_are_moved_to_the_head() {
    let mut ctx = setup();
    ctx.queue.mark_tile_rendered(&ctx.one);
    ctx.queue.mark_tile_rendered(&ctx.two);
    ctx.queue.mark_tile_rendered(&ctx.three);

    ctx.queue.mark_start_of_render_frame();

    ctx.queue.mark_tile_rendered(&ctx.one);
    ctx.queue.mark_tile_rendered(&ctx.two);
    ctx.queue.mark_tile_rendered(&ctx.three);

    ctx.queue.trim_tiles(0);
    assert_eq!(ctx.queue.count(), 3);
    assert!(Rc::ptr_eq(&ctx.queue.head().unwrap(), &ctx.three));
    assert!(Rc::ptr_eq(&ctx.queue.tail().unwrap(), &ctx.one));
}

#[test]
fn does_not_remove_a_tile_that_is_not_eligible_for_unloading() {
    let mut ctx = setup();
    ctx.queue.mark_tile_rendered(&ctx.one);
    ctx.queue.mark_tile_rendered(&ctx.two);
    ctx.queue
        .mark_tile_rendered(&ctx.not_eligible_for_unloading);
    ctx.queue.mark_tile_rendered(&ctx.three);

    ctx.queue.mark_start_of_render_frame();

    ctx.queue.trim_tiles(0);
    assert_eq!(ctx.queue.count(), 1);
    assert!(Rc::ptr_eq(
        &ctx.queue.head().unwrap(),
        &ctx.not_eligible_for_unloading
    ));
}

#[test]
fn does_not_remove_a_transitioning_tile_at_the_end_of_the_last_render_frame() {
    let mut ctx = setup();
    ctx.queue.mark_tile_rendered(&ctx.one);
    ctx.queue.mark_tile_rendered(&ctx.two);
    ctx.queue.mark_tile_rendered(&ctx.three);
    ctx.queue
        .mark_tile_rendered(&ctx.not_eligible_for_unloading);

    ctx.queue.mark_start_of_render_frame();

    ctx.queue.trim_tiles(0);
    assert_eq!(ctx.queue.count(), 1);
    assert!(Rc::ptr_eq(
        &ctx.queue.head().unwrap(),
        &ctx.not_eligible_for_unloading
    ));
}

#[test]
fn removes_two_tiles_not_used_last_render_frame() {
    let mut ctx = setup();
    ctx.queue
        .mark_tile_rendered(&ctx.not_eligible_for_unloading);
    ctx.queue.mark_tile_rendered(&ctx.one);
    ctx.queue.mark_tile_rendered(&ctx.two);
    ctx.queue.mark_start_of_render_frame();
    ctx.queue.mark_tile_rendered(&ctx.three);
    ctx.queue.mark_tile_rendered(&ctx.four);
    ctx.queue.trim_tiles(0);
    assert_eq!(ctx.queue.count(), 3);
}
