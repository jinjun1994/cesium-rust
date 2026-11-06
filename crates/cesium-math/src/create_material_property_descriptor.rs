//! createMaterialPropertyDescriptor matching CesiumJS `DataSources/createMaterialPropertyDescriptor.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    Color, ColorMaterialProperty, DeveloperError, Event, ImageMaterialProperty, ListenerId,
    MaterialProperty,
};

/// Discriminated material input for inferring material property types.
#[derive(Clone)]
pub enum MaterialValueInput {
    Color(Color),
    Image(String),
    Property(Rc<dyn MaterialProperty>),
    Invalid,
}

impl std::fmt::Debug for MaterialValueInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Color(c) => write!(f, "MaterialValueInput::Color({c:?})"),
            Self::Image(u) => write!(f, "MaterialValueInput::Image({u:?})"),
            Self::Property(_) => write!(f, "MaterialValueInput::Property(dyn MaterialProperty)"),
            Self::Invalid => write!(f, "MaterialValueInput::Invalid"),
        }
    }
}

impl From<Color> for MaterialValueInput {
    fn from(c: Color) -> Self {
        Self::Color(c)
    }
}

impl From<&str> for MaterialValueInput {
    fn from(s: &str) -> Self {
        Self::Image(s.to_string())
    }
}

impl From<String> for MaterialValueInput {
    fn from(s: String) -> Self {
        Self::Image(s)
    }
}

/// Infers and creates a MaterialProperty from an input value.
pub fn create_material_property(
    value: MaterialValueInput,
) -> Result<Rc<dyn MaterialProperty>, DeveloperError> {
    match value {
        MaterialValueInput::Color(c) => Ok(Rc::new(ColorMaterialProperty::from_color(c))),
        MaterialValueInput::Image(url) => Ok(Rc::new(ImageMaterialProperty::from_url(url))),
        MaterialValueInput::Property(p) => Ok(p),
        MaterialValueInput::Invalid => {
            Err(DeveloperError::new("Unable to infer material type."))
        }
    }
}

/// A property descriptor managing material properties on graphics objects.
#[derive(Default)]
pub struct MaterialPropertyDescriptor {
    name: String,
    value: Option<Rc<dyn MaterialProperty>>,
    definition_changed: Event<()>,
    subscription: Option<ListenerId>,
}

impl MaterialPropertyDescriptor {
    /// Creates a new MaterialPropertyDescriptor.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: None,
            definition_changed: Event::new(),
            subscription: None,
        }
    }

    /// Gets the property name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the current material property value.
    pub fn get(&self) -> Option<Rc<dyn MaterialProperty>> {
        self.value.clone()
    }

    /// Sets the material property value using type inference.
    pub fn set(&mut self, input: Option<MaterialValueInput>) -> Result<(), DeveloperError> {
        let new_val = match input {
            Some(v) => Some(create_material_property(v)?),
            None => None,
        };

        self.value = new_val.clone();
        if let Some(ref val) = new_val {
            let dc = self.definition_changed.clone();
            let lid = val.definition_changed().add_listener(move |_| {
                dc.raise_event(&());
            });
            self.subscription = Some(lid);
        } else {
            self.subscription = None;
        }

        self.definition_changed.raise_event(&());
        Ok(())
    }

    /// Convenience setter for Color.
    pub fn set_color(&mut self, color: Color) -> Result<(), DeveloperError> {
        self.set(Some(MaterialValueInput::Color(color)))
    }

    /// Convenience setter for Image / URL.
    pub fn set_image(&mut self, image: impl Into<String>) -> Result<(), DeveloperError> {
        self.set(Some(MaterialValueInput::Image(image.into())))
    }

    /// Convenience setter for an existing MaterialProperty.
    pub fn set_property(&mut self, property: Rc<dyn MaterialProperty>) -> Result<(), DeveloperError> {
        self.set(Some(MaterialValueInput::Property(property)))
    }

    /// Gets the definition_changed event.
    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }
}
