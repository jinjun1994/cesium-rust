use cesium_math::{Cartesian3, DistanceDisplayCondition, Ellipsoid};
use cesium_renderer::material::Material;
use cesium_renderer::polyline_collection::{PolylineCollection, PolylineOptions};
use glam::DMat4;

#[test]
fn test_default_constructs_a_polyline() {
    let mut polylines = PolylineCollection::new();
    let idx = polylines.add(None);
    let p = polylines.get(idx).unwrap();

    assert!(p.show);
    assert_eq!(p.positions.len(), 0);
    assert_eq!(p.width, 1.0);
    assert!(!p.loop_);
    assert!(p.id.is_none());

    let color_map = p.material.uniforms.get("color").unwrap().as_object().unwrap();
    assert_eq!(color_map.get("red").unwrap().as_f64(), Some(1.0));
    assert_eq!(color_map.get("green").unwrap().as_f64(), Some(1.0));
    assert_eq!(color_map.get("blue").unwrap().as_f64(), Some(1.0));
    assert_eq!(color_map.get("alpha").unwrap().as_f64(), Some(1.0));
}

#[test]
fn test_explicitly_constructs_a_polyline() {
    let mut polylines = PolylineCollection::new();
    let material = Material::from_type(Material::POLYLINE_OUTLINE_TYPE).unwrap();

    let options = PolylineOptions {
        show: Some(false),
        positions: Some(vec![
            Cartesian3::new(1.0, 2.0, 3.0),
            Cartesian3::new(4.0, 5.0, 6.0),
            Cartesian3::new(7.0, 8.0, 9.0),
        ]),
        loop_: Some(true),
        width: Some(2.0),
        material: Some(material.clone()),
        id: Some("id".to_string()),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
    };

    let idx = polylines.add(Some(options));
    let p = polylines.get(idx).unwrap();

    assert!(!p.show);
    assert_eq!(p.positions.len(), 3);
    assert_eq!(p.positions[0], Cartesian3::new(1.0, 2.0, 3.0));
    assert_eq!(p.positions[1], Cartesian3::new(4.0, 5.0, 6.0));
    assert_eq!(p.positions[2], Cartesian3::new(7.0, 8.0, 9.0));
    assert!(p.loop_);
    assert_eq!(p.width, 2.0);
    assert_eq!(p.id.as_deref(), Some("id"));
    assert_eq!(p.material.material_type, Material::POLYLINE_OUTLINE_TYPE);
}

#[test]
fn test_sets_polyline_properties_and_loop_constraint() {
    let mut polylines = PolylineCollection::new();
    let idx = polylines.add(None);

    {
        let p = polylines.get_mut(idx).unwrap();
        p.set_show(false);
        p.set_positions(vec![
            Cartesian3::new(1.0, 2.0, 3.0),
            Cartesian3::new(4.0, 5.0, 6.0),
        ]);
        // When positions count <= 2, loop cannot be set to true
        p.set_loop(true);
        assert!(!p.loop_);

        // Now set > 2 positions
        p.set_positions(vec![
            Cartesian3::new(1.0, 2.0, 3.0),
            Cartesian3::new(4.0, 5.0, 6.0),
            Cartesian3::new(7.0, 8.0, 9.0),
        ]);
        p.set_loop(true);
        assert!(p.loop_);

        // If positions shrink back to <= 2, loop becomes false
        p.set_positions(vec![Cartesian3::new(1.0, 2.0, 3.0)]);
        assert!(!p.loop_);

        p.set_width(5.0);
        p.set_id(Some("new_id".to_string()));
    }

    let p = polylines.get(idx).unwrap();
    assert!(!p.show);
    assert_eq!(p.width, 5.0);
    assert_eq!(p.id.as_deref(), Some("new_id"));
}

#[test]
fn test_constructor_sets_loop_only_when_positions_greater_than_2() {
    let mut polylines = PolylineCollection::new();

    // 2 positions with loop = true => loop should be false
    let p1_idx = polylines.add(Some(PolylineOptions {
        positions: Some(vec![
            Cartesian3::new(1.0, 2.0, 3.0),
            Cartesian3::new(4.0, 5.0, 6.0),
        ]),
        loop_: Some(true),
        ..Default::default()
    }));
    assert!(!polylines.get(p1_idx).unwrap().loop_);

    // 3 positions with loop = true => loop should be true
    let p2_idx = polylines.add(Some(PolylineOptions {
        positions: Some(vec![
            Cartesian3::new(1.0, 2.0, 3.0),
            Cartesian3::new(4.0, 5.0, 6.0),
            Cartesian3::new(7.0, 8.0, 9.0),
        ]),
        loop_: Some(true),
        ..Default::default()
    }));
    assert!(polylines.get(p2_idx).unwrap().loop_);
}

