use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use cesium_math::{
    DefaultProxy, DerivedResourceOptions, Proxy, QueryValue, Request, RequestErrorEvent,
    Resource, ResourceOptions, RetryCallback,
};
use serde_json::Value;

#[test]
fn constructor_sets_correct_properties() {
    let proxy = Arc::new(DefaultProxy::new("/proxy/"));
    let request = Request::default();
    let retry_func = Arc::new(|_: &Resource, _: Option<&RequestErrorEvent>| true);

    let mut query_params = BTreeMap::new();
    query_params.insert("key1".to_string(), QueryValue::Single("value1".to_string()));
    query_params.insert("key2".to_string(), QueryValue::Single("value2".to_string()));

    let mut template_values = HashMap::new();
    template_values.insert("key3".to_string(), "value3".to_string());
    template_values.insert("key4".to_string(), "value4".to_string());

    let mut headers = HashMap::new();
    headers.insert("Accept".to_string(), "application/test-type".to_string());

    let resource = Resource::new(ResourceOptions {
        url: "http://test.com/tileset".to_string(),
        query_parameters: query_params,
        template_values,
        headers,
        proxy: Some(proxy.clone()),
        retry_callback: Some(retry_func),
        retry_attempts: 4,
        request: Some(request.clone()),
        parse_url: true,
        ..Default::default()
    });

    assert_eq!(
        resource.get_url_component(false, false),
        "http://test.com/tileset"
    );
    assert_eq!(
        resource.get_url_component(true, false),
        "http://test.com/tileset?key1=value1&key2=value2"
    );
    assert_eq!(
        resource.get_url_component(false, true),
        proxy.get_url("http://test.com/tileset")
    );
    assert_eq!(
        resource.get_url_component(true, true),
        proxy.get_url("http://test.com/tileset?key1=value1&key2=value2")
    );
    assert_eq!(
        resource.url(),
        proxy.get_url("http://test.com/tileset?key1=value1&key2=value2")
    );
    assert_eq!(
        resource.to_string(),
        proxy.get_url("http://test.com/tileset?key1=value1&key2=value2")
    );

    assert_eq!(
        resource.query_parameters().get("key1"),
        Some(&QueryValue::Single("value1".to_string()))
    );
    assert_eq!(
        resource.query_parameters().get("key2"),
        Some(&QueryValue::Single("value2".to_string()))
    );
    assert_eq!(
        resource.template_values().get("key3"),
        Some(&"value3".to_string())
    );
    assert_eq!(
        resource.template_values().get("key4"),
        Some(&"value4".to_string())
    );
    assert_eq!(
        resource.headers.get("Accept"),
        Some(&"application/test-type".to_string())
    );
    assert!(resource.proxy.is_some());
    assert_eq!(resource.retry_attempts, 4);
    assert_eq!(resource.retry_count(), 0);
    assert_eq!(resource.request, request);
}

#[test]
fn constructor_with_simple_url() {
    let url = "http://invalid.domain.com/tileset";
    let resource = Resource::from_url(url);
    assert_eq!(resource.url(), url);
    assert_eq!(resource.to_string(), url);
    assert!(resource.query_parameters().is_empty());
    assert!(resource.template_values().is_empty());
    assert!(resource.headers.is_empty());
    assert!(resource.proxy.is_none());
    assert_eq!(resource.retry_attempts, 0);
}

#[test]
fn append_forward_slash_appends_slash() {
    let mut resource = Resource::from_url("http://test.com/tileset");
    assert_eq!(resource.url(), "http://test.com/tileset");
    resource.append_forward_slash();
    assert_eq!(resource.url(), "http://test.com/tileset/");
    resource.append_forward_slash();
    assert_eq!(resource.url(), "http://test.com/tileset/");
}

#[test]
fn setting_url_with_query_string_sets_query_parameters() {
    let resource = Resource::from_url("http://test.com/tileset?key=value");
    assert_eq!(resource.base_url(), "http://test.com/tileset");
    assert_eq!(
        resource.query_parameters().get("key"),
        Some(&QueryValue::Single("value".to_string()))
    );
    assert_eq!(resource.url(), "http://test.com/tileset?key=value");
}

