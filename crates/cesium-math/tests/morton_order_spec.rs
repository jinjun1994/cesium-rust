use cesium_math::MortonOrder;

#[test]
fn test_morton_order_encode_2d_throws_out_of_range() {
    assert!(MortonOrder::encode_2d(65536, 0).is_err());
    assert!(MortonOrder::encode_2d(0, 65536).is_err());
}

#[test]
fn test_morton_order_encode_2d_works() {
    assert_eq!(MortonOrder::encode_2d(0, 0).unwrap(), 0);
    assert_eq!(MortonOrder::encode_2d(1, 0).unwrap(), 1);
    assert_eq!(MortonOrder::encode_2d(0, 1).unwrap(), 2);
    assert_eq!(MortonOrder::encode_2d(1, 1).unwrap(), 3);
    assert_eq!(MortonOrder::encode_2d(2, 0).unwrap(), 4);
    assert_eq!(MortonOrder::encode_2d(3, 0).unwrap(), 5);
    assert_eq!(MortonOrder::encode_2d(2, 1).unwrap(), 6);
    assert_eq!(MortonOrder::encode_2d(3, 1).unwrap(), 7);

    assert_eq!(MortonOrder::encode_2d(7, 5).unwrap(), 55);

    assert_eq!(MortonOrder::encode_2d(65535, 65535).unwrap(), 4294967295);
    assert_eq!(MortonOrder::encode_2d(65535, 0).unwrap(), 1431655765);
    assert_eq!(MortonOrder::encode_2d(0, 65535).unwrap(), 2863311530);
}

#[test]
fn test_morton_order_decode_2d_works() {
    assert_eq!(MortonOrder::decode_2d(0), [0, 0]);
    assert_eq!(MortonOrder::decode_2d(1), [1, 0]);
    assert_eq!(MortonOrder::decode_2d(2), [0, 1]);
    assert_eq!(MortonOrder::decode_2d(3), [1, 1]);
    assert_eq!(MortonOrder::decode_2d(4), [2, 0]);
    assert_eq!(MortonOrder::decode_2d(5), [3, 0]);
    assert_eq!(MortonOrder::decode_2d(6), [2, 1]);
    assert_eq!(MortonOrder::decode_2d(7), [3, 1]);

    assert_eq!(MortonOrder::decode_2d(55), [7, 5]);

    assert_eq!(MortonOrder::decode_2d(4294967295), [65535, 65535]);
    assert_eq!(MortonOrder::decode_2d(1431655765), [65535, 0]);
    assert_eq!(MortonOrder::decode_2d(2863311530), [0, 65535]);
}

#[test]
fn test_morton_order_encode_3d_throws_out_of_range() {
    assert!(MortonOrder::encode_3d(1024, 0, 0).is_err());
    assert!(MortonOrder::encode_3d(0, 1024, 0).is_err());
    assert!(MortonOrder::encode_3d(0, 0, 1024).is_err());
}

#[test]
fn test_morton_order_encode_3d_works() {
    assert_eq!(MortonOrder::encode_3d(0, 0, 0).unwrap(), 0);
    assert_eq!(MortonOrder::encode_3d(1, 0, 0).unwrap(), 1);
    assert_eq!(MortonOrder::encode_3d(0, 1, 0).unwrap(), 2);
    assert_eq!(MortonOrder::encode_3d(1, 1, 0).unwrap(), 3);
    assert_eq!(MortonOrder::encode_3d(0, 0, 1).unwrap(), 4);
    assert_eq!(MortonOrder::encode_3d(1, 0, 1).unwrap(), 5);
    assert_eq!(MortonOrder::encode_3d(0, 1, 1).unwrap(), 6);
    assert_eq!(MortonOrder::encode_3d(1, 1, 1).unwrap(), 7);

    assert_eq!(MortonOrder::encode_3d(1, 3, 3).unwrap(), 55);

    assert_eq!(MortonOrder::encode_3d(1023, 1023, 1023).unwrap(), 1073741823);
    assert_eq!(MortonOrder::encode_3d(1023, 0, 0).unwrap(), 153391689);
    assert_eq!(MortonOrder::encode_3d(0, 1023, 0).unwrap(), 306783378);
    assert_eq!(MortonOrder::encode_3d(0, 0, 1023).unwrap(), 613566756);
}

#[test]
fn test_morton_order_decode_3d_throws_out_of_range() {
    assert!(MortonOrder::decode_3d(1073741824).is_err());
}

#[test]
fn test_morton_order_decode_3d_works() {
    assert_eq!(MortonOrder::decode_3d(0).unwrap(), [0, 0, 0]);
    assert_eq!(MortonOrder::decode_3d(1).unwrap(), [1, 0, 0]);
    assert_eq!(MortonOrder::decode_3d(2).unwrap(), [0, 1, 0]);
    assert_eq!(MortonOrder::decode_3d(3).unwrap(), [1, 1, 0]);
    assert_eq!(MortonOrder::decode_3d(4).unwrap(), [0, 0, 1]);
    assert_eq!(MortonOrder::decode_3d(5).unwrap(), [1, 0, 1]);
    assert_eq!(MortonOrder::decode_3d(6).unwrap(), [0, 1, 1]);
    assert_eq!(MortonOrder::decode_3d(7).unwrap(), [1, 1, 1]);

    assert_eq!(MortonOrder::decode_3d(55).unwrap(), [1, 3, 3]);

    assert_eq!(MortonOrder::decode_3d(1073741823).unwrap(), [1023, 1023, 1023]);
    assert_eq!(MortonOrder::decode_3d(153391689).unwrap(), [1023, 0, 0]);
    assert_eq!(MortonOrder::decode_3d(306783378).unwrap(), [0, 1023, 0]);
    assert_eq!(MortonOrder::decode_3d(613566756).unwrap(), [0, 0, 1023]);
}
