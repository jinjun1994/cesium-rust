//! Port of CesiumJS `Scene/Model/ModelImageryMappingSpec.js` (Apache-2.0).

use std::f64::consts::PI;

use cesium_math::{
    AttributeType, BoundingRectangle, Cartesian2, Cartesian3, Cartographic, Ellipsoid,
    GeoRectangle, Matrix4, WebMercatorProjection, EPSILON6,
};
use cesium_renderer::model_imagery_mapping::{MappedPositions, ModelImageryMapping};

#[test]
fn create_texture_coordinates_for_mapped_positions_throws_without_mapped_positions() {
    let projection = WebMercatorProjection::default();
    assert!(
        ModelImageryMapping::create_texture_coordinates_for_mapped_positions(
            None,
            Some(&projection)
        )
        .is_err()
    );
}

#[test]
fn create_texture_coordinates_for_mapped_positions_throws_without_projection() {
    let cartographic_positions = vec![
        Cartographic::new(0.0, 0.0, 0.0),
        Cartographic::new(0.1, 0.0, 0.0),
        Cartographic::new(0.0, 0.1, 0.0),
        Cartographic::new(0.1, 0.1, 0.0),
    ];
    let num_positions = 4;
    let cartographic_bounding_rectangle = GeoRectangle::new(0.0, 0.0, 0.1, 0.1);
    let ellipsoid = Ellipsoid::WGS84;
    let mapped_positions = MappedPositions::from_values(
        cartographic_positions,
        num_positions,
        cartographic_bounding_rectangle,
        ellipsoid,
    )
    .unwrap();

    assert!(
        ModelImageryMapping::create_texture_coordinates_for_mapped_positions(
            Some(&mapped_positions),
            None::<&WebMercatorProjection>
        )
        .is_err()
    );
}

#[test]
fn create_texture_coordinates_attribute_for_mapped_positions_throws_without_mapped_positions() {
    let projection = WebMercatorProjection::default();
    assert!(
        ModelImageryMapping::create_texture_coordinates_attribute_for_mapped_positions(
            None,
            Some(&projection)
        )
        .is_err()
    );
}

#[test]
fn create_texture_coordinates_attribute_for_mapped_positions_throws_without_projection() {
    let cartographic_positions = vec![
        Cartographic::new(0.0, 0.0, 0.0),
        Cartographic::new(0.1, 0.0, 0.0),
        Cartographic::new(0.0, 0.1, 0.0),
        Cartographic::new(0.1, 0.1, 0.0),
    ];
    let num_positions = 4;
    let cartographic_bounding_rectangle = GeoRectangle::new(0.0, 0.0, 0.1, 0.1);
    let ellipsoid = Ellipsoid::WGS84;
    let mapped_positions = MappedPositions::from_values(
        cartographic_positions,
        num_positions,
        cartographic_bounding_rectangle,
        ellipsoid,
    )
    .unwrap();

    assert!(
        ModelImageryMapping::create_texture_coordinates_attribute_for_mapped_positions(
            Some(&mapped_positions),
            None::<&WebMercatorProjection>
        )
        .is_err()
    );
}

#[test]
fn create_texture_coordinates_attribute_for_mapped_positions_creates_attribute() {
    let cartographic_positions = vec![
        Cartographic::new(0.0, 0.0, 0.0),
        Cartographic::new(0.1, 0.0, 0.0),
        Cartographic::new(0.0, 0.1, 0.0),
        Cartographic::new(0.1, 0.1, 0.0),
    ];
    let num_positions = 4;
    let cartographic_bounding_rectangle = GeoRectangle::new(0.0, 0.0, 0.1, 0.1);
    let ellipsoid = Ellipsoid::WGS84;
    let mapped_positions = MappedPositions::from_values(
        cartographic_positions,
        num_positions,
        cartographic_bounding_rectangle,
        ellipsoid,
    )
    .unwrap();
    let projection = WebMercatorProjection::default();

    let attribute =
        ModelImageryMapping::create_texture_coordinates_attribute_for_mapped_positions(
            Some(&mapped_positions),
            Some(&projection),
        )
        .unwrap();

    assert_eq!(attribute.semantic.as_deref(), Some("TEXCOORD"));
    assert_eq!(attribute.attribute_type, Some(AttributeType::Vec2));
    assert_eq!(attribute.count, 4);
}

