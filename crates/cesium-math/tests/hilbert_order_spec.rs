use cesium_math::HilbertOrder;

#[test]
fn test_encode2d_throws_for_invalid_level() {
    assert!(HilbertOrder::encode_2d(0, 0, 0).is_err());
}

#[test]
fn test_encode2d_throws_for_invalid_coordinates() {
    assert!(HilbertOrder::encode_2d(1, 2, 0).is_err());
    assert!(HilbertOrder::encode_2d(1, 0, 2).is_err());
    assert!(HilbertOrder::encode_2d(1, 2, 2).is_err());
}

#[test]
fn test_encode2d_works() {
    assert_eq!(HilbertOrder::encode_2d(1, 0, 0).unwrap(), 0);
    assert_eq!(HilbertOrder::encode_2d(1, 0, 1).unwrap(), 1);
    assert_eq!(HilbertOrder::encode_2d(1, 1, 1).unwrap(), 2);
    assert_eq!(HilbertOrder::encode_2d(1, 1, 0).unwrap(), 3);

    assert_eq!(HilbertOrder::encode_2d(2, 0, 0).unwrap(), 0);
    assert_eq!(HilbertOrder::encode_2d(2, 1, 0).unwrap(), 1);
    assert_eq!(HilbertOrder::encode_2d(2, 1, 1).unwrap(), 2);
    assert_eq!(HilbertOrder::encode_2d(2, 0, 1).unwrap(), 3);
    assert_eq!(HilbertOrder::encode_2d(2, 0, 2).unwrap(), 4);
    assert_eq!(HilbertOrder::encode_2d(2, 0, 3).unwrap(), 5);
    assert_eq!(HilbertOrder::encode_2d(2, 1, 3).unwrap(), 6);
    assert_eq!(HilbertOrder::encode_2d(2, 1, 2).unwrap(), 7);
    assert_eq!(HilbertOrder::encode_2d(2, 2, 2).unwrap(), 8);
    assert_eq!(HilbertOrder::encode_2d(2, 2, 3).unwrap(), 9);
    assert_eq!(HilbertOrder::encode_2d(2, 3, 3).unwrap(), 10);
    assert_eq!(HilbertOrder::encode_2d(2, 3, 2).unwrap(), 11);
    assert_eq!(HilbertOrder::encode_2d(2, 3, 1).unwrap(), 12);
    assert_eq!(HilbertOrder::encode_2d(2, 2, 1).unwrap(), 13);
    assert_eq!(HilbertOrder::encode_2d(2, 2, 0).unwrap(), 14);
    assert_eq!(HilbertOrder::encode_2d(2, 3, 0).unwrap(), 15);
}

#[test]
fn test_decode2d_throws_for_invalid_level() {
    assert!(HilbertOrder::decode_2d(0, 0).is_err());
}

#[test]
fn test_decode2d_throws_for_invalid_index() {
    assert!(HilbertOrder::decode_2d(1, 4).is_err());
}

#[test]
fn test_decode2d_works() {
    assert_eq!(HilbertOrder::decode_2d(1, 0).unwrap(), (0, 0));
    assert_eq!(HilbertOrder::decode_2d(1, 1).unwrap(), (0, 1));
    assert_eq!(HilbertOrder::decode_2d(1, 2).unwrap(), (1, 1));
    assert_eq!(HilbertOrder::decode_2d(1, 3).unwrap(), (1, 0));

    assert_eq!(HilbertOrder::decode_2d(2, 0).unwrap(), (0, 0));
    assert_eq!(HilbertOrder::decode_2d(2, 1).unwrap(), (1, 0));
    assert_eq!(HilbertOrder::decode_2d(2, 2).unwrap(), (1, 1));
    assert_eq!(HilbertOrder::decode_2d(2, 3).unwrap(), (0, 1));
    assert_eq!(HilbertOrder::decode_2d(2, 4).unwrap(), (0, 2));
    assert_eq!(HilbertOrder::decode_2d(2, 5).unwrap(), (0, 3));
    assert_eq!(HilbertOrder::decode_2d(2, 6).unwrap(), (1, 3));
    assert_eq!(HilbertOrder::decode_2d(2, 7).unwrap(), (1, 2));
    assert_eq!(HilbertOrder::decode_2d(2, 8).unwrap(), (2, 2));
    assert_eq!(HilbertOrder::decode_2d(2, 9).unwrap(), (2, 3));
    assert_eq!(HilbertOrder::decode_2d(2, 10).unwrap(), (3, 3));
    assert_eq!(HilbertOrder::decode_2d(2, 11).unwrap(), (3, 2));
    assert_eq!(HilbertOrder::decode_2d(2, 12).unwrap(), (3, 1));
    assert_eq!(HilbertOrder::decode_2d(2, 13).unwrap(), (2, 1));
    assert_eq!(HilbertOrder::decode_2d(2, 14).unwrap(), (2, 0));
    assert_eq!(HilbertOrder::decode_2d(2, 15).unwrap(), (3, 0));
}
