//! Direct port and verification against CesiumJS `KmlTourSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    BoundingSphere, Cartesian3, HeadingPitchRange, KmlCameraOptions, KmlCameraTarget, KmlLookAt,
    KmlTour, KmlTourFlyTo, KmlTourWait,
};

#[derive(Default)]
struct MockCamera {
    fly_to_count: usize,
    fly_to_bounding_sphere_count: usize,
    cancel_flight_count: usize,
}

impl KmlCameraTarget for MockCamera {
    fn fly_to(&mut self, _options: &KmlCameraOptions, on_complete: Box<dyn FnOnce(bool)>) {
        self.fly_to_count += 1;
        on_complete(false);
    }

    fn fly_to_bounding_sphere(
        &mut self,
        _bounding_sphere: &BoundingSphere,
        _options: &KmlCameraOptions,
        on_complete: Box<dyn FnOnce(bool)>,
    ) {
        self.fly_to_bounding_sphere_count += 1;
        on_complete(false);
    }

    fn cancel_flight(&mut self) {
        self.cancel_flight_count += 1;
    }
}

fn get_look_at() -> KmlLookAt {
    let position = Cartesian3::from_degrees(40.0, 30.0, Some(1000.0), None).unwrap();
    let hpr = HeadingPitchRange::new(
        10.0_f64.to_radians(),
        45.0_f64.to_radians(),
        10000.0,
    );
    KmlLookAt::new(position, hpr)
}

#[test]
fn test_add_entries_to_playlist() {
    let mut tour = KmlTour::new("test", "test");
    let wait = KmlTourWait::new(10.0);
    let fly_to = KmlTourFlyTo::new(5.0, "bounce", get_look_at());

    tour.add_playlist_entry(wait);
    tour.add_playlist_entry(fly_to);

    assert_eq!(tour.playlist.len(), 2);
    assert!(tour.playlist[0].is_wait());
    assert_eq!(tour.playlist[0].as_wait().unwrap().duration, 10.0);
    assert!(tour.playlist[1].is_fly_to());
    assert_eq!(tour.playlist[1].as_fly_to().unwrap().duration, 5.0);
}

#[test]
fn test_calls_entries_play() {
    let mut tour = KmlTour::new("test", "test");
    let wait = KmlTourWait::new(0.1);
    let fly_to = KmlTourFlyTo::new(0.1, "bounce", get_look_at());

    tour.add_playlist_entry(wait);
    tour.add_playlist_entry(fly_to);

    let mut mock_camera = MockCamera::default();
    tour.play(&mut mock_camera, None);

    // Wait entry triggers step
    tour.step_next(false, &mut mock_camera, None);
    assert_eq!(mock_camera.fly_to_bounding_sphere_count, 1);
}

#[test]
fn test_calls_events() {
    let mut tour = KmlTour::new("test", "test");
    let wait1 = KmlTourWait::new(0.05);
    let wait2 = KmlTourWait::new(0.02);

    tour.add_playlist_entry(wait1);
    tour.add_playlist_entry(wait2);

    let tour_start_called = Rc::new(RefCell::new(0usize));
    let tour_end_called = Rc::new(RefCell::new(Vec::new()));
    let entry_start_called = Rc::new(RefCell::new(Vec::new()));
    let entry_end_called = Rc::new(RefCell::new(Vec::new()));

    let t_start = Rc::clone(&tour_start_called);
    tour.tour_start.add_listener(move |_| {
        *t_start.borrow_mut() += 1;
    });

    let t_end = Rc::clone(&tour_end_called);
    tour.tour_end.add_listener(move |&terminated| {
        t_end.borrow_mut().push(terminated);
    });

    let e_start = Rc::clone(&entry_start_called);
    tour.entry_start.add_listener(move |&idx| {
        e_start.borrow_mut().push(idx);
    });

    let e_end = Rc::clone(&entry_end_called);
    tour.entry_end.add_listener(move |&(idx, terminated)| {
        e_end.borrow_mut().push((idx, terminated));
    });

    let mut mock_camera = MockCamera::default();
    tour.play(&mut mock_camera, None);
    assert_eq!(*tour_start_called.borrow(), 1);
    assert_eq!(*entry_start_called.borrow(), vec![0]);

    // Finish entry 0
    tour.step_next(false, &mut mock_camera, None);
    assert_eq!(*entry_end_called.borrow(), vec![(0, false)]);
    assert_eq!(*entry_start_called.borrow(), vec![0, 1]);

    // Finish entry 1
    tour.step_next(false, &mut mock_camera, None);
    assert_eq!(*entry_end_called.borrow(), vec![(0, false), (1, false)]);
    assert_eq!(*tour_end_called.borrow(), vec![false]);
}

#[test]
fn test_terminates_playback() {
    let mut tour = KmlTour::new("test", "test");
    let wait = KmlTourWait::new(60.0);
    let fly_to = KmlTourFlyTo::new(0.1, "bounce", get_look_at());

    tour.add_playlist_entry(wait);
    tour.add_playlist_entry(fly_to);

    let tour_start_called = Rc::new(RefCell::new(0usize));
    let tour_end_called = Rc::new(RefCell::new(Vec::new()));
    let entry_start_called = Rc::new(RefCell::new(Vec::new()));
    let entry_end_called = Rc::new(RefCell::new(Vec::new()));

    let t_start = Rc::clone(&tour_start_called);
    tour.tour_start.add_listener(move |_| {
        *t_start.borrow_mut() += 1;
    });

    let t_end = Rc::clone(&tour_end_called);
    tour.tour_end.add_listener(move |&terminated| {
        t_end.borrow_mut().push(terminated);
    });

    let e_start = Rc::clone(&entry_start_called);
    tour.entry_start.add_listener(move |&idx| {
        e_start.borrow_mut().push(idx);
    });

    let e_end = Rc::clone(&entry_end_called);
    tour.entry_end.add_listener(move |&(idx, terminated)| {
        e_end.borrow_mut().push((idx, terminated));
    });

    let mut mock_camera = MockCamera::default();
    tour.play(&mut mock_camera, None);

    assert_eq!(*tour_start_called.borrow(), 1);
    assert_eq!(*entry_start_called.borrow(), vec![0]);

    tour.stop(Some(&mut mock_camera));

    assert_eq!(*entry_end_called.borrow(), vec![(0, true)]);
    assert_eq!(*tour_end_called.borrow(), vec![true]);
    assert_eq!(*entry_start_called.borrow(), vec![0]);
    assert_eq!(mock_camera.fly_to_count, 0);
    assert_eq!(mock_camera.fly_to_bounding_sphere_count, 0);
    assert_eq!(mock_camera.cancel_flight_count, 1);
}
