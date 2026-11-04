use cesium_math::{
    BoundingSphere, Cartesian3, ComponentDatatype,
    EncodedCartesian3, Geometry, GeometryAttribute,
    GeometryAttributes, GeometryInstance, GeometryPipeline, Matrix4, PrimitiveType,
};

#[test]
fn test_to_wireframe_triangles() {
    let mut geometry = Geometry::new(
        GeometryAttributes::default(),
        vec![0, 1, 2, 3, 4, 5],
        PrimitiveType::Triangles,
        None,
    );

    GeometryPipeline::to_wireframe(&mut geometry).unwrap();

    assert_eq!(geometry.primitive_type, PrimitiveType::Lines);
    let v = &geometry.indices;
    assert_eq!(v.len(), 12);
    assert_eq!(v[0], 0);
    assert_eq!(v[1], 1);
    assert_eq!(v[2], 1);
    assert_eq!(v[3], 2);
    assert_eq!(v[4], 2);
    assert_eq!(v[5], 0);

    assert_eq!(v[6], 3);
    assert_eq!(v[7], 4);
    assert_eq!(v[8], 4);
    assert_eq!(v[9], 5);
    assert_eq!(v[10], 5);
    assert_eq!(v[11], 3);
}

#[test]
fn test_to_wireframe_triangle_fan() {
    let mut geometry = Geometry::new(
        GeometryAttributes::default(),
        vec![0, 1, 2, 3],
        PrimitiveType::TriangleFan,
        None,
    );

    GeometryPipeline::to_wireframe(&mut geometry).unwrap();

    assert_eq!(geometry.primitive_type, PrimitiveType::Lines);
    let v = &geometry.indices;
    assert_eq!(v.len(), 12);
    assert_eq!(v[0], 0);
    assert_eq!(v[1], 1);
    assert_eq!(v[2], 1);
    assert_eq!(v[3], 2);
    assert_eq!(v[4], 2);
    assert_eq!(v[5], 0);

    assert_eq!(v[6], 0);
    assert_eq!(v[7], 2);
    assert_eq!(v[8], 2);
    assert_eq!(v[9], 3);
    assert_eq!(v[10], 3);
    assert_eq!(v[11], 0);
}

#[test]
fn test_to_wireframe_triangle_strip() {
    let mut geometry = Geometry::new(
        GeometryAttributes::default(),
        vec![0, 1, 2, 3],
        PrimitiveType::TriangleStrip,
        None,
    );

    GeometryPipeline::to_wireframe(&mut geometry).unwrap();

    assert_eq!(geometry.primitive_type, PrimitiveType::Lines);
    let v = &geometry.indices;
    assert_eq!(v.len(), 12);
    assert_eq!(v[0], 0);
    assert_eq!(v[1], 1);
    assert_eq!(v[2], 1);
    assert_eq!(v[3], 2);
    assert_eq!(v[4], 2);
    assert_eq!(v[5], 0);

    assert_eq!(v[6], 2);
    assert_eq!(v[7], 3);
    assert_eq!(v[8], 3);
    assert_eq!(v[9], 1);
    assert_eq!(v[10], 1);
    assert_eq!(v[11], 2);
}

#[test]
fn test_to_wireframe_throws_when_not_triangle_type() {
    let mut geometry = Geometry::new(
        GeometryAttributes::default(),
        vec![0, 1],
        PrimitiveType::Lines,
        None,
    );

    assert!(GeometryPipeline::to_wireframe(&mut geometry).is_err());
}

#[test]
fn test_create_line_segments_for_vectors() {
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        )),
        normal: Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
        )),
        ..Default::default()
    };

    let bs = BoundingSphere::new(Cartesian3::ZERO, 1.0);
    let geometry = Geometry::new(attributes, vec![0, 1, 2], PrimitiveType::Triangles, Some(bs));

    let lines = GeometryPipeline::create_line_segments_for_vectors(
        &geometry,
        Some("normal"),
        Some(2.0),
    )
    .unwrap();

    assert_eq!(lines.primitive_type, PrimitiveType::Lines);
    let pos = &lines.attributes.position.as_ref().unwrap().values;
    assert_eq!(pos.len(), 18);
    // vertex 0: start and end
    assert_eq!(pos[0], 0.0);
    assert_eq!(pos[1], 0.0);
    assert_eq!(pos[2], 0.0);
    assert_eq!(pos[3], 0.0);
    assert_eq!(pos[4], 0.0);
    assert_eq!(pos[5], 2.0);

    assert_eq!(lines.bounding_sphere.unwrap().radius, 3.0);
}

