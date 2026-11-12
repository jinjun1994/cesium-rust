use std::cell::{Cell, RefCell};
use std::rc::Rc;

use cesium_math::Cartesian3;
use cesium_viewer::widgets::{
    GeocoderResult, GeocoderServiceFn, GeocoderViewModel, GeocoderViewModelOptions,
};

fn create_mock_service(name: &'static str) -> GeocoderServiceFn {
    Rc::new(move |query: &str| {
        if query.is_empty() {
            Vec::new()
        } else {
            vec![
                GeocoderResult::new(format!("{name}_1"), Cartesian3::new(1.0, 2.0, 3.0)),
                GeocoderResult::new(format!("{name}_2"), Cartesian3::new(4.0, 5.0, 6.0)),
            ]
        }
    })
}

#[test]
fn constructor_sets_expected_properties() {
    let vm = GeocoderViewModel::new(GeocoderViewModelOptions {
        scene_id: Some("scene_1".to_string()),
        flight_duration: Some(1234.0),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(vm.scene_id(), "scene_1");
    assert_eq!(vm.flight_duration(), 1234.0);
    assert!(!vm.keep_expanded());
    assert!(!vm.is_destroyed());
    vm.destroy();
    assert!(vm.is_destroyed());
}

#[test]
fn can_get_and_set_flight_duration() {
    let vm = GeocoderViewModel::new(GeocoderViewModelOptions {
        scene_id: Some("scene_1".to_string()),
        ..Default::default()
    })
    .unwrap();

    vm.set_flight_duration(324.0).unwrap();
    assert_eq!(vm.flight_duration(), 324.0);

    assert!(vm.set_flight_duration(-123.0).is_err());
}

#[test]
fn throws_without_scene() {
    assert!(GeocoderViewModel::new(GeocoderViewModelOptions::default()).is_err());
}

#[test]
fn raises_complete_event_when_search_finishes() {
    let found_called = Rc::new(Cell::new(0));
    let fc = Rc::clone(&found_called);

    let vm = GeocoderViewModel::new(GeocoderViewModelOptions {
        scene_id: Some("scene_1".to_string()),
        flight_duration: Some(0.0),
        geocoder_services: vec![create_mock_service("mock")],
        destination_found: Some(Rc::new(move |_vm, res| {
            fc.set(fc.get() + 1);
            assert_eq!(res.display_name, "mock_1");
        })),
    })
    .unwrap();

    let complete_called = Rc::new(Cell::new(0));
    let cc = Rc::clone(&complete_called);
    vm.complete.add_listener(move |()| {
        cc.set(cc.get() + 1);
    });

    vm.set_search_text("some location");
    vm.search();

    assert_eq!(found_called.get(), 1);
    assert_eq!(complete_called.get(), 1);
}

#[test]
fn automatic_suggestions_retrieved_and_empty_when_query_empty() {
    let vm = GeocoderViewModel::new(GeocoderViewModelOptions {
        scene_id: Some("scene_1".to_string()),
        geocoder_services: vec![create_mock_service("svc")],
        ..Default::default()
    })
    .unwrap();

    vm.set_search_text("hello");
    vm.update_search_suggestions();
    assert_eq!(vm.suggestions().len(), 2);
    assert_eq!(vm.suggestions()[0].display_name, "svc_1");

    vm.set_search_text("");
    vm.update_search_suggestions();
    assert_eq!(vm.suggestions().len(), 0);
}

#[test]
fn can_activate_selected_search_suggestion() {
    let found_called = Rc::new(Cell::new(0));
    let fc = Rc::clone(&found_called);

    let vm = GeocoderViewModel::new(GeocoderViewModelOptions {
        scene_id: Some("scene_1".to_string()),
        destination_found: Some(Rc::new(move |_vm, res| {
            fc.set(fc.get() + 1);
            assert_eq!(res.display_name, "custom_dest");
        })),
        ..Default::default()
    })
    .unwrap();

    let suggestion = GeocoderResult::new("custom_dest", Cartesian3::new(10.0, 20.0, 30.0));
    vm.activate_suggestion(&suggestion);

    assert_eq!(vm.search_text(), "custom_dest");
    assert_eq!(found_called.get(), 1);
}

#[test]
fn uses_first_result_from_first_geocoder_in_order() {
    let svc1 = create_mock_service("first_service");
    let svc2 = create_mock_service("second_service");

    let found_name = Rc::new(RefCell::new(String::new()));
    let fn_clone = Rc::clone(&found_name);

    let vm = GeocoderViewModel::new(GeocoderViewModelOptions {
        scene_id: Some("scene_1".to_string()),
        geocoder_services: vec![svc1, svc2],
        destination_found: Some(Rc::new(move |_vm, res| {
            *fn_clone.borrow_mut() = res.display_name.clone();
        })),
        ..Default::default()
    })
    .unwrap();

    vm.set_search_text("query");
    vm.search();

    assert_eq!(*found_name.borrow(), "first_service_1");
}