#[test]
fn constructing_with_parse_url_false_keeps_url() {
    let resource = Resource::new(ResourceOptions {
        url: "http://test.com/tileset?key=value".to_string(),
        parse_url: false,
        ..Default::default()
    });
    assert_eq!(resource.base_url(), "http://test.com/tileset?key=value");
    assert!(resource.query_parameters().is_empty());
}

#[test]
fn create_if_needed_functionality() {
    assert!(Resource::create_if_needed(None, None).is_none());

    let resource = Resource::from_url("http://test.com/tileset");
    let derived = Resource::create_if_needed(Some(&resource), None).unwrap();
    assert_eq!(derived.url(), resource.url());

    let from_str = Resource::create_if_needed(None, Some("http://test.com/tileset")).unwrap();
    assert_eq!(from_str.url(), "http://test.com/tileset");
}

#[test]
fn multiple_values_for_query_parameters() {
    let resource = Resource::from_url("http://test.com/tileset/endpoint?a=1&a=2&b=3&a=4");
    assert_eq!(
        resource.query_parameters().get("a"),
        Some(&QueryValue::Array(vec![
            "1".to_string(),
            "2".to_string(),
            "4".to_string()
        ]))
    );
    assert_eq!(
        resource.query_parameters().get("b"),
        Some(&QueryValue::Single("3".to_string()))
    );
    assert_eq!(
        resource.url(),
        "http://test.com/tileset/endpoint?a=1&a=2&a=4&b=3"
    );
}

#[test]
fn multiple_values_query_parameters_derived_without_preserve() {
    let resource = Resource::from_url("http://test.com/tileset/endpoint?a=1&a=2&b=3&a=4");
    let derived = resource.get_derived_resource(DerivedResourceOptions {
        url: Some("other_endpoint?a=5&b=6&a=7".to_string()),
        preserve_query_parameters: false,
        ..Default::default()
    });

    assert_eq!(
        derived.query_parameters().get("a"),
        Some(&QueryValue::Array(vec!["5".to_string(), "7".to_string()]))
    );
    assert_eq!(
        derived.query_parameters().get("b"),
        Some(&QueryValue::Single("6".to_string()))
    );
    assert_eq!(
        derived.url(),
        "http://test.com/tileset/other_endpoint?a=5&a=7&b=6"
    );
}

#[test]
fn multiple_values_query_parameters_derived_with_preserve() {
    let resource = Resource::from_url("http://test.com/tileset/endpoint?a=1&a=2&b=3&a=4");
    let derived = resource.get_derived_resource(DerivedResourceOptions {
        url: Some("other_endpoint?a=5&b=6&a=7".to_string()),
        preserve_query_parameters: true,
        ..Default::default()
    });

    assert_eq!(
        derived.query_parameters().get("a"),
        Some(&QueryValue::Array(vec![
            "5".to_string(),
            "7".to_string(),
            "1".to_string(),
            "2".to_string(),
            "4".to_string()
        ]))
    );
    assert_eq!(
        derived.query_parameters().get("b"),
        Some(&QueryValue::Array(vec!["6".to_string(), "3".to_string()]))
    );
    assert_eq!(
        derived.url(),
        "http://test.com/tileset/other_endpoint?a=5&a=7&a=1&a=2&a=4&b=6&b=3"
    );
}

#[test]
fn replaces_template_values_in_url() {
    let mut template_values = HashMap::new();
    template_values.insert("foo".to_string(), "test1".to_string());
    template_values.insert("bar".to_string(), "test2".to_string());

    let resource = Resource::new(ResourceOptions {
        url: "http://test.com/tileset/{foo}/{bar}".to_string(),
        template_values,
        parse_url: true,
        ..Default::default()
    });

    assert_eq!(resource.url(), "http://test.com/tileset/test1/test2");
}

