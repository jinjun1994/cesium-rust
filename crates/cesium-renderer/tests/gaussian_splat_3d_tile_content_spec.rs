use cesium_math::{Cartesian3, Matrix4};
use cesium_renderer::gaussian_splat_3d_tile_content::{
    GaussianSplat3DTileContent, GaussianSplat3DTileContentOptions,
};
use cesium_renderer::vertex_attribute_semantic::VertexAttributeSemantic;

#[test]
fn loads_gaussian_splat_content_and_exposes_attributes() {
    let content = GaussianSplat3DTileContent::new(GaussianSplat3DTileContentOptions {
        tileset_id: Some("tileset-1".into()),
        tile_id: Some("tile-0".into()),
        resource_url: "splats/tile.glb".into(),
        positions: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        rotations: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        scales: vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0],
        colors: vec![1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
        sh_degree: 0,
    });

    assert!(content.ready());
    assert_eq!(content.points_length(), 2);
    assert_eq!(content.features_length(), 0);
    assert_eq!(content.triangles_length(), 0);
    assert!(content.geometry_byte_length() > 0);
    assert!(!content.has_property(0, "any"));
    assert!(content.get_feature(0).is_none());

    let pos = content
        .gltf_primitive
        .get_attribute_by_semantic(VertexAttributeSemantic::POSITION)
        .unwrap();
    let rot = content
        .gltf_primitive
        .get_attribute_by_semantic(VertexAttributeSemantic::ROTATION)
        .unwrap();
    let scale = content
        .gltf_primitive
        .get_attribute_by_semantic(VertexAttributeSemantic::SCALE)
        .unwrap();
    let color = content
        .gltf_primitive
        .get_attribute_by_semantic(VertexAttributeSemantic::COLOR)
        .unwrap();
    assert!(!pos.data.is_empty());
    assert!(!rot.data.is_empty());
    assert!(!scale.data.is_empty());
    assert!(!color.data.is_empty());
}

#[test]
fn transforms_attributes_while_preserving_source_for_rebuilds() {
    let mut content = GaussianSplat3DTileContent::new(GaussianSplat3DTileContentOptions {
        resource_url: "tile.glb".into(),
        positions: vec![1.0, 0.0, 0.0],
        rotations: vec![0.0, 0.0, 0.0, 1.0],
        scales: vec![1.0, 2.0, 3.0],
        colors: vec![1.0, 1.0, 1.0, 1.0],
        sh_degree: 1,
        ..Default::default()
    });

    assert!(!content.is_transformed());
    assert_eq!(content.sh_degree(), 1);

    let transform = Matrix4::from_translation(Cartesian3::new(10.0, 20.0, 30.0));
    content.apply_tile_transform(&transform);
    assert!(content.is_transformed());
    assert_eq!(content.positions(), &[11.0, 20.0, 30.0]);
    assert_eq!(content.scales(), &[1.0, 2.0, 3.0]);

    // Original source attribute inside gltf_primitive remains untouched
    let src_pos = content
        .gltf_primitive
        .get_attribute_by_semantic(VertexAttributeSemantic::POSITION)
        .unwrap();
    assert_eq!(src_pos.data, vec![1.0, 0.0, 0.0]);
}

#[test]
fn create_and_destroy_gaussian_splat_content() {
    let mut content = GaussianSplat3DTileContent::new(GaussianSplat3DTileContentOptions {
        resource_url: "tile.glb".into(),
        positions: vec![0.0, 0.0, 0.0],
        rotations: vec![0.0, 0.0, 0.0, 1.0],
        scales: vec![1.0, 1.0, 1.0],
        colors: vec![1.0, 1.0, 1.0, 1.0],
        ..Default::default()
    });
    assert!(!content.is_destroyed());
    content.destroy();
    assert!(content.is_destroyed());
    assert_eq!(content.points_length(), 0);
    assert!(content.gltf_primitive.attributes.is_empty());
}
