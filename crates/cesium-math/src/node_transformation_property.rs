//! NodeTransformationProperty matching CesiumJS `DataSources/NodeTransformationProperty.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    Cartesian3, ConstantProperty, DynProperty, Event, JulianDate, Property, Quaternion,
    TranslationRotationScale,
};

#[derive(Default)]
struct NodeTransformationPropertyInner {
    translation: Option<Rc<dyn DynProperty<Cartesian3>>>,
    rotation: Option<Rc<dyn DynProperty<Quaternion>>>,
    scale: Option<Rc<dyn DynProperty<Cartesian3>>>,
    translation_sub: Option<Box<dyn FnOnce() + 'static>>,
    rotation_sub: Option<Box<dyn FnOnce() + 'static>>,
    scale_sub: Option<Box<dyn FnOnce() + 'static>>,
}

impl Drop for NodeTransformationPropertyInner {
    fn drop(&mut self) {
        if let Some(sub) = self.translation_sub.take() {
            sub();
        }
        if let Some(sub) = self.rotation_sub.take() {
            sub();
        }
        if let Some(sub) = self.scale_sub.take() {
            sub();
        }
    }
}

/// Construction options for NodeTransformationProperty.
#[derive(Default, Clone)]
pub struct NodeTransformationPropertyOptions {
    pub translation: Option<Rc<dyn DynProperty<Cartesian3>>>,
    pub rotation: Option<Rc<dyn DynProperty<Quaternion>>>,
    pub scale: Option<Rc<dyn DynProperty<Cartesian3>>>,
}

/// A Property that produces TranslationRotationScale data.
pub struct NodeTransformationProperty {
    inner: RefCell<NodeTransformationPropertyInner>,
    definition_changed: Event<()>,
}

impl std::fmt::Debug for NodeTransformationProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeTransformationProperty")
            .field("is_constant", &self.is_constant())
            .finish()
    }
}

impl Default for NodeTransformationProperty {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for NodeTransformationProperty {
    fn clone(&self) -> Self {
        let prop = Self::new();
        let inner = self.inner.borrow();
        if let Some(t) = &inner.translation {
            prop.set_translation(Some(Rc::clone(t)));
        }
        if let Some(r) = &inner.rotation {
            prop.set_rotation(Some(Rc::clone(r)));
        }
        if let Some(s) = &inner.scale {
            prop.set_scale(Some(Rc::clone(s)));
        }
        prop
    }
}

impl NodeTransformationProperty {
    /// Creates a new NodeTransformationProperty with default values.
    pub fn new() -> Self {
        Self::from_options(NodeTransformationPropertyOptions::default())
    }

    /// Creates a new NodeTransformationProperty with the specified options.
    pub fn from_options(options: NodeTransformationPropertyOptions) -> Self {
        let prop = Self {
            inner: RefCell::new(NodeTransformationPropertyInner::default()),
            definition_changed: Event::new(),
        };

        if let Some(t) = options.translation {
            prop.set_translation(Some(t));
        }
        if let Some(r) = options.rotation {
            prop.set_rotation(Some(r));
        }
        if let Some(s) = options.scale {
            prop.set_scale(Some(s));
        }

        prop
    }

