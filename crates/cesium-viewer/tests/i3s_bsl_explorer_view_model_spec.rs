use cesium_renderer::i3s_data_provider::{I3SAttributeFilter, I3SSublayer};
use cesium_viewer::widgets::{
    I3SBuildingSceneLayerExplorerViewModel, I3SLevel, SimpleI3SDataProvider, TopLayerItem,
};
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;

fn make_test_provider() -> SimpleI3SDataProvider {
    let mut provider = SimpleI3SDataProvider::new(vec![
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
        I3SSublayer {
            name: "Overview".to_string(),
            model_name: "Overview".to_string(),
            visibility: true,
            sublayers: vec![],
            ..Default::default()
        },
    ]);

    provider.set_attribute_values("BldgLevel", vec![json!(1), json!(0)]);
    provider.set_attribute_values("testAttr", vec![json!(1), json!(0)]);
    provider
}

fn make_provider_without_overview() -> SimpleI3SDataProvider {
    SimpleI3SDataProvider::new(vec![I3SSublayer {
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
    }])
}

#[test]
fn can_create_bsl_explorer_view_model() {
    let provider = Rc::new(RefCell::new(make_test_provider()));
    let view_model =
        I3SBuildingSceneLayerExplorerViewModel::new(Rc::clone(&provider) as _).unwrap();

    assert_eq!(
        view_model.levels(),
        &[I3SLevel::All, I3SLevel::Level(0), I3SLevel::Level(1)]
    );
    assert_eq!(view_model.selected_level(), &I3SLevel::All);

    let sublayers = view_model.sublayers();
    assert_eq!(sublayers.len(), 2);

    assert_eq!(sublayers[1].name, "Overview");
    assert_eq!(sublayers[1].model_name, "Overview");
    assert!(!sublayers[1].visibility);
    assert_eq!(sublayers[1].sublayers.len(), 0);

    assert_eq!(sublayers[0].name, "Full Model");
    assert_eq!(sublayers[0].model_name, "FullModel");
    assert!(!sublayers[0].visibility);
    assert_eq!(sublayers[0].sublayers.len(), 1);

    assert_eq!(sublayers[0].sublayers[0].name, "Cat1");
    assert!(sublayers[0].sublayers[0].visibility);
    assert_eq!(sublayers[0].sublayers[0].sublayers.len(), 2);

    assert_eq!(sublayers[0].sublayers[0].sublayers[0].name, "SubCat1");
    assert!(sublayers[0].sublayers[0].sublayers[0].visibility);
    assert_eq!(sublayers[0].sublayers[0].sublayers[0].sublayers.len(), 0);

    assert_eq!(sublayers[0].sublayers[0].sublayers[1].name, "SubCat2");
    assert!(!sublayers[0].sublayers[0].sublayers[1].visibility);
    assert_eq!(sublayers[0].sublayers[0].sublayers[1].sublayers.len(), 0);

    assert_eq!(view_model.top_layers().len(), 3);
    assert_eq!(
        view_model.default_layer().unwrap().model_name.as_deref(),
        Some("Overview")
    );
}

#[test]
fn can_create_bsl_explorer_view_model_if_no_overview() {
    let provider = Rc::new(RefCell::new(make_provider_without_overview()));
    let view_model =
        I3SBuildingSceneLayerExplorerViewModel::new(Rc::clone(&provider) as _).unwrap();

    let sublayers = view_model.sublayers();
    assert_eq!(sublayers.len(), 1);
    assert_eq!(sublayers[0].name, "Full Model");
    assert_eq!(sublayers[0].model_name, "FullModel");
    assert!(!sublayers[0].visibility);
    assert_eq!(sublayers[0].sublayers.len(), 1);

    assert_eq!(sublayers[0].sublayers[0].name, "Cat1");
    assert!(sublayers[0].sublayers[0].visibility);
    assert_eq!(sublayers[0].sublayers[0].sublayers.len(), 2);

    assert_eq!(sublayers[0].sublayers[0].sublayers[0].name, "SubCat1");
    assert!(sublayers[0].sublayers[0].sublayers[0].visibility);
    assert_eq!(sublayers[0].sublayers[0].sublayers[0].sublayers.len(), 0);

    assert_eq!(sublayers[0].sublayers[0].sublayers[1].name, "SubCat2");
    assert!(!sublayers[0].sublayers[0].sublayers[1].visibility);
    assert_eq!(sublayers[0].sublayers[0].sublayers[1].sublayers.len(), 0);

    assert_eq!(view_model.top_layers().len(), 2);
    assert_eq!(
        view_model.default_layer().unwrap().model_name.as_deref(),
        Some("FullModel")
    );
}

#[test]
fn can_handle_filtering_by_level() {
    let provider = Rc::new(RefCell::new(make_test_provider()));
    let mut view_model =
        I3SBuildingSceneLayerExplorerViewModel::new(Rc::clone(&provider) as _).unwrap();

    view_model.set_current_level(I3SLevel::Level(1));
    assert_eq!(
        provider.borrow().last_filters,
        Some(vec![I3SAttributeFilter::new("BldgLevel", vec![json!(1)])])
    );

    view_model.set_current_level(I3SLevel::All);
    assert_eq!(provider.borrow().last_filters, None);
}

#[test]
fn can_handle_top_layer_selection() {
    let provider = Rc::new(RefCell::new(make_test_provider()));
    let mut view_model =
        I3SBuildingSceneLayerExplorerViewModel::new(Rc::clone(&provider) as _).unwrap();

    view_model.set_current_layer(TopLayerItem::new(
        "Full Model",
        Some("FullModel".to_string()),
        false,
        1,
    ));
    view_model.set_current_level(I3SLevel::Level(1));

    view_model.set_current_layer(TopLayerItem::new(
        "Overview",
        Some("Overview".to_string()),
        false,
        0,
    ));
    assert!(view_model.sublayers()[0].visibility);
    assert!(!view_model.sublayers()[1].visibility);
    assert_eq!(view_model.selected_level(), &I3SLevel::Level(1));
    assert_eq!(view_model.current_level(), Some(&I3SLevel::All));
    assert!(!view_model.is_bsl_wrapper_visible());

    view_model.set_current_layer(TopLayerItem::new(
        "Full Model",
        Some("FullModel".to_string()),
        false,
        1,
    ));
    assert!(!view_model.sublayers()[0].visibility);
    assert!(view_model.sublayers()[1].visibility);
    assert_eq!(view_model.current_level(), Some(&I3SLevel::Level(1)));
    assert!(view_model.is_bsl_wrapper_visible());
}

#[test]
fn can_handle_top_layer_selection_if_no_overview() {
    let provider = Rc::new(RefCell::new(make_provider_without_overview()));
    let mut view_model =
        I3SBuildingSceneLayerExplorerViewModel::new(Rc::clone(&provider) as _).unwrap();

    view_model.set_current_layer(TopLayerItem::new(
        "Full Model",
        Some("FullModel".to_string()),
        false,
        0,
    ));
    assert!(view_model.sublayers()[0].visibility);
    assert!(view_model.is_bsl_wrapper_visible());
}