#[test]
fn replaces_numeric_template_values() {
    let mut template_values = HashMap::new();
    template_values.insert("0".to_string(), "test1".to_string());
    template_values.insert("1".to_string(), "test2".to_string());

    let resource = Resource::new(ResourceOptions {
        url: "http://test.com/tileset/{0}/{1}".to_string(),
        template_values,
        parse_url: true,
        ..Default::default()
    });

    assert_eq!(resource.url(), "http://test.com/tileset/test1/test2");
}

#[test]
fn leaves_template_values_unchanged_when_not_provided() {
    let resource = Resource::from_url("http://test.com/tileset/{foo}/{bar}");
    assert_eq!(resource.url(), "http://test.com/tileset/{foo}/{bar}");
}

#[test]
fn url_encodes_replacement_template_values() {
    let mut template_values = HashMap::new();
    template_values.insert("foo".to_string(), "a/b".to_string());
    template_values.insert("bar".to_string(), "x$y#".to_string());

    let resource = Resource::new(ResourceOptions {
        url: "http://test.com/tileset/{foo}/{bar}".to_string(),
        template_values,
        parse_url: true,
        ..Default::default()
    });

    assert_eq!(resource.url(), "http://test.com/tileset/a%2Fb/x%24y%23");
}

#[test]
fn get_derived_resource_sets_correct_properties() {
    let proxy = Arc::new(DefaultProxy::new("/proxy/"));
    let request = Request::default();
    let retry_func = Arc::new(|_: &Resource, _: Option<&RequestErrorEvent>| true);

    let mut parent_query = BTreeMap::new();
    parent_query.insert("foo".to_string(), QueryValue::Single("bar".to_string()));

    let mut parent_templates = HashMap::new();
    parent_templates.insert("key5".to_string(), "value5".to_string());
    parent_templates.insert("key6".to_string(), "value6".to_string());

    let mut parent = Resource::new(ResourceOptions {
        url: "http://test.com/tileset?key=value".to_string(),
        query_parameters: parent_query,
        template_values: parent_templates,
        parse_url: true,
        ..Default::default()
    });
    parent.append_forward_slash();

    let mut derived_query = BTreeMap::new();
    derived_query.insert("key1".to_string(), QueryValue::Single("value1".to_string()));
    derived_query.insert("key2".to_string(), QueryValue::Single("value2".to_string()));

    let mut derived_templates = HashMap::new();
    derived_templates.insert("key3".to_string(), "value3".to_string());
    derived_templates.insert("key4".to_string(), "value4".to_string());

    let mut derived_headers = HashMap::new();
    derived_headers.insert("Accept".to_string(), "application/test-type".to_string());

    let resource = parent.get_derived_resource(DerivedResourceOptions {
        url: Some("tileset.json".to_string()),
        query_parameters: Some(derived_query),
        template_values: Some(derived_templates),
        headers: Some(derived_headers),
        proxy: Some(proxy.clone()),
        retry_callback: Some(retry_func),
        retry_attempts: Some(4),
        request: Some(request.clone()),
        preserve_query_parameters: false,
    });

    assert_eq!(
        resource.get_url_component(false, false),
        "http://test.com/tileset/tileset.json"
    );
    assert_eq!(
        resource.get_url_component(true, false),
        "http://test.com/tileset/tileset.json?foo=bar&key=value&key1=value1&key2=value2"
    );
    assert_eq!(
        resource.get_url_component(false, true),
        proxy.get_url("http://test.com/tileset/tileset.json")
    );
    assert_eq!(
        resource.url(),
        proxy.get_url("http://test.com/tileset/tileset.json?foo=bar&key=value&key1=value1&key2=value2")
    );
    assert_eq!(resource.retry_attempts, 4);
    assert_eq!(resource.retry_count(), 0);
    assert_eq!(resource.request, request);
}

#[test]
fn get_derived_resource_with_directory_and_file_parent() {
    let dir_parent = Resource::from_url("http://test.com/tileset/");
    assert_eq!(dir_parent.url(), "http://test.com/tileset/");
    let dir_child = dir_parent.get_derived_resource(DerivedResourceOptions {
        url: Some("tileset.json".to_string()),
        ..Default::default()
    });
    assert_eq!(dir_child.url(), "http://test.com/tileset/tileset.json");

    let file_parent = Resource::from_url("http://test.com/tileset/tileset.json");
    assert_eq!(file_parent.url(), "http://test.com/tileset/tileset.json");
    let file_child = file_parent.get_derived_resource(DerivedResourceOptions {
        url: Some("0/0/0.b3dm".to_string()),
        ..Default::default()
    });
    assert_eq!(file_child.url(), "http://test.com/tileset/0/0/0.b3dm");
}

