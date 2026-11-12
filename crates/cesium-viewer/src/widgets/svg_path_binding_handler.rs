//! SvgPathBindingHandler matching CesiumJS `Widgets/SvgPathBindingHandler.js` (Apache-2.0).

use crate::widgets::knockout::{
    KnockoutElement, KnockoutObservable, KnockoutValue, SubscriptionToken,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub const SVG_CLASS_NAME: &str = "cesium-svgPath-svg";

/// SvgPath binding options.
#[derive(Clone, Debug)]
pub struct SvgPathOptions {
    pub path: KnockoutValue,
    pub width: KnockoutValue,
    pub height: KnockoutValue,
    pub css: Option<KnockoutValue>,
}

impl SvgPathOptions {
    /// Creates options with string/numeric values.
    pub fn new(
        path: impl Into<String>,
        width: f64,
        height: f64,
        css: Option<impl Into<String>>,
    ) -> Self {
        Self {
            path: KnockoutValue::String(path.into()),
            width: KnockoutValue::Number(width),
            height: KnockoutValue::Number(height),
            css: css.map(|c| KnockoutValue::String(c.into())),
        }
    }
}

/// Handler for the `cesiumSvgPath` Knockout binding.
pub struct SvgPathBindingHandler;

impl SvgPathBindingHandler {
    /// Applies the svg path binding to a DOM element.
    pub fn apply(
        element: &Rc<RefCell<KnockoutElement>>,
        options: SvgPathOptions,
    ) {
        let svg = KnockoutElement::new("svg");
        let path_elem = KnockoutElement::new("path");
        KnockoutElement::append_child(&svg, &path_elem);
        KnockoutElement::append_child(element, &svg);

        let svg_clone = Rc::clone(&svg);
        let path_clone = Rc::clone(&path_elem);
        let opts = Rc::new(RefCell::new(options));

        let update = {
            let svg = Rc::clone(&svg_clone);
            let path_elem = Rc::clone(&path_clone);
            let opts = Rc::clone(&opts);
            move || {
                let current = opts.borrow().clone();
                let path_d = current.path.as_str().unwrap_or_default();
                let width_str = current.width.as_str().unwrap_or_else(|| "0".to_string());
                let height_str = current.height.as_str().unwrap_or_else(|| "0".to_string());

                path_elem.borrow().set_attribute("d", path_d);
                svg.borrow().set_attribute("width", &width_str);
                svg.borrow().set_attribute("height", &height_str);
                svg.borrow().set_attribute(
                    "viewBox",
                    format!("0 0 {width_str} {height_str}"),
                );

                if let Some(css_val) = &current.css {
                    if let Some(css_str) = css_val.as_str() {
                        if !css_str.is_empty() {
                            svg.borrow().set_attribute(
                                "class",
                                format!("{SVG_CLASS_NAME} {css_str}"),
                            );
                            return;
                        }
                    }
                }
                svg.borrow().set_attribute("class", SVG_CLASS_NAME);
            }
        };

        // Initial render
        update();

        // Register reactive subscriptions if any fields are observable
        let mut subscriptions: Vec<(Rc<KnockoutObservable<KnockoutValue>>, SubscriptionToken)> =
            Vec::new();

        let subscribe_field = |val: &KnockoutValue,
                               opts_rc: &Rc<RefCell<SvgPathOptions>>,
                               updater: &Rc<dyn Fn()>,
                               subs: &mut Vec<(
            Rc<KnockoutObservable<KnockoutValue>>,
            SubscriptionToken,
        )>,
                               field_setter: fn(&mut SvgPathOptions, KnockoutValue)| {
            if let KnockoutValue::Observable(obs) = val {
                let opts = Rc::clone(opts_rc);
                let update_fn = Rc::clone(updater);
                let token = obs.subscribe(move |new_val| {
                    field_setter(&mut opts.borrow_mut(), new_val.clone());
                    update_fn();
                });
                subs.push((Rc::clone(obs), token));
            }
        };

        let updater_rc: Rc<dyn Fn()> = Rc::new(update);

        let initial_opts = opts.borrow().clone();
        subscribe_field(
            &initial_opts.path,
            &opts,
            &updater_rc,
            &mut subscriptions,
            |o, v| o.path = v,
        );
        subscribe_field(
            &initial_opts.width,
            &opts,
            &updater_rc,
            &mut subscriptions,
            |o, v| o.width = v,
        );
        subscribe_field(
            &initial_opts.height,
            &opts,
            &updater_rc,
            &mut subscriptions,
            |o, v| o.height = v,
        );
        if let Some(css_val) = &initial_opts.css {
            subscribe_field(
                css_val,
                &opts,
                &updater_rc,
                &mut subscriptions,
                |o, v| o.css = Some(v),
            );
        }

        // Store cleanup hook
        if !subscriptions.is_empty() {
            element.borrow_mut().cleanups.borrow_mut().push(Box::new(move || {
                for (obs, token) in &subscriptions {
                    obs.unsubscribe(*token);
                }
            }));
        }
    }

    /// Evaluates data-bind string and applies binding if `cesiumSvgPath` is present.
    pub fn apply_bindings(
        view_model: &HashMap<String, KnockoutValue>,
        element: &Rc<RefCell<KnockoutElement>>,
    ) {
        let binding_str = match element.borrow().get_attribute("data-bind") {
            Some(s) => s,
            None => return,
        };

        if let Some(options) = parse_binding(&binding_str, view_model) {
            Self::apply(element, options);
        }
    }
}

/// Parses the `cesiumSvgPath` options from data-bind attribute string.
fn parse_binding(
    data_bind: &str,
    vm: &HashMap<String, KnockoutValue>,
) -> Option<SvgPathOptions> {
    let trimmed = data_bind.trim();
    let prefix = "cesiumSvgPath:";
    let idx = trimmed.find(prefix)?;
    let content = trimmed[idx + prefix.len()..].trim();

    // Check if the argument is a single view model identifier like `cesiumSvgPath: svgPath`
    if !content.starts_with('{') {
        let ident = content.split_whitespace().next()?.trim_matches(|c| c == ',' || c == ';');
        if let Some(val) = vm.get(ident) {
            let unwrapped = val.unwrap();
            if let KnockoutValue::Object(map) = unwrapped {
                let borrowed = map.borrow();
                let path = borrowed.get("path").cloned().unwrap_or(KnockoutValue::String(String::new()));
                let width = borrowed.get("width").cloned().unwrap_or(KnockoutValue::Number(0.0));
                let height = borrowed.get("height").cloned().unwrap_or(KnockoutValue::Number(0.0));
                let css = borrowed.get("css").cloned();
                return Some(SvgPathOptions {
                    path,
                    width,
                    height,
                    css,
                });
            }
        }
        return None;
    }

    // Parsing object literal: `{ path: ..., width: ..., height: ..., css: ... }`
    let inner = content.trim_start_matches('{').trim_end_matches('}').trim();
    let mut path = KnockoutValue::String(String::new());
    let mut width = KnockoutValue::Number(0.0);
    let mut height = KnockoutValue::Number(0.0);
    let mut css = None;

    for part in inner.split(',') {
        if let Some((k, v)) = part.split_once(':') {
            let key = k.trim();
            let raw_v = v.trim().trim_matches('"').trim_matches('\'');

            let resolve_value = |raw: &str| -> KnockoutValue {
                if let Some(vm_val) = vm.get(raw) {
                    vm_val.clone()
                } else if let Ok(n) = raw.parse::<f64>() {
                    KnockoutValue::Number(n)
                } else {
                    KnockoutValue::String(raw.to_string())
                }
            };

            match key {
                "path" => path = resolve_value(raw_v),
                "width" => width = resolve_value(raw_v),
                "height" => height = resolve_value(raw_v),
                "css" => css = Some(resolve_value(raw_v)),
                _ => {}
            }
        }
    }

    Some(SvgPathOptions {
        path,
        width,
        height,
        css,
    })
}
