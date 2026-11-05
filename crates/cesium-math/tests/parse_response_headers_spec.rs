use cesium_math::parse_response_headers;
use std::collections::HashMap;

#[test]
fn test_returns_empty_map_when_given_falsy_input() {
    assert_eq!(parse_response_headers(()), HashMap::new());
    assert_eq!(parse_response_headers(None), HashMap::new());
    assert_eq!(parse_response_headers(""), HashMap::new());
}

#[test]
fn test_correctly_parses_response_headers() {
    let header_string = "Date: Sun, 24 Oct 2004 04:58:38 GMT\r\n\
         Server: Apache/1.3.31 (Unix)\r\n\
         Keep-Alive: timeout=15, max=99\r\n\
         Connection: Keep-Alive\r\n\
         Transfer-Encoding: chunked\r\n\
         Content-Type: text/plain; charset=utf-8";

    let headers = parse_response_headers(header_string);

    assert_eq!(headers.get("Date").map(|s| s.as_str()), Some("Sun, 24 Oct 2004 04:58:38 GMT"));
    assert_eq!(headers.get("Server").map(|s| s.as_str()), Some("Apache/1.3.31 (Unix)"));
    assert_eq!(headers.get("Keep-Alive").map(|s| s.as_str()), Some("timeout=15, max=99"));
    assert_eq!(headers.get("Connection").map(|s| s.as_str()), Some("Keep-Alive"));
    assert_eq!(headers.get("Transfer-Encoding").map(|s| s.as_str()), Some("chunked"));
    assert_eq!(headers.get("Content-Type").map(|s| s.as_str()), Some("text/plain; charset=utf-8"));
    assert_eq!(headers.len(), 6);
}