#[test]
fn get_derived_resource_with_only_template_values_or_query_parameters() {
    let parent = Resource::from_url("http://test.com/terrain/{z}/{x}/{y}.terrain");
    let mut tmpl = HashMap::new();
    tmpl.insert("x".to_string(), "1".to_string());
    tmpl.insert("y".to_string(), "2".to_string());
    tmpl.insert("z".to_string(), "0".to_string());
    let child = parent.get_derived_resource(DerivedResourceOptions {
        template_values: Some(tmpl),
        ..Default::default()
    });
    assert_eq!(child.url(), "http://test.com/terrain/0/1/2.terrain");

    let parent2 = Resource::from_url("http://test.com/terrain");
    let mut qp = BTreeMap::new();
    qp.insert("x".to_string(), QueryValue::Single("1".to_string()));
    qp.insert("y".to_string(), QueryValue::Single("2".to_string()));
    qp.insert("z".to_string(), QueryValue::Single("0".to_string()));
    let child2 = parent2.get_derived_resource(DerivedResourceOptions {
        query_parameters: Some(qp),
        ..Default::default()
    });
    assert_eq!(child2.url(), "http://test.com/terrain?x=1&y=2&z=0");
}

#[test]
fn set_query_parameters_with_use_as_default() {
    let mut resource = Resource::from_url("http://test.com/terrain?x=1&y=2");
    resource.set_query_parameters(
        vec![
            ("x".to_string(), QueryValue::Single("3".to_string())),
            ("y".to_string(), QueryValue::Single("4".to_string())),
            ("z".to_string(), QueryValue::Single("0".to_string())),
        ],
        true,
    );
    assert_eq!(resource.url(), "http://test.com/terrain?x=1&y=2&z=0");

    resource.set_query_parameters(
        vec![
            ("x".to_string(), QueryValue::Single("3".to_string())),
            ("y".to_string(), QueryValue::Single("4".to_string())),
            ("z".to_string(), QueryValue::Single("0".to_string())),
        ],
        false,
    );
    assert_eq!(resource.url(), "http://test.com/terrain?x=3&y=4&z=0");
}

#[test]
fn append_query_parameters_works_with_arrays_and_non_arrays() {
    let mut resource = Resource::from_url("http://test.com/terrain?x=1&y=2");
    resource.append_query_parameters(vec![
        ("x".to_string(), QueryValue::Single("3".to_string())),
        ("y".to_string(), QueryValue::Single("4".to_string())),
        ("z".to_string(), QueryValue::Single("0".to_string())),
    ]);
    assert_eq!(
        resource.query_parameters().get("x"),
        Some(&QueryValue::Array(vec!["3".to_string(), "1".to_string()]))
    );
    assert_eq!(
        resource.query_parameters().get("y"),
        Some(&QueryValue::Array(vec!["4".to_string(), "2".to_string()]))
    );
    assert_eq!(
        resource.query_parameters().get("z"),
        Some(&QueryValue::Single("0".to_string()))
    );

    let mut resource2 = Resource::new(ResourceOptions {
        url: "http://test.com/terrain".to_string(),
        query_parameters: {
            let mut m = BTreeMap::new();
            m.insert("x".to_string(), QueryValue::Array(vec!["1".to_string(), "2".to_string()]));
            m.insert("y".to_string(), QueryValue::Single("2".to_string()));
            m.insert("z".to_string(), QueryValue::Array(vec!["-1".to_string(), "-2".to_string()]));
            m
        },
        parse_url: true,
        ..Default::default()
    });

    resource2.append_query_parameters(vec![
        ("x".to_string(), QueryValue::Single("3".to_string())),
        ("y".to_string(), QueryValue::Array(vec!["4".to_string(), "5".to_string()])),
        ("z".to_string(), QueryValue::Array(vec!["-3".to_string(), "-4".to_string()])),
    ]);

    assert_eq!(
        resource2.query_parameters().get("x"),
        Some(&QueryValue::Array(vec!["3".to_string(), "1".to_string(), "2".to_string()]))
    );
    assert_eq!(
        resource2.query_parameters().get("y"),
        Some(&QueryValue::Array(vec!["4".to_string(), "5".to_string(), "2".to_string()]))
    );
    assert_eq!(
        resource2.query_parameters().get("z"),
        Some(&QueryValue::Array(vec!["-3".to_string(), "-4".to_string(), "-1".to_string(), "-2".to_string()]))
    );
}

