use cesium_renderer::i3s_data_provider::I3SSublayer;
use cesium_viewer::widgets::{
    I3SBuildingSceneLayerExplorer, I3SBuildingSceneLayerExplorerContainer, SimpleI3SDataProvider,
};
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;

fn make_test_provider() -> SimpleI3SDataProvider {
    let mut provider = SimpleI3SDataProvider::new(vec![
        I3SSublayer {
            name: "Overview".to_string(),
            model_name: "Overview".to_string(),
            visibility: true,
            sublayers: vec![],
            ..Default::default()
        },
        I3SSublayer {
            name: "Full Model".to_string(),
            model_name: "FullModel".to_string(),
            visibility: true,
            sublayers: vec![I3SSublayer {
                name: "Cat1".to_string(),
                model_name: "Cat1".to_string(),
                visibility: false,
                sublayers: vec![
                    I3SSublayer {
                        name: "SubCat1".to_string(),
                        model_name: "SubCat1".to_string(),
                        visibility: true,
                        sublayers: vec![],
                        ..Default::default()
                    },
                    I3SSublayer {
                        name: "SubCat2".to_string(),
                        model_name: "SubCat2".to_string(),
                        visibility: false,
                        sublayers: vec![],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        },
    ]);

    provider.set_attribute_values("BldgLevel", vec![json!(1), json!(0)]);
    provider.set_attribute_values("testAttr", vec![json!(1), json!(0)]);
    provider
}

#[test]
fn can_create_bsl_explorer_ui() {
    let container = Rc::new(RefCell::new(
        I3SBuildingSceneLayerExplorerContainer::new("testContainer"),
    ));
    let provider = Rc::new(RefCell::new(SimpleI3SDataProvider::new(vec![])));

    let widget = I3SBuildingSceneLayerExplorer::new(
        Some(Rc::clone(&container)),
        Some(provider as _),
    )
    .unwrap();

    assert!(!widget.is_destroyed());
    assert_eq!(container.borrow().child_element_count(), 1);

    let root = container.borrow().children[0].clone();
    assert_eq!(root.borrow().child_element_count(), 3);
    assert_eq!(root.borrow().children[0].borrow().local_name(), "h3");
    assert_eq!(
        root.borrow().children[0].borrow().text_content(),
        "Building explorer"
    );

    assert_eq!(root.borrow().children[1].borrow().local_name(), "select");
    assert_eq!(
        root.borrow().children[1].borrow().text_content(),
        "Building layers not found"
    );

    assert_eq!(root.borrow().children[2].borrow().local_name(), "div");
    assert_eq!(root.borrow().children[2].borrow().id(), Some("bsl-wrapper"));
}

#[test]
fn throws_dev_error_with_no_container() {
    let provider = Rc::new(RefCell::new(SimpleI3SDataProvider::new(vec![])));
    let result = I3SBuildingSceneLayerExplorer::new(None, Some(provider as _));
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "container is required."
    );
}

#[test]
fn throws_dev_error_with_no_i3sdataprovider() {
    let container = Rc::new(RefCell::new(
        I3SBuildingSceneLayerExplorerContainer::new("testContainer"),
    ));
    let result = I3SBuildingSceneLayerExplorer::new(Some(container), None);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "i3sProvider is required."
    );
}

#[test]
fn can_expand_collapse_bsl_tree() {
    let container = Rc::new(RefCell::new(
        I3SBuildingSceneLayerExplorerContainer::new("testContainer"),
    ));
    let provider = Rc::new(RefCell::new(make_test_provider()));

    let _widget = I3SBuildingSceneLayerExplorer::new(
        Some(Rc::clone(&container)),
        Some(provider as _),
    )
    .unwrap();

    let expander = container
        .borrow()
        .query_selector(".expandItem")
        .expect("expandItem not found");

    let parent = expander.borrow().parent_element().unwrap();
    let grandparent = parent.borrow().parent_element().unwrap();
    let nested_list = grandparent
        .borrow()
        .query_selector("#Cat1-expander")
        .expect("Cat1-expander not found");

    assert_eq!(expander.borrow().text_content(), "+");
    assert_eq!(nested_list.borrow().class_name(), "nested");

    I3SBuildingSceneLayerExplorer::fire_click(&expander);
    assert_eq!(expander.borrow().text_content(), "-");
    assert_eq!(nested_list.borrow().class_name(), "nested active");

    I3SBuildingSceneLayerExplorer::fire_click(&expander);
    assert_eq!(expander.borrow().text_content(), "+");
    assert_eq!(nested_list.borrow().class_name(), "nested");
}
