#![allow(clippy::assertions_on_constants)]

use cesium_math::VertexFormat;

#[test]
fn test_vertex_format_clone_and_constants() {
    let vf = VertexFormat {
        position: true,
        normal: true,
        st: false,
        tangent: false,
        bitangent: false,
        color: false,
    };
    let cloned = vf;
    assert_eq!(cloned, vf);
    assert_eq!(vf, VertexFormat::POSITION_AND_NORMAL);

    assert!(VertexFormat::POSITION_ONLY.position);
    assert!(!VertexFormat::POSITION_ONLY.normal);

    assert!(VertexFormat::ALL.position);
    assert!(VertexFormat::ALL.normal);
    assert!(VertexFormat::ALL.st);
    assert!(VertexFormat::ALL.tangent);
    assert!(VertexFormat::ALL.bitangent);
}

#[test]
fn test_vertex_format_pack_unpack() {
    let vf = VertexFormat::POSITION_AND_NORMAL;
    let mut packed = [0.0; 6];
    vf.pack(&mut packed, 0).unwrap();
    assert_eq!(packed, [1.0, 1.0, 0.0, 0.0, 0.0, 0.0]);

    let unpacked = VertexFormat::unpack(&packed, 0).unwrap();
    assert_eq!(unpacked, vf);
}
