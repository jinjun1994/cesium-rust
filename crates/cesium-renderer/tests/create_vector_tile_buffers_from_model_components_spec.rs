use cesium_math::{
    AttributeType, Cartesian3, ComponentDatatype, IndexDatatype, Matrix4, PrimitiveType,
};
use cesium_renderer::create_vector_tile_buffers::{
    create_vector_tile_buffers_from_model_components, BufferCollectionGet, BufferPoint,
    BufferPolygon, BufferPolyline, VectorGltf3DTileContent, VectorTileset,
};
use cesium_renderer::model_components::{
    Attribute, Components, FeatureId, FeatureIdAttribute, Indices, Node, Polygon, Primitive,
    Scene, Vector,
};

fn create_position_attribute(positions: &[f32]) -> Attribute {
    let mut bytes = Vec::with_capacity(positions.len() * 4);
    for &p in positions {
        bytes.extend_from_slice(&p.to_le_bytes());
    }
    Attribute {
        semantic: Some("POSITION".to_string()),
        count: positions.len() / 3,
        typed_array: Some(bytes),
        component_datatype: Some(ComponentDatatype::Float),
        attribute_type: Some(AttributeType::Vec3),
        ..Default::default()
    }
}

fn create_feature_id_attribute(feature_ids: &[u16]) -> Attribute {
    let mut bytes = Vec::with_capacity(feature_ids.len() * 2);
    for &id in feature_ids {
        bytes.extend_from_slice(&id.to_le_bytes());
    }
    Attribute {
        semantic: Some("FEATURE_ID".to_string()),
        set_index: Some(0),
        count: feature_ids.len(),
        typed_array: Some(bytes),
        component_datatype: Some(ComponentDatatype::UnsignedShort),
        attribute_type: Some(AttributeType::Scalar),
        ..Default::default()
    }
}

fn create_feature_id_set(feature_count: usize) -> FeatureId {
    FeatureId::Attribute(FeatureIdAttribute {
        feature_count,
        set_index: Some(0),
        positional_label: Some("featureId_0".to_string()),
        ..Default::default()
    })
}

fn create_indices_u16(indices: &[u16]) -> Indices {
    let mut bytes = Vec::with_capacity(indices.len() * 2);
    for &idx in indices {
        bytes.extend_from_slice(&idx.to_le_bytes());
    }
    Indices {
        count: indices.len(),
        typed_array: Some(bytes),
        index_datatype: Some(IndexDatatype::UnsignedShort),
        ..Default::default()
    }
}

fn create_indices_u32(indices: &[u32]) -> Indices {
    let mut bytes = Vec::with_capacity(indices.len() * 4);
    for &idx in indices {
        bytes.extend_from_slice(&idx.to_le_bytes());
    }
    Indices {
        count: indices.len(),
        typed_array: Some(bytes),
        index_datatype: Some(IndexDatatype::UnsignedInt),
        ..Default::default()
    }
}

struct PrimitiveOptions<'a> {
    primitive_type: PrimitiveType,
    positions: &'a [f32],
    indices_u16: Option<&'a [u16]>,
    indices_u32: Option<&'a [u32]>,
    vector: Option<Vector>,
    polygon: Option<Polygon>,
    feature_ids: Option<&'a [u16]>,
}

fn create_primitive(options: PrimitiveOptions<'_>) -> Primitive {
    let mut attributes = vec![create_position_attribute(options.positions)];
    let mut feature_ids_components = Vec::new();

    if let Some(fids) = options.feature_ids {
        attributes.push(create_feature_id_attribute(fids));
        let max_fid = fids.iter().copied().max().unwrap_or(0);
        feature_ids_components.push(create_feature_id_set((max_fid + 1) as usize));
    }

    let indices = if let Some(iu16) = options.indices_u16 {
        Some(create_indices_u16(iu16))
    } else {
        options.indices_u32.map(create_indices_u32)
    };

    Primitive {
        attributes,
        indices,
        primitive_type: Some(options.primitive_type),
        feature_ids: feature_ids_components,
        vector: options.vector,
        polygon: options.polygon,
        ..Default::default()
    }
}

