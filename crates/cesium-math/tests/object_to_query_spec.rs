use cesium_math::{object_to_query, query_to_object};

#[test]
fn test_can_encode_data() {
    let pairs = [("key1", &["some value"][..]), ("key2", &["a/b"][..])];
    let str = object_to_query(&pairs);
    assert_eq!(str, "key1=some%20value&key2=a%2Fb");
}

#[test]
fn test_can_encode_arrays_of_data() {
    let pairs = [("key", &["a", "b"][..])];
    let str = object_to_query(&pairs);
    assert_eq!(str, "key=a&key=b");
}

#[test]
fn test_runs_example_code() {
    let pairs = [
        ("key1", &["some value"][..]),
        ("key2", &["a/b"][..]),
        ("key3", &["x", "y"][..]),
    ];
    let str = object_to_query(&pairs);
    assert_eq!(str, "key1=some%20value&key2=a%2Fb&key3=x&key3=y");
}

#[test]
fn test_can_decode_data() {
    let str = "key1=some%20value&key2=a%2Fb";
    let obj = query_to_object(str);
    assert_eq!(obj.get("key1").unwrap(), &vec!["some value".to_string()]);
    assert_eq!(obj.get("key2").unwrap(), &vec!["a/b".to_string()]);

    let str2 = "q=query+string";
    let obj2 = query_to_object(str2);
    assert_eq!(obj2.get("q").unwrap(), &vec!["query string".to_string()]);
}

#[test]
fn test_can_decode_arrays_of_data() {
    let str = "key=a&key=b";
    let obj = query_to_object(str);
    assert_eq!(
        obj.get("key").unwrap(),
        &vec!["a".to_string(), "b".to_string()]
    );
}

#[test]
fn test_can_use_semicolon_instead_of_ampersand() {
    let str = "key=a;key=b;key2=c";
    let obj = query_to_object(str);
    assert_eq!(
        obj.get("key").unwrap(),
        &vec!["a".to_string(), "b".to_string()]
    );
    assert_eq!(obj.get("key2").unwrap(), &vec!["c".to_string()]);
}

#[test]
fn test_can_encode_and_decode_blank() {
    assert_eq!(object_to_query(&[]), "");
    assert!(query_to_object("").is_empty());
}
