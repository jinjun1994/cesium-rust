use cesium_math::create_guid;

#[test]
fn test_creates_guids() {
    let guid1 = create_guid();
    let guid2 = create_guid();
    let guid3 = create_guid();

    // Unique
    assert_ne!(guid1, guid2);
    assert_ne!(guid1, guid3);
    assert_ne!(guid2, guid3);

    // Length 36
    assert_eq!(guid1.len(), 36);
    assert_eq!(guid2.len(), 36);
    assert_eq!(guid3.len(), 36);

    // Format: 8-4-4-4-12
    let check_format = |g: &str| {
        let parts: Vec<&str> = g.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
        assert!(g.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
        // Version 4 check: parts[2] starts with '4'
        assert!(parts[2].starts_with('4'));
    };

    check_format(&guid1);
    check_format(&guid2);
    check_format(&guid3);
}
