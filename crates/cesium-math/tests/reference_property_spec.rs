//! Direct port and verification against CesiumJS `ReferencePropertySpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    BillboardGraphics, Cartesian3, Color, ColorMaterialProperty,
    ConstantPositionProperty, ConstantProperty, DynProperty, Entity, EntityCollection, JulianDate,
    ReferenceFrame, ReferenceProperty, ResolvedPropertyTarget,
};

#[test]
fn test_constructor_sets_expected_values() {
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let object_id = "testId";
    let property_names = vec!["foo".to_string(), "bar".to_string(), "baz".to_string()];
    let property =
        ReferenceProperty::new(Rc::clone(&collection), object_id, property_names.clone()).unwrap();

    assert_eq!(
        property.target_collection().borrow().id(),
        collection.borrow().id()
    );
    assert_eq!(property.target_id(), object_id);
    assert_eq!(property.target_property_names(), &property_names[..]);
}

#[test]
fn test_from_string_sets_expected_values() {
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let object_id = "testId";
    let property_names = ["foo", "bar", "baz"];

    let property =
        ReferenceProperty::from_string(Rc::clone(&collection), "testId#foo.bar.baz").unwrap();

    assert_eq!(
        property.target_collection().borrow().id(),
        collection.borrow().id()
    );
    assert_eq!(property.target_id(), object_id);
    assert_eq!(property.target_property_names(), property_names);
}

#[test]
fn test_from_string_works_with_escaped_values() {
    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    let object_id = "#identif\\#ier.";
    let property_names = ["propertyName", ".abc\\", "def"];

    let property = ReferenceProperty::from_string(
        Rc::clone(&collection),
        "\\#identif\\\\\\#ier\\.#propertyName.\\.abc\\\\.def",
    )
    .unwrap();

    assert_eq!(
        property.target_collection().borrow().id(),
        collection.borrow().id()
    );
    assert_eq!(property.target_id(), object_id);
    assert_eq!(property.target_property_names(), property_names);
}

#[test]
fn test_properly_tracks_resolved_property() {
    let time = JulianDate::now();
    let mut test_object = Entity::with_id("testId");

    let scale_prop: Rc<dyn DynProperty<f64>> = Rc::new(ConstantProperty::from_value(5.0));
    let mut billboard = BillboardGraphics::default();
    billboard.scale = Some(Rc::clone(&scale_prop));
    test_object.billboard = Some(billboard);

    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    collection.borrow_mut().add(test_object.clone()).unwrap();

    // Basic property resolution
    let property =
        ReferenceProperty::from_string(Rc::clone(&collection), "testId#billboard.scale").unwrap();
    assert!(property.reference_frame().is_none());
    assert!(property.is_constant());
    assert!(property.has_resolved_property());
    assert_eq!(property.get_value::<f64>(Some(&time)), Some(5.0));

    let call_count = Rc::new(RefCell::new(0usize));
    let cc = Rc::clone(&call_count);
    property
        .definition_changed()
        .add_listener(move |_| *cc.borrow_mut() += 1);

    // A change to existing target property is reflected in reference.
    if let Some(ResolvedPropertyTarget::F64(resolved)) = property.resolve() {
        if let Some(cp) = (resolved.as_ref() as &dyn std::any::Any)
            .downcast_ref::<ConstantProperty<f64>>()
        {
            cp.set_value(Some(6.0));
        }
    }
    // Also notify target definition changed
    test_object.definition_changed.raise_event(&"billboard".to_string());
    assert_eq!(*call_count.borrow(), 1);
    assert!(property.is_constant());
    assert_eq!(property.get_value::<f64>(Some(&time)), Some(6.0));
    *call_count.borrow_mut() = 0;

    // Assignment of new leaf property to existing target is reflected in reference.
    let new_scale_prop: Rc<dyn DynProperty<f64>> = Rc::new(ConstantProperty::from_value(7.0));
    if let Some(ent) = collection.borrow_mut().get_by_id_mut("testId") {
        if let Some(bb) = ent.billboard.as_mut() {
            bb.scale = Some(Rc::clone(&new_scale_prop));
        }
        ent.definition_changed.raise_event(&"billboard".to_string());
    }
    assert_eq!(*call_count.borrow(), 1);
    assert!(property.is_constant());
    assert_eq!(property.get_value::<f64>(Some(&time)), Some(7.0));
    *call_count.borrow_mut() = 0;

    // Assignment of non-leaf property to existing target is reflected in reference.
    let newer_scale_prop: Rc<dyn DynProperty<f64>> = Rc::new(ConstantProperty::from_value(8.0));
    let mut new_bb = BillboardGraphics::default();
    new_bb.scale = Some(Rc::clone(&newer_scale_prop));
    if let Some(ent) = collection.borrow_mut().get_by_id_mut("testId") {
        ent.set_billboard(Some(new_bb));
    }
    assert_eq!(*call_count.borrow(), 1);
    assert!(property.is_constant());
    assert_eq!(property.get_value::<f64>(Some(&time)), Some(8.0));
    *call_count.borrow_mut() = 0;

    // Removing an object should cause the reference to be severed without raising definition_changed.
    collection.borrow_mut().remove_by_id("testId");
    assert_eq!(*call_count.borrow(), 0);
    assert!(property.is_constant());
    assert_eq!(property.get_value::<f64>(Some(&time)), None);
    *call_count.borrow_mut() = 0;

    // Adding a new object should re-wire the reference and fire definition_changed.
    let mut test_object2 = Entity::with_id("testId");
    let mut bb2 = BillboardGraphics::default();
    bb2.scale = Some(Rc::new(ConstantProperty::from_value(9.0)));
    test_object2.billboard = Some(bb2);
    collection.borrow_mut().add(test_object2).unwrap();

    assert_eq!(*call_count.borrow(), 1);
    assert!(property.is_constant());
    assert_eq!(property.get_value::<f64>(Some(&time)), Some(9.0));
    *call_count.borrow_mut() = 0;

    // Setting the target property to undefined should cause the reference to be severed.
    if let Some(ent) = collection.borrow_mut().get_by_id_mut("testId") {
        if let Some(bb) = ent.billboard.as_mut() {
            bb.scale = None;
        }
        ent.definition_changed.raise_event(&"billboard".to_string());
    }
    assert_eq!(*call_count.borrow(), 1);
    assert!(property.is_constant());
    assert_eq!(property.get_value::<f64>(Some(&time)), None);
    *call_count.borrow_mut() = 0;

    // Assigning a valid property should re-connect the reference.
    if let Some(ent) = collection.borrow_mut().get_by_id_mut("testId") {
        if let Some(bb) = ent.billboard.as_mut() {
            bb.scale = Some(Rc::new(ConstantProperty::from_value(10.0)));
        }
        ent.definition_changed.raise_event(&"billboard".to_string());
    }
    assert_eq!(*call_count.borrow(), 1);
    assert!(property.is_constant());
    assert_eq!(property.get_value::<f64>(Some(&time)), Some(10.0));
}