#[test]
fn create_texture_coordinates_throws_without_cartographic_positions() {
    let cartographic_bounding_rectangle = GeoRectangle::new(0.0, 0.0, 0.1, 0.1);
    let projection = WebMercatorProjection::default();
    assert!(ModelImageryMapping::create_texture_coordinates(
        None,
        Some(4),
        Some(&cartographic_bounding_rectangle),
        Some(&projection),
    )
    .is_err());
}

#[test]
fn create_texture_coordinates_throws_with_invalid_num_positions() {
    let cartographic_positions = vec![
        Cartographic::new(0.0, 0.0, 0.0),
        Cartographic::new(0.1, 0.0, 0.0),
        Cartographic::new(0.0, 0.1, 0.0),
        Cartographic::new(0.1, 0.1, 0.0),
    ];
    let cartographic_bounding_rectangle = GeoRectangle::new(0.0, 0.0, 0.1, 0.1);
    let projection = WebMercatorProjection::default();
    assert!(ModelImageryMapping::create_texture_coordinates(
        Some(&cartographic_positions),
        Some(-1),
        Some(&cartographic_bounding_rectangle),
        Some(&projection),
    )
    .is_err());
}

#[test]
fn create_texture_coordinates_throws_without_cartographic_bounding_rectangle() {
    let cartographic_positions = vec![
        Cartographic::new(0.0, 0.0, 0.0),
        Cartographic::new(0.1, 0.0, 0.0),
        Cartographic::new(0.0, 0.1, 0.0),
        Cartographic::new(0.1, 0.1, 0.0),
    ];
    let projection = WebMercatorProjection::default();
    assert!(ModelImageryMapping::create_texture_coordinates(
        Some(&cartographic_positions),
        Some(4),
        None,
        Some(&projection),
    )
    .is_err());
}

#[test]
fn create_texture_coordinates_throws_without_projection() {
    let cartographic_positions = vec![
        Cartographic::new(0.0, 0.0, 0.0),
        Cartographic::new(0.1, 0.0, 0.0),
        Cartographic::new(0.0, 0.1, 0.0),
        Cartographic::new(0.1, 0.1, 0.0),
    ];
    let cartographic_bounding_rectangle = GeoRectangle::new(0.0, 0.0, 0.1, 0.1);
    assert!(ModelImageryMapping::create_texture_coordinates(
        Some(&cartographic_positions),
        Some(4),
        Some(&cartographic_bounding_rectangle),
        None::<&WebMercatorProjection>,
    )
    .is_err());
}

#[test]
fn create_texture_coordinates_creates_texture_coordinates() {
    let cartographic_positions = vec![
        Cartographic::new(0.0, 0.0, 0.0),
        Cartographic::new(0.1, 0.0, 0.0),
        Cartographic::new(0.0, 0.1, 0.0),
        Cartographic::new(0.1, 0.1, 0.0),
    ];
    let cartographic_bounding_rectangle = GeoRectangle::new(0.0, 0.0, 0.1, 0.1);
    let projection = WebMercatorProjection::default();

    let actual = ModelImageryMapping::create_texture_coordinates(
        Some(&cartographic_positions),
        Some(4),
        Some(&cartographic_bounding_rectangle),
        Some(&projection),
    )
    .unwrap();

    let expected = [0.0f32, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert!((a - e).abs() < 1e-5);
    }
}

#[test]
fn create_tex_coord_attribute_throws_without_array() {
    assert!(ModelImageryMapping::create_tex_coord_attribute(None).is_err());
}

#[test]
fn create_tex_coord_attribute_creates_attribute() {
    let tex_coords = vec![0.0f32, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let attr = ModelImageryMapping::create_tex_coord_attribute(Some(&tex_coords)).unwrap();

    assert_eq!(attr.semantic.as_deref(), Some("TEXCOORD"));
    assert_eq!(attr.attribute_type, Some(AttributeType::Vec2));
    assert_eq!(attr.count, 4);
    assert!(attr.typed_array.is_some());
}

#[test]
fn create_iterable_cartesian3_from_typed_array_throws_with_undefined() {
    assert!(ModelImageryMapping::create_iterable_cartesian3_from_typed_array(None, Some(3)).is_err());
}

#[test]
fn create_iterable_cartesian3_from_typed_array_throws_with_invalid_stride() {
    let arr = vec![0.0f32, 0.1, 0.2, 1.0, 1.1, 1.2, 2.0, 2.1, 2.2];
    assert!(
        ModelImageryMapping::create_iterable_cartesian3_from_typed_array(Some(&arr), Some(2))
            .is_err()
    );
}

#[test]
fn create_iterable_cartesian3_from_typed_array_stride_3() {
    let arr = vec![0.0f32, 0.1, 0.2, 1.0, 1.1, 1.2, 2.0, 2.1, 2.2];
    let actual =
        ModelImageryMapping::create_iterable_cartesian3_from_typed_array(Some(&arr), Some(3))
            .unwrap();

    let expected = [
        Cartesian3::new(0.0, 0.10000000149011612, 0.20000000298023224),
        Cartesian3::new(1.0, 1.100000023841858, 1.2000000476837158),
        Cartesian3::new(2.0, 2.0999999046325684, 2.200000047683716),
    ];
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert!(a.equals_epsilon(e, Some(EPSILON6), None));
    }
}