#[test]
fn test_sets_removed_polyline_properties() {
    let mut polylines = PolylineCollection::new();
    let idx = polylines.add(None);
    let mut removed_p = polylines.get(idx).unwrap().clone();

    polylines.remove(idx);
    assert_eq!(polylines.len(), 0);

    // Modifying removed polyline succeeds without panicking
    removed_p.set_width(10.0);
    removed_p.set_positions(vec![Cartesian3::new(0.0, 0.0, 0.0)]);
    assert_eq!(removed_p.width, 10.0);
    assert_eq!(removed_p.positions.len(), 1);
}

#[test]
fn test_collection_add_remove_and_contains() {
    let mut polylines = PolylineCollection::new();
    assert_eq!(polylines.len(), 0);
    assert!(polylines.is_empty());

    let i1 = polylines.add(None);
    let p1 = polylines.get(i1).unwrap().clone();
    let i2 = polylines.add(None);
    let p2 = polylines.get(i2).unwrap().clone();
    let i3 = polylines.add(None);
    let p3 = polylines.get(i3).unwrap().clone();

    assert_eq!(polylines.len(), 3);
    assert!(polylines.contains(Some(&p1)));
    assert!(polylines.contains(Some(&p2)));
    assert!(polylines.contains(Some(&p3)));

    // Remove first
    assert!(polylines.remove_polyline(Some(&p1)));
    assert_eq!(polylines.len(), 2);
    assert!(!polylines.contains(Some(&p1)));

    // Remove same twice returns false
    assert!(!polylines.remove_polyline(Some(&p1)));

    // Remove last
    assert!(polylines.remove_polyline(Some(&p3)));
    assert_eq!(polylines.len(), 1);

    // Remove undefined returns false
    assert!(!polylines.remove_polyline(None));
    assert!(!polylines.contains(None));

    // Remove all
    polylines.remove_all();
    assert_eq!(polylines.len(), 0);
    assert!(!polylines.contains(Some(&p2)));
}

#[test]
fn test_get_validations() {
    let mut polylines = PolylineCollection::new();
    let idx = polylines.add(None);

    assert!(polylines.get(None).is_err());
    assert!(polylines.get(Some(5)).is_err());
    assert!(polylines.get(idx).is_ok());
}

#[test]
fn test_visibility_and_picking_conditions() {
    let mut p = cesium_renderer::polyline_collection::Polyline::default();
    assert!(p.is_visible_for_picking());

    // Hide polyline
    p.set_show(false);
    assert!(!p.is_visible_for_picking());
    p.set_show(true);

    // Set alpha to 0.0
    let mut color_map = std::collections::HashMap::new();
    color_map.insert("red".to_string(), serde_json::json!(1.0));
    color_map.insert("green".to_string(), serde_json::json!(1.0));
    color_map.insert("blue".to_string(), serde_json::json!(1.0));
    color_map.insert("alpha".to_string(), serde_json::json!(0.0));
    p.material.uniforms.insert("color".to_string(), serde_json::json!(color_map));

    assert!(!p.is_visible_for_picking());
}

#[test]
fn test_bounding_spheres_3d_2d_and_columbus_view() {
    let mut polylines = PolylineCollection::new();
    let ellipsoid = Ellipsoid::WGS84;

    let p1 = Cartesian3::from_degrees(-50.0, -50.0, Some(0.0), Some(&ellipsoid)).unwrap();
    let p2 = Cartesian3::from_degrees(-50.0, 50.0, Some(0.0), Some(&ellipsoid)).unwrap();

    polylines.add(Some(PolylineOptions {
        positions: Some(vec![p1, p2]),
        width: Some(4.0),
        ..Default::default()
    }));

    // Empty or 1-vertex polylines should not crash bounding sphere computation
    polylines.add(Some(PolylineOptions {
        positions: Some(vec![]),
        ..Default::default()
    }));
    polylines.add(Some(PolylineOptions {
        positions: Some(vec![p1]),
        ..Default::default()
    }));

    // 3D bounding sphere
    let bs3d = polylines.compute_bounding_sphere_3d().expect("3D sphere");
    assert!(bs3d.radius > 0.0);

    // 2D bounding sphere
    let bs2d = polylines
        .compute_bounding_sphere_2d(&ellipsoid)
        .expect("2D sphere");
    assert_eq!(bs2d.center.x, 0.0);
    assert!(bs2d.radius > 0.0);

    // Columbus view bounding sphere
    let bscv = polylines
        .compute_bounding_sphere_columbus_view(&ellipsoid)
        .expect("CV sphere");
    assert!(bscv.radius > 0.0);

    // Model matrix affects 3D bounding sphere
    polylines.model_matrix = Some(DMat4::from_translation(glam::DVec3::new(100.0, 0.0, 0.0)));
    let bs3d_transformed = polylines.compute_bounding_sphere_3d().unwrap();
    assert_eq!(
        bs3d_transformed.center.x,
        bs3d.center.x + 100.0
    );

    // Per material bounding volumes
    let per_mat = polylines.compute_bounding_spheres_per_material();
    assert!(!per_mat.is_empty());

    // Destruction
    polylines.destroy();
    assert!(polylines.is_destroyed());
    assert_eq!(polylines.len(), 0);
}