#[test]
fn set_template_values_with_use_as_default() {
    let mut template_values = HashMap::new();
    template_values.insert("x".to_string(), "1".to_string());
    template_values.insert("y".to_string(), "2".to_string());
    template_values.insert("map".to_string(), "my map".to_string());

    let mut resource = Resource::new(ResourceOptions {
        url: "http://test.com/terrain/{z}/{x}/{y}.terrain".to_string(),
        template_values,
        parse_url: true,
        ..Default::default()
    });

    let updates = vec![
        ("x".to_string(), "3".to_string()),
        ("y".to_string(), "4".to_string()),
        ("z".to_string(), "0".to_string()),
        ("style".to_string(), "my style".to_string()),
    ];

    resource.set_template_values(updates.clone(), true);
    assert_eq!(resource.template_values().get("x").unwrap(), "1");
    assert_eq!(resource.template_values().get("y").unwrap(), "2");
    assert_eq!(resource.template_values().get("z").unwrap(), "0");
    assert_eq!(resource.template_values().get("map").unwrap(), "my map");
    assert_eq!(resource.template_values().get("style").unwrap(), "my style");

    resource.set_template_values(updates, false);
    assert_eq!(resource.template_values().get("x").unwrap(), "3");
    assert_eq!(resource.template_values().get("y").unwrap(), "4");
    assert_eq!(resource.template_values().get("z").unwrap(), "0");
    assert_eq!(resource.template_values().get("map").unwrap(), "my map");
    assert_eq!(resource.template_values().get("style").unwrap(), "my style");
}

#[test]
fn retry_on_error_does_not_exceed_retry_attempts() {
    let call_count = Arc::new(AtomicU32::new(0));
    let call_count_clone = call_count.clone();

    let retry_cb: RetryCallback = Arc::new(move |_, _| {
        call_count_clone.fetch_add(1, Ordering::SeqCst);
        true
    });

    let mut resource = Resource::new(ResourceOptions {
        url: "http://test.com/endpoint".to_string(),
        retry_attempts: 2,
        retry_callback: Some(retry_cb),
        ..Default::default()
    });

    let err = RequestErrorEvent::new(Some(500), None, None);
    assert!(resource.retry_on_error(Some(&err)));
    assert_eq!(resource.retry_count(), 1);
    assert_eq!(call_count.load(Ordering::SeqCst), 1);

    assert!(resource.retry_on_error(Some(&err)));
    assert_eq!(resource.retry_count(), 2);
    assert_eq!(call_count.load(Ordering::SeqCst), 2);

    assert!(!resource.retry_on_error(Some(&err)));
    assert_eq!(resource.retry_count(), 2);
    assert_eq!(call_count.load(Ordering::SeqCst), 2);
}

#[test]
fn is_data_uri_and_is_blob_uri() {
    let data_res = Resource::from_url("data:text/plain;base64,SGVsbG8sIFdvcmxkIQ%3D%3D");
    assert!(data_res.is_data_uri());
    assert!(!data_res.is_blob_uri());

    let blob_res = Resource::from_url("blob:d3958f5c-0777-0845-9dcf-2cb28783acaf");
    assert!(!blob_res.is_data_uri());
    assert!(blob_res.is_blob_uri());

    let normal_res = Resource::from_url("http://invalid.uri/tileset");
    assert!(!normal_res.is_data_uri());
    assert!(!normal_res.is_blob_uri());
}