#[test]
fn test_create_attribute_locations() {
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![0.0, 0.0, 0.0],
        )),
        normal: Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            vec![0.0, 0.0, 1.0],
        )),
        ..Default::default()
    };

    let geometry = Geometry::new(attributes, vec![], PrimitiveType::Triangles, None);
    let locations = GeometryPipeline::create_attribute_locations(&geometry);

    assert_eq!(locations.get("position"), Some(&0));
    assert_eq!(locations.get("normal"), Some(&1));
}

#[test]
fn test_reorder_for_pre_vertex_cache() {
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![
                0.0, 0.0, 0.0, // 0
                1.0, 0.0, 0.0, // 1
                2.0, 0.0, 0.0, // 2
                3.0, 0.0, 0.0, // 3 (unused)
            ],
        )),
        ..Default::default()
    };

    let mut geometry = Geometry::new(
        attributes,
        vec![2, 0, 1],
        PrimitiveType::Triangles,
        None,
    );

    GeometryPipeline::reorder_for_pre_vertex_cache(&mut geometry).unwrap();

    // 2 becomes 0, 0 becomes 1, 1 becomes 2
    assert_eq!(geometry.indices, vec![0, 1, 2]);
    let pos = &geometry.attributes.position.as_ref().unwrap().values;
    // Length is now 3 vertices (unused vertex 3 removed)
    assert_eq!(pos.len(), 9);
    assert_eq!(pos[0], 2.0);
    assert_eq!(pos[3], 0.0);
    assert_eq!(pos[6], 1.0);
}

#[test]
fn test_reorder_for_pre_vertex_cache_throws_different_vertex_count() {
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0], // 2 vertices
        )),
        normal: Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            vec![0.0, 0.0, 1.0], // 1 vertex
        )),
        ..Default::default()
    };

    let mut geometry = Geometry::new(
        attributes,
        vec![0, 1],
        PrimitiveType::Lines,
        None,
    );

    assert!(GeometryPipeline::reorder_for_pre_vertex_cache(&mut geometry).is_err());
}

#[test]
fn test_reorder_for_post_vertex_cache() {
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![
                0.0, 0.0, 0.0,
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
            ],
        )),
        ..Default::default()
    };

    let mut geometry = Geometry::new(
        attributes,
        vec![0, 1, 2],
        PrimitiveType::Triangles,
        None,
    );

    GeometryPipeline::reorder_for_post_vertex_cache(&mut geometry, Some(24));
    assert_eq!(geometry.indices.len(), 3);
}

#[test]
fn test_fit_to_unsigned_short_indices_small_geometry() {
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        )),
        ..Default::default()
    };

    let geometry = Geometry::new(
        attributes,
        vec![0, 1, 2],
        PrimitiveType::Triangles,
        None,
    );

    let res = GeometryPipeline::fit_to_unsigned_short_indices(&geometry).unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].indices, vec![0, 1, 2]);
}

#[test]
fn test_encode_attribute() {
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![1234567.89, 9876543.21, 555555.55],
        )),
        ..Default::default()
    };

    let mut geometry = Geometry::new(attributes, vec![], PrimitiveType::Points, None);

    GeometryPipeline::encode_attribute(
        &mut geometry,
        "position",
        "positionHigh",
        "positionLow",
    )
    .unwrap();

    assert!(geometry.attributes.position.is_none());
    let high = geometry.attributes.get("positionHigh").unwrap();
    let low = geometry.attributes.get("positionLow").unwrap();

    assert_eq!(high.values.len(), 3);
    assert_eq!(low.values.len(), 3);

    let (exp_high, exp_low) = EncodedCartesian3::encode(1234567.89);
    assert_eq!(high.values[0], exp_high);
    assert_eq!(low.values[0], exp_low);
}

