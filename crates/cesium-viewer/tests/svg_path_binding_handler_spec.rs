use cesium_viewer::widgets::knockout::{
    Knockout, KnockoutElement, KnockoutValue,
};
use cesium_viewer::widgets::svg_path_binding_handler::SvgPathBindingHandler;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_svg_path_binding_with_constants() {
    let div = KnockoutElement::new("div");
    div.borrow().set_attribute(
        "data-bind",
        "cesiumSvgPath: { path: \"M 100 100 L 300 100 L 200 300 Z\", width: 28, height: 40, css: \"someClass\" }",
    );

    SvgPathBindingHandler::apply_bindings(&HashMap::new(), &div);

    let svg = KnockoutElement::query_selector(&div, "svg.cesium-svgPath-svg");
    assert!(svg.is_some());
    let svg_node = svg.unwrap();
    assert_eq!(svg_node.borrow().get_attribute("width").as_deref(), Some("28"));
    assert_eq!(svg_node.borrow().get_attribute("height").as_deref(), Some("40"));

    let class_attr = svg_node.borrow().get_attribute("class").unwrap();
    let classes: Vec<&str> = class_attr.split_whitespace().collect();
    assert!(classes.contains(&"someClass"));

    let path = KnockoutElement::query_selector(&div, "svg > path");
    assert!(path.is_some());
    let path_node = path.unwrap();
    assert_eq!(
        path_node.borrow().get_attribute("d").as_deref(),
        Some("M 100 100 L 300 100 L 200 300 Z")
    );

    Knockout::clean_node(&div);
    assert!(div.borrow().children.borrow().is_empty());
}

#[test]
fn test_svg_path_binding_with_observables() {
    let div = KnockoutElement::new("div");
    div.borrow().set_attribute(
        "data-bind",
        "cesiumSvgPath: { path: p, width: w, height: h, css: c }",
    );

    let p = Knockout::observable(KnockoutValue::String(
        "M 100 100 L 300 100 L 200 300 Z".to_string(),
    ));
    let w = Knockout::observable(KnockoutValue::Number(28.0));
    let h = Knockout::observable(KnockoutValue::Number(40.0));
    let c = Knockout::observable(KnockoutValue::String("someClass".to_string()));

    let mut vm = HashMap::new();
    vm.insert("p".to_string(), KnockoutValue::Observable(Rc::clone(&p)));
    vm.insert("w".to_string(), KnockoutValue::Observable(Rc::clone(&w)));
    vm.insert("h".to_string(), KnockoutValue::Observable(Rc::clone(&h)));
    vm.insert("c".to_string(), KnockoutValue::Observable(Rc::clone(&c)));

    SvgPathBindingHandler::apply_bindings(&vm, &div);

    let svg = KnockoutElement::query_selector(&div, "svg.cesium-svgPath-svg").unwrap();
    assert_eq!(svg.borrow().get_attribute("width").as_deref(), Some("28"));
    assert_eq!(svg.borrow().get_attribute("height").as_deref(), Some("40"));

    let path = KnockoutElement::query_selector(&div, "svg > path").unwrap();
    assert_eq!(
        path.borrow().get_attribute("d").as_deref(),
        Some("M 100 100 L 300 100 L 200 300 Z")
    );

    // Reactive update
    p.set(KnockoutValue::String("M 0 0 L 10 10 Z".to_string()));
    assert_eq!(
        path.borrow().get_attribute("d").as_deref(),
        Some("M 0 0 L 10 10 Z")
    );

    w.set(KnockoutValue::Number(50.0));
    assert_eq!(svg.borrow().get_attribute("width").as_deref(), Some("50"));

    Knockout::clean_node(&div);
    assert!(div.borrow().children.borrow().is_empty());
}

#[test]
fn test_svg_path_binding_with_observable_parameter_object() {
    let div = KnockoutElement::new("div");
    div.borrow().set_attribute("data-bind", "cesiumSvgPath: svgPath");

    let mut inner_map = HashMap::new();
    inner_map.insert(
        "path".to_string(),
        KnockoutValue::Observable(Knockout::observable(KnockoutValue::String(
            "M 100 100 L 300 100 L 200 300 Z".to_string(),
        ))),
    );
    inner_map.insert(
        "width".to_string(),
        KnockoutValue::Observable(Knockout::observable(KnockoutValue::Number(28.0))),
    );
    inner_map.insert(
        "height".to_string(),
        KnockoutValue::Observable(Knockout::observable(KnockoutValue::Number(40.0))),
    );
    inner_map.insert(
        "css".to_string(),
        KnockoutValue::Observable(Knockout::observable(KnockoutValue::String(
            "someClass".to_string(),
        ))),
    );

    let svg_path_obj = Knockout::observable(KnockoutValue::Object(Rc::new(RefCell::new(inner_map))));

    let mut vm = HashMap::new();
    vm.insert("svgPath".to_string(), KnockoutValue::Observable(svg_path_obj));

    SvgPathBindingHandler::apply_bindings(&vm, &div);

    let svg = KnockoutElement::query_selector(&div, "svg.cesium-svgPath-svg");
    assert!(svg.is_some());
    let svg_node = svg.unwrap();
    assert_eq!(svg_node.borrow().get_attribute("width").as_deref(), Some("28"));
    assert_eq!(svg_node.borrow().get_attribute("height").as_deref(), Some("40"));

    let path = KnockoutElement::query_selector(&div, "svg > path");
    assert!(path.is_some());
    let path_node = path.unwrap();
    assert_eq!(
        path_node.borrow().get_attribute("d").as_deref(),
        Some("M 100 100 L 300 100 L 200 300 Z")
    );

    Knockout::clean_node(&div);
    assert!(div.borrow().children.borrow().is_empty());
}