fn create_components(root_node: Node) -> Components {
    Components {
        scene: Some(Scene { nodes: vec![0] }),
        nodes: vec![root_node],
        ..Default::default()
    }
}

fn mock_tile_content() -> VectorGltf3DTileContent {
    VectorGltf3DTileContent {
        tileset: VectorTileset {
            feature_id_label: Some("featureId_0".to_string()),
        },
    }
}

#[test]
fn test_creates_one_point_collection_per_mesh_primitive_and_preserves_local_transforms() {
    let first_primitive = create_primitive(PrimitiveOptions {
        primitive_type: PrimitiveType::Points,
        positions: &[1.0, 2.0, 3.0],
        indices_u16: Some(&[0]),
        indices_u32: None,
        vector: Some(Vector {
            count: 1,
            ..Default::default()
        }),
        polygon: None,
        feature_ids: Some(&[7]),
    });

    let second_primitive = create_primitive(PrimitiveOptions {
        primitive_type: PrimitiveType::Points,
        positions: &[4.0, 5.0, 6.0],
        indices_u16: Some(&[0]),
        indices_u32: None,
        vector: Some(Vector {
            count: 1,
            ..Default::default()
        }),
        polygon: None,
        feature_ids: Some(&[9]),
    });

    let node = Node {
        translation: Some(Cartesian3::new(10.0, 20.0, 30.0)),
        primitives: vec![first_primitive, second_primitive],
        ..Default::default()
    };

    let result = create_vector_tile_buffers_from_model_components(
        &mock_tile_content(),
        &create_components(node),
    );

    assert_eq!(result.collections.len(), 2);
    assert_eq!(result.collections[0].primitive_count(), 1);
    assert_eq!(result.collections[1].primitive_count(), 1);

    let expected_transform = Matrix4::from_translation(Cartesian3::new(10.0, 20.0, 30.0));
    assert_eq!(result.collection_local_matrices[0], expected_transform);
    assert_eq!(result.collection_local_matrices[1], expected_transform);

    let mut point = BufferPoint::default();
    result.collections[0].get(0, &mut point);
    assert_eq!(point.get_position(None), Cartesian3::new(1.0, 2.0, 3.0));
    assert_eq!(point.feature_id, 7);

    result.collections[1].get(0, &mut point);
    assert_eq!(point.get_position(None), Cartesian3::new(4.0, 5.0, 6.0));
    assert_eq!(point.feature_id, 9);
}

#[test]
fn test_splits_line_strips_using_primitive_restart_values() {
    let primitive = create_primitive(PrimitiveOptions {
        primitive_type: PrimitiveType::LineStrip,
        positions: &[
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 0.0, 4.0, 0.0, 0.0,
        ],
        indices_u16: None,
        indices_u32: Some(&[0, 1, 2, 0xffffffff, 2, 3, 4]),
        vector: Some(Vector {
            count: 2,
            ..Default::default()
        }),
        polygon: None,
        feature_ids: Some(&[10, 11, 12, 13, 14]),
    });

    let node = Node {
        translation: Some(Cartesian3::new(5.0, 6.0, 7.0)),
        primitives: vec![primitive],
        ..Default::default()
    };

    let result = create_vector_tile_buffers_from_model_components(
        &mock_tile_content(),
        &create_components(node),
    );

    assert_eq!(result.collections.len(), 1);

    let collection = &result.collections[0];
    assert_eq!(collection.primitive_count(), 2);
    assert_eq!(
        result.collection_local_matrices[0],
        Matrix4::from_translation(Cartesian3::new(5.0, 6.0, 7.0))
    );

    let mut polyline = BufferPolyline::default();
    collection.get(0, &mut polyline);
    assert_eq!(
        polyline.get_positions(),
        &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 2.0, 0.0, 0.0]
    );
    assert_eq!(polyline.feature_id, 10);

    collection.get(1, &mut polyline);
    assert_eq!(
        polyline.get_positions(),
        &[2.0, 0.0, 0.0, 3.0, 0.0, 0.0, 4.0, 0.0, 0.0]
    );
    assert_eq!(polyline.feature_id, 12);
}

