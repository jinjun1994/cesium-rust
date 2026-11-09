use cesium_math::{Cartesian2, Cartesian3, Color};
use cesium_renderer::{
    CloudCollection, CloudCollectionOptions, CloudType, CumulusCloudOptions,
};

#[test]
fn test_constructs_default_cloud() {
    let mut clouds = CloudCollection::default();
    let c = clouds.add(None).expect("add default cloud");

    assert!(c.show());
    assert_eq!(c.position(), Cartesian3::ZERO);
    assert_eq!(c.scale(), Cartesian2::new(20.0, 12.0));
    assert_eq!(c.maximum_size(), Cartesian3::new(20.0, 12.0, 8.0));
    assert_eq!(c.slice(), -1.0);
    assert_eq!(c.brightness(), 1.0);
    assert_eq!(c.color(), Color::WHITE);
    assert_eq!(c.cloud_type(), CloudType::Cumulus);
}

#[test]
fn test_explicitly_constructs_cloud() {
    let mut clouds = CloudCollection::default();
    let c = clouds
        .add(Some(CumulusCloudOptions {
            cloud_type: Some(CloudType::Cumulus),
            show: Some(false),
            position: Some(Cartesian3::new(1.0, 2.0, 3.0)),
            scale: Some(Cartesian2::new(2.0, 3.0)),
            maximum_size: Some(Cartesian3::new(4.0, 5.0, 6.0)),
            slice: Some(0.5),
            brightness: Some(0.7),
            color: Some(Color::RED),
        }))
        .expect("add explicit cloud");

    assert!(!c.show());
    assert_eq!(c.position(), Cartesian3::new(1.0, 2.0, 3.0));
    assert_eq!(c.scale(), Cartesian2::new(2.0, 3.0));
    assert_eq!(c.maximum_size(), Cartesian3::new(4.0, 5.0, 6.0));
    assert_eq!(c.slice(), 0.5);
    assert_eq!(c.brightness(), 0.7);
    assert_eq!(c.color(), Color::RED);
}

#[test]
fn test_constructs_cloud_with_only_maximum_size() {
    let mut clouds = CloudCollection::default();
    let c = clouds
        .add(Some(CumulusCloudOptions {
            maximum_size: Some(Cartesian3::new(1.0, 2.0, 3.0)),
            ..Default::default()
        }))
        .expect("add cloud with max size");

    assert_eq!(c.maximum_size(), Cartesian3::new(1.0, 2.0, 3.0));
    assert_eq!(c.scale(), Cartesian2::new(1.0, 2.0));
}

#[test]
fn test_sets_cloud_properties() {
    let mut clouds = CloudCollection::default();
    let c = clouds.add(None).expect("add cloud");

    c.set_show(false);
    c.set_position(Cartesian3::new(1.0, 2.0, 3.0));
    c.set_scale(Cartesian2::new(2.0, 3.0));
    c.set_maximum_size(Cartesian3::new(4.0, 5.0, 6.0));
    c.set_slice(0.5);
    c.set_brightness(0.7);
    c.set_color(Color::BLUE);

    assert!(!c.show());
    assert_eq!(c.position(), Cartesian3::new(1.0, 2.0, 3.0));
    assert_eq!(c.scale(), Cartesian2::new(2.0, 3.0));
    assert_eq!(c.maximum_size(), Cartesian3::new(4.0, 5.0, 6.0));
    assert_eq!(c.slice(), 0.5);
    assert_eq!(c.brightness(), 0.7);
    assert_eq!(c.color(), Color::BLUE);
}

#[test]
fn test_sets_removed_cloud_property() {
    let mut clouds = CloudCollection::default();
    let c = clouds.add(None).expect("add cloud");

    assert!(clouds.remove(Some(&c)));
    c.set_show(false);
    assert!(!c.show());
}

