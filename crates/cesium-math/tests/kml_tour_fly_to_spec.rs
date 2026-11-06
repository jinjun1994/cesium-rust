//! Direct port and verification against CesiumJS `KmlTourFlyToSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    BoundingSphere, Cartesian3, HeadingPitchRange, HeadingPitchRoll, KmlCamera, KmlCameraOptions,
    KmlCameraTarget, KmlLookAt, KmlTourFlyTo,
};

#[derive(Default)]
struct MockCamera {
    fly_to_called: bool,
    fly_to_destination: Option<Cartesian3>,
    fly_to_orientation: Option<HeadingPitchRoll>,

    fly_to_bounding_sphere_called: bool,
    bounding_sphere_center: Option<Cartesian3>,
    bounding_sphere_offset: Option<HeadingPitchRange>,
}

impl KmlCameraTarget for MockCamera {
    fn fly_to(&mut self, options: &KmlCameraOptions, on_complete: Box<dyn FnOnce(bool)>) {
        self.fly_to_called = true;
        self.fly_to_destination = options.destination;
        self.fly_to_orientation = options.orientation;
        on_complete(false);
    }

    fn fly_to_bounding_sphere(
        &mut self,
        bounding_sphere: &BoundingSphere,
        options: &KmlCameraOptions,
        on_complete: Box<dyn FnOnce(bool)>,
    ) {
        self.fly_to_bounding_sphere_called = true;
        self.bounding_sphere_center = Some(Cartesian3::new(
            bounding_sphere.center.x,
            bounding_sphere.center.y,
            bounding_sphere.center.z,
        ));
        self.bounding_sphere_offset = options.offset;
        on_complete(false);
    }

    fn cancel_flight(&mut self) {}
}

#[test]
fn test_generates_camera_options_for_kml_look_at() {
    let position = Cartesian3::from_degrees(40.0, 30.0, Some(1000.0), None).unwrap();
    let hpr = HeadingPitchRange::new(
        10.0_f64.to_radians(),
        45.0_f64.to_radians(),
        10000.0,
    );

    let flyto = KmlTourFlyTo::new(10.0, "bounce", KmlLookAt::new(position, hpr));
    let options = flyto.get_camera_options(None);

    assert_eq!(options.duration, 10.0);
    assert!(!options.has_complete);
    assert!(options.easing_function.is_none());
    assert_eq!(options.offset, Some(hpr));
    assert!(options.destination.is_none());
    assert!(options.orientation.is_none());
}

#[test]
fn test_generates_camera_options_for_kml_camera() {
    let position = Cartesian3::from_degrees(40.0, 30.0, Some(1000.0), None).unwrap();
    let hpr = HeadingPitchRoll::new(
        10.0_f64.to_radians(),
        45.0_f64.to_radians(),
        0.0,
    );

    let flyto = KmlTourFlyTo::new(10.0, "bounce", KmlCamera::new(position, hpr));
    let options = flyto.get_camera_options(None);

    assert_eq!(options.duration, 10.0);
    assert!(!options.has_complete);
    assert!(options.easing_function.is_none());
    assert!(options.offset.is_none());
    assert_eq!(options.destination.unwrap().x, position.x);
    assert_eq!(options.destination.unwrap().y, position.y);
    assert_eq!(options.destination.unwrap().z, position.z);
    assert_eq!(options.orientation, Some(hpr));
}

#[test]
fn test_adds_active_callback_to_options() {
    let position = Cartesian3::from_degrees(40.0, 30.0, Some(1000.0), None).unwrap();
    let hpr = HeadingPitchRange::new(
        10.0_f64.to_radians(),
        45.0_f64.to_radians(),
        10000.0,
    );

    let mut flyto = KmlTourFlyTo::new(10.0, "bounce", KmlLookAt::new(position, hpr));
    let called = Rc::new(RefCell::new(false));
    let called_clone = Rc::clone(&called);
    flyto.set_active_callback(move |_terminated| {
        *called_clone.borrow_mut() = true;
    });

    let options = flyto.get_camera_options(None);
    assert!(options.has_complete);

    flyto.invoke_complete(false);
    assert!(*called.borrow());
}

#[test]
fn test_calls_camera_fly_to_for_kml_camera() {
    let position = Cartesian3::from_degrees(40.0, 30.0, Some(1000.0), None).unwrap();
    let hpr = HeadingPitchRoll::new(
        10.0_f64.to_radians(),
        45.0_f64.to_radians(),
        0.0,
    );

    let mut flyto = KmlTourFlyTo::new(0.01, "bounce", KmlCamera::new(position, hpr));
    let done_called = Rc::new(RefCell::new(false));
    let done_clone = Rc::clone(&done_called);

    let mut mock_camera = MockCamera::default();
    flyto.play(
        Some(Box::new(move |_terminated| {
            *done_clone.borrow_mut() = true;
        })),
        &mut mock_camera,
        None,
    );

    assert!(mock_camera.fly_to_called);
    assert_eq!(mock_camera.fly_to_destination, Some(position));
    assert_eq!(mock_camera.fly_to_orientation, Some(hpr));
    assert!(*done_called.borrow());
}

#[test]
fn test_calls_camera_fly_to_bounding_sphere_for_kml_look_at() {
    let position = Cartesian3::from_degrees(40.0, 30.0, Some(1000.0), None).unwrap();
    let hpr = HeadingPitchRange::new(
        10.0_f64.to_radians(),
        45.0_f64.to_radians(),
        10000.0,
    );

    let mut flyto = KmlTourFlyTo::new(0.01, "bounce", KmlLookAt::new(position, hpr));
    let done_called = Rc::new(RefCell::new(false));
    let done_clone = Rc::clone(&done_called);

    let mut mock_camera = MockCamera::default();
    flyto.play(
        Some(Box::new(move |_terminated| {
            *done_clone.borrow_mut() = true;
        })),
        &mut mock_camera,
        None,
    );

    assert!(mock_camera.fly_to_bounding_sphere_called);
    let center = mock_camera.bounding_sphere_center.unwrap();
    assert_eq!(center.x, position.x);
    assert_eq!(center.y, position.y);
    assert_eq!(center.z, position.z);
    assert_eq!(mock_camera.bounding_sphere_offset, Some(hpr));
    assert!(*done_called.borrow());
}
