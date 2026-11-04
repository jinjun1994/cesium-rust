use cesium_math::{Request, RequestOptions, RequestState, RequestType};

#[test]
fn test_request_constructor_properties() {
    let options = RequestOptions {
        url: Some("https://www.example.com".to_string()),
        priority: 1.0,
        throttle: true,
        throttle_by_server: true,
        request_type: RequestType::IMAGERY,
        server_key: Some("customKey".to_string()),
    };

    let request = Request::new(options);

    assert_eq!(request.url.as_deref(), Some("https://www.example.com"));
    assert_eq!(request.priority, 1.0);
    assert!(request.throttle);
    assert!(request.throttle_by_server);
    assert_eq!(request.request_type, RequestType::IMAGERY);
    assert_eq!(request.server_key.as_deref(), Some("customKey"));
    assert_eq!(request.state, RequestState::UNISSUED);
    assert!(!request.cancelled);
}

#[test]
fn test_request_cancel() {
    let mut request = Request::default();
    assert!(!request.cancelled);
    request.cancel();
    assert!(request.cancelled);
}

#[test]
fn test_request_clone() {
    let options = RequestOptions {
        url: Some("https://www.example.com".to_string()),
        priority: 1.0,
        throttle: true,
        throttle_by_server: true,
        request_type: RequestType::IMAGERY,
        server_key: Some("customKey".to_string()),
    };

    let request = Request::new(options);
    let clone = request.clone();

    assert_eq!(clone.url, request.url);
    assert_eq!(clone.priority, request.priority);
    assert_eq!(clone.throttle, request.throttle);
    assert_eq!(clone.throttle_by_server, request.throttle_by_server);
    assert_eq!(clone.request_type, request.request_type);
    assert_eq!(clone.server_key, request.server_key);
}

#[test]
fn test_request_clone_into() {
    let options1 = RequestOptions {
        url: Some("https://www.example.com".to_string()),
        priority: 1.0,
        throttle: true,
        throttle_by_server: true,
        request_type: RequestType::IMAGERY,
        server_key: Some("customKey".to_string()),
    };

    let options2 = RequestOptions {
        url: Some("http://example.com".to_string()),
        priority: 2.0,
        throttle: false,
        throttle_by_server: false,
        request_type: RequestType::TERRAIN,
        server_key: Some("updatedKey".to_string()),
    };

    let mut request1 = Request::new(options1);
    let request2 = Request::new(options2);

    request2.clone_into(&mut request1);

    assert_eq!(request1.url.as_deref(), Some("http://example.com"));
    assert_eq!(request1.priority, 2.0);
    assert!(!request1.throttle);
    assert!(!request1.throttle_by_server);
    assert_eq!(request1.request_type, RequestType::TERRAIN);
    assert_eq!(request1.server_key.as_deref(), Some("updatedKey"));
}