#[test]
fn test_add_remove_and_contains() {
    let mut clouds = CloudCollection::default();
    assert_eq!(clouds.len(), 0);
    assert!(clouds.is_empty());

    let one = clouds
        .add(Some(CumulusCloudOptions {
            position: Some(Cartesian3::new(1.0, 2.0, 3.0)),
            ..Default::default()
        }))
        .expect("add one");
    assert_eq!(clouds.len(), 1);
    assert_eq!(clouds.get(Some(0)).unwrap(), one);

    let two = clouds
        .add(Some(CumulusCloudOptions {
            position: Some(Cartesian3::new(4.0, 5.0, 6.0)),
            ..Default::default()
        }))
        .expect("add two");
    assert_eq!(clouds.len(), 2);
    assert!(clouds.contains(Some(&one)));
    assert!(clouds.contains(Some(&two)));

    // Remove first
    assert!(clouds.remove(Some(&one)));
    assert_eq!(clouds.len(), 1);
    assert_eq!(clouds.get(Some(0)).unwrap(), two);
    assert!(!clouds.contains(Some(&one)));

    // Remove twice
    assert!(!clouds.remove(Some(&one)));

    // Remove None
    assert!(!clouds.remove(None));
    assert!(!clouds.contains(None));

    // Remove all
    let three = clouds.add(None).expect("add three");
    assert_eq!(clouds.len(), 2);
    assert!(clouds.contains(Some(&three)));
    clouds.remove_all();
    assert_eq!(clouds.len(), 0);
    assert!(clouds.is_empty());
}

#[test]
fn test_cloud_rendering_commands() {
    let mut clouds = CloudCollection::new(CloudCollectionOptions {
        debug_billboards: Some(true),
        ..Default::default()
    });

    // Empty collection -> no draw command
    let (_, draw) = clouds.update().expect("update empty");
    assert!(draw.is_none());

    // Add a cloud
    let c = clouds
        .add(Some(CumulusCloudOptions {
            position: Some(Cartesian3::ZERO),
            scale: Some(Cartesian2::new(20.0, 12.0)),
            ..Default::default()
        }))
        .expect("add cloud");

    let (_, draw) = clouds.update().expect("update with cloud");
    assert!(draw.is_some());

    // c.show = false
    c.set_show(false);
    let (_, draw) = clouds.update().expect("update hidden");
    assert!(draw.is_none());
    c.set_show(true);

    // c.scale = ZERO
    c.set_scale(Cartesian2::ZERO);
    let (_, draw) = clouds.update().expect("update zero scale");
    assert!(draw.is_none());
    c.set_scale(Cartesian2::new(10.0, 5.0));
    let (_, draw) = clouds.update().expect("update valid scale");
    assert!(draw.is_some());

    // Invalid slice > 1.0
    c.set_slice(10.0);
    let (_, draw) = clouds.update().expect("update invalid slice");
    assert!(draw.is_none());
    c.set_slice(0.5);
    let (_, draw) = clouds.update().expect("update valid slice");
    assert!(draw.is_some());

    // Zero brightness
    c.set_brightness(0.0);
    let (_, draw) = clouds.update().expect("update zero brightness");
    assert!(draw.is_none());
    c.set_brightness(0.5);
    let (_, draw) = clouds.update().expect("update valid brightness");
    assert!(draw.is_some());

    // Collection show = false
    clouds.show = false;
    let (_, draw) = clouds.update().expect("update collection hidden");
    assert!(draw.is_none());
    clouds.show = true;

    // Remove all
    clouds.remove_all();
    let (_, draw) = clouds.update().expect("update after remove all");
    assert!(draw.is_none());
}

#[test]
fn test_compute_command_and_noise_texture() {
    let mut clouds = CloudCollection::new(CloudCollectionOptions {
        debug_billboards: Some(false),
        ..Default::default()
    });

    clouds
        .add(Some(CumulusCloudOptions {
            position: Some(Cartesian3::ZERO),
            scale: Some(Cartesian2::new(20.0, 12.0)),
            ..Default::default()
        }))
        .expect("add cloud");

    // Before texture is ready, compute command is produced
    let (compute, draw) = clouds.update().expect("update not ready");
    assert!(compute.is_some());
    assert!(draw.is_none());

    // Once texture is ready, draw command is produced
    clouds.set_ready(true);
    let (compute, draw) = clouds.update().expect("update ready");
    assert!(compute.is_none());
    assert!(draw.is_some());
}

#[test]
fn test_developer_errors() {
    let mut clouds = CloudCollection::default();

    // Access without index
    assert!(clouds.get(None).is_err());

    // Invalid number of noise texture rows (8 / 16 < 1)
    clouds.debug_billboards = false;
    clouds.set_texture_slice_width(8);
    clouds.set_noise_texture_rows(16);
    assert!(clouds.update().is_err());

    // Destroy
    assert!(!clouds.is_destroyed());
    clouds.destroy();
    assert!(clouds.is_destroyed());
    assert!(clouds.add(None).is_err());
    assert!(clouds.get(Some(0)).is_err());
    assert!(clouds.update().is_err());
}