#[test]
fn test_works_with_position_properties() {
    let time = JulianDate::now();
    let mut test_object = Entity::with_id("testId");
    test_object.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1.0, 2.0, 3.0)),
        Some(ReferenceFrame::Fixed),
    ));

    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    collection.borrow_mut().add(test_object.clone()).unwrap();

    let property = ReferenceProperty::from_string(Rc::clone(&collection), "testId#position").unwrap();
    assert!(property.is_constant());
    assert_eq!(property.reference_frame(), Some(ReferenceFrame::Fixed));
    assert_eq!(
        property.get_value::<Cartesian3>(Some(&time)),
        test_object.position.as_ref().unwrap().get_value(Some(&time))
    );
    assert_eq!(
        property.get_value_in_reference_frame(Some(&time), ReferenceFrame::Inertial),
        test_object
            .position
            .as_ref()
            .unwrap()
            .get_value_in_reference_frame(Some(&time), ReferenceFrame::Inertial)
    );

    let nonexistent =
        ReferenceProperty::from_string(Rc::clone(&collection), "nonExistent#position").unwrap();
    assert!(nonexistent.is_constant());
    assert!(nonexistent.reference_frame().is_none());
    assert_eq!(nonexistent.get_value::<Cartesian3>(Some(&time)), None);
    assert_eq!(
        nonexistent.get_value_in_reference_frame(Some(&time), ReferenceFrame::Inertial),
        None
    );
}

#[test]
fn test_works_with_material_properties() {
    let time = JulianDate::now();
    let mut test_object = Entity::with_id("testId");
    test_object.add_property("testMaterial").unwrap();
    let color_mat = Rc::new(ColorMaterialProperty::from_color(Color::WHITE));
    test_object.set_material_property("testMaterial", Some(color_mat.clone()));

    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    collection.borrow_mut().add(test_object).unwrap();

    let property =
        ReferenceProperty::from_string(Rc::clone(&collection), "testId#testMaterial").unwrap();
    assert!(property.is_constant());
    assert_eq!(property.get_type(Some(&time)), Some("Color".to_string()));
    assert_eq!(property.get_value::<Color>(Some(&time)), Some(Color::WHITE));

    let nonexistent =
        ReferenceProperty::from_string(Rc::clone(&collection), "nonExistent#testMaterial").unwrap();
    assert!(nonexistent.is_constant());
    assert!(nonexistent.reference_frame().is_none());
    assert!(nonexistent.get_type(Some(&time)).is_none());
    assert_eq!(nonexistent.get_value::<Color>(Some(&time)), None);
}

#[test]
fn test_equals_works() {
    let entity_collection = Rc::new(RefCell::new(EntityCollection::new()));

    let left =
        ReferenceProperty::from_string(Rc::clone(&entity_collection), "objectId#foo.bar").unwrap();
    let mut right =
        ReferenceProperty::from_string(Rc::clone(&entity_collection), "objectId#foo.bar").unwrap();
    assert_eq!(left, right);
    assert!(left.equals(&right));

    // collection differs
    let other_col = Rc::new(RefCell::new(EntityCollection::new()));
    right = ReferenceProperty::from_string(other_col, "objectId#foo.bar").unwrap();
    assert_ne!(left, right);

    // target id differs
    right = ReferenceProperty::from_string(
        Rc::clone(&entity_collection),
        "otherObjectId#foo.bar",
    )
    .unwrap();
    assert_ne!(left, right);

    // number of sub-properties differ
    right = ReferenceProperty::from_string(Rc::clone(&entity_collection), "objectId#foo").unwrap();
    assert_ne!(left, right);

    // sub-properties of same length differ
    right =
        ReferenceProperty::from_string(Rc::clone(&entity_collection), "objectId#foo.baz").unwrap();
    assert_ne!(left, right);
}

