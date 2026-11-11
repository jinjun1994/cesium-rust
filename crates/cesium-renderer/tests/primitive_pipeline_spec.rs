use cesium_math::{
    BoundingSphere, BoxGeometry, Cartesian3, ComponentDatatype, Geometry, GeometryAttribute,
    GeometryAttributes, PrimitiveType,
};
use cesium_renderer::primitive_pipeline::PrimitivePipeline;
use glam::DVec3;

#[test]
fn test_primitive_pipeline_can_pack_and_unpack_geometry() {
    let box1 = BoxGeometry::from_dimensions(Cartesian3::new(1.0, 2.0, 3.0), None);
    let box_geom1 = BoxGeometry::create_geometry(&box1).unwrap();

    let box2 = BoxGeometry::from_dimensions(Cartesian3::new(3.0, 4.0, 7.0), None);
    let box_geom2 = BoxGeometry::create_geometry(&box2).unwrap();

    let geometry_to_pack = vec![Some(box_geom1), Some(box_geom2)];
    let mut transferable_objects = Vec::new();

    let results = PrimitivePipeline::pack_create_geometry_results(
        &geometry_to_pack,
        &mut transferable_objects,
    );
    let unpacked_geometry = PrimitivePipeline::unpack_create_geometry_results(&results);

    assert_eq!(transferable_objects.len(), 1);
    assert_eq!(geometry_to_pack, unpacked_geometry);
}

#[test]
fn test_primitive_pipeline_can_pack_and_unpack_geometry_without_indices() {
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        )),
        ..Default::default()
    };

    let p1 = DVec3::new(1.0, 2.0, 3.0);
    let p2 = DVec3::new(4.0, 5.0, 6.0);
    let bounding_sphere = BoundingSphere::from_points(&[p1, p2]).ok();

    let geometry = Geometry::new(
        attributes,
        Vec::new(),
        PrimitiveType::Points,
        bounding_sphere,
    );

    let geometry_to_pack = vec![Some(geometry)];
    let mut transferable_objects = Vec::new();

    let results = PrimitivePipeline::pack_create_geometry_results(
        &geometry_to_pack,
        &mut transferable_objects,
    );
    let unpacked_geometry = PrimitivePipeline::unpack_create_geometry_results(&results);

    assert_eq!(transferable_objects.len(), 1);
    assert_eq!(geometry_to_pack, unpacked_geometry);
}