    /// Creates a new NodeTransformationProperty from constant raw values.
    pub fn from_values(
        translation: Option<Cartesian3>,
        rotation: Option<Quaternion>,
        scale: Option<Cartesian3>,
    ) -> Self {
        Self::from_options(NodeTransformationPropertyOptions {
            translation: translation
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Cartesian3>>),
            rotation: rotation
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Quaternion>>),
            scale: scale
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Cartesian3>>),
        })
    }

    /// Gets a value indicating if this property is constant.
    pub fn is_constant(&self) -> bool {
        let inner = self.inner.borrow();
        inner.translation.as_ref().is_none_or(|p| p.is_constant())
            && inner.rotation.as_ref().is_none_or(|p| p.is_constant())
            && inner.scale.as_ref().is_none_or(|p| p.is_constant())
    }

    /// Gets the event that is raised whenever the definition of this property changes.
    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    /// Gets the translation property.
    pub fn translation(&self) -> Option<Rc<dyn DynProperty<Cartesian3>>> {
        self.inner.borrow().translation.clone()
    }

    /// Sets the translation property.
    pub fn set_translation(&self, property: Option<Rc<dyn DynProperty<Cartesian3>>>) {
        let mut inner = self.inner.borrow_mut();
        let is_same = match (&inner.translation, &property) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        };

        if is_same {
            return;
        }

        if let Some(sub) = inner.translation_sub.take() {
            sub();
        }

        if let Some(ref p) = property {
            let dc = self.definition_changed.clone();
            let sub = p.definition_changed().add_listener_scoped(move |_| {
                dc.raise_event(&());
            });
            inner.translation_sub = Some(sub);
        }

        inner.translation = property;
        drop(inner);
        self.definition_changed.raise_event(&());
    }

    /// Sets the translation property from a raw constant value.
    pub fn set_translation_value(&self, value: Cartesian3) {
        self.set_translation(Some(Rc::new(ConstantProperty::from_value(value))));
    }

    /// Gets the rotation property.
    pub fn rotation(&self) -> Option<Rc<dyn DynProperty<Quaternion>>> {
        self.inner.borrow().rotation.clone()
    }

    /// Sets the rotation property.
    pub fn set_rotation(&self, property: Option<Rc<dyn DynProperty<Quaternion>>>) {
        let mut inner = self.inner.borrow_mut();
        let is_same = match (&inner.rotation, &property) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        };

        if is_same {
            return;
        }

        if let Some(sub) = inner.rotation_sub.take() {
            sub();
        }

        if let Some(ref p) = property {
            let dc = self.definition_changed.clone();
            let sub = p.definition_changed().add_listener_scoped(move |_| {
                dc.raise_event(&());
            });
            inner.rotation_sub = Some(sub);
        }

        inner.rotation = property;
        drop(inner);
        self.definition_changed.raise_event(&());
    }

    /// Sets the rotation property from a raw constant value.
    pub fn set_rotation_value(&self, value: Quaternion) {
        self.set_rotation(Some(Rc::new(ConstantProperty::from_value(value))));
    }

    /// Gets the scale property.
    pub fn scale(&self) -> Option<Rc<dyn DynProperty<Cartesian3>>> {
        self.inner.borrow().scale.clone()
    }

    /// Sets the scale property.
    pub fn set_scale(&self, property: Option<Rc<dyn DynProperty<Cartesian3>>>) {
        let mut inner = self.inner.borrow_mut();
        let is_same = match (&inner.scale, &property) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        };

        if is_same {
            return;
        }

        if let Some(sub) = inner.scale_sub.take() {
            sub();
        }

        if let Some(ref p) = property {
            let dc = self.definition_changed.clone();
            let sub = p.definition_changed().add_listener_scoped(move |_| {
                dc.raise_event(&());
            });
            inner.scale_sub = Some(sub);
        }

        inner.scale = property;
        drop(inner);
        self.definition_changed.raise_event(&());
    }

    /// Sets the scale property from a raw constant value.
    pub fn set_scale_value(&self, value: Cartesian3) {
        self.set_scale(Some(Rc::new(ConstantProperty::from_value(value))));
    }

    /// Gets the value of the property at the provided time.
    pub fn get_value(&self, time: Option<&JulianDate>) -> Option<TranslationRotationScale> {
        let mut result = TranslationRotationScale::default();
        self.get_value_to(time, &mut result);
        Some(result)
    }

    /// Stores the evaluated value into the provided result struct.
    pub fn get_value_to(&self, time: Option<&JulianDate>, result: &mut TranslationRotationScale) {
        let inner = self.inner.borrow();
        result.translation = inner
            .translation
            .as_ref()
            .and_then(|p| p.get_value(time))
            .unwrap_or(Cartesian3::ZERO);

        result.rotation = inner
            .rotation
            .as_ref()
            .and_then(|p| p.get_value(time))
            .unwrap_or(Quaternion::IDENTITY);

        result.scale = inner
            .scale
            .as_ref()
            .and_then(|p| p.get_value(time))
            .unwrap_or_else(|| Cartesian3::new(1.0, 1.0, 1.0));
    }

    /// Compares this property to the provided property and returns true if they are equal.
    pub fn equals(&self, other: Option<&Self>) -> bool {
        let other = match other {
            Some(o) => o,
            None => return false,
        };
        if std::ptr::eq(self, other) {
            return true;
        }
        let s = self.inner.borrow();
        let o = other.inner.borrow();

        fn prop_eq<T: 'static + Clone>(
            a: &Option<Rc<dyn DynProperty<T>>>,
            b: &Option<Rc<dyn DynProperty<T>>>,
        ) -> bool {
            match (a, b) {
                (None, None) => true,
                (Some(a), Some(b)) => a.equals_dyn(&**b),
                _ => false,
            }
        }

        prop_eq(&s.translation, &o.translation)
            && prop_eq(&s.rotation, &o.rotation)
            && prop_eq(&s.scale, &o.scale)
    }
}

impl DynProperty<TranslationRotationScale> for NodeTransformationProperty {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<TranslationRotationScale> {
        self.get_value(time)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn equals_dyn(&self, other: &dyn DynProperty<TranslationRotationScale>) -> bool {
        if let Some(o) = other.as_any().downcast_ref::<Self>() {
            self.equals(Some(o))
        } else {
            false
        }
    }
}

impl Property<TranslationRotationScale> for NodeTransformationProperty {
    fn is_constant(&self) -> bool {
        self.is_constant()
    }
    fn definition_changed(&self) -> &Event<()> {
        self.definition_changed()
    }
    fn get_value(&self, time: Option<&JulianDate>) -> Option<TranslationRotationScale> {
        self.get_value(time)
    }
    fn equals(&self, other: Option<&Self>) -> bool {
        self.equals(other)
    }
}
