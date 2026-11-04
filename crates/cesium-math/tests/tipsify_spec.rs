use cesium_math::Tipsify;

#[test]
fn test_can_calculate_the_acmr() {
    // Hexagon formed from 6 triangles, 7 vertices
    let indices = [
        0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 1, 6,
    ];
    let acmr = Tipsify::calculate_acmr(&indices, Some(6), Some(3)).unwrap();
    assert_eq!(acmr, 2.0);
}

#[test]
fn test_can_calculate_the_acmr_without_maximum_index() {
    let indices = [
        0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 1, 6,
    ];
    let acmr = Tipsify::calculate_acmr(&indices, None, Some(3)).unwrap();
    assert_eq!(acmr, 2.0);
}

#[test]
fn test_throws_when_calculating_acmr() {
    // not multiple of 3
    assert!(Tipsify::calculate_acmr(&[1, 2, 3, 4], Some(1), Some(3)).is_err());
    // maximum index == 0
    assert!(Tipsify::calculate_acmr(&[0, 1, 2], Some(0), Some(2)).is_err());
    // cache size < 3
    assert!(Tipsify::calculate_acmr(&[0, 1, 2], Some(2), Some(2)).is_err());
}

#[test]
fn test_throws_when_executing_tipsify() {
    // not multiple of 3
    assert!(Tipsify::tipsify(&[1, 2, 3, 4], Some(1), Some(3)).is_err());
    // maximum index == 0
    assert!(Tipsify::tipsify(&[0, 1, 2], Some(0), Some(3)).is_err());
    // cache size < 3
    assert!(Tipsify::tipsify(&[0, 1, 2], Some(2), Some(2)).is_err());
}

#[test]
fn test_can_lower_acmr_using_tipsify_algorithm() {
    let indices = [
        0, 1, 7, 1, 7, 8, 1, 2, 8, 2, 8, 9, 2, 3, 9, 3, 9, 10, 3, 4, 10, 4, 10,
        11, 4, 5, 11, 5, 11, 12, 6, 13, 14, 6, 7, 14, 7, 14, 15, 7, 8, 15, 8, 15,
        16, 8, 9, 16, 9, 16, 17, 9, 10, 17, 10, 17, 18, 10, 11, 18, 11, 18, 19,
        11, 12, 19, 12, 19, 20, 13, 21, 22, 13, 14, 22, 14, 22, 23, 14, 15, 23,
        15, 23, 24, 15, 16, 24, 16, 24, 25, 16, 17, 25, 17, 25, 26, 17, 18, 26,
        18, 26, 27, 18, 19, 27, 19, 27, 28, 19, 20, 28,
    ];

    let acmr_before = Tipsify::calculate_acmr(&indices, Some(28), Some(6)).unwrap();
    let result = Tipsify::tipsify(&indices, Some(28), Some(6)).unwrap();
    let acmr_after = Tipsify::calculate_acmr(&result, Some(28), Some(6)).unwrap();

    assert!(acmr_after < acmr_before, "expected {acmr_after} < {acmr_before}");
}

#[test]
fn test_can_tipsify_without_knowing_maximum_index() {
    let indices = [
        0, 1, 7, 1, 7, 8, 1, 2, 8, 2, 8, 9, 2, 3, 9, 3, 9, 10, 3, 4, 10, 4, 10,
        11, 4, 5, 11, 5, 11, 12, 6, 13, 14, 6, 7, 14, 7, 14, 15, 7, 8, 15, 8, 15,
        16, 8, 9, 16, 9, 16, 17, 9, 10, 17, 10, 17, 18, 10, 11, 18, 11, 18, 19,
        11, 12, 19, 12, 19, 20, 13, 21, 22, 13, 14, 22, 14, 22, 23, 14, 15, 23,
        15, 23, 24, 15, 16, 24, 16, 24, 25, 16, 17, 25, 17, 25, 26, 17, 18, 26,
        18, 26, 27, 18, 19, 27, 19, 27, 28, 19, 20, 28,
    ];

    let r1 = Tipsify::tipsify(&indices, None, Some(6)).unwrap();
    let r2 = Tipsify::tipsify(&indices, Some(28), Some(6)).unwrap();
    assert_eq!(r1, r2);
}
