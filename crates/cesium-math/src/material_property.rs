//! Material property interfaces and implementations matching CesiumJS `DataSources/*MaterialProperty.js` (Apache-2.0).

use crate::{Cartesian2, Color, ConstantProperty, Event, JulianDate};

/// A MaterialProperty that maps to flat Color materials.
#[derive(Clone, Debug, Default)]
pub struct ColorMaterialProperty {
    pub color: Option<ConstantProperty<Color>>,
    definition_changed: Event<()>,
}

impl ColorMaterialProperty {
    pub fn new(color: Option<ConstantProperty<Color>>) -> Self {
        Self {
            color,
            definition_changed: Event::new(),
        }
    }

    pub fn from_color(color: Color) -> Self {
        Self::new(Some(ConstantProperty::from_value(color)))
    }

    pub fn get_type(&self) -> &'static str {
        "Color"
    }

    pub fn is_constant(&self) -> bool {
        self.color.as_ref().is_none_or(|c| c.is_constant())
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> Color {
        self.color
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(Color::WHITE)
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => self.color == o.color,
            None => false,
        }
    }
}

/// The evaluated value of a CheckerboardMaterialProperty.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckerboardMaterialValue {
    pub light_color: Color,
    pub dark_color: Color,
    pub repeat: Cartesian2,
}

/// A MaterialProperty that maps to checkerboard pattern materials.
#[derive(Clone, Debug, Default)]
pub struct CheckerboardMaterialProperty {
    pub even_color: Option<ConstantProperty<Color>>,
    pub odd_color: Option<ConstantProperty<Color>>,
    pub repeat: Option<ConstantProperty<Cartesian2>>,
    definition_changed: Event<()>,
}

impl CheckerboardMaterialProperty {
    pub fn new(
        even_color: Option<ConstantProperty<Color>>,
        odd_color: Option<ConstantProperty<Color>>,
        repeat: Option<ConstantProperty<Cartesian2>>,
    ) -> Self {
        Self {
            even_color,
            odd_color,
            repeat,
            definition_changed: Event::new(),
        }
    }

    pub fn get_type(&self) -> &'static str {
        "Checkerboard"
    }

    pub fn is_constant(&self) -> bool {
        self.even_color.as_ref().is_none_or(|c| c.is_constant())
            && self.odd_color.as_ref().is_none_or(|c| c.is_constant())
            && self.repeat.as_ref().is_none_or(|c| c.is_constant())
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> CheckerboardMaterialValue {
        let light_color = self
            .even_color
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(Color::WHITE);
        let dark_color = self
            .odd_color
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(Color::BLACK);
        let repeat = self
            .repeat
            .as_ref()
            .and_then(|r| r.get_value(time))
            .unwrap_or(Cartesian2::new(2.0, 2.0));
        CheckerboardMaterialValue {
            light_color,
            dark_color,
            repeat,
        }
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => {
                self.even_color == o.even_color
                    && self.odd_color == o.odd_color
                    && self.repeat == o.repeat
            }
            None => false,
        }
    }
}
