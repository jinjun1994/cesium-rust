use std::cell::Cell;
use std::rc::Rc;

use cesium_viewer::widgets::{create_command, ProviderViewModel, ProviderViewModelOptions};

#[test]
fn constructor_sets_expected_parameters() {
    let call_count = Rc::new(Cell::new(0));
    let cc = Rc::clone(&call_count);

    let command = create_command(move |()| {
        cc.set(cc.get() + 1);
        "layer_created"
    });

    let options = ProviderViewModelOptions {
        name: Some("Bing Maps".to_string()),
        tooltip: Some("Bing Maps Aerial".to_string()),
        icon_url: Some("http://example.com/icon.png".to_string()),
        category: Some("Aerial".to_string()),
        creation_command: Some(command),
    };

    let vm = ProviderViewModel::new(options).unwrap();
    assert_eq!(vm.name(), "Bing Maps");
    assert_eq!(vm.tooltip(), "Bing Maps Aerial");
    assert_eq!(vm.icon_url(), "http://example.com/icon.png");
    assert_eq!(vm.category(), "Aerial");

    let result = vm.creation_command().call().unwrap();
    assert_eq!(result, Some("layer_created"));
    assert_eq!(call_count.get(), 1);
}

#[test]
fn from_fn_convenience_constructor() {
    let call_count = Rc::new(Cell::new(0));
    let cc = Rc::clone(&call_count);

    let vm = ProviderViewModel::from_fn(
        "OSM",
        "OpenStreetMap",
        "http://example.com/osm.png",
        move |()| {
            cc.set(cc.get() + 1);
            42
        },
    )
    .unwrap();

    assert_eq!(vm.name(), "OSM");
    assert_eq!(vm.tooltip(), "OpenStreetMap");
    assert_eq!(vm.icon_url(), "http://example.com/osm.png");
    assert_eq!(vm.category(), "");

    assert_eq!(vm.creation_command().call().unwrap(), Some(42));
    assert_eq!(call_count.get(), 1);
}

#[test]
fn constructor_throws_with_no_name() {
    let options: ProviderViewModelOptions<()> = ProviderViewModelOptions {
        name: None,
        tooltip: Some("tooltip".to_string()),
        icon_url: Some("icon.png".to_string()),
        category: None,
        creation_command: Some(create_command(|()| ())),
    };

    let err = ProviderViewModel::new(options).unwrap_err();
    assert!(err.message.contains("options.name is required."));
}

#[test]
fn constructor_throws_with_no_tooltip() {
    let options: ProviderViewModelOptions<()> = ProviderViewModelOptions {
        name: Some("name".to_string()),
        tooltip: None,
        icon_url: Some("icon.png".to_string()),
        category: None,
        creation_command: Some(create_command(|()| ())),
    };

    let err = ProviderViewModel::new(options).unwrap_err();
    assert!(err.message.contains("options.tooltip is required."));
}

#[test]
fn constructor_throws_with_no_icon_url() {
    let options: ProviderViewModelOptions<()> = ProviderViewModelOptions {
        name: Some("name".to_string()),
        tooltip: Some("tooltip".to_string()),
        icon_url: None,
        category: None,
        creation_command: Some(create_command(|()| ())),
    };

    let err = ProviderViewModel::new(options).unwrap_err();
    assert!(err.message.contains("options.iconUrl is required."));
}

#[test]
fn constructor_throws_with_no_creation_function() {
    let options: ProviderViewModelOptions<()> = ProviderViewModelOptions {
        name: Some("name".to_string()),
        tooltip: Some("tooltip".to_string()),
        icon_url: Some("icon.png".to_string()),
        category: None,
        creation_command: None,
    };

    let err = ProviderViewModel::new(options).unwrap_err();
    assert!(err.message.contains("options.creationFunction is required."));
}
