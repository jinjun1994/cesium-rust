use cesium_math::RequestErrorEvent;
use std::collections::HashMap;

#[test]
fn test_request_error_event_parses_response_headers_string() {
    let raw = "This-is-a-test: first\r\nAnother: second value!";
    let event = RequestErrorEvent::from_raw_headers(Some(404), Some("foo".to_string()), raw);

    assert_eq!(
        event.response_headers.get("This-is-a-test").map(|s| s.as_str()),
        Some("first")
    );
    assert_eq!(
        event.response_headers.get("Another").map(|s| s.as_str()),
        Some("second value!")
    );
}

#[test]
fn test_request_error_event_leaves_headers_map_intact() {
    let mut map = HashMap::new();
    map.insert("This-is-a-test".to_string(), "first".to_string());
    map.insert("Another".to_string(), "second value!".to_string());

    let event = RequestErrorEvent::from_headers_map(Some(404), Some("foo".to_string()), map.clone());
    assert_eq!(event.response_headers, map);
}