#[test]
fn create_iterable_cartesian3_from_typed_array_stride_4() {
    let arr = vec![
        0.0f32, 0.1, 0.2, 9.9, 1.0, 1.1, 1.2, 9.9, 2.0, 2.1, 2.2, 9.9,
    ];
    let actual =
        ModelImageryMapping::create_iterable_cartesian3_from_typed_array(Some(&arr), Some(4))
            .unwrap();

    let expected = [
        Cartesian3::new(0.0, 0.10000000149011612, 0.20000000298023224),
        Cartesian3::new(1.0, 1.100000023841858, 1.2000000476837158),
        Cartesian3::new(2.0, 2.0999999046325684, 2.200000047683716),
    ];
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert!(a.equals_epsilon(e, Some(EPSILON6), None));
    }
}

#[test]
fn map_throws_with_undefined() {
    assert!(ModelImageryMapping::map::<Cartesian3, f64, _>(None, Some(|c: &Cartesian3| c.magnitude())).is_err());
    assert!(ModelImageryMapping::map::<Cartesian3, f64, fn(&Cartesian3) -> f64>(
        Some(&[Cartesian3::UNIT_X]),
        None
    )
    .is_err());
}

#[test]
fn map_maps() {
    let items = vec![
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(0.0, 2.0, 0.0),
        Cartesian3::new(0.0, 0.0, 3.0),
    ];
    let actual = ModelImageryMapping::map(Some(&items), Some(|c: &Cartesian3| c.magnitude())).unwrap();
    let expected = vec![1.0, 2.0, 3.0];
    assert_eq!(actual, expected);
}

#[test]
fn compute_cartographic_bounding_rectangle_throws_with_undefined() {
    assert!(ModelImageryMapping::compute_cartographic_bounding_rectangle(None, None).is_err());
}

#[test]
fn compute_cartographic_bounding_rectangle_computes_bounding_rectangle() {
    let positions = vec![
        Cartographic::new(0.0, 0.0, 0.0),
        Cartographic::new(0.1, 0.0, 0.0),
        Cartographic::new(0.0, 0.1, 0.0),
        Cartographic::new(0.1, 0.1, 0.0),
    ];
    let mut result = GeoRectangle::default();
    let actual =
        ModelImageryMapping::compute_cartographic_bounding_rectangle(Some(&positions), Some(&mut result))
            .unwrap();

    let expected = GeoRectangle::new(0.0, 0.0, 0.1, 0.1);
    assert!((actual.west - expected.west).abs() < EPSILON6);
    assert!((actual.south - expected.south).abs() < EPSILON6);
    assert!((actual.east - expected.east).abs() < EPSILON6);
    assert!((actual.north - expected.north).abs() < EPSILON6);
}

#[test]
fn transform_cartesians3_throws_with_undefined() {
    let mat = Matrix4::IDENTITY;
    assert!(ModelImageryMapping::transform_cartesians3(None, Some(&mat)).is_err());
    assert!(ModelImageryMapping::transform_cartesians3(Some(&[Cartesian3::ZERO]), None).is_err());
}

#[test]
fn transform_cartesians3_transforms() {
    let positions = vec![
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(0.0, 2.0, 0.0),
        Cartesian3::new(0.0, 0.0, 3.0),
    ];
    let mat = Matrix4::from_translation(Cartesian3::new(1.0, 2.0, 3.0));
    let actual = ModelImageryMapping::transform_cartesians3(Some(&positions), Some(&mat)).unwrap();

    let expected = [
        Cartesian3::new(2.0, 2.0, 3.0),
        Cartesian3::new(1.0, 4.0, 3.0),
        Cartesian3::new(1.0, 2.0, 6.0),
    ];
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert!(a.equals_epsilon(e, Some(EPSILON6), None));
    }
}

