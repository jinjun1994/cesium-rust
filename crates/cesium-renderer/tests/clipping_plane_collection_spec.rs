use cesium_math::{
    Cartesian2, Cartesian3, Color, Matrix4, Sphere,
};
use cesium_renderer::{
    ClippingPlane, ClippingPlaneCollection, ClippingPlaneCollectionOptions, Intersect,
};
use glam::DVec3;
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn test_default_constructor() {
    let collection = ClippingPlaneCollection::default();
    assert_eq!(collection.len(), 0);
    assert!(collection.is_empty());
    assert!(collection.enabled);
    assert_eq!(collection.model_matrix, Matrix4::IDENTITY);
    assert_eq!(collection.edge_color, Color::WHITE);
    assert_eq!(collection.edge_width, 0.0);
    assert!(!collection.union_clipping_regions);
}

#[test]
fn test_length_and_add() {
    let mut collection = ClippingPlaneCollection::default();
    assert_eq!(collection.len(), 0);

    let plane_x = ClippingPlane::new(Cartesian3::UNIT_X, 1.0);
    let plane_y = ClippingPlane::new(Cartesian3::UNIT_Y, 2.0);

    let added_count = Rc::new(Cell::new(0));
    let added_clone = added_count.clone();
    let _ = collection.plane_added.add_listener(move |_idx| {
        added_clone.set(added_clone.get() + 1);
    });

    collection.add(plane_x.clone());
    assert_eq!(collection.len(), 1);
    assert_eq!(added_count.get(), 1);
    assert_eq!(collection.get(0), Some(&plane_x));

    collection.add(plane_y.clone());
    assert_eq!(collection.len(), 2);
    assert_eq!(added_count.get(), 2);
    assert_eq!(collection.get(1), Some(&plane_y));
    assert_eq!(collection.get(2), None);
}

#[test]
fn test_contains_and_remove() {
    let plane_x = ClippingPlane::new(Cartesian3::UNIT_X, 1.0);
    let plane_y = ClippingPlane::new(Cartesian3::UNIT_Y, 2.0);
    let plane_z = ClippingPlane::new(Cartesian3::UNIT_Z, 3.0);

    let mut collection = ClippingPlaneCollection::new(Some(ClippingPlaneCollectionOptions {
        planes: Some(vec![plane_x.clone(), plane_y.clone()]),
        ..Default::default()
    }));

    assert!(collection.contains(&plane_x));
    assert!(collection.contains(&ClippingPlane::new(Cartesian3::UNIT_Y, 2.0)));
    assert!(!collection.contains(&plane_z));

    let removed_count = Rc::new(Cell::new(0));
    let removed_clone = removed_count.clone();
    let _ = collection.plane_removed.add_listener(move |_idx| {
        removed_clone.set(removed_clone.get() + 1);
    });

    assert!(collection.remove(&plane_x));
    assert!(!collection.contains(&plane_x));
    assert_eq!(collection.len(), 1);
    assert_eq!(collection.get(0), Some(&plane_y));
    assert_eq!(removed_count.get(), 1);

    // Remove already removed plane
    assert!(!collection.remove(&plane_x));
    assert_eq!(removed_count.get(), 1);

    // Remove all
    collection.remove_all();
    assert_eq!(collection.len(), 0);
    assert!(collection.is_empty());
    assert_eq!(removed_count.get(), 2);
}

#[test]
fn test_intersection_with_bounding_volumes_intersection_mode() {
    let mut collection = ClippingPlaneCollection::new(Some(ClippingPlaneCollectionOptions {
        union_clipping_regions: Some(false),
        ..Default::default()
    }));

    let bounding_volume = Sphere::new(DVec3::ZERO, 1.0);

    let intersect = collection.compute_intersection_with_bounding_volume(&bounding_volume, None);
    assert_eq!(intersect, Intersect::Inside);

    collection.add(ClippingPlane::new(Cartesian3::UNIT_Y, 0.0));
    let intersect = collection.compute_intersection_with_bounding_volume(&bounding_volume, None);
    assert_eq!(intersect, Intersect::Intersecting);

    collection.add(ClippingPlane::new(Cartesian3::UNIT_Z, 1.0));
    let intersect = collection.compute_intersection_with_bounding_volume(&bounding_volume, None);
    assert_eq!(intersect, Intersect::Inside);

    collection.add(ClippingPlane::new(Cartesian3::UNIT_Z, 0.0));
    let intersect = collection.compute_intersection_with_bounding_volume(&bounding_volume, None);
    assert_eq!(intersect, Intersect::Inside);
}

