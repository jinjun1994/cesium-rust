use cesium_viewer::widgets::knockout::{
    Knockout, KnockoutValue, TrackableObject,
};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_knockout_can_track_all_properties() {
    let obj = Rc::new(RefCell::new(TrackableObject::new()));

    obj.borrow().set_property("one", KnockoutValue::Number(1.0));
    obj.borrow().set_property("two", KnockoutValue::Undefined);
    obj.borrow().set_property(
        "three",
        KnockoutValue::Observable(Knockout::observable(KnockoutValue::Undefined)),
    );

    assert!(Knockout::get_observable(&obj, "one").is_none());
    assert!(Knockout::get_observable(&obj, "two").is_none());
    assert!(Knockout::get_observable(&obj, "three").is_none());

    Knockout::track(&obj);

    assert!(Knockout::get_observable(&obj, "one").is_some());
    assert!(Knockout::get_observable(&obj, "two").is_some());
    assert!(Knockout::get_observable(&obj, "three").is_some());

    let obs_one = Knockout::get_observable(&obj, "one").unwrap();
    if let KnockoutValue::Number(val) = obs_one.get() {
        assert_eq!(val, 1.0);
    } else {
        panic!("Expected number value");
    }
}