#[test]
fn test_project_to_2d() {
    // Point on equator at lon 0
    let pt = Cartesian3::new(6378137.0, 0.0, 0.0);
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![pt.x, pt.y, pt.z],
        )),
        ..Default::default()
    };

    let mut geometry = Geometry::new(attributes, vec![], PrimitiveType::Points, None);
    GeometryPipeline::project_to_2d(
        &mut geometry,
        "position",
        "position3D",
        "position2D",
        None,
    )
    .unwrap();

    assert!(geometry.attributes.position.is_none());
    assert!(geometry.attributes.get("position3D").is_some());
    let pos2d = geometry.attributes.get("position2D").unwrap();
    assert_eq!(pos2d.values.len(), 3);
    // lon 0, lat 0 -> x=0, y=0, z=0 in GeographicProjection
    assert!(pos2d.values[0].abs() < 1e-4);
    assert!(pos2d.values[1].abs() < 1e-4);
    assert!(pos2d.values[2].abs() < 1e-4);
}

#[test]
fn test_transform_to_world_coordinates() {
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![1.0, 2.0, 3.0],
        )),
        normal: Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            vec![0.0, 0.0, 1.0],
        )),
        ..Default::default()
    };

    let bs = BoundingSphere::new(Cartesian3::new(1.0, 2.0, 3.0), 5.0);
    let geometry = Geometry::new(attributes, vec![], PrimitiveType::Points, Some(bs));

    // Translation matrix by (10, 20, 30)
    let model_matrix = Matrix4::from_translation(Cartesian3::new(10.0, 20.0, 30.0));
    let mut instance = GeometryInstance::new(geometry, Some(model_matrix), None, None);

    GeometryPipeline::transform_to_world_coordinates(&mut instance);

    assert_eq!(instance.model_matrix, Matrix4::IDENTITY);
    let pos = &instance.geometry.attributes.position.as_ref().unwrap().values;
    assert!((pos[0] - 11.0).abs() < 1e-6);
    assert!((pos[1] - 22.0).abs() < 1e-6);
    assert!((pos[2] - 33.0).abs() < 1e-6);

    let norm = &instance.geometry.attributes.normal.as_ref().unwrap().values;
    assert!((norm[0] - 0.0).abs() < 1e-6);
    assert!((norm[1] - 0.0).abs() < 1e-6);
    assert!((norm[2] - 1.0).abs() < 1e-6);

    let new_bs = instance.geometry.bounding_sphere.unwrap();
    assert!((new_bs.center.x - 11.0).abs() < 1e-6);
    assert!((new_bs.center.y - 22.0).abs() < 1e-6);
    assert!((new_bs.center.z - 33.0).abs() < 1e-6);
    assert!((new_bs.radius - 5.0).abs() < 1e-6);
}

#[test]
fn test_combine_instances() {
    let attrs1 = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        )),
        ..Default::default()
    };
    let geom1 = Geometry::new(
        attrs1,
        vec![0, 1, 2],
        PrimitiveType::Triangles,
        Some(BoundingSphere::new(Cartesian3::ZERO, 1.0)),
    );

    let attrs2 = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![2.0, 0.0, 0.0, 3.0, 0.0, 0.0, 2.0, 1.0, 0.0],
        )),
        ..Default::default()
    };
    let geom2 = Geometry::new(
        attrs2,
        vec![0, 1, 2],
        PrimitiveType::Triangles,
        Some(BoundingSphere::new(Cartesian3::new(2.5, 0.5, 0.0), 1.0)),
    );

    let inst1 = GeometryInstance::new(geom1, None, None, None);
    let inst2 = GeometryInstance::new(geom2, None, None, None);

    let combined = GeometryPipeline::combine_instances(&[inst1, inst2]).unwrap();
    assert_eq!(combined.len(), 1);
    let g = &combined[0];
    assert_eq!(g.primitive_type, PrimitiveType::Triangles);
    assert_eq!(g.attributes.position.as_ref().unwrap().values.len(), 18);
    // Indices are offset for second instance: [0, 1, 2, 3, 4, 5]
    assert_eq!(g.indices, vec![0, 1, 2, 3, 4, 5]);
    assert!(g.bounding_sphere.is_some());
}

