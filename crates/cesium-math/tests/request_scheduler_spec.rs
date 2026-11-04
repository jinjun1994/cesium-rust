use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cesium_math::{
    get_server_key, is_blob_uri, is_data_uri, Request, RequestOptions, RequestScheduler,
    RequestState,
};

#[test]
fn test_get_server_key_and_uri_predicates() {
    assert!(is_data_uri("data:text/plain;base64,SGVsbG8="));
    assert!(!is_data_uri("https://example.com"));

    assert!(is_blob_uri("blob:http://example.com/uuid"));
    assert!(!is_blob_uri("https://example.com"));

    assert!(get_server_key("").is_err());
    assert_eq!(
        get_server_key("https://test.invalid/1").unwrap(),
        "test.invalid:443"
    );
    assert_eq!(
        get_server_key("http://test.invalid/1").unwrap(),
        "test.invalid:80"
    );
    assert_eq!(
        get_server_key("https://test.invalid:80/1").unwrap(),
        "test.invalid:80"
    );
    assert_eq!(
        get_server_key("http://myhost:8080/path").unwrap(),
        "myhost:8080"
    );
}

#[test]
fn test_request_validation() {
    let mut scheduler = RequestScheduler::new();
    let invalid_req = Rc::new(RefCell::new(Request::default()));
    assert!(scheduler.request(&invalid_req).is_err());
}

#[test]
fn test_honors_maximum_requests() {
    let mut scheduler = RequestScheduler::new();
    scheduler.maximum_requests = 2;

    let create_request = || {
        Rc::new(RefCell::new(Request::new(RequestOptions {
            url: Some("http://test.invalid/1".to_string()),
            throttle: true,
            ..Default::default()
        })))
    };

    let r1 = create_request();
    let r2 = create_request();
    let r3 = create_request();

    assert!(scheduler.request(&r1).unwrap());
    assert!(scheduler.request(&r2).unwrap());
    scheduler.update();

    assert_eq!(scheduler.statistics.number_of_active_requests, 2);
    assert_eq!(r1.borrow().state, RequestState::Active);
    assert_eq!(r2.borrow().state, RequestState::Active);

    // Scheduler is full, r3 will be rejected (returns false)
    assert!(!scheduler.request(&r3).unwrap());
    scheduler.update();
    assert_eq!(scheduler.statistics.number_of_active_requests, 2);

    // Complete r1, slot opens up
    scheduler.complete_request(&r1);
    assert_eq!(r1.borrow().state, RequestState::Received);
    scheduler.update();
    assert_eq!(scheduler.statistics.number_of_active_requests, 1);

    let r4 = create_request();
    assert!(scheduler.request(&r4).unwrap());
    scheduler.update();
    assert_eq!(scheduler.statistics.number_of_active_requests, 2);
    assert_eq!(r4.borrow().state, RequestState::Active);

    // Full again
    let r5 = create_request();
    assert!(!scheduler.request(&r5).unwrap());

    // Increase maximum requests to 3
    scheduler.maximum_requests = 3;
    let r6 = create_request();
    assert!(scheduler.request(&r6).unwrap());
    scheduler.update();
    assert_eq!(scheduler.statistics.number_of_active_requests, 3);
}

#[test]
fn test_honors_maximum_requests_per_server() {
    let mut scheduler = RequestScheduler::new();
    scheduler.maximum_requests_per_server = 2;

    let url = "http://test.invalid/1".to_string();
    let server = get_server_key(&url).unwrap();

    let create_request = || {
        Rc::new(RefCell::new(Request::new(RequestOptions {
            url: Some(url.clone()),
            throttle_by_server: true,
            throttle: true,
            ..Default::default()
        })))
    };

    let r1 = create_request();
    let r2 = create_request();
    assert!(scheduler.request(&r1).unwrap());
    assert!(scheduler.request(&r2).unwrap());
    scheduler.update();

    assert_eq!(
        scheduler.number_of_active_requests_by_server(&server),
        2
    );

    let r3 = create_request();
    // Server slots saturated, returns false
    assert!(!scheduler.request(&r3).unwrap());

    scheduler.complete_request(&r1);
    scheduler.update();
    assert_eq!(
        scheduler.number_of_active_requests_by_server(&server),
        1
    );

    let r4 = create_request();
    assert!(scheduler.request(&r4).unwrap());
    scheduler.update();
    assert_eq!(
        scheduler.number_of_active_requests_by_server(&server),
        2
    );

    // Increase maximum requests per server to 3
    scheduler.maximum_requests_per_server = 3;
    let r5 = create_request();
    assert!(scheduler.request(&r5).unwrap());
    scheduler.update();
    assert_eq!(
        scheduler.number_of_active_requests_by_server(&server),
        3
    );
}

