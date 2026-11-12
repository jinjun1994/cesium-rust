use std::cell::RefCell;
use std::rc::Rc;

use cesium_viewer::widgets::{
    BaseLayerPickerViewModel, BaseLayerPickerViewModelOptions, ProviderViewModel, StandaloneGlobe,
};

fn create_test_provider(name: &str, category: &str) -> ProviderViewModel<String> {
    let name_str = name.to_string();
    ProviderViewModel::from_fn(
        name,
        format!("tooltip {name}"),
        format!("http://example.com/{name}.png"),
        move |()| name_str.clone(),
    )
    .and_then(|vm| {
        let options = cesium_viewer::widgets::ProviderViewModelOptions {
            name: Some(vm.name().to_string()),
            tooltip: Some(vm.tooltip().to_string()),
            icon_url: Some(vm.icon_url().to_string()),
            category: Some(category.to_string()),
            creation_command: Some(vm.creation_command().clone()),
        };
        ProviderViewModel::new(options)
    })
    .unwrap()
}

#[test]
fn constructor_sets_expected_values() {
    let globe = Rc::new(RefCell::new(StandaloneGlobe::new()));
    let vm: BaseLayerPickerViewModel<StandaloneGlobe, ()> = BaseLayerPickerViewModel::new(BaseLayerPickerViewModelOptions {
        globe: Some(Rc::clone(&globe)),
        ..Default::default()
    })
    .unwrap();

    assert!(Rc::ptr_eq(&vm.globe(), &globe));
    assert_eq!(vm.imagery_provider_view_models().len(), 0);
    assert_eq!(vm.terrain_provider_view_models().len(), 0);
}

#[test]
fn separates_providers_into_categories() {
    let img1 = create_test_provider("img1", "cat1");
    let img2 = create_test_provider("img2", "cat1");
    let img3 = create_test_provider("img3", "cat2");

    let trn1 = create_test_provider("trn1", "catA");
    let trn2 = create_test_provider("trn2", "catB");
    let trn3 = create_test_provider("trn3", "catB");

    let globe = Rc::new(RefCell::new(StandaloneGlobe::new()));
    let vm = BaseLayerPickerViewModel::new(BaseLayerPickerViewModelOptions {
        globe: Some(globe),
        imagery_provider_view_models: vec![img1, img2, img3],
        terrain_provider_view_models: vec![trn1, trn2, trn3],
        ..Default::default()
    })
    .unwrap();

    let img_cats = vm.categorized_imagery_providers();
    assert_eq!(img_cats.len(), 2);
    assert_eq!(img_cats[0].name, "cat1");
    assert_eq!(img_cats[0].providers.len(), 2);
    assert_eq!(img_cats[1].name, "cat2");
    assert_eq!(img_cats[1].providers.len(), 1);

    let trn_cats = vm.categorized_terrain_providers();
    assert_eq!(trn_cats.len(), 2);
    assert_eq!(trn_cats[0].name, "catA");
    assert_eq!(trn_cats[0].providers.len(), 1);
    assert_eq!(trn_cats[1].name, "catB");
    assert_eq!(trn_cats[1].providers.len(), 2);
}

#[test]
fn selecting_imagery_closes_drop_down() {
    let img = create_test_provider("img1", "cat1");
    let globe = Rc::new(RefCell::new(StandaloneGlobe::new()));
    let vm = BaseLayerPickerViewModel::new(BaseLayerPickerViewModelOptions {
        globe: Some(globe),
        imagery_provider_view_models: vec![img.clone()],
        ..Default::default()
    })
    .unwrap();

    vm.set_drop_down_visible(true);
    assert!(vm.drop_down_visible());

    vm.set_selected_imagery(Some(img));
    assert!(!vm.drop_down_visible());
}

#[test]
fn selecting_terrain_closes_drop_down() {
    let trn = create_test_provider("trn1", "cat1");
    let globe = Rc::new(RefCell::new(StandaloneGlobe::new()));
    let vm = BaseLayerPickerViewModel::new(BaseLayerPickerViewModelOptions {
        globe: Some(globe),
        terrain_provider_view_models: vec![trn.clone()],
        ..Default::default()
    })
    .unwrap();

    vm.set_drop_down_visible(true);
    assert!(vm.drop_down_visible());

    vm.set_selected_terrain(Some(trn));
    assert!(!vm.drop_down_visible());
}

#[test]
fn tooltip_and_button_image_url() {
    let img = create_test_provider("Aerial", "cat1");
    let trn = create_test_provider("WorldTerrain", "cat1");
    let globe = Rc::new(RefCell::new(StandaloneGlobe::new()));

    let vm = BaseLayerPickerViewModel::new(BaseLayerPickerViewModelOptions {
        globe: Some(globe),
        imagery_provider_view_models: vec![img.clone()],
        terrain_provider_view_models: vec![trn.clone()],
        selected_imagery_provider_view_model: Some(img.clone()),
        selected_terrain_provider_view_model: Some(trn.clone()),
    })
    .unwrap();

    assert_eq!(
        vm.button_tooltip(),
        Some(format!("{}\n{}", img.name(), trn.name()))
    );
    assert_eq!(vm.button_image_url(), Some(img.icon_url().to_string()));

    vm.set_selected_imagery(None);
    assert_eq!(vm.button_tooltip(), Some(trn.name().to_string()));
    assert_eq!(vm.button_image_url(), None);
}

#[test]
fn toggle_drop_down_command() {
    let globe = Rc::new(RefCell::new(StandaloneGlobe::new()));
    let vm: BaseLayerPickerViewModel = BaseLayerPickerViewModel::new(BaseLayerPickerViewModelOptions {
        globe: Some(globe),
        ..Default::default()
    })
    .unwrap();

    assert!(!vm.drop_down_visible());
    vm.toggle_drop_down().call().unwrap();
    assert!(vm.drop_down_visible());
    vm.toggle_drop_down().call().unwrap();
    assert!(!vm.drop_down_visible());
}
