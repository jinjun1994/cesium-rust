use std::f64::consts::TAU;

use cesium_math::{JulianDate, Rotation, Sampleable, SampledProperty};

#[test]
fn pack_and_unpack_works() {
    let rotation = Rotation(1.23);
    let mut packed = vec![0.0; Rotation::PACKED_LENGTH];
    rotation.pack(&mut packed);
    assert_eq!(packed[0], 1.23);

    let unpacked = Rotation::unpack(&packed);
    assert_eq!(unpacked, rotation);
}

#[test]
fn interpolates_towards_the_closest_angle() {
    let time1 = JulianDate::from_iso8601("2010-05-07T00:00:00Z").unwrap();
    let time2 = JulianDate::from_iso8601("2010-05-07T00:01:00Z").unwrap();
    let time3 = JulianDate::from_iso8601("2010-05-07T00:02:00Z").unwrap();
    let time4 = JulianDate::from_iso8601("2010-05-07T00:03:00Z").unwrap();
    let time5 = JulianDate::from_iso8601("2010-05-07T00:04:00Z").unwrap();

    let mut property = SampledProperty::<Rotation>::new();
    property.add_sample(time1, Rotation(0.0));
    property.add_sample(time3, Rotation(350.0_f64.to_radians()));
    property.add_sample(time5, Rotation(10.0_f64.to_radians()));

    let val2 = property.get_value(Some(&time2)).unwrap();
    let expected2 = 355.0_f64.to_radians();
    assert!((val2.0 - expected2).abs() < 1e-10);

    let val4 = property.get_value(Some(&time4)).unwrap();
    let expected4 = 0.0;
    let diff = (val4.0 - expected4).abs() % TAU;
    assert!(diff < 1e-10 || (diff - TAU).abs() < 1e-10);
}
