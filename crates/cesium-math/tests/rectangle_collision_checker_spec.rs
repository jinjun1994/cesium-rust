use cesium_math::{GeoRectangle, RectangleCollisionChecker};

#[test]
fn test_rectangle_collision_checker_collides_and_removes() {
    let test_rectangle1 = GeoRectangle::new(0.0, 0.0, 1.0, 1.0);
    let test_rectangle2 = GeoRectangle::new(1.1, 1.1, 2.1, 2.1);
    let test_rectangle3 = GeoRectangle::new(1.1, 1.1, 1.2, 1.2);

    let mut checker = RectangleCollisionChecker::new();
    checker.insert("test1", &test_rectangle1);

    assert!(!checker.collides(&test_rectangle2));

    checker.insert("test3", &test_rectangle3);
    assert!(checker.collides(&test_rectangle2));

    checker.remove("test3", &test_rectangle3);
    assert!(!checker.collides(&test_rectangle2));
}