#[test]
fn test_intersection_with_bounding_volumes_union_mode() {
    let mut collection = ClippingPlaneCollection::new(Some(ClippingPlaneCollectionOptions {
        union_clipping_regions: Some(true),
        ..Default::default()
    }));

    let bounding_volume = Sphere::new(DVec3::ZERO, 1.0);

    let intersect = collection.compute_intersection_with_bounding_volume(&bounding_volume, None);
    assert_eq!(intersect, Intersect::Inside);

    collection.add(ClippingPlane::new(Cartesian3::UNIT_Z, 1.0));
    let intersect = collection.compute_intersection_with_bounding_volume(&bounding_volume, None);
    assert_eq!(intersect, Intersect::Inside);

    let temp = ClippingPlane::new(Cartesian3::UNIT_Y, -2.0);
    collection.add(temp.clone());
    let intersect = collection.compute_intersection_with_bounding_volume(&bounding_volume, None);
    assert_eq!(intersect, Intersect::Outside);

    collection.add(ClippingPlane::new(Cartesian3::UNIT_X, 0.0));
    let intersect = collection.compute_intersection_with_bounding_volume(&bounding_volume, None);
    assert_eq!(intersect, Intersect::Outside);

    collection.remove(&temp);
    let intersect = collection.compute_intersection_with_bounding_volume(&bounding_volume, None);
    assert_eq!(intersect, Intersect::Intersecting);
}

#[test]
fn test_intersection_with_optional_transform() {
    let mut collection = ClippingPlaneCollection::default();
    let bounding_volume = Sphere::new(DVec3::ZERO, 1.0);
    let transform = Matrix4::from_translation(Cartesian3::new(1.0, 3.0, 2.0));

    let intersect = collection.compute_intersection_with_bounding_volume(&bounding_volume, Some(&transform));
    assert_eq!(intersect, Intersect::Inside);

    collection.add(ClippingPlane::new(Cartesian3::UNIT_X, -1.0));
    let intersect = collection.compute_intersection_with_bounding_volume(&bounding_volume, Some(&transform));
    assert_ne!(intersect, Intersect::Inside);
}

#[test]
fn test_clipping_planes_state() {
    let mut collection = ClippingPlaneCollection::default();
    assert_eq!(collection.clipping_planes_state(), 0);

    collection.add(ClippingPlane::new(Cartesian3::UNIT_X, -1.0));
    assert_eq!(collection.clipping_planes_state(), -1);

    let hold = ClippingPlane::new(Cartesian3::UNIT_X, -1.0);
    collection.add(hold.clone());
    assert_eq!(collection.clipping_planes_state(), -2);

    collection.union_clipping_regions = true;
    assert_eq!(collection.clipping_planes_state(), 2);

    collection.remove(&hold);
    assert_eq!(collection.clipping_planes_state(), 1);
}

#[test]
fn test_texture_resolution() {
    let mut collection = ClippingPlaneCollection::new(Some(ClippingPlaneCollectionOptions {
        planes: Some(vec![
            ClippingPlane::new(Cartesian3::UNIT_X, 1.0),
            ClippingPlane::new(Cartesian3::UNIT_Y, 2.0),
        ]),
        ..Default::default()
    }));

    // uint8 mode: 2 planes -> 4 pixels needed -> (4, 2)
    let res_uint8 = collection.get_texture_resolution(false, None);
    assert_eq!(res_uint8, Cartesian2::new(4.0, 2.0));

    // float mode: 2 planes -> 2 pixels needed -> (2, 2)
    let res_float = collection.get_texture_resolution(true, None);
    assert_eq!(res_float, Cartesian2::new(2.0, 2.0));

    // Destroy
    assert!(!collection.is_destroyed());
    collection.destroy();
    assert!(collection.is_destroyed());
    assert_eq!(collection.len(), 0);
}
