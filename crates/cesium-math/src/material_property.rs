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

/// The evaluated value of a GridMaterialProperty.
#[derive(Clone, Debug, PartialEq)]
pub struct GridMaterialValue {
    pub color: Color,
    pub cell_alpha: f64,
    pub line_count: Cartesian2,
    pub line_thickness: Cartesian2,
    pub line_offset: Cartesian2,
}

/// A MaterialProperty that maps to grid pattern materials.
#[derive(Clone, Debug, Default)]
pub struct GridMaterialProperty {
    pub color: Option<ConstantProperty<Color>>,
    pub cell_alpha: Option<ConstantProperty<f64>>,
    pub line_count: Option<ConstantProperty<Cartesian2>>,
    pub line_thickness: Option<ConstantProperty<Cartesian2>>,
    pub line_offset: Option<ConstantProperty<Cartesian2>>,
    definition_changed: Event<()>,
}

impl GridMaterialProperty {
    pub fn new(
        color: Option<ConstantProperty<Color>>,
        cell_alpha: Option<ConstantProperty<f64>>,
        line_count: Option<ConstantProperty<Cartesian2>>,
        line_thickness: Option<ConstantProperty<Cartesian2>>,
        line_offset: Option<ConstantProperty<Cartesian2>>,
    ) -> Self {
        Self {
            color,
            cell_alpha,
            line_count,
            line_thickness,
            line_offset,
            definition_changed: Event::new(),
        }
    }

    pub fn get_type(&self) -> &'static str {
        "Grid"
    }

    pub fn is_constant(&self) -> bool {
        self.color.as_ref().is_none_or(|c| c.is_constant())
            && self.cell_alpha.as_ref().is_none_or(|c| c.is_constant())
            && self.line_count.as_ref().is_none_or(|c| c.is_constant())
            && self.line_thickness.as_ref().is_none_or(|c| c.is_constant())
            && self.line_offset.as_ref().is_none_or(|c| c.is_constant())
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> GridMaterialValue {
        let color = self
            .color
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(Color::WHITE);
        let cell_alpha = self
            .cell_alpha
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(0.1);
        let line_count = self
            .line_count
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(Cartesian2::new(8.0, 8.0));
        let line_thickness = self
            .line_thickness
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(Cartesian2::new(1.0, 1.0));
        let line_offset = self
            .line_offset
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(Cartesian2::new(0.0, 0.0));
        GridMaterialValue {
            color,
            cell_alpha,
            line_count,
            line_thickness,
            line_offset,
        }
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => {
                self.color == o.color
                    && self.cell_alpha == o.cell_alpha
                    && self.line_count == o.line_count
                    && self.line_thickness == o.line_thickness
                    && self.line_offset == o.line_offset
            }
            None => false,
        }
    }
}

/// The orientation of stripes in StripeMaterialProperty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StripeOrientation {
    #[default]
    Horizontal = 0,
    Vertical = 1,
}

/// The evaluated value of a StripeMaterialProperty.
#[derive(Clone, Debug, PartialEq)]
pub struct StripeMaterialValue {
    pub horizontal: bool,
    pub even_color: Color,
    pub odd_color: Color,
    pub offset: f64,
    pub repeat: f64,
}

/// A MaterialProperty that maps to stripe pattern materials.
#[derive(Clone, Debug, Default)]
pub struct StripeMaterialProperty {
    pub orientation: Option<ConstantProperty<StripeOrientation>>,
    pub even_color: Option<ConstantProperty<Color>>,
    pub odd_color: Option<ConstantProperty<Color>>,
    pub offset: Option<ConstantProperty<f64>>,
    pub repeat: Option<ConstantProperty<f64>>,
    definition_changed: Event<()>,
}

impl StripeMaterialProperty {
    pub fn new(
        orientation: Option<ConstantProperty<StripeOrientation>>,
        even_color: Option<ConstantProperty<Color>>,
        odd_color: Option<ConstantProperty<Color>>,
        offset: Option<ConstantProperty<f64>>,
        repeat: Option<ConstantProperty<f64>>,
    ) -> Self {
        Self {
            orientation,
            even_color,
            odd_color,
            offset,
            repeat,
            definition_changed: Event::new(),
        }
    }

    pub fn get_type(&self) -> &'static str {
        "Stripe"
    }

    pub fn is_constant(&self) -> bool {
        self.orientation.as_ref().is_none_or(|c| c.is_constant())
            && self.even_color.as_ref().is_none_or(|c| c.is_constant())
            && self.odd_color.as_ref().is_none_or(|c| c.is_constant())
            && self.offset.as_ref().is_none_or(|c| c.is_constant())
            && self.repeat.as_ref().is_none_or(|c| c.is_constant())
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> StripeMaterialValue {
        let orientation = self
            .orientation
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(StripeOrientation::Horizontal);
        let even_color = self
            .even_color
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(Color::WHITE);
        let odd_color = self
            .odd_color
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(Color::BLACK);
        let offset = self
            .offset
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(0.0);
        let repeat = self
            .repeat
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(1.0);
        StripeMaterialValue {
            horizontal: orientation == StripeOrientation::Horizontal,
            even_color,
            odd_color,
            offset,
            repeat,
        }
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => {
                self.orientation == o.orientation
                    && self.even_color == o.even_color
                    && self.odd_color == o.odd_color
                    && self.offset == o.offset
                    && self.repeat == o.repeat
            }
            None => false,
        }
    }
}

/// The evaluated value of an ImageMaterialProperty.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageMaterialValue {
    pub image: Option<String>,
    pub repeat: Cartesian2,
    pub color: Color,
    pub transparent: bool,
}

/// A MaterialProperty that maps to image materials.
#[derive(Clone, Debug, Default)]
pub struct ImageMaterialProperty {
    pub image: Option<ConstantProperty<String>>,
    pub repeat: Option<ConstantProperty<Cartesian2>>,
    pub color: Option<ConstantProperty<Color>>,
    pub transparent: Option<ConstantProperty<bool>>,
    definition_changed: Event<()>,
}

impl ImageMaterialProperty {
    pub fn new(
        image: Option<ConstantProperty<String>>,
        repeat: Option<ConstantProperty<Cartesian2>>,
        color: Option<ConstantProperty<Color>>,
        transparent: Option<ConstantProperty<bool>>,
    ) -> Self {
        Self {
            image,
            repeat,
            color,
            transparent,
            definition_changed: Event::new(),
        }
    }

    pub fn get_type(&self) -> &'static str {
        "Image"
    }

    pub fn is_constant(&self) -> bool {
        self.image.as_ref().is_none_or(|c| c.is_constant())
            && self.repeat.as_ref().is_none_or(|c| c.is_constant())
            && self.color.as_ref().is_none_or(|c| c.is_constant())
            && self.transparent.as_ref().is_none_or(|c| c.is_constant())
    }

    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    pub fn get_value(&self, time: Option<&JulianDate>) -> ImageMaterialValue {
        let image = self.image.as_ref().and_then(|c| c.get_value(time));
        let repeat = self
            .repeat
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(Cartesian2::new(1.0, 1.0));
        let color = self
            .color
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(Color::WHITE);
        let transparent = self
            .transparent
            .as_ref()
            .and_then(|c| c.get_value(time))
            .unwrap_or(false);
        ImageMaterialValue {
            image,
            repeat,
            color,
            transparent,
        }
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => {
                self.image == o.image
                    && self.repeat == o.repeat
                    && self.color == o.color
                    && self.transparent == o.transparent
            }
            None => false,
        }
    }
}