#[test]
fn test_combine_instances_throws_on_mismatched_model_matrices() {
    let geom = Geometry::new(GeometryAttributes::default(), vec![], PrimitiveType::Triangles, None);
    let inst1 = GeometryInstance::new(geom.clone(), Some(Matrix4::IDENTITY), None, None);
    let inst2 = GeometryInstance::new(
        geom,
        Some(Matrix4::from_translation(Cartesian3::new(1.0, 0.0, 0.0))),
        None,
        None,
    );

    assert!(GeometryPipeline::combine_instances(&[inst1, inst2]).is_err());
}

#[test]
fn test_compute_normal() {
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![
                0.0, 0.0, 0.0,
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
            ],
        )),
        ..Default::default()
    };

    let mut geometry = Geometry::new(
        attributes,
        vec![0, 1, 2],
        PrimitiveType::Triangles,
        None,
    );

    GeometryPipeline::compute_normal(&mut geometry).unwrap();

    let normal = &geometry.attributes.normal.as_ref().unwrap().values;
    assert_eq!(normal.len(), 9);
    for chunk in normal.as_chunks::<3>().0 {
        assert!((chunk[0] - 0.0).abs() < 1e-6);
        assert!((chunk[1] - 0.0).abs() < 1e-6);
        assert!((chunk[2] - 1.0).abs() < 1e-6);
    }
}

#[test]
fn test_compute_tangent_and_bitangent() {
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![
                0.0, 0.0, 0.0,
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
            ],
        )),
        normal: Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            vec![
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
            ],
        )),
        st: Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            2,
            vec![
                0.0, 0.0,
                1.0, 0.0,
                0.0, 1.0,
            ],
        )),
        ..Default::default()
    };

    let mut geometry = Geometry::new(
        attributes,
        vec![0, 1, 2],
        PrimitiveType::Triangles,
        None,
    );

    GeometryPipeline::compute_tangent_and_bitangent(&mut geometry).unwrap();

    assert!(geometry.attributes.tangent.is_some());
    assert!(geometry.attributes.bitangent.is_some());
    let tan = &geometry.attributes.tangent.as_ref().unwrap().values;
    let bitan = &geometry.attributes.bitangent.as_ref().unwrap().values;
    assert_eq!(tan.len(), 9);
    assert_eq!(bitan.len(), 9);
    // Tangent points along +X, bitangent along +Y
    assert!((tan[0] - 1.0).abs() < 1e-6);
    assert!((bitan[1] - 1.0).abs() < 1e-6);
}

#[test]
fn test_compress_vertices() {
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![
                0.0, 0.0, 0.0,
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
            ],
        )),
        normal: Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            3,
            vec![
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
            ],
        )),
        st: Some(GeometryAttribute::new(
            ComponentDatatype::Float,
            2,
            vec![
                0.0, 0.0,
                1.0, 0.0,
                0.0, 1.0,
            ],
        )),
        ..Default::default()
    };

    let mut geometry = Geometry::new(
        attributes,
        vec![0, 1, 2],
        PrimitiveType::Triangles,
        None,
    );

    GeometryPipeline::compress_vertices(&mut geometry).unwrap();

    assert!(geometry.attributes.normal.is_none());
    assert!(geometry.attributes.st.is_none());
    assert!(geometry.attributes.contains_key("compressedAttributes"));
}

#[test]
fn test_split_longitude_unindexed() {
    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute::new(
            ComponentDatatype::Double,
            3,
            vec![
                0.0, 0.0, 0.0,
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
            ],
        )),
        ..Default::default()
    };

    let geom = Geometry::new(attributes, vec![], PrimitiveType::Triangles, None);
    let mut instance = GeometryInstance::new(geom, None, None, None);

    GeometryPipeline::split_longitude(&mut instance).unwrap();

    // Auto-indexed
    assert_eq!(instance.geometry.indices, vec![0, 1, 2]);
}