#[test]
fn test_data_and_blob_uri_go_through_immediately() {
    let mut scheduler = RequestScheduler::new();
    let event_raised = Rc::new(RefCell::new(false));
    let raised_clone = event_raised.clone();
    scheduler.request_completed_event.add_listener(move |_| {
        *raised_clone.borrow_mut() = true;
    });

    let data_req = Rc::new(RefCell::new(Request::new(RequestOptions {
        url: Some("data:text/plain;base64,SGVsbG8=".to_string()),
        ..Default::default()
    })));

    assert!(scheduler.request(&data_req).unwrap());
    assert_eq!(data_req.borrow().state, RequestState::Received);
    assert!(*event_raised.borrow());
    assert_eq!(scheduler.statistics.number_of_active_requests, 0);

    *event_raised.borrow_mut() = false;
    let blob_req = Rc::new(RefCell::new(Request::new(RequestOptions {
        url: Some("blob:https://test.invalid/uuid-1234".to_string()),
        ..Default::default()
    })));

    assert!(scheduler.request(&blob_req).unwrap());
    assert_eq!(blob_req.borrow().state, RequestState::Received);
    assert!(*event_raised.borrow());
}

#[test]
fn test_unthrottled_request_immediate_active() {
    let mut scheduler = RequestScheduler::new();
    let req = Rc::new(RefCell::new(Request::new(RequestOptions {
        url: Some("https://test.invalid/1".to_string()),
        throttle: false,
        ..Default::default()
    })));

    assert!(scheduler.request(&req).unwrap());
    assert_eq!(req.borrow().state, RequestState::Active);
    assert_eq!(scheduler.statistics.number_of_active_requests, 1);
    assert_eq!(
        scheduler.number_of_active_requests_by_server("test.invalid:443"),
        1
    );

    scheduler.complete_request(&req);
    assert_eq!(req.borrow().state, RequestState::Received);
    assert_eq!(scheduler.statistics.number_of_active_requests, 0);
    assert_eq!(
        scheduler.number_of_active_requests_by_server("test.invalid:443"),
        0
    );
}

#[test]
fn test_cancels_issued_and_active_request() {
    let mut scheduler = RequestScheduler::new();
    let cancelled_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = cancelled_flag.clone();

    let req = Rc::new(RefCell::new(Request::new(RequestOptions {
        url: Some("https://test.invalid/1".to_string()),
        throttle: true,
        cancel_function: Some(Rc::new(move || {
            flag_clone.store(true, Ordering::SeqCst);
        })),
        ..Default::default()
    })));

    assert!(scheduler.request(&req).unwrap());
    assert_eq!(req.borrow().state, RequestState::Issued);

    // Cancel while issued
    req.borrow_mut().cancel();
    scheduler.update();
    assert_eq!(req.borrow().state, RequestState::Cancelled);
    assert_eq!(scheduler.statistics.number_of_cancelled_requests, 1);
    assert_eq!(
        scheduler.statistics.number_of_cancelled_active_requests,
        0
    );
    assert!(cancelled_flag.load(Ordering::SeqCst));

    // Test cancelling while active
    let req2 = Rc::new(RefCell::new(Request::new(RequestOptions {
        url: Some("https://test.invalid/2".to_string()),
        throttle: true,
        ..Default::default()
    })));
    assert!(scheduler.request(&req2).unwrap());
    scheduler.update();
    assert_eq!(req2.borrow().state, RequestState::Active);

    req2.borrow_mut().cancel();
    scheduler.update();
    assert_eq!(req2.borrow().state, RequestState::Cancelled);
    assert_eq!(scheduler.statistics.number_of_cancelled_requests, 2);
    assert_eq!(
        scheduler.statistics.number_of_cancelled_active_requests,
        1
    );
    assert_eq!(
        scheduler.number_of_active_requests_by_server("test.invalid:443"),
        0
    );
}

#[test]
fn test_prioritizes_requests() {
    let mut scheduler = RequestScheduler::new();
    scheduler.maximum_requests = 1;

    let r_low = Rc::new(RefCell::new(Request::new(RequestOptions {
        url: Some("https://test.invalid/low".to_string()),
        priority: 10.0,
        throttle: true,
        ..Default::default()
    })));
    let r_high = Rc::new(RefCell::new(Request::new(RequestOptions {
        url: Some("https://test.invalid/high".to_string()),
        priority: 1.0,
        throttle: true,
        ..Default::default()
    })));

    scheduler.request(&r_low).unwrap();
    scheduler.request(&r_high).unwrap();

    // Since maximum_requests is 1, the highest priority (lowest numerical value) should become active first
    scheduler.update();
    assert_eq!(r_high.borrow().state, RequestState::Active);
    assert_eq!(r_low.borrow().state, RequestState::Issued);
}