#[test]
fn data_uri_decoding_and_payloads() {
    let uri_escaped = Resource::from_url("data:,Hello%2C%20World!");
    assert_eq!(uri_escaped.fetch_text().unwrap(), "Hello, World!");

    let base64_text = Resource::from_url("data:text/plain;base64,SGVsbG8sIFdvcmxkIQ==");
    assert_eq!(base64_text.fetch_text().unwrap(), "Hello, World!");

    let base64_uri_encoded =
        Resource::from_url("data:text/plain;base64,SGVsbG8sIFdvcmxkIQ%3D%3D");
    assert_eq!(base64_uri_encoded.fetch_text().unwrap(), "Hello, World!");

    let escaped_html = Resource::from_url("data:text/html,%3Ch1%3EHello%2C%20World!%3C%2Fh1%3E");
    assert_eq!(escaped_html.fetch_text().unwrap(), "<h1>Hello, World!</h1>");

    let escaped_json = Resource::from_url("data:application/json,%7B%22key%22%3A%22value%22%7D");
    let val: Value = escaped_json.fetch_json().unwrap();
    assert_eq!(val["key"], "value");

    let base64_json = Resource::from_url("data:application/json;base64,eyJrZXkiOiJ2YWx1ZSJ9");
    let val2: Value = base64_json.fetch_json().unwrap();
    assert_eq!(val2["key"], "value");

    let tile = "data:;base64,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DwABABIAAgAAAAIAEwACAAEABgADAAIAEwAAABMAFAACAAEAAgAAAAAAAgAEAAYAAwAEAAEACAACAAYACAAHAAAACQAAAAQAAgABAAoAAAAKAAAACwAOAAAABQAMAAMAAgAMAAEADwAQAAAAAgAQAAEAAAACABIAEgAUAAEAAAADAAIAAAAEAAIABAABAAUABQABAAYAAAAIAAcABwACAAEACAAAAAAAAwACAAoAAQAMAAoAAAAMAA0AAQANAAIAAAANAAAAAAASAA8AAwABAA8ADwAEAAIAEQASAAEAAAASAAAAEwACABUAAwABABMAAAAEAAYAAQACAAQAAAAAAAgACAABAAMABwACAAgAAgAHAAAAAQAIAAoACgAAAAIADAABAAsADQABAAwADQAOAAAAAgAOAAAAAQAPAAAAAwABABAAEgAAABIABAASAAEAEwAAAAIAFQABABQAAQAAAAMAAwAAAAcABAACAAEAAQAFAAcABQAAAAAABwAAAAkAAQAIAAIAAwAIAAAAAQAJAAUAAgAAAAsACwABAAAADAABAA4ADwAOAAEAAAASABAAAgABABAAAQARABIAAAASAAIAEgABAAAAFQATAAEAAAAEAAAABgAHAAUAAQAFAAcABQACAAQABwAAAAIACAAKAAEACgAAAAIAAQALAAAAAQAMAA0ADgABAA0AAAACAA8AAQAPAAAADQABABAAEQASAA0AEgABAA0AAQASAAAAAgABAAAAAQAEAAMAAAAFAAAAAAAHAAMAAQAIAAcAAgAHAAQACAAAAAoAAQAJAAAACgADAAEACwAAAA0ADAADAAAAAQACAA0ADwAOAAIAAgAQAA8AAgAAABEAEwARAAAAAQASAAIAAQAAABUAFAAVAAEAAQAAABUAFQABACkAAgAAAAAAAgAEAAUAAwAEAAAAAQAFAAIABwAAAAcABwABAAQAAAAAAAoAAwAKAAEACgAMAAIAAAANAAwADQABAAMAAAACAA0ADQAQAAEAAAAAAAQABgAEAAEAAwACAAQAAQAFAAYABQAAAAAABwADAAIABwAAAAoAAQAIAAIAAAAOAAsACwACAAEADQAOAAEAAAAAAA8ADwABAA4AAgAPAAAAAQAQAAQAAAATABAAEAADAAEAPAA7ABMAAQA8ABMAPAABAAAAAQACAAAAAwAFAAEABQAGAAAAAAAHAAIAAwAHAAEACAAAAAAACAACAAoAAQAEAAoACAAMAAAAAQADAAkADAANAAkADgABAA0ADwAAAA8AAgAPAAAAAQAQAAIAAAASAAAAEwAUAAEABAATAAIAFAAVAAEAAQAVAAAAAwACAAEAAQAAAAQAAQAGAAQAAAAHAAAACAACAAcAAQAIAAMAAAAKAAgACAADAAEADAAKAAEADAAAAAwAAgAAAA4AAgAOAAEAAAARAA4ADgADAAEAAAAAABMAAwACABMAEwABABIAAAAVABMAAQATAAIAAgAAAAAABAABAAMABQACAAQABgAIAAUAAAADAAYABgAJAAEAAAACAAoACQABAAoAAAAKAAAAAwALAAAAAQAMAAMAAgAMAA4ADgAQAAIAAAADABEAEQAQAAEAAAAAABIAFAACABIAAwASAAEAFAAVAAIAAgAAAAIAAwAWAAAAAAADAAUAAgABAAUAAAAFAAQABQABAAcAAAAKAAgACAACAAEACgABAAsACwABAAAACgAMAAEACgAAAAAAAQAPAAwAAwACAAwAAAAQAAIAEAABABEAAQATABEAAQAAAAAAAAAEAAIAAAAXAAUABQACAAEABAAFAAYABgAHAAQAAAAIAAkACQAAAAIACwABAAoACwAMAAEAAQAMAAAAEAABAA0AEAAOAAAAAgARAAEAAAAQABEAAQARAAAAAgABAAAAAgATAAAAAwAAAAMABAAAAAAABgAEAAIAAQADAAYABABCAAAAQwAFAAAABAAGAAIAAQAGABkABAAAAAQAVQAFAAMABQBVAAEAAAAHAAUABQACAAEABwABAAAAAgADAFoAAgAAAAIAAQADAFsAWwAEAFkAWQAEAFcABABYAFcAWAAGAEkABgBIAEkAMwBIAAUAHQAzAAUAWwBaAAEAAQBaAAAAAABcAF4AXAABAAIAAAACAGAAXwBgAAIAAAAAAAMAAgADAGIAYgB0AAIAAAACAAMAdQABAAMAAgABAAAAAgAAAAIAAwB3AHgAAQADAAAAAQAEAHkAAAADAAIAAwABAAAAewCPAAMAAwCPAAIAAAADAAAAkQCSAAQABAACAAMAAQAEAJIAAQAAAAMAkwCmAAIApgABAAIApgClAAEAIQAAACEAIQABAAAAAQA4ACIAAAACAAMAOQACADgAOAACAAAAAAACAAQABAADAAEAAwAAAAIAAAAEAAMAAwACAAEABAABAAAAPQAFAD8AAQAAAAYAQAAGAAEAAAAEAAAABAAFAAAABgAHAAIAAQAGAAMABwAAAAMAAQAEAAMAvQC+AAQABAABAL0ABAC+AAIAvgAGAAIAAAAGAAcAAQAHALsAuwAHAL8ABgABAAAABwBaAEYAWgAHAAEAAQACAAAAvgADAL0AAQADAL4AAgBcAFsAAQCZAAIAAgCZAFwArACZAAEAvgCsAAEAXACZAJgAwAAAAMMAwQAFAAAAAgDCAAEA2AACANYAxAACANgAEQAAAAEAnwCoANUA2QAFANoAzgDLAMkApwCmAKMAqgCsAK0AqwACAAAAAQAEABEAAAAEAK4ADAAkAMYAxQDIAMIAwQC/ALsAugC5ALIAsAC8AL0AAgAAAJ8ArgA=";
    let tile_res = Resource::from_url(tile);
    let bytes = tile_res.fetch_array_buffer().unwrap();
    assert_eq!(bytes.len(), 3914);

    let blob = tile_res.fetch_blob().unwrap();
    assert_eq!(blob.data.len(), 3914);
}
