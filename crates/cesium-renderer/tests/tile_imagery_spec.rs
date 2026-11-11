use std::cell::RefCell;
use std::rc::Rc;

use cesium_renderer::imagery_pipeline_stage::ImageryState;
use cesium_renderer::{ImageryNode, TileImagery};

#[test]
fn test_does_not_use_ancestor_ready_imagery_that_needs_to_be_reprojected() {
    let grandparent = Rc::new(RefCell::new(ImageryNode::new(
        0,
        0,
        0,
        ImageryState::Ready,
    )));
    grandparent.borrow_mut().texture = Some("texture".to_string());
    grandparent.borrow_mut().texture_web_mercator = Some("texture_wm".to_string());

    let parent = Rc::new(RefCell::new(ImageryNode::new(
        1,
        0,
        0,
        ImageryState::Ready,
    )));
    parent.borrow_mut().parent = Some(Rc::clone(&grandparent));
    parent.borrow_mut().texture = None; // needs reproject
    parent.borrow_mut().texture_web_mercator = Some("texture_wm".to_string());

    let this_node = Rc::new(RefCell::new(ImageryNode::new(
        2,
        0,
        0,
        ImageryState::Failed,
    )));
    this_node.borrow_mut().parent = Some(Rc::clone(&parent));

    let mut tile_imagery = TileImagery::new(Some(Rc::clone(&this_node)), None, false);

    let done = tile_imagery.process_state_machine();
    assert!(!done);

    assert!(tile_imagery.ready_imagery.is_some());
    assert!(Rc::ptr_eq(
        tile_imagery.ready_imagery.as_ref().unwrap(),
        &grandparent
    ));
    assert_eq!(parent.borrow().process_state_machine_calls, 1);
}

#[test]
fn test_transitions_to_ready_imagery_when_loading_imagery_is_ready() {
    let node = Rc::new(RefCell::new(ImageryNode::new(
        0,
        0,
        0,
        ImageryState::Ready,
    )));

    let mut tile_imagery = TileImagery::new(Some(Rc::clone(&node)), None, false);
    let done = tile_imagery.process_state_machine();

    assert!(done);
    assert!(tile_imagery.ready_imagery.is_some());
    assert!(Rc::ptr_eq(
        tile_imagery.ready_imagery.as_ref().unwrap(),
        &node
    ));
    assert!(tile_imagery.loading_imagery.is_none());
}

#[test]
fn test_free_resources() {
    let node = Rc::new(RefCell::new(ImageryNode::new(
        0,
        0,
        0,
        ImageryState::Ready,
    )));

    let mut tile_imagery = TileImagery::new(Some(Rc::clone(&node)), None, false);
    tile_imagery.process_state_machine();
    assert_eq!(node.borrow().reference_count, 1);

    tile_imagery.free_resources();
    assert_eq!(node.borrow().reference_count, 0);
}