#[test]
fn test_does_not_raise_definition_changed_when_target_entity_has_not_changed() {
    let mut test_object = Entity::with_id("testId");
    let mut bb = BillboardGraphics::default();
    bb.scale = Some(Rc::new(ConstantProperty::from_value(5.0)));
    test_object.billboard = Some(bb);

    let other_object = Entity::with_id("other");

    let collection = Rc::new(RefCell::new(EntityCollection::new()));
    collection.borrow_mut().add(test_object).unwrap();
    collection.borrow_mut().add(other_object).unwrap();

    let property =
        ReferenceProperty::from_string(Rc::clone(&collection), "testId#billboard.scale").unwrap();
    assert!(property.has_resolved_property());

    let call_count = Rc::new(RefCell::new(0usize));
    let cc = Rc::clone(&call_count);
    property
        .definition_changed()
        .add_listener(move |_| *cc.borrow_mut() += 1);

    collection.borrow_mut().remove_by_id("other");
    assert_eq!(*call_count.borrow(), 0);

    collection.borrow_mut().remove_by_id("testId");
    assert_eq!(*call_count.borrow(), 0);
}

#[test]
fn test_attaches_to_a_target_entity_created_later() {
    let time = JulianDate::now();
    let collection = Rc::new(RefCell::new(EntityCollection::new()));

    let property =
        ReferenceProperty::from_string(Rc::clone(&collection), "testId#billboard.scale").unwrap();
    assert!(!property.has_resolved_property());

    let call_count = Rc::new(RefCell::new(0usize));
    let cc = Rc::clone(&call_count);
    property
        .definition_changed()
        .add_listener(move |_| *cc.borrow_mut() += 1);

    let other_object = Entity::with_id("other");
    collection.borrow_mut().add(other_object).unwrap();

    assert_eq!(*call_count.borrow(), 0);
    assert!(!property.has_resolved_property());

    let mut test_object = Entity::with_id("testId");
    let mut bb = BillboardGraphics::default();
    bb.scale = Some(Rc::new(ConstantProperty::from_value(5.0)));
    test_object.billboard = Some(bb);
    collection.borrow_mut().add(test_object).unwrap();

    assert_eq!(*call_count.borrow(), 1);
    assert!(property.has_resolved_property());
    assert_eq!(property.get_value::<f64>(Some(&time)), Some(5.0));
}

#[test]
fn test_constructor_and_from_string_validation_errors() {
    let collection = Rc::new(RefCell::new(EntityCollection::new()));

    // Empty target_id
    assert!(ReferenceProperty::new(
        Rc::clone(&collection),
        "",
        vec!["property".to_string()]
    )
    .is_err());

    // Empty targetPropertyNames
    assert!(ReferenceProperty::new(Rc::clone(&collection), "objectId", vec![]).is_err());

    // Invalid property item
    assert!(ReferenceProperty::new(
        Rc::clone(&collection),
        "objectId",
        vec!["".to_string()]
    )
    .is_err());

    // Invalid referenceStrings
    assert!(ReferenceProperty::from_string(Rc::clone(&collection), "").is_err());
    assert!(ReferenceProperty::from_string(Rc::clone(&collection), "foo").is_err());
    assert!(ReferenceProperty::from_string(Rc::clone(&collection), "foo#").is_err());
    assert!(ReferenceProperty::from_string(Rc::clone(&collection), "#bar").is_err());
}

#[test]
fn test_get_value_returns_none_when_unresolvable() {
    let time = JulianDate::now();
    let collection = Rc::new(RefCell::new(EntityCollection::new()));

    // Target entity not in collection
    let prop1 =
        ReferenceProperty::from_string(Rc::clone(&collection), "testId#foo.bar").unwrap();
    assert_eq!(prop1.get_value::<f64>(Some(&time)), None);

    // Target property not resolved
    let test_object = Entity::with_id("testId");
    collection.borrow_mut().add(test_object).unwrap();
    let prop2 =
        ReferenceProperty::from_string(Rc::clone(&collection), "testId#billboard").unwrap();
    assert_eq!(prop2.get_value::<f64>(Some(&time)), None);

    // Sub-property not resolved
    let mut test_object2 = Entity::with_id("testId2");
    test_object2.billboard = Some(BillboardGraphics::default());
    collection.borrow_mut().add(test_object2).unwrap();
    let prop3 =
        ReferenceProperty::from_string(Rc::clone(&collection), "testId2#billboard.foo").unwrap();
    assert_eq!(prop3.get_value::<f64>(Some(&time)), None);
}