#[test]
fn test_honors_priority_heap_length_and_dynamic_priority() {
    let mut scheduler = RequestScheduler::new();
    scheduler.set_priority_heap_length(1);

    let first_request = Rc::new(RefCell::new(Request::new(RequestOptions {
        url: Some("http://test.invalid/1".to_string()),
        priority: 0.0,
        throttle: true,
        ..Default::default()
    })));

    assert!(scheduler.request(&first_request).unwrap());
    assert_eq!(first_request.borrow().state, RequestState::Issued);

    let second_request = Rc::new(RefCell::new(Request::new(RequestOptions {
        url: Some("http://test.invalid/1".to_string()),
        priority: 1.0,
        throttle: true,
        ..Default::default()
    })));
    // Heap length is 1, lower priority request is rejected
    assert!(!scheduler.request(&second_request).unwrap());

    scheduler.set_priority_heap_length(3);
    let r3 = Rc::new(RefCell::new(Request::new(RequestOptions {
        url: Some("http://test.invalid/1".to_string()),
        priority: 2.0,
        throttle: true,
        ..Default::default()
    })));
    let r4 = Rc::new(RefCell::new(Request::new(RequestOptions {
        url: Some("http://test.invalid/1".to_string()),
        priority: 3.0,
        throttle: true,
        ..Default::default()
    })));
    let r5 = Rc::new(RefCell::new(Request::new(RequestOptions {
        url: Some("http://test.invalid/1".to_string()),
        priority: 4.0,
        throttle: true,
        ..Default::default()
    })));

    assert!(scheduler.request(&r3).unwrap());
    assert!(scheduler.request(&r4).unwrap());
    assert!(!scheduler.request(&r5).unwrap());

    // Shrinking heap from 3 to 2 cancels the request popped from the top of the heap (first_request with 0.0)
    scheduler.set_priority_heap_length(2);
    assert_eq!(first_request.borrow().state, RequestState::Cancelled);
}

#[test]
fn test_custom_requests_by_server_and_open_slots() {
    let mut scheduler = RequestScheduler::new();
    scheduler
        .requests_by_server
        .insert("custom.server:80".to_string(), 4);

    assert!(scheduler.server_has_open_slots("custom.server:80", 4));
    assert!(!scheduler.server_has_open_slots("custom.server:80", 5));

    assert!(scheduler.heap_has_open_slots(20));
    assert!(!scheduler.heap_has_open_slots(21));
}

#[test]
fn test_failed_request_event() {
    let mut scheduler = RequestScheduler::new();
    let error_received = Rc::new(RefCell::new(None));
    let err_clone = error_received.clone();
    scheduler.request_completed_event.add_listener(move |err: &Option<String>| {
        *err_clone.borrow_mut() = err.clone();
    });

    let req = Rc::new(RefCell::new(Request::new(RequestOptions {
        url: Some("https://test.invalid/err".to_string()),
        throttle: false,
        ..Default::default()
    })));

    scheduler.request(&req).unwrap();
    assert_eq!(scheduler.statistics.number_of_active_requests, 1);

    scheduler.fail_request(&req, "Network timeout".to_string());
    assert_eq!(req.borrow().state, RequestState::Failed);
    assert_eq!(scheduler.statistics.number_of_failed_requests, 1);
    assert_eq!(scheduler.statistics.number_of_active_requests, 0);
    assert_eq!(
        *error_received.borrow(),
        Some("Network timeout".to_string())
    );
}

#[test]
fn test_updates_priority() {
    let mut scheduler = RequestScheduler::new();
    let length = scheduler.priority_heap_length();
    let invert_priority = Rc::new(RefCell::new(false));

    let mut requests = Vec::new();
    for i in 0..length {
        let priority = (i as f64) / ((length - 1) as f64);
        let invert = invert_priority.clone();
        let p_fn = Rc::new(move || {
            if *invert.borrow() {
                1.0 - priority
            } else {
                priority
            }
        });
        let req = Rc::new(RefCell::new(Request::new(RequestOptions {
            url: Some("https://test.invalid/1".to_string()),
            throttle: true,
            priority_function: Some(p_fn),
            ..Default::default()
        })));
        scheduler.request(&req).unwrap();
        requests.push(req);
    }

    scheduler.maximum_requests = 0;
    scheduler.update();

    let mut current_priority = -1.0;
    while let Some(req) = scheduler.request_heap_mut().pop(0) {
        let p = req.borrow().priority;
        assert!(p >= current_priority);
        current_priority = p;
    }

    for req in &requests {
        scheduler.request_heap_mut().insert(req.clone());
    }
    *invert_priority.borrow_mut() = true;
    scheduler.update();

    let mut current_inverted_p = -1.0;
    while let Some(req) = scheduler.request_heap_mut().pop(0) {
        let p = req.borrow().priority;
        assert!(p >= current_inverted_p);
        current_inverted_p = p;
    }
}

#[test]
fn test_request_throttled_by_server_is_cancelled() {
    let mut scheduler = RequestScheduler::new();
    scheduler.maximum_requests_per_server = 2;

    let create_request = |throttle_by_server: bool| {
        Rc::new(RefCell::new(Request::new(RequestOptions {
            url: Some("http://test.invalid/1".to_string()),
            throttle: throttle_by_server,
            throttle_by_server,
            ..Default::default()
        })))
    };

    let r1 = create_request(false);
    scheduler.request(&r1).unwrap();
    assert_eq!(r1.borrow().state, RequestState::Active);

    let throttled_req = create_request(true);
    scheduler.request(&throttled_req).unwrap();
    assert_eq!(throttled_req.borrow().state, RequestState::Issued);

    let r2 = create_request(false);
    scheduler.request(&r2).unwrap();
    assert_eq!(r2.borrow().state, RequestState::Active);

    scheduler.update();
    assert_eq!(throttled_req.borrow().state, RequestState::Cancelled);
}