#[test]
fn transform_to_cartographic_throws_with_undefined() {
    let ell = Ellipsoid::WGS84;
    assert!(ModelImageryMapping::transform_to_cartographic(None, Some(&ell)).is_err());
    assert!(ModelImageryMapping::transform_to_cartographic(Some(&[Cartesian3::ZERO]), None).is_err());
}

#[test]
fn transform_to_cartographic_transforms() {
    let positions = vec![
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(0.0, 2.0, 0.0),
        Cartesian3::new(0.0, 0.0, 3.0),
    ];
    let ell = Ellipsoid::WGS84;
    let actual = ModelImageryMapping::transform_to_cartographic(Some(&positions), Some(&ell)).unwrap();

    let expected = [
        Cartographic::new(0.0, 0.0, -6378137.0 + 1.0),
        Cartographic::new(PI / 2.0, 0.0, -6378137.0 + 2.0),
        Cartographic::new(0.0, PI / 2.0, -6356752.314245179 + 3.0),
    ];
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert!((a.longitude - e.longitude).abs() < EPSILON6);
        assert!((a.latitude - e.latitude).abs() < EPSILON6);
        assert!((a.height - e.height).abs() < 1e-4);
    }
}

#[test]
fn create_projected_positions_throws_with_undefined() {
    let proj = WebMercatorProjection::default();
    assert!(ModelImageryMapping::create_projected_positions(None, Some(&proj)).is_err());
    assert!(ModelImageryMapping::create_projected_positions(
        Some(&[Cartographic::default()]),
        None::<&WebMercatorProjection>
    )
    .is_err());
}

#[test]
fn create_projected_positions_creates_projected_positions() {
    let positions = vec![
        Cartographic::new(0.0, 0.0, 0.0),
        Cartographic::new(0.1, 0.0, 0.0),
        Cartographic::new(0.0, 0.1, 0.0),
        Cartographic::new(0.1, 0.1, 0.0),
    ];
    let proj = WebMercatorProjection::default();
    let actual = ModelImageryMapping::create_projected_positions(Some(&positions), Some(&proj)).unwrap();

    let expected = [
        Cartesian3::new(0.0, 0.0, 0.0),
        Cartesian3::new(637813.7, 0.0, 0.0),
        Cartesian3::new(0.0, 638879.3881344117, 0.0),
        Cartesian3::new(637813.7, 638879.3881344117, 0.0),
    ];
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert!(a.equals_epsilon(e, Some(EPSILON6), None));
    }
}

#[test]
fn compute_tex_coords_throws_with_undefined() {
    let bounds = BoundingRectangle::new(0.0, 0.0, 2.0, 2.0);
    assert!(ModelImageryMapping::compute_tex_coords(None, Some(&bounds)).is_err());
    assert!(ModelImageryMapping::compute_tex_coords(Some(&[Cartesian3::ZERO]), None).is_err());
}

#[test]
fn compute_tex_coords_computes_texture_coordinates() {
    let positions = vec![
        Cartesian3::new(0.0, 0.0, 0.0),
        Cartesian3::new(1.0, 0.0, 0.0),
        Cartesian3::new(0.0, 1.0, 0.0),
        Cartesian3::new(-4.0, 4.0, 0.0),
    ];
    let bounds = BoundingRectangle::new(0.0, 0.0, 2.0, 2.0);
    let actual = ModelImageryMapping::compute_tex_coords(Some(&positions), Some(&bounds)).unwrap();

    let expected = [
        Cartesian2::new(0.0, 0.0),
        Cartesian2::new(0.5, 0.0),
        Cartesian2::new(0.0, 0.5),
        Cartesian2::new(0.0, 1.0), // Clamped
    ];
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert!(a.equals_epsilon(e, Some(EPSILON6), None));
    }
}

#[test]
fn create_typed_array_from_cartesians2_throws_with_invalid() {
    let elems = vec![Cartesian2::new(0.0, 0.0)];
    assert!(ModelImageryMapping::create_typed_array_from_cartesians2(Some(-1), Some(&elems)).is_err());
    assert!(ModelImageryMapping::create_typed_array_from_cartesians2(Some(1), None).is_err());
}

#[test]
fn create_typed_array_from_cartesians2_creates_array() {
    let elems = vec![
        Cartesian2::new(0.0, 0.0),
        Cartesian2::new(1.0, 0.0),
        Cartesian2::new(0.0, 1.0),
        Cartesian2::new(1.0, 1.0),
    ];
    let actual = ModelImageryMapping::create_typed_array_from_cartesians2(Some(4), Some(&elems)).unwrap();
    let expected = [0.0f32, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    assert_eq!(actual, expected);
}