#[test]
fn test_creates_polygon_collections_from_cesium_mesh_vector() {
    let primitive = create_primitive(PrimitiveOptions {
        primitive_type: PrimitiveType::Triangles,
        positions: &[
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 0.0, 2.0,
            1.0, 0.0,
        ],
        indices_u16: None,
        indices_u32: Some(&[0, 1, 2, 3, 4, 5]),
        vector: Some(Vector {
            count: 2,
            polygon_attribute_offsets: vec![0, 3],
            polygon_indices_offsets: vec![0, 3],
            ..Default::default()
        }),
        polygon: None,
        feature_ids: Some(&[100, 100, 100, 200, 200, 200]),
    });

    let node = Node {
        primitives: vec![primitive],
        ..Default::default()
    };

    let result = create_vector_tile_buffers_from_model_components(
        &mock_tile_content(),
        &create_components(node),
    );

    assert_eq!(result.collections.len(), 1);

    let collection = &result.collections[0];
    assert_eq!(collection.primitive_count(), 2);
    assert_eq!(collection.triangle_count(), 2);
    assert_eq!(collection.hole_count(), 0);

    let mut polygon = BufferPolygon::default();
    collection.get(0, &mut polygon);
    assert_eq!(
        polygon.get_positions(),
        &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]
    );
    assert_eq!(polygon.get_triangles(), &[0, 1, 2]);
    assert_eq!(polygon.feature_id, 100);

    collection.get(1, &mut polygon);
    assert_eq!(
        polygon.get_positions(),
        &[2.0, 0.0, 0.0, 3.0, 0.0, 0.0, 2.0, 1.0, 0.0]
    );
    assert_eq!(polygon.get_triangles(), &[0, 1, 2]);
    assert_eq!(polygon.feature_id, 200);
}

#[test]
fn test_creates_polygon_collections_from_ext_mesh_polygon() {
    let loop_indices = vec![0, 1, 2, 3, 0xffffffff, 4, 5, 6, 7];
    let primitive = create_primitive(PrimitiveOptions {
        primitive_type: PrimitiveType::LineLoop,
        positions: &[
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0,
            1.0, 0.0, 0.0, 2.0, 0.0, 1.0, 2.0, 0.0,
        ],
        indices_u16: None,
        indices_u32: Some(&loop_indices),
        vector: None,
        polygon: Some(Polygon {
            count: 2,
            loop_indices: loop_indices.clone(),
            loop_indices_offsets: vec![0, 5],
            triangle_indices: Some(vec![0, 1, 2, 2, 1, 3, 4, 5, 6, 6, 5, 7]),
            triangle_indices_offsets: vec![0, 6],
        }),
        feature_ids: Some(&[100, 100, 100, 100, 200, 200, 200, 200]),
    });

    let node = Node {
        primitives: vec![primitive],
        ..Default::default()
    };

    let result = create_vector_tile_buffers_from_model_components(
        &mock_tile_content(),
        &create_components(node),
    );

    assert_eq!(result.collections.len(), 1);

    let collection = &result.collections[0];
    assert_eq!(collection.primitive_count(), 2);
    assert_eq!(collection.triangle_count(), 4);
    assert_eq!(collection.hole_count(), 0);

    let mut polygon = BufferPolygon::default();
    collection.get(0, &mut polygon);
    assert_eq!(
        polygon.get_positions(),
        &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0]
    );
    assert_eq!(polygon.get_triangles(), &[0, 1, 2, 2, 1, 3]);
    assert_eq!(polygon.feature_id, 100);

    collection.get(1, &mut polygon);
    assert_eq!(
        polygon.get_positions(),
        &[0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 2.0, 0.0, 1.0, 2.0, 0.0]
    );
    // indices in BufferPolygon are relative to current polygon's positions
    assert_eq!(polygon.get_triangles(), &[0, 1, 2, 2, 1, 3]);
    assert_eq!(polygon.feature_id, 200);
}

