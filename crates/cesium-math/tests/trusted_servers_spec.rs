use cesium_math::TrustedServers;

#[test]
fn test_arguments_validation() {
    let mut ts = TrustedServers::new();
    assert!(ts.add("", 80).is_err());
    assert!(ts.add("host", 0).is_err());
    assert!(ts.remove("", 80).is_err());
    assert!(ts.remove("host", 0).is_err());
    assert!(ts.contains("").is_err());
}

#[test]
fn test_http_without_a_port() {
    let mut ts = TrustedServers::new();
    ts.add("cesiumjs.org", 80).unwrap();
    assert!(ts.contains("http://cesiumjs.org/index.html").unwrap());
    assert!(!ts.contains("https://cesiumjs.org/index.html").unwrap());
}

#[test]
fn test_https_without_a_port() {
    let mut ts = TrustedServers::new();
    ts.add("cesiumjs.org", 443).unwrap();
    assert!(ts.contains("https://cesiumjs.org/index.html").unwrap());
    assert!(!ts.contains("http://cesiumjs.org/index.html").unwrap());
}

#[test]
fn test_add_and_remove() {
    let mut ts = TrustedServers::new();
    assert!(!ts.contains("http://cesiumjs.org:81/index.html").unwrap());

    ts.add("cesiumjs.org", 81).unwrap();
    assert!(!ts.contains("http://cesiumjs.org/index.html").unwrap());
    assert!(ts.contains("http://cesiumjs.org:81/index.html").unwrap());

    ts.remove("cesiumjs.org", 8080).unwrap();
    assert!(ts.contains("http://cesiumjs.org:81/index.html").unwrap());

    ts.remove("cesiumjs.org", 81).unwrap();
    assert!(!ts.contains("http://cesiumjs.org:81/index.html").unwrap());
}

#[test]
fn test_handles_credentials() {
    let mut ts = TrustedServers::new();
    ts.add("cesiumjs.org", 81).unwrap();
    assert!(ts.contains("http://user:pass@cesiumjs.org:81/index.html").unwrap());
}

#[test]
fn test_returns_false_for_relative_paths() {
    let ts = TrustedServers::new();
    assert!(!ts.contains("./data/index.html").unwrap());
    assert!(!ts.contains("/data/index.html").unwrap());
}

#[test]
fn test_handles_protocol_relative_urls() {
    let mut ts = TrustedServers::new();
    ts.add("cesiumjs.org", 80).unwrap();
    assert!(ts.contains("//cesiumjs.org/index.html").unwrap());
}

#[test]
fn test_clear() {
    let mut ts = TrustedServers::new();
    ts.add("cesiumjs.org", 80).unwrap();
    assert!(ts.contains("http://cesiumjs.org/index.html").unwrap());

    ts.clear();
    assert!(!ts.contains("http://cesiumjs.org/index.html").unwrap());

    ts.add("cesiumjs.org", 80).unwrap();
    assert!(ts.contains("http://cesiumjs.org/index.html").unwrap());
}

#[test]
fn test_global_singleton_api() {
    TrustedServers::clear_global();
    assert!(!TrustedServers::contains_global("http://cesiumjs.org/index.html").unwrap());

    TrustedServers::add_global("cesiumjs.org", 80).unwrap();
    assert!(TrustedServers::contains_global("http://cesiumjs.org/index.html").unwrap());

    TrustedServers::clear_global();
    assert!(!TrustedServers::contains_global("http://cesiumjs.org/index.html").unwrap());
}
