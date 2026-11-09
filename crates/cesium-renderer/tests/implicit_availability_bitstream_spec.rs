use cesium_renderer::implicit_availability_bitstream::{
    ImplicitAvailabilityBitstream, ImplicitAvailabilityBitstreamOptions,
};

#[test]
fn throws_on_missing_length_bits() {
    let res = ImplicitAvailabilityBitstream::new(ImplicitAvailabilityBitstreamOptions {
        length_bits: None,
        ..Default::default()
    });
    assert!(res.is_err());
    assert!(res.unwrap_err().message.contains("lengthBits is required"));
}

#[test]
fn throws_on_mismatched_bit_length_and_bitstream_length() {
    let res = ImplicitAvailabilityBitstream::new(ImplicitAvailabilityBitstreamOptions {
        length_bits: Some(17),
        bitstream: Some(vec![0xff, 0x02]),
        ..Default::default()
    });
    assert!(res.is_err());
    assert!(res.unwrap_err().message.contains("must be exactly 3 bytes long"));
}

#[test]
fn reads_bits_from_constant() {
    let length = 21;
    let bitstream = ImplicitAvailabilityBitstream::new(ImplicitAvailabilityBitstreamOptions {
        length_bits: Some(length),
        constant: Some(true),
        ..Default::default()
    })
    .expect("constant bitstream");

    for i in 0..length {
        assert!(bitstream.get_bit(i).unwrap());
    }
}

#[test]
fn reads_bits_from_bitstream() {
    let bitstream_u8 = vec![0xfa, 0x01];
    let expected = [false, true, false, true, true, true, true, true, true];
    let bitstream = ImplicitAvailabilityBitstream::new(ImplicitAvailabilityBitstreamOptions {
        length_bits: Some(expected.len()),
        bitstream: Some(bitstream_u8),
        ..Default::default()
    })
    .expect("valid bitstream");

    for (i, &exp) in expected.iter().enumerate() {
        assert_eq!(bitstream.get_bit(i).unwrap(), exp);
    }
}

#[test]
fn throws_on_out_of_bounds() {
    let bitstream = ImplicitAvailabilityBitstream::new(ImplicitAvailabilityBitstreamOptions {
        length_bits: Some(10),
        bitstream: Some(vec![0xff, 0x02]),
        ..Default::default()
    })
    .unwrap();

    assert!(bitstream.get_bit(10).is_err());
    assert!(bitstream.get_bit(15).is_err());
}

#[test]
fn stores_available_count() {
    let bitstream = ImplicitAvailabilityBitstream::new(ImplicitAvailabilityBitstreamOptions {
        length_bits: Some(10),
        available_count: Some(3),
        bitstream: Some(vec![0x07, 0x00]),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(bitstream.available_count(), Some(3));
}

#[test]
fn computes_available_count_if_enabled_and_available_count_is_undefined() {
    let bitstream = ImplicitAvailabilityBitstream::new(ImplicitAvailabilityBitstreamOptions {
        length_bits: Some(10),
        bitstream: Some(vec![0xff, 0x02]),
        compute_available_count_enabled: true,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(bitstream.available_count(), Some(9));
}

#[test]
fn does_not_compute_available_count_if_disabled_and_available_count_is_undefined() {
    let bitstream = ImplicitAvailabilityBitstream::new(ImplicitAvailabilityBitstreamOptions {
        length_bits: Some(10),
        bitstream: Some(vec![0xff, 0x02]),
        compute_available_count_enabled: false,
        ..Default::default()
    })
    .unwrap();
    assert!(bitstream.available_count().is_none());
}