#[test]
fn test_creates_polygon_collections_with_holes_from_cesium_mesh_vector() {
    let primitive = create_primitive(PrimitiveOptions {
        primitive_type: PrimitiveType::Triangles,
        positions: &[
            0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 4.0, 4.0, 0.0, 0.0, 4.0, 0.0, 1.0, 1.0, 0.0, 2.0,
            1.0, 0.0, 1.0, 2.0, 0.0,
        ],
        indices_u16: None,
        indices_u32: Some(&[0, 1, 2, 0, 2, 3]),
        vector: Some(Vector {
            count: 1,
            polygon_attribute_offsets: vec![0],
            polygon_indices_offsets: vec![0],
            polygon_hole_counts: vec![1],
            polygon_hole_offsets: vec![4],
        }),
        polygon: None,
        feature_ids: Some(&[5, 5, 5, 5, 5, 5, 5]),
    });

    let node = Node {
        primitives: vec![primitive],
        ..Default::default()
    };

    let result = create_vector_tile_buffers_from_model_components(
        &mock_tile_content(),
        &create_components(node),
    );

    assert_eq!(result.collections.len(), 1);

    let collection = &result.collections[0];
    assert_eq!(collection.primitive_count(), 1);
    assert_eq!(collection.hole_count(), 1);
    assert_eq!(collection.triangle_count(), 2);

    let mut polygon = BufferPolygon::default();
    collection.get(0, &mut polygon);
    assert_eq!(polygon.hole_count(), 1);
    assert_eq!(polygon.get_holes(), &[4]);
    assert_eq!(polygon.get_triangles(), &[0, 1, 2, 0, 2, 3]);
    assert_eq!(polygon.feature_id, 5);
}

#[test]
fn test_creates_polygon_collections_with_holes_from_ext_mesh_polygon() {
    let loop_indices = vec![0, 1, 2, 3, 0xffffffff, 4, 5, 6, 7];
    let primitive = create_primitive(PrimitiveOptions {
        primitive_type: PrimitiveType::LineLoop,
        positions: &[
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.25, 0.25, 0.0,
            0.75, 0.25, 0.0, 0.25, 0.75, 0.0, 0.75, 0.75, 0.0,
        ],
        indices_u16: None,
        indices_u32: Some(&loop_indices),
        vector: None,
        polygon: Some(Polygon {
            count: 1,
            loop_indices: loop_indices.clone(),
            loop_indices_offsets: vec![0],
            triangle_indices: Some(vec![
                0, 1, 5, 0, 5, 4, 0, 4, 2, 1, 3, 5, 2, 4, 6, 2, 6, 3, 3, 6, 7, 3, 7, 5,
            ]),
            triangle_indices_offsets: vec![0],
        }),
        feature_ids: Some(&[5, 5, 5, 5, 5, 5, 5, 5]),
    });

    let node = Node {
        primitives: vec![primitive],
        ..Default::default()
    };

    let result = create_vector_tile_buffers_from_model_components(
        &mock_tile_content(),
        &create_components(node),
    );

    assert_eq!(result.collections.len(), 1);

    let collection = &result.collections[0];
    assert_eq!(collection.primitive_count(), 1);
    assert_eq!(collection.hole_count(), 1);
    assert_eq!(collection.triangle_count(), 8);

    let mut polygon = BufferPolygon::default();
    collection.get(0, &mut polygon);
    assert_eq!(polygon.hole_count(), 1);
    assert_eq!(polygon.get_holes(), &[4]);
    assert_eq!(polygon.get_triangles().len(), 8 * 3);
    assert_eq!(polygon.feature_id, 5);
}

#[test]
fn test_returns_empty_result_when_model_contains_no_vector_primitives() {
    let node = Node::default();
    let result = create_vector_tile_buffers_from_model_components(
        &mock_tile_content(),
        &create_components(node),
    );

    assert!(result.collections.is_empty());
    assert!(result.collection_local_matrices.is_empty());
}
